//! Five HP-67 ROM serial endpoints. RAM ports and autonomous bank decoding are
//! separate: the source still supplies the existing selected-bank view.

use crate::emulation::{Drive, LogicLevel};
use super::{
    fetch::{Hp67RomWordSource, SerialFetchError},
    isa::{rom_word_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},
    timing::{isa_window_for_bit, IsaWindow, ROM_ADDRESS_BITS},
    wiring::Hp67Driver,
};

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
    Hp67RomChipProfile { driver: Hp67Driver::RomDisplay1818_0268, bank: 0, first_address: 0x000 },
    Hp67RomChipProfile { driver: Hp67Driver::RomRam1818_0550, bank: 0, first_address: 0x400 },
    Hp67RomChipProfile { driver: Hp67Driver::RomRam1818_0551, bank: 0, first_address: 0x800 },
    Hp67RomChipProfile { driver: Hp67Driver::RomRam1818_0232, bank: 0, first_address: 0xc00 },
    Hp67RomChipProfile { driver: Hp67Driver::RomRam1818_0231, bank: 1, first_address: 0x400 },
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
    response_word: Option<u16>,
}

impl Hp67RomChipEndpoint {
    fn new(profile: Hp67RomChipProfile) -> Self {
        Self { profile, received_address: 0, received_mask: 0, selected: false, response_word: None }
    }

    pub const fn profile(&self) -> Hp67RomChipProfile { self.profile }
    pub const fn selected(&self) -> bool { self.selected }

    pub const fn received_address(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_ADDRESS {
            return Err(SerialFetchError::IncompleteAddress { received_mask: self.received_mask });
        }
        Ok(self.received_address & ROM_ADDRESS_MASK)
    }

    fn sample_address_bit(&mut self, serial_bit: u8, high: bool) {
        if high { self.received_address |= 1 << serial_bit; }
        else { self.received_address &= !(1 << serial_bit); }
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
}

impl Default for Hp67RomChipSet {
    fn default() -> Self { Self { chips: HP67_ROM_PROFILES.map(Hp67RomChipEndpoint::new) } }
}

impl Hp67RomChipSet {
    pub fn begin_cycle(&mut self) {
        for chip in &mut self.chips { *chip = Hp67RomChipEndpoint::new(chip.profile); }
    }

    pub const fn chips(&self) -> &[Hp67RomChipEndpoint; HP67_ROM_CHIPS] { &self.chips }

    pub(crate) fn sample_for_bit<S: Hp67RomWordSource>(
        &mut self, word_bit: u8, level: LogicLevel, source: &S,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomAddress { serial_bit } = isa_window_for_bit(word_bit) else { return Ok(()); };
        let high = match level {
            LogicLevel::High => true,
            LogicLevel::Low => false,
            LogicLevel::Floating => return Err(SerialFetchError::FloatingIsa { word_bit }),
            LogicLevel::Contention => return Err(SerialFetchError::IsaContention { word_bit }),
        };
        for chip in &mut self.chips { chip.sample_address_bit(serial_bit, high); }
        if serial_bit + 1 != ROM_ADDRESS_BITS { return Ok(()); }
        let address = self.chips[0].received_address()?;
        let bank = source.selected_rom_bank(address);
        let mut selected = 0;
        for chip in &mut self.chips {
            let local_address = chip.received_address()?;
            chip.selected = chip.profile.bank == bank
                && (local_address & 0xc00) == chip.profile.first_address;
            selected += u8::from(chip.selected);
        }
        if selected != 1 {
            return Err(SerialFetchError::InvalidRomSelection { address, bank, selected });
        }
        for chip in &mut self.chips {
            if chip.selected {
                chip.response_word = Some(source.read_word(chip.received_address()?)
                    .ok_or(SerialFetchError::MissingRomWord { address })? & ROM_WORD_MASK);
            }
        }
        Ok(())
    }

    pub(crate) fn is_drives_for_bit(&self, word_bit: u8) -> RomIsDrives {
        RomIsDrives::Chips(std::array::from_fn(|index| self.chips[index].drive_for_bit(word_bit)))
    }

    /// Aggregate annotation for the legacy IS/PHI trace only. Production commits
    /// the five contributions separately and resolves the actual electrical net.
    pub(crate) fn trace_drive_for_bit(&self, word_bit: u8) -> Drive {
        if self.chips.iter().any(|chip| chip.drive_for_bit(word_bit) == Drive::High) {
            Drive::High
        } else { Drive::HighZ }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::cell::Cell;

    struct Source { bank: Cell<u8>, reads: Cell<usize> }
    impl Hp67RomWordSource for Source {
        fn selected_rom_bank(&self, _address: u16) -> u8 { self.bank.get() }
        fn read_word(&self, _address: u16) -> Option<u16> {
            self.reads.set(self.reads.get() + 1);
            Some(if self.bank.get() == 0 { 0x200 } else { 0x100 })
        }
    }
    fn address(chips: &mut Hp67RomChipSet, source: &Source, pc: u16) -> Result<(), SerialFetchError> {
        for bit in 16..28 {
            chips.sample_for_bit(bit, if pc & (1 << (bit - 16)) != 0 { LogicLevel::High } else { LogicLevel::Low }, source)?;
            if bit < 27 { assert!(chips.chips().iter().all(|chip| !chip.selected())); }
        }
        Ok(())
    }

    #[test]
    fn m15j_each_chip_receives_address_and_only_selected_chip_holds_response() {
        for (index, profile) in HP67_ROM_PROFILES.into_iter().enumerate() {
            let mut chips = Hp67RomChipSet::default();
            let source = Source { bank: Cell::new(profile.bank), reads: Cell::new(0) };
            let pc = profile.first_address | 0x3ff;
            address(&mut chips, &source, pc).unwrap();
            assert_eq!(source.reads.get(), 1);
            for (other, chip) in chips.chips().iter().enumerate() {
                assert_eq!(chip.received_address(), Ok(pc));
                assert_eq!(chip.selected(), other == index);
                for bit in 0..46 { assert_eq!(chip.drive_for_bit(bit), Drive::HighZ); }
            }
            let expected = if profile.bank == 0 { 0x200 } else { 0x100 };
            source.bank.set(profile.bank ^ 1); // Source changes cannot rewrite the latched word.
            let mut rebuilt = 0;
            for bit in 46..56 {
                for (other, chip) in chips.chips().iter().enumerate() {
                    if other != index { assert_eq!(chip.drive_for_bit(bit), Drive::HighZ); }
                }
                if chips.chips()[index].drive_for_bit(bit) == Drive::High { rebuilt |= 1 << (bit - 46); }
            }
            assert_eq!(rebuilt, expected);
            assert_eq!(source.reads.get(), 1);
            chips.begin_cycle();
            assert!(chips.chips().iter().all(|chip| !chip.selected() && chip.drive_for_bit(55) == Drive::HighZ));
        }
    }

    #[test]
    fn m15j_selection_rejects_no_chip_or_two_chips_before_lookup() {
        for duplicate in [false, true] {
            let mut chips = Hp67RomChipSet::default();
            if duplicate { chips.chips[2].profile = chips.chips[1].profile; }
            let source = Source { bank: Cell::new(if duplicate { 0 } else { 1 }), reads: Cell::new(0) };
            let pc = if duplicate { 0x400 } else { 0x800 };
            assert_eq!(address(&mut chips, &source, pc), Err(SerialFetchError::InvalidRomSelection {
                address: pc, bank: source.bank.get(), selected: if duplicate { 2 } else { 0 },
            }));
            assert_eq!(source.reads.get(), 0);
            assert!(chips.chips().iter().all(|chip| chip.drive_for_bit(55) == Drive::HighZ));
        }
    }
}
