//! Five HP-67 ROM serial endpoints with autonomous IS/SYNC bank latches.
//! Explicit provider-view mode remains only for reference/comparison fixtures.

use super::{
    fetch::{Hp67RomWordSource, SerialFetchError},
    isa::{rom_word_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},
    timing::{isa_window_for_bit, IsaWindow, ROM_ADDRESS_BITS, ROM_WORD_BITS},
    wiring::Hp67Driver,
};
use crate::emulation::{Drive, LogicLevel};

pub const HP67_ROM_CHIPS: usize = 5;
const COMPLETE_ADDRESS: u16 = (1 << ROM_ADDRESS_BITS) - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67RomChipProfile {
    pub driver: Hp67Driver,
    pub bank: u8,
    pub first_address: u16,
}

/// ROM1/2 identities use the owner's HP-97 equivalence assumption. ROM4's
/// 0231 identity follows by elimination from the five-chip HP-67 inventory;
/// this particular part/range assignment remains an explicit working inference.
pub const HP67_ROM_PROFILES: [Hp67RomChipProfile; HP67_ROM_CHIPS] = [
    Hp67RomChipProfile {
        driver: Hp67Driver::RomDisplay1818_0268,
        bank: 0,
        first_address: 0x000,
    },
    Hp67RomChipProfile {
        driver: Hp67Driver::RomRam1818_0550,
        bank: 0,
        first_address: 0x400,
    },
    Hp67RomChipProfile {
        driver: Hp67Driver::RomRam1818_0551,
        bank: 0,
        first_address: 0x800,
    },
    Hp67RomChipProfile {
        driver: Hp67Driver::RomRam1818_0232,
        bank: 0,
        first_address: 0xc00,
    },
    Hp67RomChipProfile {
        driver: Hp67Driver::RomRam1818_0231,
        bank: 1,
        first_address: 0x400,
    },
];

/// Source image for the shared IS launch; physical contributions remain distinct.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RomIsDrives {
    Aggregate(Drive),
    Chips([Drive; HP67_ROM_CHIPS]),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67RomChipEndpoint {
    profile: Hp67RomChipProfile,
    received_address: u16,
    received_mask: u16,
    selected: bool,
    bank_latch: u8,
    response_word: Option<u16>,
}

impl Hp67RomChipEndpoint {
    fn new(profile: Hp67RomChipProfile) -> Self {
        Self {
            profile,
            received_address: 0,
            received_mask: 0,
            selected: false,
            bank_latch: 0,
            response_word: None,
        }
    }

    pub const fn profile(&self) -> Hp67RomChipProfile {
        self.profile
    }
    pub const fn bank_latch(&self) -> u8 {
        self.bank_latch
    }

    pub const fn selected(&self) -> bool {
        self.selected
    }

    pub const fn received_address(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_ADDRESS {
            return Err(SerialFetchError::IncompleteAddress {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_address & ROM_ADDRESS_MASK)
    }

    fn sample_address_bit(&mut self, serial_bit: u8, high: bool) {
        if high {
            self.received_address |= 1 << serial_bit;
        } else {
            self.received_address &= !(1 << serial_bit);
        }
        self.received_mask |= 1 << serial_bit;
    }

    pub const fn drive_for_bit(&self, word_bit: u8) -> Drive {
        match self.response_word {
            Some(word) if self.selected => rom_word_drive(word, word_bit),
            _ => Drive::HighZ,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67RomChipSet {
    chips: [Hp67RomChipEndpoint; HP67_ROM_CHIPS],
    autonomous: bool,
    observed_word: u16,
    received_address_bits: u8,
    received_return_bits: u8,
    observed_sync: Option<bool>,
}

impl Default for Hp67RomChipSet {
    fn default() -> Self {
        Self {
            chips: HP67_ROM_PROFILES.map(Hp67RomChipEndpoint::new),
            autonomous: false,
            observed_word: 0,
            received_address_bits: 0,
            received_return_bits: 0,
            observed_sync: None,
        }
    }
}

impl Hp67RomChipSet {
    /// Production mode: bank selection is reconstructed from the resolved bus.
    pub fn autonomous() -> Self {
        Self {
            autonomous: true,
            ..Self::default()
        }
    }

    pub const fn has_autonomous_banks(&self) -> bool {
        self.autonomous
    }

    pub fn begin_cycle(&mut self) {
        for chip in &mut self.chips {
            let bank = chip.bank_latch;
            *chip = Hp67RomChipEndpoint::new(chip.profile);
            chip.bank_latch = bank;
        }
        self.observed_word = 0;
        self.received_address_bits = 0;
        self.received_return_bits = 0;
        self.observed_sync = None;
    }

    pub const fn chips(&self) -> &[Hp67RomChipEndpoint; HP67_ROM_CHIPS] {
        &self.chips
    }

    pub(crate) fn sample_for_bit<S: Hp67RomWordSource>(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
        source: &S,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomAddress { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };
        if serial_bit != self.received_address_bits {
            return Err(SerialFetchError::UnexpectedRomAddressBit {
                word_bit,
                received_bits: self.received_address_bits,
            });
        }
        let high = match level {
            LogicLevel::High => true,
            LogicLevel::Low => false,
            LogicLevel::Floating => return Err(SerialFetchError::FloatingIsa { word_bit }),
            LogicLevel::Contention => return Err(SerialFetchError::IsaContention { word_bit }),
        };
        for chip in &mut self.chips {
            chip.sample_address_bit(serial_bit, high);
        }
        self.received_address_bits += 1;
        if serial_bit + 1 != ROM_ADDRESS_BITS {
            return Ok(());
        }
        let address = self.chips[0].received_address()?;
        // Teenix printed p71: addressing ROM0 restores bank 0 in all chips.
        // Commit only after the complete resolved address, before selection.
        if self.autonomous && address < 0x400 {
            for chip in &mut self.chips {
                chip.bank_latch = 0;
            }
        }
        let bank = if self.autonomous {
            self.chips[0].bank_latch
        } else {
            source.selected_rom_bank(address)
        };
        let mut selected = 0;
        for chip in &mut self.chips {
            let local_address = chip.received_address()?;
            let active_bank = if self.autonomous {
                chip.bank_latch
            } else {
                bank
            };
            chip.selected = chip.profile.bank == active_bank
                && (local_address & 0xc00) == chip.profile.first_address;
            selected += u8::from(chip.selected);
        }
        if selected != 1 {
            return Err(SerialFetchError::InvalidRomSelection {
                address,
                bank,
                selected,
            });
        }
        for chip in &mut self.chips {
            if chip.selected {
                chip.response_word = Some(
                    (if self.autonomous {
                        source.read_word_in_bank(chip.profile.bank, chip.received_address()?)
                    } else {
                        source.read_word(chip.received_address()?)
                    })
                    .ok_or(SerialFetchError::MissingRomWord { address })?
                        & ROM_WORD_MASK,
                );
            }
        }
        Ok(())
    }

    /// Observe the same resolved return as ACT. This shared observation is
    /// validated before changing any chip latch; iteration order cannot matter.
    /// Bank-set affects the NEXT address, never the response already in flight.
    pub(crate) fn sample_instruction_for_bit(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
        sync: LogicLevel,
    ) -> Result<(), SerialFetchError> {
        if !self.autonomous {
            return Ok(());
        }
        let IsaWindow::RomWord { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };
        if serial_bit != self.received_return_bits {
            return Err(SerialFetchError::UnexpectedRomReturnBit {
                word_bit,
                received_bits: self.received_return_bits,
            });
        }
        let qualifier = match sync {
            LogicLevel::High => true,
            LogicLevel::Low => false,
            _ => {
                return Err(SerialFetchError::UnresolvedSync {
                    word_bit,
                    level: sync,
                })
            }
        };
        if self
            .observed_sync
            .is_some_and(|previous| previous != qualifier)
        {
            return Err(SerialFetchError::ChangedSyncQualifier { word_bit });
        }
        let high = match level {
            LogicLevel::High => true,
            LogicLevel::Low => false,
            LogicLevel::Floating => return Err(SerialFetchError::FloatingIsa { word_bit }),
            LogicLevel::Contention => return Err(SerialFetchError::IsaContention { word_bit }),
        };
        self.observed_sync = Some(qualifier);
        if high {
            self.observed_word |= 1 << serial_bit;
        } else {
            self.observed_word &= !(1 << serial_bit);
        }
        self.received_return_bits += 1;
        if self.received_return_bits == ROM_WORD_BITS && qualifier && self.observed_word == 0x230 {
            // Set, not toggle: a repeated bank instruction cannot restore bank 0.
            for chip in &mut self.chips {
                chip.bank_latch = 1;
            }
        }
        Ok(())
    }

    /// Complete return observed on resolved IS by autonomous ROM receivers.
    /// Provider-view fixtures do not monitor the return and remain incomplete.
    pub const fn received_word(&self) -> Result<u16, SerialFetchError> {
        if self.received_return_bits != ROM_WORD_BITS {
            return Err(SerialFetchError::IncompleteRomWord {
                received_mask: (1u16 << self.received_return_bits) - 1,
            });
        }
        Ok(self.observed_word & ROM_WORD_MASK)
    }

    pub(crate) fn is_drives_for_bit(&self, word_bit: u8) -> RomIsDrives {
        RomIsDrives::Chips(std::array::from_fn(|index| {
            self.chips[index].drive_for_bit(word_bit)
        }))
    }

    /// Aggregate annotation for the legacy IS/PHI trace only. Production commits
    /// the five contributions separately and resolves the actual electrical net.
    pub(crate) fn trace_drive_for_bit(&self, word_bit: u8) -> Drive {
        if self
            .chips
            .iter()
            .any(|chip| chip.drive_for_bit(word_bit) == Drive::High)
        {
            Drive::High
        } else {
            Drive::HighZ
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Source {
        bank: Cell<u8>,
        reads: Cell<usize>,
    }
    impl Hp67RomWordSource for Source {
        fn selected_rom_bank(&self, _address: u16) -> u8 {
            self.bank.get()
        }
        fn read_word(&self, _address: u16) -> Option<u16> {
            self.reads.set(self.reads.get() + 1);
            Some(if self.bank.get() == 0 { 0x200 } else { 0x100 })
        }
    }
    fn address(
        chips: &mut Hp67RomChipSet,
        source: &Source,
        pc: u16,
    ) -> Result<(), SerialFetchError> {
        for bit in 16..28 {
            chips.sample_for_bit(
                bit,
                if pc & (1 << (bit - 16)) != 0 {
                    LogicLevel::High
                } else {
                    LogicLevel::Low
                },
                source,
            )?;
            if bit < 27 {
                assert!(chips.chips().iter().all(|chip| !chip.selected()));
            }
        }
        Ok(())
    }

    fn observe(chips: &mut Hp67RomChipSet, word: u16, sync: LogicLevel) {
        for bit in 46..56 {
            chips
                .sample_instruction_for_bit(
                    bit,
                    if word & (1 << (bit - 46)) != 0 {
                        LogicLevel::High
                    } else {
                        LogicLevel::Low
                    },
                    sync,
                )
                .unwrap();
        }
    }

    fn resolved_bit(word: u16, serial_bit: u8) -> LogicLevel {
        if word & (1 << serial_bit) != 0 {
            LogicLevel::High
        } else {
            LogicLevel::Low
        }
    }

    #[test]
    fn m15m_address_receiver_rejects_skips_repeats_and_replays_before_mutation() {
        for autonomous in [false, true] {
            for prefix in 0..=ROM_ADDRESS_BITS {
                let mut chips = if autonomous {
                    Hp67RomChipSet::autonomous()
                } else {
                    Hp67RomChipSet::default()
                };
                let source = Source {
                    bank: Cell::new(0),
                    reads: Cell::new(0),
                };
                for serial_bit in 0..prefix {
                    chips
                        .sample_for_bit(16 + serial_bit, resolved_bit(0x123, serial_bit), &source)
                        .unwrap();
                }
                let before = chips;
                let reads = source.reads.get();
                for serial_bit in 0..ROM_ADDRESS_BITS {
                    if serial_bit == prefix {
                        continue;
                    }
                    assert_eq!(
                        chips.sample_for_bit(16 + serial_bit, LogicLevel::High, &source),
                        Err(SerialFetchError::UnexpectedRomAddressBit {
                            word_bit: 16 + serial_bit,
                            received_bits: prefix
                        })
                    );
                    assert_eq!(chips, before);
                    assert_eq!(source.reads.get(), reads);
                }
                // A rejected callback consumes no bit; the valid suffix works.
                for serial_bit in prefix..ROM_ADDRESS_BITS {
                    chips
                        .sample_for_bit(16 + serial_bit, resolved_bit(0x123, serial_bit), &source)
                        .unwrap();
                }
                assert_eq!(source.reads.get(), 1);
                assert!(chips
                    .chips()
                    .iter()
                    .all(|chip| chip.received_address() == Ok(0x123)));
                assert_eq!(
                    chips.chips().iter().filter(|chip| chip.selected()).count(),
                    1
                );
                chips.begin_cycle();
                chips.sample_for_bit(16, LogicLevel::High, &source).unwrap();
            }
        }
    }

    #[test]
    fn m15m_return_receiver_rejects_reorder_and_only_decodes_complete_bank_word() {
        for prefix in 0..=ROM_WORD_BITS {
            let mut chips = Hp67RomChipSet::autonomous();
            for serial_bit in 0..prefix {
                chips
                    .sample_instruction_for_bit(
                        46 + serial_bit,
                        resolved_bit(0x230, serial_bit),
                        LogicLevel::High,
                    )
                    .unwrap();
            }
            let before = chips;
            for serial_bit in 0..ROM_WORD_BITS {
                if serial_bit == prefix {
                    continue;
                }
                assert_eq!(
                    chips.sample_instruction_for_bit(
                        46 + serial_bit,
                        LogicLevel::High,
                        LogicLevel::High
                    ),
                    Err(SerialFetchError::UnexpectedRomReturnBit {
                        word_bit: 46 + serial_bit,
                        received_bits: prefix
                    })
                );
                assert_eq!(chips, before);
            }
            if prefix < ROM_WORD_BITS {
                assert_eq!(
                    chips.received_word(),
                    Err(SerialFetchError::IncompleteRomWord {
                        received_mask: (1u16 << prefix) - 1,
                    })
                );
                assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 0));
            }
            for serial_bit in prefix..ROM_WORD_BITS {
                chips
                    .sample_instruction_for_bit(
                        46 + serial_bit,
                        resolved_bit(0x230, serial_bit),
                        LogicLevel::High,
                    )
                    .unwrap();
            }
            assert_eq!(chips.received_word(), Ok(0x230));
            assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 1));
            chips.begin_cycle();
            assert!(chips.received_word().is_err());
            assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 1));
        }
    }

    #[test]
    fn m15m_invalid_levels_preserve_the_next_bit_and_valid_suffix_can_complete() {
        for prefix in 0..ROM_WORD_BITS {
            for fault in 0..5 {
                if fault == 4 && prefix == 0 {
                    continue;
                }
                let mut chips = Hp67RomChipSet::autonomous();
                for serial_bit in 0..prefix {
                    chips
                        .sample_instruction_for_bit(
                            46 + serial_bit,
                            resolved_bit(0x230, serial_bit),
                            LogicLevel::High,
                        )
                        .unwrap();
                }
                let before = chips;
                let (level, sync) = match fault {
                    0 => (LogicLevel::Floating, LogicLevel::High),
                    1 => (LogicLevel::Contention, LogicLevel::High),
                    2 => (LogicLevel::Low, LogicLevel::Floating),
                    3 => (LogicLevel::Low, LogicLevel::Contention),
                    _ => (LogicLevel::Low, LogicLevel::Low),
                };
                assert!(chips
                    .sample_instruction_for_bit(46 + prefix, level, sync)
                    .is_err());
                assert_eq!(chips, before);
                for serial_bit in prefix..ROM_WORD_BITS {
                    chips
                        .sample_instruction_for_bit(
                            46 + serial_bit,
                            resolved_bit(0x230, serial_bit),
                            LogicLevel::High,
                        )
                        .unwrap();
                }
                assert_eq!(chips.received_word(), Ok(0x230));
                assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 1));
            }
        }
        for prefix in 0..ROM_ADDRESS_BITS {
            for level in [LogicLevel::Floating, LogicLevel::Contention] {
                let mut chips = Hp67RomChipSet::autonomous();
                let source = Source {
                    bank: Cell::new(0),
                    reads: Cell::new(0),
                };
                for serial_bit in 0..prefix {
                    chips
                        .sample_for_bit(16 + serial_bit, resolved_bit(0x123, serial_bit), &source)
                        .unwrap();
                }
                let before = chips;
                assert!(chips.sample_for_bit(16 + prefix, level, &source).is_err());
                assert_eq!(chips, before);
                assert_eq!(source.reads.get(), 0);
                for serial_bit in prefix..ROM_ADDRESS_BITS {
                    chips
                        .sample_for_bit(16 + serial_bit, resolved_bit(0x123, serial_bit), &source)
                        .unwrap();
                }
                assert_eq!(source.reads.get(), 1);
            }
        }
    }

    #[test]
    fn m15k_bank_latches_require_complete_normal_instruction_and_survive_cycle() {
        for word in 0..=ROM_WORD_MASK {
            for sync in [LogicLevel::Low, LogicLevel::High] {
                let mut chips = Hp67RomChipSet::autonomous();
                observe(&mut chips, word, sync);
                let bank = u8::from(word == 0x230 && sync == LogicLevel::High);
                assert!(chips.chips().iter().all(|chip| chip.bank_latch() == bank));
                chips.begin_cycle();
                assert!(chips.chips().iter().all(|chip| chip.bank_latch() == bank));
            }
        }
        let mut chips = Hp67RomChipSet::autonomous();
        observe(&mut chips, 0x230, LogicLevel::High);
        chips.begin_cycle();
        observe(&mut chips, 0x230, LogicLevel::High);
        assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 1));
    }

    #[test]
    fn m15k_incomplete_or_invalid_return_never_changes_bank_latches() {
        for fault in [
            LogicLevel::Floating,
            LogicLevel::Contention,
            LogicLevel::Low,
        ] {
            let mut chips = Hp67RomChipSet::autonomous();
            for bit in 46..55 {
                chips
                    .sample_instruction_for_bit(
                        bit,
                        if 0x230 & (1 << (bit - 46)) != 0 {
                            LogicLevel::High
                        } else {
                            LogicLevel::Low
                        },
                        LogicLevel::High,
                    )
                    .unwrap();
                assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 0));
            }
            assert!(chips
                .sample_instruction_for_bit(55, LogicLevel::High, fault)
                .is_err());
            assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 0));
        }
        for fault in [LogicLevel::Floating, LogicLevel::Contention] {
            let mut chips = Hp67RomChipSet::autonomous();
            assert!(chips
                .sample_instruction_for_bit(46, fault, LogicLevel::High)
                .is_err());
            assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 0));
        }
        let mut chips = Hp67RomChipSet::autonomous();
        // Missing b46 now rejects the first out-of-order sample immediately.
        let before = chips;
        assert_eq!(
            chips.sample_instruction_for_bit(47, LogicLevel::Low, LogicLevel::High),
            Err(SerialFetchError::UnexpectedRomReturnBit {
                word_bit: 47,
                received_bits: 0
            })
        );
        assert_eq!(chips, before);
        assert!(chips.chips().iter().all(|chip| chip.bank_latch() == 0));
    }

    #[test]
    fn m15k_autonomous_map_covers_all_addresses_without_provider_bank_control() {
        struct RawSource;
        impl Hp67RomWordSource for RawSource {
            fn read_word(&self, _: u16) -> Option<u16> {
                panic!("provider view used")
            }
            fn selected_rom_bank(&self, _: u16) -> u8 {
                panic!("host bank control used")
            }
            fn read_word_in_bank(&self, bank: u8, pc: u16) -> Option<u16> {
                Some((pc ^ (u16::from(bank) << 9)) & ROM_WORD_MASK)
            }
        }
        for initial_bank in 0..2 {
            for pc in 0..=ROM_ADDRESS_MASK {
                let mut chips = Hp67RomChipSet::autonomous();
                if initial_bank == 1 {
                    observe(&mut chips, 0x230, LogicLevel::High);
                }
                chips.begin_cycle();
                let mut result = Ok(());
                for bit in 16..28 {
                    result = chips.sample_for_bit(
                        bit,
                        if pc & (1 << (bit - 16)) != 0 {
                            LogicLevel::High
                        } else {
                            LogicLevel::Low
                        },
                        &RawSource,
                    );
                    if bit < 27 {
                        assert!(result.is_ok());
                    }
                }
                let bank = if pc < 0x400 { 0 } else { initial_bank };
                if bank == 1 && pc >= 0x800 {
                    assert_eq!(
                        result,
                        Err(SerialFetchError::InvalidRomSelection {
                            address: pc,
                            bank,
                            selected: 0
                        })
                    );
                    assert!(chips
                        .chips()
                        .iter()
                        .all(|chip| chip.drive_for_bit(55) == Drive::HighZ));
                } else {
                    result.unwrap();
                    let selected = chips.chips().iter().find(|chip| chip.selected()).unwrap();
                    assert_eq!(selected.profile().bank, bank);
                    assert_eq!(
                        selected.response_word,
                        Some((pc ^ (u16::from(bank) << 9)) & ROM_WORD_MASK)
                    );
                }
                assert!(chips.chips().iter().all(|chip| chip.bank_latch() == bank));
            }
        }
    }

    #[test]
    fn m15j_each_chip_receives_address_and_only_selected_chip_holds_response() {
        for (index, profile) in HP67_ROM_PROFILES.into_iter().enumerate() {
            let mut chips = Hp67RomChipSet::default();
            let source = Source {
                bank: Cell::new(profile.bank),
                reads: Cell::new(0),
            };
            let pc = profile.first_address | 0x3ff;
            address(&mut chips, &source, pc).unwrap();
            assert_eq!(source.reads.get(), 1);
            for (other, chip) in chips.chips().iter().enumerate() {
                assert_eq!(chip.received_address(), Ok(pc));
                assert_eq!(chip.selected(), other == index);
                for bit in 0..46 {
                    assert_eq!(chip.drive_for_bit(bit), Drive::HighZ);
                }
            }
            let expected = if profile.bank == 0 { 0x200 } else { 0x100 };
            source.bank.set(profile.bank ^ 1); // Source changes cannot rewrite the latched word.
            let mut rebuilt = 0;
            for bit in 46..56 {
                for (other, chip) in chips.chips().iter().enumerate() {
                    if other != index {
                        assert_eq!(chip.drive_for_bit(bit), Drive::HighZ);
                    }
                }
                if chips.chips()[index].drive_for_bit(bit) == Drive::High {
                    rebuilt |= 1 << (bit - 46);
                }
            }
            assert_eq!(rebuilt, expected);
            assert_eq!(source.reads.get(), 1);
            chips.begin_cycle();
            assert!(chips
                .chips()
                .iter()
                .all(|chip| !chip.selected() && chip.drive_for_bit(55) == Drive::HighZ));
        }
    }

    #[test]
    fn m15j_selection_rejects_no_chip_or_two_chips_before_lookup() {
        for duplicate in [false, true] {
            let mut chips = Hp67RomChipSet::default();
            if duplicate {
                chips.chips[2].profile = chips.chips[1].profile;
            }
            let source = Source {
                bank: Cell::new(if duplicate { 0 } else { 1 }),
                reads: Cell::new(0),
            };
            let pc = if duplicate { 0x400 } else { 0x800 };
            assert_eq!(
                address(&mut chips, &source, pc),
                Err(SerialFetchError::InvalidRomSelection {
                    address: pc,
                    bank: source.bank.get(),
                    selected: if duplicate { 2 } else { 0 },
                })
            );
            assert_eq!(source.reads.get(), 0);
            assert!(chips
                .chips()
                .iter()
                .all(|chip| chip.drive_for_bit(55) == Drive::HighZ));
        }
    }
}
