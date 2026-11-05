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

fn parse_blocks(blocks: [&str; 10]) -> Result<[u16; FIRMWARE_WORDS], String> {
    const BLOCK_WORDS: usize = 512;
    let mut image = [0u16; FIRMWARE_WORDS];
    for (block_index, block) in blocks.into_iter().enumerate() {
        let mut count = 0;
        for token in block
            .split(|ch: char| ch == ',' || ch.is_whitespace())
            .filter(|token| !token.is_empty())
        {
            if count == BLOCK_WORDS {
                return Err(format!(
                    "HP-67 firmware block {block_index:02} exceeds {BLOCK_WORDS} words"
                ));
            }
            let word = u16::from_str_radix(token, 8).map_err(|_| {
                format!("invalid HP-67 firmware word {token} in block {block_index:02} at {count}")
            })?;
            if word > WORD_MASK {
                return Err(format!(
                    "HP-67 firmware word exceeds 10 bits in block {block_index:02} at {count}"
                ));
            }
            image[block_index * BLOCK_WORDS + count] = word;
            count += 1;
        }
        if count != BLOCK_WORDS {
            return Err(format!("HP-67 firmware block {block_index:02} contains {count} words; expected {BLOCK_WORDS}"));
        }
    }
    Ok(image)
}

fn words() -> &'static [u16; FIRMWARE_WORDS] {
    WORDS.get_or_init(|| parse_blocks(BLOCKS).unwrap_or_else(|error| panic!("{error}")))
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
    fn read_word_in_bank(&self, bank: u8, address: u16) -> Option<u16> {
        let pc = usize::from(address);
        if pc >= BANK0_WORDS {
            return None;
        }
        match bank {
            0 => Some(words()[pc]),
            1 if (BANK1_BASE_PC..BANK1_BASE_PC + BANK1_WORDS).contains(&pc) => {
                Some(words()[BANK0_WORDS + pc - BANK1_BASE_PC])
            }
            _ => None,
        }
    }

    fn selected_rom_bank(&self, address: u16) -> u8 {
        // Preserve the existing image view outside the populated bank-1 window.
        // Reference-only compatibility view; live autonomous ROMs do not use it.
        if (0x400..0x800).contains(&address) {
            self.requested_bank.get() & 1
        } else {
            0
        }
    }

    fn read_word(&self, address: u16) -> Option<u16> {
        let pc = usize::from(address);
        if pc >= BANK0_WORDS {
            return None;
        }
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
    fn m15j_physical_rom_set_preserves_every_firmware_word_in_both_bank_views() {
        use super::super::rom_devices::Hp67RomChipSet;
        use crate::emulation::{Drive, LogicLevel};
        let source = Hp67Firmware::default();
        let mut chips = Hp67RomChipSet::default();
        for bank in 0..=1 {
            source.select_bank(bank);
            for address in 0..0x1000u16 {
                chips.begin_cycle();
                for bit in 16..28 {
                    let level = if address & (1 << (bit - 16)) != 0 {
                        LogicLevel::High
                    } else {
                        LogicLevel::Low
                    };
                    chips.sample_for_bit(bit, level, &source).unwrap();
                }
                assert_eq!(
                    chips.chips().iter().filter(|chip| chip.selected()).count(),
                    1
                );
                let mut received = 0u16;
                for bit in 46..56 {
                    if chips.trace_drive_for_bit(bit) == Drive::High {
                        received |= 1 << (bit - 46);
                    }
                }
                assert_eq!(
                    Some(received),
                    source.read_word(address),
                    "bank={bank} address={address:03x}"
                );
            }
        }
    }

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
    #[test]
    fn core_audit_firmware_block_boundaries_cannot_shift_while_total_stays_valid() {
        let mut blocks: Vec<String> = BLOCKS.iter().map(|block| block.to_string()).collect();
        let tokens: Vec<_> = blocks[0]
            .split(|ch: char| ch == ',' || ch.is_whitespace())
            .filter(|token| !token.is_empty())
            .map(str::to_owned)
            .collect();
        let moved = tokens.last().unwrap().clone();
        blocks[0] = tokens[..511].join(",");
        blocks[1] = format!("{moved},{}", blocks[1]);
        assert!(parse_blocks(std::array::from_fn(|i| blocks[i].as_str())).is_err());
        assert_eq!(parse_blocks(BLOCKS).unwrap(), *words());
    }

    #[test]
    fn core_audit_firmware_rejects_invalid_tokens_counts_and_words() {
        for bad in [
            "0,".repeat(513),
            "0,".repeat(511),
            format!("2000,{}", "0,".repeat(511)),
            format!("8,{}", "0,".repeat(511)),
        ] {
            let mut blocks = BLOCKS;
            blocks[9] = &bad;
            assert!(parse_blocks(blocks).is_err());
        }
    }

    #[test]
    fn core_audit_firmware_addresses_never_wrap_into_valid_rom() {
        let rom = Hp67Firmware::default();
        for address in [0x1000, 0x1400, u16::MAX] {
            for bank in [0, 1] {
                rom.select_bank(bank);
                assert_eq!(rom.read_word(address), None);
                assert_eq!(rom.read_word_in_bank(bank, address), None);
            }
        }
        assert!(rom.read_word(0xfff).is_some());
        assert_eq!(rom.read_word_in_bank(1, 0x800), None);
        assert_eq!(rom.read_word_in_bank(2, 0x400), None);
    }
}
