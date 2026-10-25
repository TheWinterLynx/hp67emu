//! RAM control reception from the resolved IS/SYNC instruction window.
//!
//! No opcode or ACT state enters the receiver. Sampling uses the existing
//! provisional PHI checkpoint; this does not establish physical RAM latch edges.

use super::{
    machine::Hp67ElectricalBackplane,
    timing::{ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT},
    wiring::Hp67Net,
};
use crate::emulation::LogicLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67RamCommand {
    Idle,
    Address,
    Write(Option<u8>),
    Read(Option<u8>),
    Clear,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67RamCommandFrame {
    pub word: u16,
    pub instruction: bool,
}

impl Hp67RamCommandFrame {
    pub const fn command(self) -> Hp67RamCommand {
        if !self.instruction {
            return Hp67RamCommand::Idle;
        }
        match self.word {
            0o1160 => Hp67RamCommand::Address,
            0o1260 => Hp67RamCommand::Clear,
            0o1360 => Hp67RamCommand::Write(None),
            0o0070 => Hp67RamCommand::Read(None),
            _ => match self.word & 0o77 {
                0o50 => Hp67RamCommand::Write(Some(((self.word >> 6) & 15) as u8)),
                0o70 => Hp67RamCommand::Read(Some(((self.word >> 6) & 15) as u8)),
                _ => Hp67RamCommand::Idle,
            },
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67RamCommandError {
    InvalidSyncSource(super::sync::Hp67SyncFault),
    InvalidPhase {
        word_bit: u8,
        tick: u64,
    },
    UnexpectedBit {
        expected: u8,
        actual: u8,
    },
    InvalidInstruction {
        word_bit: u8,
        is: LogicLevel,
        sync: LogicLevel,
    },
    ChangedSync {
        word_bit: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67RamCommandReceiver {
    expected_bit: u8,
    word: u16,
    qualifier: Option<bool>,
    received: Option<Hp67RamCommandFrame>,
}

impl Default for Hp67RamCommandReceiver {
    fn default() -> Self {
        Self {
            expected_bit: ROM_WORD_FIRST_BIT,
            word: 0,
            qualifier: None,
            received: None,
        }
    }
}

impl Hp67RamCommandReceiver {
    /// A complete word only; partial or faulted windows cannot publish a frame.
    pub const fn received_frame(&self) -> Option<Hp67RamCommandFrame> {
        self.received
    }

    /// Observe the ten instruction-window samples in order. All checks precede
    /// mutation, so a failed sample can be inspected/retried without losing bits.
    pub fn sample_at_edge(
        &mut self,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<(), Hp67RamCommandError> {
        let bit = bus.word_bit();
        if !bus.at_transport_sample_edge() {
            return Err(Hp67RamCommandError::InvalidPhase {
                word_bit: bit,
                tick: bus.tick().get(),
            });
        }
        if bit != self.expected_bit {
            return Err(Hp67RamCommandError::UnexpectedBit {
                expected: self.expected_bit,
                actual: bit,
            });
        }
        let is = bus.level(Hp67Net::Isa);
        let sync = bus.level(Hp67Net::Sync);
        let (value, instruction) = match (is, sync) {
            (LogicLevel::High, LogicLevel::High) => (true, true),
            (LogicLevel::Low, LogicLevel::High) => (false, true),
            (LogicLevel::High, LogicLevel::Low) => (true, false),
            (LogicLevel::Low, LogicLevel::Low) => (false, false),
            _ => {
                return Err(Hp67RamCommandError::InvalidInstruction {
                    word_bit: bit,
                    is,
                    sync,
                })
            }
        };
        super::sync::validate_sync_source(bus).map_err(Hp67RamCommandError::InvalidSyncSource)?;
        if bit != ROM_WORD_FIRST_BIT && self.qualifier != Some(instruction) {
            return Err(Hp67RamCommandError::ChangedSync { word_bit: bit });
        }
        if bit == ROM_WORD_FIRST_BIT {
            self.word = 0;
            self.received = None;
        }
        self.qualifier = Some(instruction);
        if value {
            self.word |= 1 << (bit - ROM_WORD_FIRST_BIT);
        }
        if bit == ROM_WORD_LAST_BIT {
            self.received = Some(Hp67RamCommandFrame {
                word: self.word,
                instruction,
            });
            self.expected_bit = ROM_WORD_FIRST_BIT;
        } else {
            self.expected_bit += 1;
        }
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
            wiring::Hp67Driver,
        },
    };

    fn launch(bus: &mut Hp67ElectricalBackplane, word: u16, instruction: bool) {
        let bit = bus.word_bit();
        bus.commit_serial_launch(
            act_address_drive(0x123, bit),
            RomIsDrives::Aggregate(rom_word_drive(word, bit)),
            if (46..56).contains(&bit) && instruction {
                Drive::High
            } else {
                Drive::Low
            },
            None,
        );
    }

    #[test]
    fn m15p_ram_command_receives_every_word_and_both_qualifiers_on_real_nets() {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut receiver = Hp67RamCommandReceiver::default();
        for instruction in [false, true] {
            for word in 0..=0x3ff {
                let tick = bus.tick().get();
                for bit in 0..56 {
                    launch(&mut bus, word, instruction);
                    bus.advance_clock();
                    bus.advance_clock();
                    if bit >= 46 {
                        receiver.sample_at_edge(&bus).unwrap();
                        if bit < 55 {
                            assert_eq!(receiver.received_frame(), None);
                        }
                    }
                    bus.advance_clock();
                    bus.advance_clock();
                }
                let frame = receiver.received_frame().unwrap();
                assert_eq!(frame, Hp67RamCommandFrame { word, instruction });
                if !instruction {
                    assert_eq!(frame.command(), Hp67RamCommand::Idle);
                }
                assert_eq!(bus.tick().get() - tick, 224);
            }
        }
    }

    #[test]
    fn m15p_ram_command_decodes_captured_ports_and_all_low_selectors() {
        for (word, expected) in [
            (0x270, Hp67RamCommand::Address),
            (0x2f0, Hp67RamCommand::Write(None)),
            (0x038, Hp67RamCommand::Read(None)),
            (0o1260, Hp67RamCommand::Clear),
        ] {
            assert_eq!(
                Hp67RamCommandFrame {
                    word,
                    instruction: true
                }
                .command(),
                expected
            );
        }
        for low in 0..16 {
            for (suffix, expected) in [
                (0o50, Hp67RamCommand::Write(Some(low))),
                (0o70, Hp67RamCommand::Read(Some(low))),
            ] {
                let word = (u16::from(low) << 6) | suffix;
                // Fixed 0070 means current address rather than selector zero.
                let expected = if word == 0o0070 {
                    Hp67RamCommand::Read(None)
                } else {
                    expected
                };
                assert_eq!(
                    Hp67RamCommandFrame {
                        word,
                        instruction: true
                    }
                    .command(),
                    expected
                );
            }
        }
    }

    #[test]
    fn m15p_ram_command_decodes_the_observed_word_not_a_source_payload() {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut receiver = Hp67RamCommandReceiver::default();
        for bit in 0..56 {
            launch(&mut bus, 0x270, true);
            bus.advance_clock();
            bus.advance_clock();
            if bit == 46 {
                bus.drive(
                    Hp67Net::Isa,
                    Hp67Driver::StructuralRomResponder,
                    Drive::High,
                );
            }
            if bit >= 46 {
                receiver.sample_at_edge(&bus).unwrap();
            }
            bus.advance_clock();
            bus.advance_clock();
        }
        let frame = receiver.received_frame().unwrap();
        assert_eq!(frame.word, 0x271);
        assert_eq!(frame.command(), Hp67RamCommand::Idle);
    }

    #[test]
    fn m15p_ram_command_phase_order_and_pin_faults_preserve_receiver() {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut receiver = Hp67RamCommandReceiver::default();
        for bit in 0..56 {
            launch(&mut bus, 0x270, true);
            if bit >= 46 {
                let before = receiver;
                assert!(receiver.sample_at_edge(&bus).is_err());
                assert_eq!(receiver, before);
            }
            bus.advance_clock();
            bus.advance_clock();
            if bit >= 46 {
                let before = receiver;
                for net in [Hp67Net::Isa, Hp67Net::Sync] {
                    let mut fault = bus.clone();
                    fault.drive(net, Hp67Driver::Act1820_2530, Drive::High);
                    fault.drive(net, Hp67Driver::External, Drive::Low);
                    assert!(receiver.sample_at_edge(&fault).is_err());
                    assert_eq!(receiver, before);
                }
                if bit > 46 {
                    let mut changed = bus.clone();
                    changed.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::Low);
                    assert!(matches!(
                        receiver.sample_at_edge(&changed),
                        Err(Hp67RamCommandError::ChangedSync { .. })
                    ));
                    assert_eq!(receiver, before);
                }
                receiver.sample_at_edge(&bus).unwrap();
                let after = receiver;
                assert!(receiver.sample_at_edge(&bus).is_err());
                assert_eq!(receiver, after);
            }
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(
            receiver.received_frame().unwrap().command(),
            Hp67RamCommand::Address
        );
    }
}
