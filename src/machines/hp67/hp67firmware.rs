//! Versioned HP-67 firmware image.
//!
//! The firmware words live in ten repository-tracked data blocks named
//! `hp67firmware.00` through `hp67firmware.09`. `include_str!` embeds those
//! blocks into the executable at compile time; no firmware file is opened at
//! runtime and no build-time generated source is required.

use std::{cell::Cell, sync::OnceLock};

use super::Hp67RomWordSource;

const BANK0_WORDS: usize = 4096;
const BANK1_BASE_PC: usize = 0x400;
const BANK1_WORDS: usize = 1024;
const FIRMWARE_WORDS: usize = BANK0_WORDS + BANK1_WORDS;
const WORD_MASK: u16 = 0x03ff;

const BLOCKS: [&str; 10] = [
    include_str!("hp67firmware.00"),
    include_str!("hp67firmware.01"),
    include_str!("hp67firmware.02"),
    include_str!("hp67firmware.03"),
    include_str!("hp67firmware.04"),
    include_str!("hp67firmware.05"),
    include_str!("hp67firmware.06"),
    include_str!("hp67firmware.07"),
    include_str!("hp67firmware.08"),
    include_str!("hp67firmware.09"),
];

static WORDS: OnceLock<[u16; FIRMWARE_WORDS]> = OnceLock::new();

fn words() -> &'static [u16; FIRMWARE_WORDS] {
    WORDS.get_or_init(|| {
        let mut image = [0u16; FIRMWARE_WORDS];
        let mut index = 0usize;

        for block in BLOCKS {
            for token in block
                .split(|ch: char| ch == ',' || ch.is_whitespace())
                .filter(|token| !token.is_empty())
            {
                assert!(index < FIRMWARE_WORDS, "HP-67 firmware contains too many words");
                let word = u16::from_str_radix(token, 8)
                    .unwrap_or_else(|_| panic!("invalid HP-67 firmware word {token}"));
                assert!(word <= WORD_MASK, "HP-67 firmware word exceeds 10 bits");
                image[index] = word;
                index += 1;
            }
        }

        assert_eq!(
            index, FIRMWARE_WORDS,
            "HP-67 firmware contains {index} words; expected {FIRMWARE_WORDS}"
        );
        image
    })
}

#[derive(Debug, Clone, Default)]
pub struct Hp67Firmware {
    requested_bank: Cell<u8>,
}

impl Hp67Firmware {
    pub const fn populated_words() -> usize {
        FIRMWARE_WORDS
    }

    pub fn select_bank(&self, bank: u8) {
        self.requested_bank.set(bank & 1);
    }

    pub fn reset_bank(&self) {
        self.requested_bank.set(0);
    }
}

impl Hp67RomWordSource for Hp67Firmware {
    fn read_word(&self, address: u16) -> Option<u16> {
        let pc = usize::from(address & 0x0fff);
        let image = words();

        if self.requested_bank.get() & 1 != 0
            && (BANK1_BASE_PC..BANK1_BASE_PC + BANK1_WORDS).contains(&pc)
        {
            return Some(image[BANK0_WORDS + pc - BANK1_BASE_PC]);
        }

        Some(image[pc])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn versioned_firmware_contains_the_complete_hp67_image() {
        let rom = Hp67Firmware::default();
        assert_eq!(Hp67Firmware::populated_words(), 5120);
        assert_eq!(words().len(), 5120);
        rom.select_bank(0);
        assert_eq!(rom.read_word(0x000), Some(0x000));
        assert_eq!(rom.read_word(0x001), Some(0x3e3));
        assert_eq!(rom.read_word(0x0f8), Some(0x11a));
    }

    #[test]
    fn bank_one_is_present_only_for_the_versioned_0400_through_07ff_window() {
        let rom = Hp67Firmware::default();
        rom.select_bank(0);
        let bank0_page0 = rom.read_word(0x000);
        let bank0_page1 = rom.read_word(0x400);

        rom.select_bank(1);
        assert_eq!(rom.read_word(0x000), bank0_page0);
        assert_ne!(rom.read_word(0x400), bank0_page1);
        assert_eq!(rom.read_word(0x800), Some(words()[0x800]));
    }
}
