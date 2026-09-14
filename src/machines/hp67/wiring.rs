//! Verified HP-67 chip inventory and named electrical nets.
//!
//! Part numbers come from HP-67 hardware documentation/board inspection sources.
//! Signal names are restricted to lines we have direct schematic or timing-source
//! evidence for; uncertain guessed wiring belongs in TODOs, not production code.

/// ICs and active display/card-reader devices confirmed for the HP-67.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hp67Chip {
    Act1820_2530,
    RomRam1818_0231,
    RomRam1818_0232,
    RomDisplay1818_0268,
    RomRam1818_0550,
    RomRam1818_0551,
    CathodeDriver1820_1749,
    CardReaderController1820_1751,
    CardSenseAmplifier1826_0322,
    DisplayTransistorArray1858_0050,
}

impl Hp67Chip {
    pub const fn part_number(self) -> &'static str {
        match self {
            Self::Act1820_2530 => "1820-2530",
            Self::RomRam1818_0231 => "1818-0231",
            Self::RomRam1818_0232 => "1818-0232",
            Self::RomDisplay1818_0268 => "1818-0268",
            Self::RomRam1818_0550 => "1818-0550",
            Self::RomRam1818_0551 => "1818-0551",
            Self::CathodeDriver1820_1749 => "1820-1749",
            Self::CardReaderController1820_1751 => "1820-1751",
            Self::CardSenseAmplifier1826_0322 => "1826-0322",
            Self::DisplayTransistorArray1858_0050 => "1858-0050",
        }
    }
}

pub const CHIPSET: &[Hp67Chip] = &[
    Hp67Chip::Act1820_2530,
    Hp67Chip::RomRam1818_0231,
    Hp67Chip::RomRam1818_0232,
    Hp67Chip::RomDisplay1818_0268,
    Hp67Chip::RomRam1818_0550,
    Hp67Chip::RomRam1818_0551,
    Hp67Chip::CathodeDriver1820_1749,
    Hp67Chip::CardReaderController1820_1751,
    Hp67Chip::CardSenseAmplifier1826_0322,
    Hp67Chip::DisplayTransistorArray1858_0050,
];

/// Named digital nets already supported by direct HP-67 evidence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Hp67Net {
    Phi1,
    Phi2,
    Isa,
    Data,
    Sync,
    Rcd,
    Str,
    F1,
    F2,
    Kc1,
    Kc2,
    Kc3,
    Kc4,
    Kc5,
}

impl Hp67Net {
    pub const ALL: [Self; 14] = [
        Self::Phi1,
        Self::Phi2,
        Self::Isa,
        Self::Data,
        Self::Sync,
        Self::Rcd,
        Self::Str,
        Self::F1,
        Self::F2,
        Self::Kc1,
        Self::Kc2,
        Self::Kc3,
        Self::Kc4,
        Self::Kc5,
    ];
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn inventory_contains_unique_part_numbers() {
        let mut parts: Vec<_> = CHIPSET.iter().map(|chip| chip.part_number()).collect();
        let count = parts.len();
        parts.sort_unstable();
        parts.dedup();
        assert_eq!(parts.len(), count);
    }

    #[test]
    fn required_serial_and_display_control_nets_are_present() {
        for required in [
            Hp67Net::Phi1,
            Hp67Net::Phi2,
            Hp67Net::Isa,
            Hp67Net::Data,
            Hp67Net::Sync,
            Hp67Net::Rcd,
            Hp67Net::Str,
        ] {
            assert!(Hp67Net::ALL.contains(&required));
        }
    }
}
