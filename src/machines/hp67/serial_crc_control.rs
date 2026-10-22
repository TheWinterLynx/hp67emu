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

    /// Check the mirror before freezing contacts. Never import internal latches.
    pub fn freeze_inputs(&mut self, mirror: &CrcArchitecturalCore) -> Result<(), String> {
        if self.expected_bit != 0 {
            return Err("CRC contacts must freeze before b0".into());
        }
        self.check_mirror(mirror)?;
        for (index, input) in self.external.iter_mut().enumerate() {
            *input = mirror.external_flag(index).expect("CRC contact index");
        }
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
        let bit = bus.word_bit();
        if !bus.at_transport_sample_edge() || bit != self.expected_bit {
            return Err(Hp67CrcControlError::Phase {
                expected: self.expected_bit,
                actual: bit,
                tick: bus.tick().get(),
            });
        }
        // Prepare the small control image: a failed command sample cannot change
        // latches, pending instruction, contacts or the partial receive window.
        let mut next = *self;
        if bit == 0 {
            next.executing = match next.receiver.received_frame() {
                Some(frame) if frame.instruction => decode_crc_opcode(frame.word)
                    .map_err(|_| Hp67CrcControlError::Decode(frame.word))?,
                _ => None,
            };
            next.result = None;
        }
        if bit >= 46 {
            next.receiver
                .sample_at_edge(bus)
                .map_err(Hp67CrcControlError::Command)?;
        }
        if bit == 55 {
            if let Some(instruction) = next.executing {
                let flag = match instruction {
                    CrcInstruction::SetFlag { flag }
                    | CrcInstruction::TestFlagAndClear { flag } => flag,
                };
                let index = usize::from(flag);
                let (value, condition) = match instruction {
                    CrcInstruction::SetFlag { .. } => (true, None),
                    CrcInstruction::TestFlagAndClear { .. } => {
                        (false, Some(next.flags[index] || next.external[index]))
                    }
                };
                next.flags[index] = value;
                next.result = Some(Hp67CrcControlResult {
                    instruction,
                    flag,
                    value,
                    condition,
                });
            }
        }
        next.expected_bit = if bit == 55 { 0 } else { bit + 1 };
        *self = next;
        Ok(())
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
