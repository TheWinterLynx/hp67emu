//! Build-embedded HP-67 firmware source.
//!
//! The validated research corpus is consumed only by `build.rs`. Cargo emits a
//! generated direct-address ROM table into `OUT_DIR`, and this module links that
//! table into every executable that uses the HP-67 machine. Runtime firmware
//! fetches therefore perform no filesystem or network I/O.

use std::cell::Cell;

use super::Hp67RomWordSource;

const WORDS_PER_BANK: usize = 4096;
const WORDS_PER_PAGE: usize = 1024;
const MISSING_WORD: u16 = 0xffff;

include!(concat!(env!("OUT_DIR"), "/hp67_embedded_rom.rs"));

#[derive(Debug, Clone, Default)]
pub struct EmbeddedHp67Rom {
    requested_bank: Cell<u8>,
}

impl EmbeddedHp67Rom {
    pub const fn populated_words() -> usize {
        EMBEDDED_HP67_POPULATED_WORDS
    }

    pub fn select_bank(&self, bank: u8) {
        self.requested_bank.set(bank & 1);
    }

    pub fn reset_bank(&self) {
        self.requested_bank.set(0);
    }
}

impl Hp67RomWordSource for EmbeddedHp67Rom {
    fn read_word(&self, address: u16) -> Option<u16> {
        let pc = usize::from(address & 0x0fff);
        let requested = usize::from(self.requested_bank.get() & 1);
        let page = pc / WORDS_PER_PAGE;
        let effective_bank = if EMBEDDED_HP67_PAGE_BANK_MASK[page] & (1u8 << requested) != 0 {
            requested
        } else {
            0
        };
        let word = EMBEDDED_HP67_WORDS[effective_bank * WORDS_PER_BANK + pc];
        (word != MISSING_WORD).then_some(word)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_firmware_contains_the_verified_hp67_corpus() {
        let rom = EmbeddedHp67Rom::default();
        assert_eq!(EmbeddedHp67Rom::populated_words(), 5120);
        rom.select_bank(0);
        assert_eq!(rom.read_word(0x000), Some(0x000));
        assert_eq!(rom.read_word(0x001), Some(0x3e3));
        assert_eq!(rom.read_word(0x0f8), Some(0x11a));
    }

    #[test]
    fn embedded_source_preserves_hp67_page_bank_fallback() {
        let rom = EmbeddedHp67Rom::default();
        rom.select_bank(1);
        assert_eq!(rom.read_word(0x000), Some(0x000));
        assert!(rom.read_word(0x400).is_some());
    }
}
