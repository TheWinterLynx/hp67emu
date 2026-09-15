//! Structural HP-67 ACT↔ROM serial fetch path.
//!
//! This module models the evidence-backed bit-cell transport on the shared IS/ISA
//! line without yet claiming the exact PHI launch/sample edge.  The ACT emits a
//! 12-bit ROM address LSB-first at b16..b27, a ROM-side endpoint reconstructs the
//! address through the resolved electrical net, the selected ROM word is looked
//! up through a caller-supplied source, and the ROM returns the 10-bit word
//! LSB-first at b46..b55 for the ACT endpoint to reconstruct.

use crate::emulation::{Drive, LogicLevel};

use super::{
    isa::{act_address_drive, rom_word_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},
    timing::{isa_window_for_bit, IsaWindow, ROM_ADDRESS_BITS, ROM_WORD_BITS},
};

const COMPLETE_ADDRESS_MASK: u16 = (1u16 << ROM_ADDRESS_BITS) - 1;
const COMPLETE_WORD_MASK: u16 = (1u16 << ROM_WORD_BITS) - 1;

/// ROM lookup boundary used by the serial fetch responder.
///
/// Physical 1818-* devices will implement this boundary later.  Keeping lookup
/// behind a trait lets the serial transport be tested without embedding HP ROM
/// payloads or depending on the semantic reference ROM implementation.
pub trait Hp67RomWordSource {
    fn read_word(&self, address: u16) -> Option<u16>;
}

/// Errors that make a serial fetch electrically or structurally invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialFetchError {
    FloatingIsa { word_bit: u8 },
    IsaContention { word_bit: u8 },
    MissingRomWord { address: u16 },
    IncompleteAddress { received_mask: u16 },
    IncompleteRomWord { received_mask: u16 },
}

fn sample_isa(level: LogicLevel, word_bit: u8) -> Result<bool, SerialFetchError> {
    match level {
        LogicLevel::Low => Ok(false),
        LogicLevel::High => Ok(true),
        LogicLevel::Floating => Err(SerialFetchError::FloatingIsa { word_bit }),
        LogicLevel::Contention => Err(SerialFetchError::IsaContention { word_bit }),
    }
}

/// ACT-side state for one 56-bit ROM fetch cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActFetchEndpoint {
    address: u16,
    received_word: u16,
    received_mask: u16,
}

impl ActFetchEndpoint {
    pub const fn new(address: u16) -> Self {
        Self {
            address: address & ROM_ADDRESS_MASK,
            received_word: 0,
            received_mask: 0,
        }
    }

    /// Start another fetch word while retaining no bits from the prior response.
    pub fn begin_cycle(&mut self, address: u16) {
        self.address = address & ROM_ADDRESS_MASK;
        self.received_word = 0;
        self.received_mask = 0;
    }

    pub const fn address(&self) -> u16 {
        self.address
    }

    /// ACT contribution to IS/ISA for the current bit cell.
    pub const fn drive_for_bit(&self, word_bit: u8) -> Drive {
        act_address_drive(self.address, word_bit)
    }

    /// Sample a resolved IS/ISA level during the ROM return window.
    pub fn sample_for_bit(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomWord { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };

        if sample_isa(level, word_bit)? {
            self.received_word |= 1u16 << serial_bit;
        } else {
            self.received_word &= !(1u16 << serial_bit);
        }
        self.received_mask |= 1u16 << serial_bit;
        Ok(())
    }

    /// Return the complete reconstructed 10-bit word after b55 has been sampled.
    pub const fn fetched_word(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_WORD_MASK {
            return Err(SerialFetchError::IncompleteRomWord {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_word & ROM_WORD_MASK)
    }
}

/// ROM-side state for one 56-bit serial fetch cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RomFetchEndpoint {
    received_address: u16,
    received_mask: u16,
    response_word: Option<u16>,
}

impl RomFetchEndpoint {
    pub fn begin_cycle(&mut self) {
        self.received_address = 0;
        self.received_mask = 0;
        self.response_word = None;
    }

    /// Sample ACT address bits from the resolved IS/ISA line.  When b27 is
    /// received, the complete 12-bit address is used to latch the ROM response.
    pub fn sample_for_bit<S: Hp67RomWordSource>(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
        source: &S,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomAddress { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };

        if sample_isa(level, word_bit)? {
            self.received_address |= 1u16 << serial_bit;
        } else {
            self.received_address &= !(1u16 << serial_bit);
        }
        self.received_mask |= 1u16 << serial_bit;

        if serial_bit + 1 == ROM_ADDRESS_BITS {
            if self.received_mask != COMPLETE_ADDRESS_MASK {
                return Err(SerialFetchError::IncompleteAddress {
                    received_mask: self.received_mask,
                });
            }
            let address = self.received_address & ROM_ADDRESS_MASK;
            self.response_word = Some(
                source
                    .read_word(address)
                    .ok_or(SerialFetchError::MissingRomWord { address })?
                    & ROM_WORD_MASK,
            );
        }

        Ok(())
    }

    pub const fn received_address(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_ADDRESS_MASK {
            return Err(SerialFetchError::IncompleteAddress {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_address & ROM_ADDRESS_MASK)
    }

    /// ROM contribution to IS/ISA for the current bit cell.  No response is
    /// driven until all twelve address bits have been captured and a word found.
    pub const fn drive_for_bit(&self, word_bit: u8) -> Drive {
        match self.response_word {
            Some(word) => rom_word_drive(word, word_bit),
            None => Drive::HighZ,
        }
    }
}

/// One-word pipeline latch separating a just-fetched word from the word that is
/// eligible to execute in the following 56-bit cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FetchPipelineLatch {
    executing: Option<u16>,
    prefetched: Option<u16>,
}

impl FetchPipelineLatch {
    /// Commit the word reconstructed during the just-finished machine cycle.
    /// The previously prefetched word becomes executable for the next cycle.
    pub fn complete_cycle(&mut self, fetched_word: u16) {
        self.executing = self.prefetched.take();
        self.prefetched = Some(fetched_word & ROM_WORD_MASK);
    }

    pub const fn executing_word(&self) -> Option<u16> {
        self.executing
    }

    pub const fn prefetched_word(&self) -> Option<u16> {
        self.prefetched
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        emulation::{DriverId, LogicLevel},
        machines::hp67::{Hp67ElectricalBackplane, Hp67Net, BITS_PER_WORD},
    };

    const ACT_IS_DRIVER: DriverId = DriverId::new("test-act-is");
    const ROM_IS_DRIVER: DriverId = DriverId::new("test-rom-is");

    struct FixtureRom {
        words: [(u16, u16); 2],
    }

    impl Hp67RomWordSource for FixtureRom {
        fn read_word(&self, address: u16) -> Option<u16> {
            self.words
                .iter()
                .find_map(|(candidate, word)| (*candidate == address).then_some(*word))
        }
    }

    fn run_fetch_cycle<S: Hp67RomWordSource>(
        backplane: &mut Hp67ElectricalBackplane,
        act: &mut ActFetchEndpoint,
        rom: &mut RomFetchEndpoint,
        source: &S,
    ) -> Result<u16, SerialFetchError> {
        rom.begin_cycle();

        for expected_bit in 0..BITS_PER_WORD {
            assert_eq!(backplane.word_bit(), expected_bit);

            backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, act.drive_for_bit(expected_bit));
            backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom.drive_for_bit(expected_bit));

            if matches!(isa_window_for_bit(expected_bit), IsaWindow::RomAddress { .. }) {
                rom.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa), source)?;
                // The last address bit can latch the response, but the address
                // window itself remains ACT-owned; the ROM cannot drive until
                // the later b46..b55 response window.
                backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom.drive_for_bit(expected_bit));
            }

            if matches!(isa_window_for_bit(expected_bit), IsaWindow::RomWord { .. }) {
                act.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa))?;
            }

            assert_ne!(backplane.level(Hp67Net::Isa), LogicLevel::Contention);
            for _ in 0..4 {
                backplane.advance_clock();
            }
        }

        act.fetched_word()
    }

    #[test]
    fn real_hp67_example_crosses_the_resolved_is_net_bit_by_bit() {
        let source = FixtureRom {
            // Measured key-wait example plus the physical startup word at 0x001.
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActFetchEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();

        let fetched = run_fetch_cycle(&mut backplane, &mut act, &mut rom, &source)
            .expect("serial fetch must complete");

        assert_eq!(rom.received_address(), Ok(0x07b));
        assert_eq!(fetched, 0x04c);
        assert_eq!(backplane.word_index(), 1);
        assert_eq!(backplane.word_bit(), 0);
    }

    #[test]
    fn fetched_word_enters_execution_only_on_the_following_machine_cycle() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActFetchEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut pipeline = FetchPipelineLatch::default();

        let first = run_fetch_cycle(&mut backplane, &mut act, &mut rom, &source)
            .expect("first serial fetch must complete");
        pipeline.complete_cycle(first);
        assert_eq!(pipeline.executing_word(), None);
        assert_eq!(pipeline.prefetched_word(), Some(0x04c));

        act.begin_cycle(0x001);
        let second = run_fetch_cycle(&mut backplane, &mut act, &mut rom, &source)
            .expect("second serial fetch must complete");
        pipeline.complete_cycle(second);

        assert_eq!(pipeline.executing_word(), Some(0x04c));
        assert_eq!(pipeline.prefetched_word(), Some(0x3e3));
        assert_eq!(backplane.word_index(), 2);
    }

    #[test]
    fn missing_rom_location_fails_at_the_end_of_the_address_window() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut rom = RomFetchEndpoint::default();
        let mut address = ActFetchEndpoint::new(0x222);

        for bit in 16..=27 {
            let level = match address.drive_for_bit(bit) {
                Drive::High => LogicLevel::High,
                Drive::HighZ => LogicLevel::Low,
                Drive::Low => unreachable!("IS zeros are represented by release"),
            };
            let result = rom.sample_for_bit(bit, level, &source);
            if bit == 27 {
                assert_eq!(result, Err(SerialFetchError::MissingRomWord { address: 0x222 }));
            } else {
                assert_eq!(result, Ok(()));
            }
        }
    }
}
