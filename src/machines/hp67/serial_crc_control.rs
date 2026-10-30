//! Autonomous CRC control latches fed only by resolved IS/SYNC.
//!
//! Commands execute one word after reception. Completion at b55 is a logical
//! scheduling approximation, not evidence for the physical F2/CRC latch edge.

use super::{
    decode_crc_opcode, CrcArchitecturalCore, CrcInstruction, Hp67ElectricalBackplane,
    Hp67RamCommandError, Hp67RamCommandFrame, Hp67RamCommandReceiver, CRC_FLAG_BUFFER_READY,
    CRC_FLAG_COUNT, CRC_FLAG_F7_STATUS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CrcControlError {
    Phase { expected: u8, actual: u8, tick: u64 },
    Command(Hp67RamCommandError),
    Decode(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67CrcControlResult {
    pub instruction: CrcInstruction,
    pub flag: u8,
    pub value: bool,
    pub condition: Option<bool>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67SerialCrcControl {
    receiver: Hp67RamCommandReceiver,
    flags: [bool; CRC_FLAG_COUNT],
    external: [bool; CRC_FLAG_COUNT],
    expected_bit: u8,
    executing: Option<CrcInstruction>,
    result: Option<Hp67CrcControlResult>,
}

// Small, non-Copy proposals are consumed once by the composition. No frozen
// contacts or retained flag array travel through the per-cell DATA transaction.
#[derive(Debug)]
pub(crate) struct Hp67CrcControlSample {
    bit: u8,
    change: Hp67CrcControlChange,
}

#[derive(Debug)]
enum Hp67CrcControlChange {
    Begin(Option<CrcInstruction>),
    Advance,
    Receive(Hp67RamCommandReceiver),
}

impl Hp67SerialCrcControl {
    pub const fn flags(&self) -> &[bool; CRC_FLAG_COUNT] {
        &self.flags
    }

    pub const fn result(&self) -> Option<Hp67CrcControlResult> {
        self.result
    }

    pub const fn received_frame(&self) -> Option<Hp67RamCommandFrame> {
        self.receiver.received_frame()
    }

    /// Explicit reference composition: snapshot logical external inputs.
    pub fn freeze_inputs(&mut self, mirror: &CrcArchitecturalCore) -> Result<(), String> {
        self.freeze_checked_inputs(mirror, None, None)
    }

    /// Production card-present input comes from MTRS. The mirror is checked,
    /// never imported as the source for that external contact.
    pub fn freeze_inputs_with_mtrs(
        &mut self,
        mirror: &CrcArchitecturalCore,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<(), String> {
        let present = super::card_contact::sample_card_present(bus)
            .map_err(|error| format!("CRC MTRS input: {error:?}"))?;
        self.freeze_checked_inputs(mirror, Some(present), None)
    }

    /// Production physical contacts: both selected inputs are sampled from
    /// resolved conductors before the frozen input image can change.
    pub fn freeze_inputs_with_contacts(
        &mut self,
        mirror: &CrcArchitecturalCore,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<(), String> {
        let present = super::card_contact::sample_card_present(bus)
            .map_err(|error| format!("CRC MTRS input: {error:?}"))?;
        let program = super::program_switch::sample_program_mode(bus)
            .map_err(|error| format!("CRC PGMSW input: {error:?}"))?;
        self.freeze_checked_inputs(mirror, Some(present), Some(program))
    }

    fn freeze_checked_inputs(
        &mut self,
        mirror: &CrcArchitecturalCore,
        mtrs: Option<bool>,
        program: Option<bool>,
    ) -> Result<(), String> {
        if self.expected_bit != 0 {
            return Err("CRC contacts must freeze before b0".into());
        }
        self.check_mirror(mirror)?;
        if let Some(present) = mtrs {
            if mirror.external_flag(super::CRC_FLAG_CARD_PRESENT) != Some(present) {
                return Err(format!("CRC MTRS/oracle contact mismatch: pin={present}"));
            }
        }
        if let Some(selected) = program {
            if mirror.external_flag(super::CRC_FLAG_PROGRAM_MODE) != Some(selected) {
                return Err(format!("CRC PGMSW/oracle contact mismatch: pin={selected}"));
            }
        }
        // Validate everything before replacing the frozen word image.
        self.external = std::array::from_fn(|index| {
            if index == super::CRC_FLAG_CARD_PRESENT {
                if let Some(present) = mtrs {
                    return present;
                }
            }
            if index == super::CRC_FLAG_PROGRAM_MODE {
                if let Some(selected) = program {
                    return selected;
                }
            }
            mirror.external_flag(index).expect("CRC contact index")
        });
        Ok(())
    }

    pub fn check_mirror(&self, mirror: &CrcArchitecturalCore) -> Result<(), String> {
        for (index, value) in self.flags.iter().enumerate() {
            if mirror.flag(index) != Some(*value) {
                return Err(format!(
                    "autonomous CRC latch {index} mismatch: device={value} mirror={:?}",
                    mirror.flag(index)
                ));
            }
        }
        Ok(())
    }

    /// Only logical FIFO/transport events own these two shared handshake latches.
    /// Other flags, including motor and write mode, never enter through this port.
    pub fn apply_buffer_status(&mut self, ready: bool, error: bool) {
        self.flags[CRC_FLAG_BUFFER_READY] = ready;
        self.flags[CRC_FLAG_F7_STATUS] = error;
    }

    pub fn sample_at_edge(
        &mut self,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<(), Hp67CrcControlError> {
        let sample = self.prepare_sample(bus)?;
        self.commit_sample(sample);
        Ok(())
    }

    /// Validate without copying retained latches or contacts. The caller must
    /// commit before modifying this endpoint again; only the shared DATA
    /// transaction holds this crate-private proposal across another check.
    pub(crate) fn prepare_sample(
        &self,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<Hp67CrcControlSample, Hp67CrcControlError> {
        let bit = bus.word_bit();
        if !bus.at_transport_sample_edge() || bit != self.expected_bit {
            return Err(Hp67CrcControlError::Phase {
                expected: self.expected_bit,
                actual: bit,
                tick: bus.tick().get(),
            });
        }
        let change = if bit == 0 {
            let executing = match self.receiver.received_frame() {
                Some(frame) if frame.instruction => decode_crc_opcode(frame.word)
                    .map_err(|_| Hp67CrcControlError::Decode(frame.word))?,
                _ => None,
            };
            Hp67CrcControlChange::Begin(executing)
        } else if bit >= 46 {
            let mut receiver = self.receiver;
            receiver
                .sample_at_edge(bus)
                .map_err(Hp67CrcControlError::Command)?;
            Hp67CrcControlChange::Receive(receiver)
        } else {
            Hp67CrcControlChange::Advance
        };
        Ok(Hp67CrcControlSample { bit, change })
    }

    pub(crate) fn commit_sample(&mut self, sample: Hp67CrcControlSample) {
        let Hp67CrcControlSample { bit, change } = sample;
        debug_assert_eq!(bit, self.expected_bit);
        match change {
            Hp67CrcControlChange::Begin(executing) => {
                self.executing = executing;
                self.result = None;
            }
            Hp67CrcControlChange::Receive(receiver) => self.receiver = receiver,
            Hp67CrcControlChange::Advance => {}
        }
        if bit == 55 {
            if let Some(instruction) = self.executing {
                let flag = match instruction {
                    CrcInstruction::SetFlag { flag }
                    | CrcInstruction::TestFlagAndClear { flag } => flag,
                };
                let index = usize::from(flag);
                let (value, condition) = match instruction {
                    CrcInstruction::SetFlag { .. } => (true, None),
                    CrcInstruction::TestFlagAndClear { .. } => {
                        (false, Some(self.flags[index] || self.external[index]))
                    }
                };
                self.flags[index] = value;
                self.result = Some(Hp67CrcControlResult {
                    instruction,
                    flag,
                    value,
                    condition,
                });
            }
        }
        self.expected_bit = if bit == 55 { 0 } else { bit + 1 };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        emulation::Drive,
        machines::hp67::{
            isa::{act_address_drive, rom_word_drive},
            rom_devices::RomIsDrives,
            wiring::{Hp67Driver, Hp67Net},
        },
    };

    fn launch(bus: &mut Hp67ElectricalBackplane, word: u16, instruction: bool) {
        let bit = bus.word_bit();
        bus.commit_serial_launch(
            act_address_drive(0x123, bit),
            RomIsDrives::Aggregate(rom_word_drive(word, bit)),
            if bit >= 46 && instruction {
                Drive::High
            } else {
                Drive::Low
            },
            None,
        );
    }

    fn word(
        crc: &mut Hp67SerialCrcControl,
        bus: &mut Hp67ElectricalBackplane,
        opcode: u16,
        instruction: bool,
    ) {
        let tick = bus.tick().get();
        for _ in 0..56 {
            launch(bus, opcode, instruction);
            bus.advance_clock();
            bus.advance_clock();
            crc.sample_at_edge(bus).unwrap();
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(bus.tick().get() - tick, 224);
    }

    #[test]
    fn m15t_crc_all_words_qualifiers_latches_and_contacts_match_oracle() {
        for internal in [false, true] {
            for external in [false, true] {
                for instruction in [false, true] {
                    for opcode in 0..=0x3ff {
                        let mut crc = Hp67SerialCrcControl::default();
                        let mut mirror = CrcArchitecturalCore::default();
                        for index in 0..CRC_FLAG_COUNT {
                            crc.flags[index] = internal;
                            mirror.commit_control_flag(index as u8, internal).unwrap();
                            mirror.set_external_flag(index as u8, external).unwrap();
                        }
                        crc.freeze_inputs(&mirror).unwrap();
                        let mut bus = Hp67ElectricalBackplane::default();
                        word(&mut crc, &mut bus, opcode, instruction);
                        assert_eq!(crc.result(), None, "fetch must not execute immediately");
                        crc.check_mirror(&mirror).unwrap();
                        let decoded = if instruction {
                            decode_crc_opcode(opcode).unwrap()
                        } else {
                            None
                        };
                        let condition = if decoded.is_some() {
                            mirror.execute_opcode(opcode).unwrap()
                        } else {
                            None
                        };
                        word(&mut crc, &mut bus, 0, true);
                        assert_eq!(crc.result().map(|result| result.instruction), decoded);
                        assert_eq!(crc.result().and_then(|result| result.condition), condition);
                        crc.check_mirror(&mirror).unwrap();
                        assert_eq!(crc.external, [external; CRC_FLAG_COUNT]);
                    }
                }
            }
        }
    }

    #[test]
    fn m15t_crc_faults_preserve_image_and_pin_changes_own_command() {
        let mut crc = Hp67SerialCrcControl::default();
        let mut bus = Hp67ElectricalBackplane::default();
        for bit in 0..56 {
            launch(&mut bus, 0o1000, true);
            let before = crc;
            assert!(crc.sample_at_edge(&bus).is_err());
            assert_eq!(crc, before);
            bus.advance_clock();
            bus.advance_clock();
            if bit == 46 {
                // Replace observed bit zero: a non-control word must be received.
                bus.drive(
                    Hp67Net::Isa,
                    Hp67Driver::StructuralRomResponder,
                    Drive::High,
                );
            }
            if bit == 47 {
                bus.drive(
                    Hp67Net::Sync,
                    Hp67Driver::StructuralRomResponder,
                    Drive::Low,
                );
                assert!(crc.sample_at_edge(&bus).is_err());
                assert_eq!(crc, before);
                bus.drive(
                    Hp67Net::Sync,
                    Hp67Driver::StructuralRomResponder,
                    Drive::HighZ,
                );
            }
            crc.sample_at_edge(&bus).unwrap();
            let after = crc;
            assert!(crc.sample_at_edge(&bus).is_err());
            assert_eq!(crc, after);
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(crc.received_frame().unwrap().word, 0o1001);
        word(&mut crc, &mut bus, 0, true);
        assert_eq!(crc.result(), None);
        assert_eq!(crc.flags(), &[false; CRC_FLAG_COUNT]);
    }

    #[test]
    fn m15t_crc_buffer_port_does_not_import_motor_or_external_latches() {
        let mut crc = Hp67SerialCrcControl::default();
        crc.flags[9] = true;
        crc.external[10] = true;
        crc.apply_buffer_status(true, true);
        assert!(crc.flags[0] && crc.flags[7] && crc.flags[9]);
        crc.apply_buffer_status(false, false);
        assert!(!crc.flags[0] && !crc.flags[7] && crc.flags[9]);
        assert!(crc.external[10]);
        assert!(crc.freeze_inputs(&CrcArchitecturalCore::default()).is_err());
        assert!(crc.flags[9]);
    }
    #[test]
    fn m15t_crc_shared_sample_is_not_committed_on_data_fault() {
        let mut path = super::super::Hp67DataSerialWordPath::default()
            .with_ram_command_receiver()
            .with_crc_control();
        path.enable_electrical_level_bridge(None);
        path.begin_word(None);
        let mut bus = Hp67ElectricalBackplane::default();
        for bit in 0..56 {
            launch(&mut bus, 0o1000, true);
            bus.advance_clock();
            bus.advance_clock();
            if bit == 55 {
                let before = path;
                bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::High);
                assert!(path.sample_transport_word_bit_at_edge(bit, &bus).is_err());
                assert_eq!(path, before);
                bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::HighZ);
            }
            path.sample_transport_word_bit_at_edge(bit, &bus).unwrap();
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(
            path.crc_control().unwrap().received_frame().unwrap().word,
            0o1000
        );
    }

    #[test]
    fn m15t_crc_consecutive_set_test_clear_and_external_level_are_pipelined() {
        let mut crc = Hp67SerialCrcControl::default();
        let mut bus = Hp67ElectricalBackplane::default();
        word(&mut crc, &mut bus, 0o1000, true);
        word(&mut crc, &mut bus, 0o1100, true);
        assert!(crc.flags[4]);
        word(&mut crc, &mut bus, 0o1100, true);
        assert_eq!(crc.result().unwrap().condition, Some(true));
        assert!(!crc.flags[4]);
        word(&mut crc, &mut bus, 0o1100, true);
        assert_eq!(crc.result().unwrap().condition, Some(false));
        let mut mirror = CrcArchitecturalCore::default();
        mirror.set_external_flag(4, true).unwrap();
        crc.freeze_inputs(&mirror).unwrap();
        word(&mut crc, &mut bus, 0o1100, true);
        assert_eq!(crc.result().unwrap().condition, Some(true));
        word(&mut crc, &mut bus, 0, true);
        assert_eq!(crc.result().unwrap().condition, Some(true));
        assert!(!crc.flags[4] && crc.external[4]);
    }
}

#[cfg(test)]
#[path = "crc_transaction_tests.rs"]
mod transaction_tests;
