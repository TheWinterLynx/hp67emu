//! HP-67 DATA phase primitives and optional resolved-net logical-level bridge.
//!
//! Direct HP-67 captures establish a continuous 56-bit DATA stream whose
//! serial bit 0 appears at machine-word bit b2. Bits 54 and 55 therefore appear
//! at b0/b1 of the following machine word. This module models that cross-word
//! phase exactly at the logical-bit level while deliberately leaving electrical
//! polarity, passive bias and physical latch edges unresolved. Production now
//! separates publication from PHI1-rising sampling using a provisional policy,
//! including the genuinely clocked following-word b0/b1 tail. Combined unclocked
//! visitors remain reference adapters and do not establish physical timing.

use crate::emulation::{Drive, LogicLevel};

use super::{
    act::{ActArchitecturalState, ActInstructionState, ActRegister, ACT_WORD_DIGITS},
    machine::Hp67ElectricalBackplane,
    ram_command::{Hp67RamCommandError, Hp67RamCommandFrame, Hp67RamCommandReceiver},
    timing::{data_serial_bit_for_word_bit, BITS_PER_DIGIT, BITS_PER_WORD},
    wiring::{Hp67Driver, Hp67Net},
};

const DATA_TAIL_BITS: u8 = 2;
const DATA_BODY_LAST_WORD_BIT: u8 = BITS_PER_WORD - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67DataTransferDirection {
    ActToPeripheral,
    PeripheralToAct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67DataTransferPlan {
    pub direction: Hp67DataTransferDirection,
    pub address: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67DataSerialError {
    RamCommand(Hp67RamCommandError),
    CrcControl(super::serial_crc_control::Hp67CrcControlError),
    UnexpectedWordBit {
        expected: u8,
        actual: u8,
    },
    MissingLogicalBit {
        word_bit: u8,
        serial_bit: u8,
    },
    UnexpectedLogicalBit {
        word_bit: u8,
        serial_bit: u8,
    },
    OverlappingFrameStart,
    ElectricalBridgeRequired,
    InvalidTransportPhase {
        word_bit: u8,
        tick: u64,
    },
    MissingElectricalOwner {
        word_bit: u8,
    },
    InvalidElectricalOwner {
        word_bit: u8,
        expected: Hp67Driver,
        level: LogicLevel,
    },
    InvalidElectricalSample {
        word_bit: u8,
        level: LogicLevel,
        active_drivers: u8,
    },
}

/// Decode the ACT instructions that exchange one 56-bit register through DATA.
///
/// This is an instruction-semantic classification only. It does not claim DATA
/// electrical polarity or a PHI launch/sample edge.
pub const fn act_data_transfer_plan(
    word: u16,
    state: &ActArchitecturalState,
) -> Option<Hp67DataTransferPlan> {
    if matches!(state.instruction_state, ActInstructionState::ThenGoto) {
        return None;
    }

    let low = word & 0o77;
    let operand = ((word >> 6) & 0x0f) as u8;
    if word == 0o1360 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::ActToPeripheral,
            address: state.ram_address,
        })
    } else if word == 0o0070 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::PeripheralToAct,
            address: state.ram_address,
        })
    } else if low == 0o50 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::ActToPeripheral,
            address: (state.ram_address & 0xf0) | operand,
        })
    } else if low == 0o70 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::PeripheralToAct,
            address: (state.ram_address & 0xf0) | operand,
        })
    } else {
        None
    }
}

pub const fn data_register_serial_bit(register: &ActRegister, serial_bit: u8) -> bool {
    debug_assert!(serial_bit < BITS_PER_WORD);
    let digit = (serial_bit / BITS_PER_DIGIT) as usize;
    let bit_in_digit = serial_bit % BITS_PER_DIGIT;
    ((register[digit] >> bit_in_digit) & 1) != 0
}

fn set_data_register_serial_bit(register: &mut ActRegister, serial_bit: u8, bit: bool) {
    debug_assert!(serial_bit < BITS_PER_WORD);
    let digit = (serial_bit / BITS_PER_DIGIT) as usize;
    let bit_in_digit = serial_bit % BITS_PER_DIGIT;
    let mask = 1u8 << bit_in_digit;
    if bit {
        register[digit] |= mask;
    } else {
        register[digit] &= !mask;
    }
}

/// Continuous DATA source whose b0/b1 output belongs to the frame started in
/// the preceding machine word and whose b2..b55 output belongs to this word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67DataSerialSource {
    previous_tail: Option<[bool; DATA_TAIL_BITS as usize]>,
    current: Option<ActRegister>,
}

impl Hp67DataSerialSource {
    pub fn begin_word(&mut self, current: Option<ActRegister>) {
        self.current = current;
    }

    pub fn logical_bit_for_word_bit(&self, word_bit: u8) -> Option<bool> {
        debug_assert!(word_bit < BITS_PER_WORD);
        match word_bit {
            0 | 1 => self.previous_tail.map(|tail| tail[word_bit as usize]),
            _ => self.current.as_ref().map(|register| {
                data_register_serial_bit(register, data_serial_bit_for_word_bit(word_bit))
            }),
        }
    }

    pub fn complete_word(&mut self) {
        self.previous_tail = self.current.as_ref().map(|register| {
            [
                data_register_serial_bit(register, BITS_PER_WORD - 2),
                data_register_serial_bit(register, BITS_PER_WORD - 1),
            ]
        });
        self.current = None;
    }
}

/// Continuous DATA sink that completes a frame only after consuming b0/b1 of
/// the following machine word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67DataSerialSink {
    expected_word_bit: u8,
    start_frame_at_b2: bool,
    assembling: Option<ActRegister>,
}

impl Default for Hp67DataSerialSink {
    fn default() -> Self {
        Self {
            expected_word_bit: 0,
            start_frame_at_b2: false,
            assembling: None,
        }
    }
}

impl Hp67DataSerialSink {
    pub const fn frame_in_progress(&self) -> bool {
        self.assembling.is_some()
    }

    pub const fn expected_word_bit(&self) -> u8 {
        self.expected_word_bit
    }

    pub fn begin_word(&mut self, start_frame_at_b2: bool) {
        debug_assert!(matches!(self.expected_word_bit, 0 | DATA_TAIL_BITS));
        self.start_frame_at_b2 = start_frame_at_b2;
    }

    pub fn sample_word_bit(
        &mut self,
        word_bit: u8,
        logical_bit: Option<bool>,
    ) -> Result<Option<ActRegister>, Hp67DataSerialError> {
        if word_bit != self.expected_word_bit {
            return Err(Hp67DataSerialError::UnexpectedWordBit {
                expected: self.expected_word_bit,
                actual: word_bit,
            });
        }

        let serial_bit = data_serial_bit_for_word_bit(word_bit);
        let mut completed = None;

        if word_bit == 2 && self.start_frame_at_b2 {
            if self.assembling.is_some() {
                return Err(Hp67DataSerialError::OverlappingFrameStart);
            }
            self.assembling = Some([0; ACT_WORD_DIGITS]);
        }

        match (self.assembling.as_mut(), logical_bit) {
            (Some(register), Some(bit)) => {
                set_data_register_serial_bit(register, serial_bit, bit);
                if word_bit == 1 {
                    completed = self.assembling.take();
                }
            }
            (Some(_), None) => {
                return Err(Hp67DataSerialError::MissingLogicalBit {
                    word_bit,
                    serial_bit,
                })
            }
            (None, Some(_)) => {
                return Err(Hp67DataSerialError::UnexpectedLogicalBit {
                    word_bit,
                    serial_bit,
                })
            }
            (None, None) => {}
        }

        self.expected_word_bit = if word_bit == DATA_BODY_LAST_WORD_BIT {
            0
        } else {
            word_bit + 1
        };
        Ok(completed)
    }
}

/// One DATA phase participant normally advanced from the structural b0..b55
/// loop. M14C can preconsume only the preceding frame's b0/b1 tail so the next
/// instruction sees the reconstructed transfer before its b2 DATA body begins.
///
/// Default callers retain logical-only transport. The live machine opts into
/// resolved DATA through an explicitly approximate logical-level adapter. No
/// physical polarity/output stage or PHI launch/sample edge is claimed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67DataSerialWordPath {
    source: Hp67DataSerialSource,
    sink: Hp67DataSerialSink,
    current_payload: Option<ActRegister>,
    completed_payload: Option<ActRegister>,
    prefix_preconsumed: bool,
    electrical_level_bridge: bool,
    current_direction: Option<Hp67DataTransferDirection>,
    previous_direction: Option<Hp67DataTransferDirection>,
    ram_command_receiver: Option<Hp67RamCommandReceiver>,
    crc_control: Option<super::Hp67SerialCrcControl>,
    autonomous_ram_owner: bool,
    receive_only: bool,
    previous_receive_only: bool,
}

impl Hp67DataSerialWordPath {
    pub fn with_crc_control(mut self) -> Self {
        self.crc_control = Some(super::Hp67SerialCrcControl::default());
        self
    }

    pub fn crc_control(&self) -> Option<&super::Hp67SerialCrcControl> {
        self.crc_control.as_ref()
    }

    pub fn crc_control_mut(&mut self) -> Option<&mut super::Hp67SerialCrcControl> {
        self.crc_control.as_mut()
    }

    pub fn disable_crc_control(&mut self) {
        self.crc_control = None;
    }

    /// Attach the RAM control receiver to the same resolved sampling schedule.
    /// This observes commands only; read sources/storage retain their owners.
    pub fn with_ram_command_receiver(mut self) -> Self {
        self.ram_command_receiver = Some(Hp67RamCommandReceiver::default());
        self
    }

    pub const fn ram_command_receiver_enabled(&self) -> bool {
        self.ram_command_receiver.is_some()
    }

    pub fn received_ram_command(&self) -> Option<Hp67RamCommandFrame> {
        self.ram_command_receiver
            .as_ref()
            .and_then(|receiver| receiver.received_frame())
    }

    pub(crate) fn use_autonomous_ram_owner(&mut self) {
        self.autonomous_ram_owner = true;
    }

    pub const fn frame_in_progress(&self) -> bool {
        self.sink.frame_in_progress()
    }

    /// Opt in to resolved-net transport for this word. Low/High encode logical
    /// bits only: physical DATA polarity/bias/output circuitry remain unknown.
    /// Aggregate RAM ownership is an explicit bridge, not physical chip mapping.
    pub fn enable_electrical_level_bridge(&mut self, direction: Option<Hp67DataTransferDirection>) {
        debug_assert!(self.current_payload.is_none());
        debug_assert!(!self.receive_only);
        self.electrical_level_bridge = true;
        self.current_direction = direction;
    }

    fn direction_for_word_bit(&self, word_bit: u8) -> Option<Hp67DataTransferDirection> {
        if word_bit < DATA_TAIL_BITS {
            self.previous_direction
        } else {
            self.current_direction
        }
    }

    fn level_bridge_drives(&self, word_bit: u8) -> Result<(Drive, Drive), Hp67DataSerialError> {
        let source_bit = self.source.logical_bit_for_word_bit(word_bit);
        let owner = self.direction_for_word_bit(word_bit);
        let (act_drive, ram_drive) = match (source_bit, owner) {
            (Some(bit), Some(direction)) => {
                // Digital logic-level adapter only; does not assert push-pull
                // transistors, passive polarity, enable edge or drive strength.
                let level = if bit { Drive::High } else { Drive::Low };
                match direction {
                    Hp67DataTransferDirection::ActToPeripheral => (level, Drive::HighZ),
                    Hp67DataTransferDirection::PeripheralToAct => (Drive::HighZ, level),
                }
            }
            (Some(_), None) => {
                return Err(Hp67DataSerialError::MissingElectricalOwner { word_bit })
            }
            (None, _) => (Drive::HighZ, Drive::HighZ),
        };
        Ok((act_drive, ram_drive))
    }

    fn drive_level_bridge(
        &self,
        word_bit: u8,
        backplane: &mut Hp67ElectricalBackplane,
    ) -> Result<(), Hp67DataSerialError> {
        let (act_drive, ram_drive) = self.level_bridge_drives(word_bit)?;
        backplane.commit_data_level_bridge(act_drive, ram_drive);
        Ok(())
    }

    fn sample_level_bridge(
        &self,
        word_bit: u8,
        backplane: &Hp67ElectricalBackplane,
    ) -> Result<Option<bool>, Hp67DataSerialError> {
        let receiving = if word_bit < DATA_TAIL_BITS {
            self.previous_receive_only
        } else {
            self.receive_only
        };
        let frame_bit = receiving || self.source.logical_bit_for_word_bit(word_bit).is_some();
        let level = backplane.level(Hp67Net::Data);
        let active_drivers = backplane.active_data_driver_count();
        let expected_drivers = u8::from(frame_bit);
        if active_drivers != expected_drivers {
            return Err(Hp67DataSerialError::InvalidElectricalSample {
                word_bit,
                level,
                active_drivers,
            });
        }
        if frame_bit {
            let expected = match self.direction_for_word_bit(word_bit) {
                Some(Hp67DataTransferDirection::ActToPeripheral) => Hp67Driver::Act1820_2530,
                Some(Hp67DataTransferDirection::PeripheralToAct) => {
                    if self.autonomous_ram_owner {
                        Hp67Driver::RamAggregate
                    } else {
                        Hp67Driver::StructuralRomResponder
                    }
                }
                None => return Err(Hp67DataSerialError::MissingElectricalOwner { word_bit }),
            };
            // Count alone accepts a sole impostor with a perfectly valid level.
            // Check identity, not expected bit value: the sink still consumes
            // the actual resolved DATA sample without republishing anything.
            if backplane.driver_drive(Hp67Net::Data, expected) == Drive::HighZ {
                return Err(Hp67DataSerialError::InvalidElectricalOwner {
                    word_bit,
                    expected,
                    level,
                });
            }
        }
        match (frame_bit, level) {
            (true, LogicLevel::High) => Ok(Some(true)),
            (true, LogicLevel::Low) => Ok(Some(false)),
            (false, LogicLevel::Floating) => Ok(None),
            _ => Err(Hp67DataSerialError::InvalidElectricalSample {
                word_bit,
                level,
                active_drivers,
            }),
        }
    }

    fn drive_and_sample_level_bridge(
        &self,
        word_bit: u8,
        backplane: &mut Hp67ElectricalBackplane,
    ) -> Result<Option<bool>, Hp67DataSerialError> {
        self.drive_level_bridge(word_bit, backplane)?;
        self.sample_level_bridge(word_bit, backplane)
    }

    fn validate_transport_bit(&self, word_bit: u8) -> Result<(), Hp67DataSerialError> {
        if !(self.prefix_preconsumed && word_bit < DATA_TAIL_BITS)
            && word_bit != self.sink.expected_word_bit()
        {
            return Err(Hp67DataSerialError::UnexpectedWordBit {
                expected: self.sink.expected_word_bit(),
                actual: word_bit,
            });
        }
        Ok(())
    }

    /// Compute a proposal from immutable DATA state. Publication belongs to the
    /// shared serial commit, so errors cannot leave IS/SYNC partially changed.
    pub(crate) fn propose_transport_word_bit(
        &self,
        word_bit: u8,
        backplane: &Hp67ElectricalBackplane,
    ) -> Result<Option<(Drive, Drive)>, Hp67DataSerialError> {
        self.validate_transport_bit(word_bit)?;
        if !backplane.at_transport_launch_boundary() || backplane.word_bit() != word_bit {
            return Err(Hp67DataSerialError::InvalidTransportPhase {
                word_bit,
                tick: backplane.tick().get(),
            });
        }
        if self.electrical_level_bridge {
            self.level_bridge_drives(word_bit).map(Some)
        } else {
            Ok(None)
        }
    }

    /// Isolated M15H receiver fixtures; production publishes the shared image.
    #[cfg(test)]
    fn launch_transport_word_bit(
        &self,
        word_bit: u8,
        backplane: &mut Hp67ElectricalBackplane,
    ) -> Result<(), Hp67DataSerialError> {
        if let Some((act, ram)) = self.propose_transport_word_bit(word_bit, backplane)? {
            backplane.commit_data_level_bridge(act, ram);
        }
        Ok(())
    }

    /// Sample existing resolved DATA without republishing the source. Clock
    /// phase failures occur before sink mutation, including on following-word tails.
    pub(crate) fn sample_transport_word_bit_at_edge(
        &mut self,
        word_bit: u8,
        backplane: &Hp67ElectricalBackplane,
    ) -> Result<(), Hp67DataSerialError> {
        self.validate_transport_bit(word_bit)?;
        if !backplane.at_transport_sample_edge() || backplane.word_bit() != word_bit {
            return Err(Hp67DataSerialError::InvalidTransportPhase {
                word_bit,
                tick: backplane.tick().get(),
            });
        }
        let crc_sample = self.crc_control.as_ref()
            .map(|crc| crc.prepare_sample(backplane))
            .transpose()
            .map_err(Hp67DataSerialError::CrcControl)?;
        let command_receiver = if (super::timing::ROM_WORD_FIRST_BIT..=super::timing::ROM_WORD_LAST_BIT).contains(&word_bit) {
            self.ram_command_receiver.map(|mut receiver| {
                receiver.sample_at_edge(backplane)
                    .map(|()| receiver)
                    .map_err(Hp67DataSerialError::RamCommand)
            }).transpose()?
        } else {
            None
        };
        if !self.electrical_level_bridge {
            self.visit_transport_word_bit(word_bit)?;
            if let Some(receiver) = command_receiver {
                self.ram_command_receiver = Some(receiver);
            }
            if let Some(sample) = crc_sample {
                self.crc_control.as_mut().expect("prepared CRC endpoint").commit_sample(sample);
            }
            return Ok(());
        }
        let bit = self.sample_level_bridge(word_bit, backplane)?;
        if self.prefix_preconsumed && word_bit < DATA_TAIL_BITS {
            return Ok(());
        }
        let completed = self.sink.sample_word_bit(word_bit, bit)?;
        if completed.is_some() {
            debug_assert!(self.completed_payload.is_none());
            self.completed_payload = completed;
        }
        if let Some(receiver) = command_receiver {
            self.ram_command_receiver = Some(receiver);
        }
        if let Some(sample) = crc_sample {
            self.crc_control.as_mut().expect("prepared CRC endpoint").commit_sample(sample);
        }
        Ok(())
    }

    /// Existing b0/b1 scheduling bridge, now reconstructed through resolved DATA.
    /// Time is still not advanced here; the later structural loop replays pins
    /// while avoiding a second sink commit. This is not PHI-relative timing.
    pub fn preconsume_previous_tail_through_net(
        &mut self,
        backplane: &mut Hp67ElectricalBackplane,
    ) -> Result<Option<ActRegister>, Hp67DataSerialError> {
        if !self.electrical_level_bridge {
            return self.preconsume_previous_tail();
        }
        if !self.sink.frame_in_progress() {
            return Ok(None);
        }
        let mut completed = None;
        for word_bit in 0..DATA_TAIL_BITS {
            let bit = self.drive_and_sample_level_bridge(word_bit, backplane)?;
            let next = self.sink.sample_word_bit(word_bit, bit)?;
            if next.is_some() {
                completed = next;
            }
        }
        self.prefix_preconsumed = true;
        Ok(completed)
    }

    pub fn visit_transport_word_bit_through_net(
        &mut self,
        word_bit: u8,
        backplane: &mut Hp67ElectricalBackplane,
    ) -> Result<(), Hp67DataSerialError> {
        if !self.electrical_level_bridge {
            return self.visit_transport_word_bit(word_bit);
        }
        if !(self.prefix_preconsumed && word_bit < DATA_TAIL_BITS)
            && word_bit != self.sink.expected_word_bit()
        {
            return Err(Hp67DataSerialError::UnexpectedWordBit {
                expected: self.sink.expected_word_bit(),
                actual: word_bit,
            });
        }
        let bit = self.drive_and_sample_level_bridge(word_bit, backplane)?;
        if self.prefix_preconsumed && word_bit < DATA_TAIL_BITS {
            return Ok(());
        }
        let completed = self.sink.sample_word_bit(word_bit, bit)?;
        if completed.is_some() {
            debug_assert!(self.completed_payload.is_none());
            self.completed_payload = completed;
        }
        Ok(())
    }

    pub(crate) const fn electrical_level_bridge_enabled(&self) -> bool {
        self.electrical_level_bridge
    }

    /// Finish a genuinely clocked b0/b1 prefix without replaying its sink later.
    pub(crate) fn take_clocked_prefix_result(&mut self) -> Option<ActRegister> {
        debug_assert_eq!(self.sink.expected_word_bit(), DATA_TAIL_BITS);
        self.prefix_preconsumed = true;
        self.completed_payload.take()
    }

    pub const fn prefix_preconsumed(&self) -> bool {
        self.prefix_preconsumed
    }

    /// Consume b0/b1 of a frame started in the preceding word before the
    /// instruction-boundary bridge for the new word runs.
    ///
    /// This preserves the directly observed DATA phase while allowing the live
    /// machine to make the reconstructed transfer visible to the next
    /// instruction before that instruction can depend on C or RAM. It is a
    /// logical scheduler split only: no PHI-relative sample/commit edge is
    /// asserted here.
    pub fn preconsume_previous_tail(&mut self) -> Result<Option<ActRegister>, Hp67DataSerialError> {
        if !self.sink.frame_in_progress() {
            return Ok(None);
        }
        debug_assert!(self.current_payload.is_none());
        debug_assert!(self.completed_payload.is_none());
        debug_assert!(!self.prefix_preconsumed);

        let mut completed = None;
        for word_bit in 0..DATA_TAIL_BITS {
            let next = self
                .sink
                .sample_word_bit(word_bit, self.source.logical_bit_for_word_bit(word_bit))?;
            if next.is_some() {
                debug_assert!(completed.is_none());
                completed = next;
            }
        }
        self.prefix_preconsumed = true;
        Ok(completed)
    }

    pub fn begin_word(&mut self, payload: Option<ActRegister>) {
        debug_assert!(self.completed_payload.is_none());
        debug_assert!(self.current_payload.is_none());
        debug_assert!(matches!(self.sink.expected_word_bit(), 0 | DATA_TAIL_BITS));
        self.receive_only = false;
        self.current_payload = payload;
        self.source.begin_word(payload);
        self.sink.begin_word(payload.is_some());
    }

    /// Autonomous reads bind a sink, with no local payload/source or tail bits.
    /// The selected RAM participant is the sole source at joint publication.
    pub(crate) fn begin_autonomous_word(&mut self, payload: Option<ActRegister>) {
        self.use_autonomous_ram_owner();
        let receiving = self.current_direction == Some(Hp67DataTransferDirection::PeripheralToAct);
        self.begin_word(if receiving { None } else { payload });
        if receiving {
            self.receive_only = true;
            self.sink.begin_word(true);
        }
    }

    pub fn visit_word_bit(&mut self, word_bit: u8) -> Result<(), Hp67DataSerialError> {
        let completed = self
            .sink
            .sample_word_bit(word_bit, self.source.logical_bit_for_word_bit(word_bit))?;
        if completed.is_some() {
            debug_assert!(self.completed_payload.is_none());
            self.completed_payload = completed;
        }
        Ok(())
    }

    pub fn visit_transport_word_bit(&mut self, word_bit: u8) -> Result<(), Hp67DataSerialError> {
        if self.prefix_preconsumed && word_bit < DATA_TAIL_BITS {
            return Ok(());
        }
        self.visit_word_bit(word_bit)
    }

    pub fn complete_word(&mut self) -> Option<ActRegister> {
        self.previous_direction = if self.receive_only {
            self.current_direction
        } else {
            self.current_payload.and(self.current_direction)
        };
        self.previous_receive_only = self.receive_only;
        self.receive_only = false;
        self.current_direction = None;
        self.source.complete_word();
        self.current_payload = None;
        self.prefix_preconsumed = false;
        self.completed_payload.take()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterned_register(seed: u8) -> ActRegister {
        std::array::from_fn(|digit| (seed.wrapping_add(digit as u8 * 3)) & 0x0f)
    }

    #[test]
    fn m15s_receive_only_data_has_no_source_and_hands_off_to_act_write() {
        use super::super::rom_devices::RomIsDrives;
        let read: ActRegister = std::array::from_fn(|digit| (digit as u8 * 3 + 1) & 15);
        let write = [7; 14];
        let mut path = Hp67DataSerialWordPath::default();
        let mut bus = Hp67ElectricalBackplane::default();
        for word in 0..3 {
            path.enable_electrical_level_bridge(match word {
                0 => Some(Hp67DataTransferDirection::PeripheralToAct),
                1 => Some(Hp67DataTransferDirection::ActToPeripheral),
                _ => None,
            });
            path.begin_autonomous_word(if word == 1 { Some(write) } else { None });
            if word == 0 {
                assert!(path.current_payload.is_none());
                assert!(path.source.current.is_none());
            }
            for bit in 0..BITS_PER_WORD {
                let (act, proposed_ram) =
                    path.propose_transport_word_bit(bit, &bus).unwrap().unwrap();
                let receiving = (word == 0 && bit >= 2) || (word == 1 && bit < 2);
                let ram = if receiving {
                    assert_eq!((act, proposed_ram), (Drive::HighZ, Drive::HighZ));
                    assert!(path.source.logical_bit_for_word_bit(bit).is_none());
                    let serial = if bit < 2 { bit + 54 } else { bit - 2 };
                    if data_register_serial_bit(&read, serial) {
                        Drive::High
                    } else {
                        Drive::Low
                    }
                } else {
                    Drive::HighZ
                };
                bus.commit_autonomous_ram_launch(
                    Drive::HighZ,
                    RomIsDrives::Aggregate(Drive::HighZ),
                    Drive::Low,
                    Some((act, ram)),
                );
                bus.advance_clock();
                bus.advance_clock();
                if receiving {
                    let before = path;
                    let mut missing = bus.clone();
                    missing.drive(Hp67Net::Data, Hp67Driver::RamAggregate, Drive::HighZ);
                    assert!(path
                        .sample_transport_word_bit_at_edge(bit, &missing)
                        .is_err());
                    assert_eq!(path, before);
                }
                path.sample_transport_word_bit_at_edge(bit, &bus).unwrap();
                bus.advance_clock();
                bus.advance_clock();
            }
            assert_eq!(
                path.complete_word(),
                match word {
                    0 => None,
                    1 => Some(read),
                    _ => Some(write),
                }
            );
            if word == 0 {
                assert!(path.source.previous_tail.is_none());
            }
        }
        assert_eq!(bus.tick().get(), 672);
        assert_eq!(bus.level(Hp67Net::Data), LogicLevel::Floating);
    }

    #[test]
    fn m15r_autonomous_ram_identity_is_required_for_read_body_and_tail() {
        use super::super::rom_devices::RomIsDrives;
        for seed in [0, 15] {
            let mut path = Hp67DataSerialWordPath::default();
            path.use_autonomous_ram_owner();
            let mut bus = Hp67ElectricalBackplane::default();
            for word in 0..2 {
                path.enable_electrical_level_bridge(if word == 0 {
                    Some(Hp67DataTransferDirection::PeripheralToAct)
                } else {
                    None
                });
                path.begin_word(if word == 0 { Some([seed; 14]) } else { None });
                for bit in 0..BITS_PER_WORD {
                    let drives = path.propose_transport_word_bit(bit, &bus).unwrap();
                    bus.commit_autonomous_ram_launch(
                        Drive::HighZ,
                        RomIsDrives::Aggregate(Drive::HighZ),
                        Drive::Low,
                        drives,
                    );
                    bus.advance_clock();
                    bus.advance_clock();
                    if (word == 0 && bit >= 2) || (word == 1 && bit < 2) {
                        let before = path;
                        let drive = bus.driver_drive(Hp67Net::Data, Hp67Driver::RamAggregate);
                        for wrong in Hp67Driver::ALL {
                            if wrong == Hp67Driver::RamAggregate {
                                continue;
                            }
                            let mut broken = bus.clone();
                            broken.drive(Hp67Net::Data, Hp67Driver::RamAggregate, Drive::HighZ);
                            broken.drive(Hp67Net::Data, wrong, drive);
                            assert_eq!(broken.level(Hp67Net::Data), bus.level(Hp67Net::Data));
                            assert!(matches!(
                                path.sample_transport_word_bit_at_edge(bit, &broken),
                                Err(Hp67DataSerialError::InvalidElectricalOwner {
                                    expected: Hp67Driver::RamAggregate,
                                    ..
                                })
                            ));
                            assert_eq!(path, before);
                        }
                        let mut overlap = bus.clone();
                        overlap.drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder, drive);
                        assert!(path
                            .sample_transport_word_bit_at_edge(bit, &overlap)
                            .is_err());
                        assert_eq!(path, before);
                    }
                    path.sample_transport_word_bit_at_edge(bit, &bus).unwrap();
                    bus.advance_clock();
                    bus.advance_clock();
                }
                assert_eq!(
                    path.complete_word(),
                    if word == 0 { None } else { Some([seed; 14]) }
                );
            }
            assert_eq!(bus.tick().get(), 448);
            assert_eq!(bus.level(Hp67Net::Data), LogicLevel::Floating);
        }
    }

    #[test]
    fn m15p_invalid_ram_command_cannot_consume_a_data_bit() {
        use super::super::{isa::rom_word_drive, rom_devices::RomIsDrives};
        let mut path = Hp67DataSerialWordPath::default().with_ram_command_receiver();
        let mut bus = Hp67ElectricalBackplane::default();
        path.enable_electrical_level_bridge(Some(Hp67DataTransferDirection::ActToPeripheral));
        path.begin_word(Some([7; ACT_WORD_DIGITS]));
        for bit in 0..BITS_PER_WORD {
            let drives = path.propose_transport_word_bit(bit, &bus).unwrap();
            bus.commit_serial_launch(
                Drive::HighZ,
                RomIsDrives::Aggregate(rom_word_drive(0x270, bit)),
                if bit >= 46 { Drive::High } else { Drive::Low },
                drives,
            );
            bus.advance_clock();
            bus.advance_clock();
            if bit >= 46 {
                let before = path;
                let mut broken = bus.clone();
                broken.drive(Hp67Net::Sync, Hp67Driver::External, Drive::Low);
                assert!(matches!(
                    path.sample_transport_word_bit_at_edge(bit, &broken),
                    Err(Hp67DataSerialError::RamCommand(_))
                ));
                assert_eq!(path, before);
            }
            path.sample_transport_word_bit_at_edge(bit, &bus).unwrap();
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(path.received_ram_command().unwrap().word, 0x270);
        assert_eq!(bus.tick().get(), 224);
    }

    #[test]
    fn m15l_data_rejects_every_sole_wrong_driver_across_bodies_and_tails() {
        for seed in [0, 15] {
            let mut path = Hp67DataSerialWordPath::default();
            let mut backplane = Hp67ElectricalBackplane::default();
            for (word, direction, payload) in [
                (
                    0,
                    Some(Hp67DataTransferDirection::ActToPeripheral),
                    Some([seed; ACT_WORD_DIGITS]),
                ),
                (
                    1,
                    Some(Hp67DataTransferDirection::PeripheralToAct),
                    Some([seed; ACT_WORD_DIGITS]),
                ),
                (2, None, None),
            ] {
                path.enable_electrical_level_bridge(direction);
                path.begin_word(payload);
                for bit in 0..BITS_PER_WORD {
                    path.launch_transport_word_bit(bit, &mut backplane).unwrap();
                    backplane.advance_clock();
                    backplane.advance_clock();
                    // Independent fixture roles: write body/tail, read body/tail.
                    let expected = match (word, bit) {
                        (0, 2..=55) | (1, 0..=1) => Some(Hp67Driver::Act1820_2530),
                        (1, 2..=55) | (2, 0..=1) => Some(Hp67Driver::StructuralRomResponder),
                        _ => None,
                    };
                    if let Some(expected) = expected {
                        let before = path;
                        let level = backplane.level(Hp67Net::Data);
                        let drive = backplane.driver_drive(Hp67Net::Data, expected);
                        for wrong in Hp67Driver::ALL {
                            if wrong == expected {
                                continue;
                            }
                            let mut faulted = backplane.clone();
                            faulted.drive(Hp67Net::Data, expected, Drive::HighZ);
                            faulted.drive(Hp67Net::Data, wrong, drive);
                            assert_eq!(faulted.active_data_driver_count(), 1);
                            assert_eq!(faulted.level(Hp67Net::Data), level);
                            assert_eq!(
                                path.sample_transport_word_bit_at_edge(bit, &faulted),
                                Err(Hp67DataSerialError::InvalidElectricalOwner {
                                    word_bit: bit,
                                    expected,
                                    level
                                })
                            );
                            assert_eq!(path, before);
                            assert_eq!(faulted.tick(), backplane.tick());
                        }
                    } else {
                        let before = path;
                        for wrong in Hp67Driver::ALL {
                            let mut faulted = backplane.clone();
                            let drive = if seed == 0 { Drive::Low } else { Drive::High };
                            let level = if seed == 0 {
                                LogicLevel::Low
                            } else {
                                LogicLevel::High
                            };
                            faulted.drive(Hp67Net::Data, wrong, drive);
                            assert_eq!(
                                path.sample_transport_word_bit_at_edge(bit, &faulted),
                                Err(Hp67DataSerialError::InvalidElectricalSample {
                                    word_bit: bit,
                                    level,
                                    active_drivers: 1
                                })
                            );
                            assert_eq!(path, before);
                        }
                    }
                    path.sample_transport_word_bit_at_edge(bit, &backplane)
                        .unwrap();
                    backplane.advance_clock();
                    backplane.advance_clock();
                }
                assert_eq!(
                    path.complete_word(),
                    if word == 0 {
                        None
                    } else {
                        Some([seed; ACT_WORD_DIGITS])
                    }
                );
            }
            assert_eq!(backplane.tick().get(), 3 * 224);
        }
    }

    #[test]
    fn m15l_valid_data_owner_delivers_observed_bits_in_both_directions_and_tails() {
        let mut path = Hp67DataSerialWordPath::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        for (word, direction, payload) in [
            (
                0,
                Some(Hp67DataTransferDirection::ActToPeripheral),
                Some([0; ACT_WORD_DIGITS]),
            ),
            (
                1,
                Some(Hp67DataTransferDirection::PeripheralToAct),
                Some([0; ACT_WORD_DIGITS]),
            ),
            (2, None, None),
        ] {
            path.enable_electrical_level_bridge(direction);
            path.begin_word(payload);
            for bit in 0..BITS_PER_WORD {
                path.launch_transport_word_bit(bit, &mut backplane).unwrap();
                backplane.advance_clock();
                let expected = match (word, bit) {
                    (0, 2..=55) | (1, 0..=1) => Some(Hp67Driver::Act1820_2530),
                    (1, 2..=55) | (2, 0..=1) => Some(Hp67Driver::StructuralRomResponder),
                    _ => None,
                };
                if let Some(owner) = expected {
                    // Same owner, changed committed bit: zero source payload
                    // must reconstruct as ones from the real sampled net.
                    backplane.drive(Hp67Net::Data, owner, Drive::High);
                }
                backplane.advance_clock();
                path.sample_transport_word_bit_at_edge(bit, &backplane)
                    .unwrap();
                backplane.advance_clock();
                backplane.advance_clock();
            }
            assert_eq!(
                path.complete_word(),
                if word == 0 {
                    None
                } else {
                    Some([15; ACT_WORD_DIGITS])
                }
            );
        }
        assert_eq!(backplane.tick().get(), 3 * 224);
    }

    #[test]
    fn m15h_data_waits_for_phi1_rising_across_direction_and_tail_boundaries() {
        let mut path = Hp67DataSerialWordPath::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        let a = patterned_register(3);
        let b = patterned_register(8);
        for (direction, payload, completed) in [
            (
                Some(Hp67DataTransferDirection::ActToPeripheral),
                Some(a),
                None,
            ),
            (
                Some(Hp67DataTransferDirection::PeripheralToAct),
                Some(b),
                Some(a),
            ),
            (None, None, Some(b)),
        ] {
            path.enable_electrical_level_bridge(direction);
            path.begin_word(payload);
            for bit in 0..BITS_PER_WORD {
                let before = path.sink;
                path.launch_transport_word_bit(bit, &mut backplane).unwrap();
                assert_eq!(path.sink, before);
                assert!(matches!(
                    path.sample_transport_word_bit_at_edge(bit, &backplane),
                    Err(Hp67DataSerialError::InvalidTransportPhase { .. })
                ));
                assert_eq!(path.sink, before);
                backplane.advance_clock();
                assert!(matches!(
                    path.sample_transport_word_bit_at_edge(bit, &backplane),
                    Err(Hp67DataSerialError::InvalidTransportPhase { .. })
                ));
                assert_eq!(path.sink, before);
                let (_, edge) = backplane.advance_clock_edge();
                assert_eq!(
                    edge,
                    super::super::timing::HP67_WORKING_TRANSPORT_SAMPLE_EDGE
                );
                path.sample_transport_word_bit_at_edge(bit, &backplane)
                    .unwrap();
                assert_eq!(path.sink.expected_word_bit(), (bit + 1) % BITS_PER_WORD);
                let sampled = path.sink;
                backplane.advance_clock();
                assert!(matches!(
                    path.sample_transport_word_bit_at_edge((bit + 1) % BITS_PER_WORD, &backplane),
                    Err(Hp67DataSerialError::InvalidTransportPhase { .. })
                ));
                assert_eq!(path.sink, sampled);
                backplane.advance_clock();
            }
            assert_eq!(path.complete_word(), completed);
        }
        assert_eq!(backplane.tick().get(), 3 * 224);
    }

    #[test]
    fn m15h_data_samples_resolved_pin_without_republishing_at_receive_edge() {
        use super::super::wiring::Hp67Driver;
        for extra in [None, Some(Drive::High), Some(Drive::Low)] {
            let mut path = Hp67DataSerialWordPath::default();
            let mut backplane = Hp67ElectricalBackplane::default();
            let mut payload = [0; ACT_WORD_DIGITS];
            payload[0] = 1;
            path.enable_electrical_level_bridge(Some(Hp67DataTransferDirection::ActToPeripheral));
            path.begin_word(Some(payload));
            for bit in 0..2 {
                path.launch_transport_word_bit(bit, &mut backplane).unwrap();
                backplane.advance_clock();
                backplane.advance_clock();
                path.sample_transport_word_bit_at_edge(bit, &backplane)
                    .unwrap();
                backplane.advance_clock();
                backplane.advance_clock();
            }
            path.launch_transport_word_bit(2, &mut backplane).unwrap();
            assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::High);
            let before = path.sink;
            backplane.advance_clock();
            if let Some(drive) = extra {
                backplane.drive(Hp67Net::Data, Hp67Driver::External, drive);
            } else {
                // Stimulus changes the committed source contribution after launch.
                // A receiver that republishes its expected bit would hide this.
                backplane.drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::Low);
            }
            backplane.advance_clock();
            let result = path.sample_transport_word_bit_at_edge(2, &backplane);
            if extra.is_some() {
                assert!(matches!(
                    result,
                    Err(Hp67DataSerialError::InvalidElectricalSample {
                        word_bit: 2,
                        active_drivers: 2,
                        ..
                    })
                ));
                assert_eq!(path.sink, before);
            } else {
                result.unwrap();
                assert_eq!(path.sink.assembling.unwrap()[0] & 1, 0);
                assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::Low);
            }
            assert_eq!(backplane.tick().get(), 10);
        }
    }

    #[test]
    fn m15e_resolved_data_matches_logical_frames_through_direction_handoffs() {
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut wired = Hp67DataSerialWordPath::default();
        let mut logical = Hp67DataSerialWordPath::default();
        let a = patterned_register(1);
        let b = patterned_register(9);
        for (direction, payload, completed) in [
            (
                Some(Hp67DataTransferDirection::ActToPeripheral),
                Some(a),
                None,
            ),
            (
                Some(Hp67DataTransferDirection::PeripheralToAct),
                Some(b),
                Some(a),
            ),
            (None, None, Some(b)),
        ] {
            wired.enable_electrical_level_bridge(direction);
            wired.begin_word(payload);
            logical.begin_word(payload);
            for bit in 0..BITS_PER_WORD {
                wired
                    .visit_transport_word_bit_through_net(bit, &mut backplane)
                    .unwrap();
                logical.visit_transport_word_bit(bit).unwrap();
                assert_eq!(wired.sink, logical.sink);
                let active = wired.source.logical_bit_for_word_bit(bit).is_some();
                assert_eq!(backplane.active_data_driver_count(), u8::from(active));
                if !active {
                    assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::Floating);
                }
                for _ in 0..4 {
                    backplane.advance_clock();
                }
            }
            assert_eq!(wired.complete_word(), completed);
            assert_eq!(logical.complete_word(), completed);
        }
        assert_eq!(backplane.tick().get(), 3 * 224);
        assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::Floating);
    }

    #[test]
    fn m15e_preconsumed_tail_uses_net_and_pin_replay_does_not_duplicate_commit() {
        let payload = patterned_register(3);
        let mut path = Hp67DataSerialWordPath::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        path.enable_electrical_level_bridge(Some(Hp67DataTransferDirection::PeripheralToAct));
        path.begin_word(Some(payload));
        for bit in 0..BITS_PER_WORD {
            path.visit_transport_word_bit_through_net(bit, &mut backplane)
                .unwrap();
            for _ in 0..4 {
                backplane.advance_clock();
            }
        }
        assert_eq!(path.complete_word(), None);
        let tick = backplane.tick();
        assert_eq!(
            path.preconsume_previous_tail_through_net(&mut backplane)
                .unwrap(),
            Some(payload)
        );
        assert_eq!(backplane.tick(), tick);
        assert!(!path.frame_in_progress());
        path.enable_electrical_level_bridge(None);
        path.begin_word(None);
        for bit in 0..BITS_PER_WORD {
            path.visit_transport_word_bit_through_net(bit, &mut backplane)
                .unwrap();
            for _ in 0..4 {
                backplane.advance_clock();
            }
        }
        assert_eq!(path.complete_word(), None);
        assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(backplane.tick().get(), 2 * 224);
    }

    #[test]
    fn m15e_data_rejects_same_level_overlap_and_contention_before_sink_mutation() {
        use super::super::wiring::Hp67Driver;
        for extra in [Drive::High, Drive::Low] {
            let mut path = Hp67DataSerialWordPath::default();
            let mut backplane = Hp67ElectricalBackplane::default();
            let mut payload = [0; ACT_WORD_DIGITS];
            payload[0] = 1;
            path.enable_electrical_level_bridge(Some(Hp67DataTransferDirection::ActToPeripheral));
            path.begin_word(Some(payload));
            for bit in 0..2 {
                path.visit_transport_word_bit_through_net(bit, &mut backplane)
                    .unwrap();
            }
            backplane.drive(Hp67Net::Data, Hp67Driver::External, extra);
            let before = path.sink;
            let level = if extra == Drive::High {
                LogicLevel::High
            } else {
                LogicLevel::Contention
            };
            assert_eq!(
                path.visit_transport_word_bit_through_net(2, &mut backplane),
                Err(Hp67DataSerialError::InvalidElectricalSample {
                    word_bit: 2,
                    level,
                    active_drivers: 2
                })
            );
            assert_eq!(path.sink, before);
            assert_eq!(backplane.tick().get(), 0);
        }
    }

    #[test]
    fn m15e_out_of_order_visit_does_not_change_pins_or_sink() {
        let mut path = Hp67DataSerialWordPath::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        path.enable_electrical_level_bridge(Some(Hp67DataTransferDirection::ActToPeripheral));
        path.begin_word(Some([1; ACT_WORD_DIGITS]));
        let sink = path.sink;
        assert_eq!(
            path.visit_transport_word_bit_through_net(56, &mut backplane),
            Err(Hp67DataSerialError::UnexpectedWordBit {
                expected: 0,
                actual: 56,
            })
        );
        assert_eq!(path.sink, sink);
        assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(backplane.tick().get(), 0);
    }

    #[test]
    fn m15e_data_requires_bound_direction_without_guessing_chip_owner() {
        let mut path = Hp67DataSerialWordPath::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        path.enable_electrical_level_bridge(None);
        path.begin_word(Some([1; ACT_WORD_DIGITS]));
        for bit in 0..2 {
            path.visit_transport_word_bit_through_net(bit, &mut backplane)
                .unwrap();
        }
        assert_eq!(
            path.visit_transport_word_bit_through_net(2, &mut backplane),
            Err(Hp67DataSerialError::MissingElectricalOwner { word_bit: 2 })
        );
    }

    #[test]
    fn register_bits_are_digit_lsb_first() {
        let mut register = [0; ACT_WORD_DIGITS];
        register[0] = 0b1010;
        register[1] = 0b0101;

        assert!(!data_register_serial_bit(&register, 0));
        assert!(data_register_serial_bit(&register, 1));
        assert!(!data_register_serial_bit(&register, 2));
        assert!(data_register_serial_bit(&register, 3));
        assert!(data_register_serial_bit(&register, 4));
        assert!(!data_register_serial_bit(&register, 5));
    }

    #[test]
    fn source_keeps_previous_bits_54_55_at_next_word_b0_b1() {
        let first = patterned_register(1);
        let second = patterned_register(7);
        let mut source = Hp67DataSerialSource::default();

        source.begin_word(Some(first));
        assert_eq!(
            source.logical_bit_for_word_bit(2),
            Some(data_register_serial_bit(&first, 0))
        );
        source.complete_word();

        source.begin_word(Some(second));
        assert_eq!(
            source.logical_bit_for_word_bit(0),
            Some(data_register_serial_bit(&first, 54))
        );
        assert_eq!(
            source.logical_bit_for_word_bit(1),
            Some(data_register_serial_bit(&first, 55))
        );
        assert_eq!(
            source.logical_bit_for_word_bit(2),
            Some(data_register_serial_bit(&second, 0))
        );
    }

    #[test]
    fn sink_reconstructs_back_to_back_frames_across_word_boundaries() {
        let first = patterned_register(2);
        let second = patterned_register(9);
        let mut source = Hp67DataSerialSource::default();
        let mut sink = Hp67DataSerialSink::default();

        source.begin_word(Some(first));
        sink.begin_word(true);
        for word_bit in 0..BITS_PER_WORD {
            assert_eq!(
                sink.sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                    .unwrap(),
                None
            );
        }
        source.complete_word();

        source.begin_word(Some(second));
        sink.begin_word(true);
        let mut first_completed = None;
        for word_bit in 0..BITS_PER_WORD {
            let completed = sink
                .sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                .unwrap();
            if completed.is_some() {
                assert_eq!(word_bit, 1);
                first_completed = completed;
            }
        }
        source.complete_word();
        assert_eq!(first_completed, Some(first));

        source.begin_word(None);
        sink.begin_word(false);
        let mut second_completed = None;
        for word_bit in 0..BITS_PER_WORD {
            let completed = sink
                .sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                .unwrap();
            if completed.is_some() {
                assert_eq!(word_bit, 1);
                second_completed = completed;
            }
        }
        assert_eq!(second_completed, Some(second));
    }

    #[test]
    fn word_path_completes_previous_frame_and_starts_next_in_same_word() {
        let first = patterned_register(4);
        let second = patterned_register(11);
        let mut path = Hp67DataSerialWordPath::default();

        path.begin_word(Some(first));
        for word_bit in 0..BITS_PER_WORD {
            path.visit_word_bit(word_bit).unwrap();
        }
        assert_eq!(path.complete_word(), None);
        assert!(path.frame_in_progress());

        path.begin_word(Some(second));
        for word_bit in 0..BITS_PER_WORD {
            path.visit_word_bit(word_bit).unwrap();
        }
        assert_eq!(path.complete_word(), Some(first));
        assert!(path.frame_in_progress());

        path.begin_word(None);
        for word_bit in 0..BITS_PER_WORD {
            path.visit_word_bit(word_bit).unwrap();
        }
        assert_eq!(path.complete_word(), Some(second));
        assert!(!path.frame_in_progress());
    }

    #[test]
    fn preconsumed_tail_can_commit_before_a_back_to_back_frame_starts_at_b2() {
        let first = patterned_register(4);
        let second = patterned_register(11);
        let mut path = Hp67DataSerialWordPath::default();

        path.begin_word(Some(first));
        for word_bit in 0..BITS_PER_WORD {
            path.visit_word_bit(word_bit).unwrap();
        }
        assert_eq!(path.complete_word(), None);
        assert!(path.frame_in_progress());

        assert_eq!(path.preconsume_previous_tail().unwrap(), Some(first));
        assert!(path.prefix_preconsumed());
        assert!(!path.frame_in_progress());

        path.begin_word(Some(second));
        for word_bit in 0..BITS_PER_WORD {
            path.visit_transport_word_bit(word_bit).unwrap();
        }
        assert_eq!(path.complete_word(), None);
        assert!(path.frame_in_progress());

        assert_eq!(path.preconsume_previous_tail().unwrap(), Some(second));
        path.begin_word(None);
        for word_bit in 0..BITS_PER_WORD {
            path.visit_transport_word_bit(word_bit).unwrap();
        }
        assert_eq!(path.complete_word(), None);
        assert!(!path.frame_in_progress());
        assert!(!path.prefix_preconsumed());
    }

    #[test]
    fn transfer_plan_matches_ram_read_write_address_semantics() {
        let mut state = ActArchitecturalState::default();
        state.ram_address = 0x27;
        assert_eq!(
            act_data_transfer_plan(0o1360, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::ActToPeripheral,
                address: 0x27,
            })
        );
        assert_eq!(
            act_data_transfer_plan(0o0070, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::PeripheralToAct,
                address: 0x27,
            })
        );
        state.ram_address = 0x20;
        assert_eq!(
            act_data_transfer_plan((3 << 6) | 0o50, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::ActToPeripheral,
                address: 0x23,
            })
        );
        assert_eq!(
            act_data_transfer_plan((5 << 6) | 0o70, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::PeripheralToAct,
                address: 0x25,
            })
        );
        assert_eq!(act_data_transfer_plan(0o0000, &state), None);
    }

    #[test]
    fn then_goto_payload_is_never_misclassified_as_a_data_transfer() {
        let mut state = ActArchitecturalState {
            instruction_state: ActInstructionState::ThenGoto,
            ram_address: 0x20,
            ..ActArchitecturalState::default()
        };

        assert_eq!(act_data_transfer_plan(0o1360, &state), None);
        assert_eq!(act_data_transfer_plan((3 << 6) | 0o50, &state), None);
        assert_eq!(act_data_transfer_plan((5 << 6) | 0o70, &state), None);

        state.instruction_state = ActInstructionState::Normal;
        assert!(act_data_transfer_plan(0o1360, &state).is_some());
    }

    #[test]
    fn missing_or_unexpected_logical_data_is_rejected() {
        let mut sink = Hp67DataSerialSink::default();
        sink.begin_word(true);
        assert_eq!(
            sink.sample_word_bit(0, Some(false)),
            Err(Hp67DataSerialError::UnexpectedLogicalBit {
                word_bit: 0,
                serial_bit: 54,
            })
        );

        let mut sink = Hp67DataSerialSink::default();
        sink.begin_word(true);
        assert_eq!(sink.sample_word_bit(0, None), Ok(None));
        assert_eq!(sink.sample_word_bit(1, None), Ok(None));
        assert_eq!(
            sink.sample_word_bit(2, None),
            Err(Hp67DataSerialError::MissingLogicalBit {
                word_bit: 2,
                serial_bit: 0,
            })
        );
    }
}
