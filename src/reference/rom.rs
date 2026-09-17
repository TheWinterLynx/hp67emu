//! Host-independent Woodstock ROM image and HP-67 fetch helper.
//!
//! ROM bytes/words are supplied by the caller. This module deliberately does
//! not perform filesystem I/O so tests can construct tiny fixtures.

use super::woodstock::{
    ArchitecturalState, ExecutionError, ReferenceMachine, BANK_COUNT, OPCODE_MASK, PAGE_COUNT,
    PAGE_WORDS, PC_MASK,
};

const ROM_WORDS: usize = BANK_COUNT * PAGE_COUNT * PAGE_WORDS;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomError {
    BankOutOfRange(u8),
    PageOutOfRange(u8),
    AddressOutOfRange(u16),
    OpcodeOutOfRange(u16),
    PageTooLarge(usize),
    MissingWord { bank: u8, address: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomRunError {
    Rom(RomError),
    Execute(ExecutionError),
}

impl From<RomError> for RomRunError {
    fn from(value: RomError) -> Self {
        Self::Rom(value)
    }
}

impl From<ExecutionError> for RomRunError {
    fn from(value: ExecutionError) -> Self {
        Self::Execute(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomImage {
    words: Vec<Option<u16>>,
    page_bank_mask: [u8; PAGE_COUNT],
}

impl Default for RomImage {
    fn default() -> Self {
        Self {
            words: vec![None; ROM_WORDS],
            page_bank_mask: [0; PAGE_COUNT],
        }
    }
}

impl RomImage {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn install_word(&mut self, bank: u8, address: u16, opcode: u16) -> Result<(), RomError> {
        let index = Self::index(bank, address)?;
        if opcode > OPCODE_MASK {
            return Err(RomError::OpcodeOutOfRange(opcode));
        }

        self.words[index] = Some(opcode);
        let page = usize::from(address) / PAGE_WORDS;
        self.page_bank_mask[page] |= 1 << bank;
        Ok(())
    }

    pub fn install_page(&mut self, bank: u8, page: u8, words: &[u16]) -> Result<(), RomError> {
        if usize::from(page) >= PAGE_COUNT {
            return Err(RomError::PageOutOfRange(page));
        }
        if words.len() > PAGE_WORDS {
            return Err(RomError::PageTooLarge(words.len()));
        }

        let base = usize::from(page) * PAGE_WORDS;
        for (offset, opcode) in words.iter().copied().enumerate() {
            self.install_word(bank, (base + offset) as u16, opcode)?;
        }
        Ok(())
    }

    pub fn read_word(&self, bank: u8, address: u16) -> Result<Option<u16>, RomError> {
        Ok(self.words[Self::index(bank, address)?])
    }

    pub fn page_has_bank(&self, page: u8, bank: u8) -> Result<bool, RomError> {
        if usize::from(page) >= PAGE_COUNT {
            return Err(RomError::PageOutOfRange(page));
        }
        if usize::from(bank) >= BANK_COUNT {
            return Err(RomError::BankOutOfRange(bank));
        }
        Ok((self.page_bank_mask[usize::from(page)] & (1 << bank)) != 0)
    }

    pub fn effective_bank(&self, state: &ArchitecturalState) -> u8 {
        let page = usize::from(state.pc & PC_MASK) / PAGE_WORDS;
        let requested = state.bank & 0x01;
        if (self.page_bank_mask[page] & (1 << requested)) != 0 {
            requested
        } else {
            0
        }
    }

    pub fn fetch(&self, state: &ArchitecturalState) -> Result<u16, RomError> {
        let address = state.pc & PC_MASK;
        let bank = self.effective_bank(state);
        self.read_word(bank, address)?
            .ok_or(RomError::MissingWord { bank, address })
    }

    /// Fetches and executes one HP-67 microinstruction.
    ///
    /// The HP-67 path applies the machine rule that bank 1 cannot remain
    /// selected while executing page 0. The rule is applied before the fetch,
    /// matching the architectural boundary used by the semantic machine.
    pub fn step_hp67(&self, machine: &mut ReferenceMachine) -> Result<u16, RomRunError> {
        if machine.cpu.pc < 0o2000 && machine.cpu.bank == 1 {
            machine.cpu.bank = 0;
        }
        let opcode = self.fetch(&machine.cpu)?;
        machine.step_word(opcode)?;
        Ok(opcode)
    }

    fn index(bank: u8, address: u16) -> Result<usize, RomError> {
        if usize::from(bank) >= BANK_COUNT {
            return Err(RomError::BankOutOfRange(bank));
        }
        if address > PC_MASK {
            return Err(RomError::AddressOutOfRange(address));
        }
        Ok(usize::from(bank) * PAGE_COUNT * PAGE_WORDS + usize::from(address))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn page_installation_is_independent_of_filesystem_io() {
        let mut rom = RomImage::new();
        rom.install_page(0, 0, &[0o0000, 0o0010, 0o0420])
            .expect("fixture page must install");

        assert_eq!(rom.read_word(0, 0).expect("address is valid"), Some(0o0000));
        assert_eq!(rom.read_word(0, 1).expect("address is valid"), Some(0o0010));
        assert_eq!(rom.read_word(0, 2).expect("address is valid"), Some(0o0420));
        assert_eq!(rom.read_word(0, 3).expect("address is valid"), None);
    }

    #[test]
    fn absent_requested_bank_falls_back_to_bank_zero() {
        let mut rom = RomImage::new();
        rom.install_word(0, 0x400, 0o0000)
            .expect("bank zero word must install");

        let state = ArchitecturalState {
            pc: 0x400,
            bank: 1,
            ..ArchitecturalState::default()
        };

        assert_eq!(rom.effective_bank(&state), 0);
        assert_eq!(rom.fetch(&state), Ok(0o0000));
    }

    #[test]
    fn populated_bank_one_is_selected_when_present() {
        let mut rom = RomImage::new();
        rom.install_word(0, 0x400, 0o0000)
            .expect("bank zero word must install");
        rom.install_word(1, 0x400, 0o0420)
            .expect("bank one word must install");

        let state = ArchitecturalState {
            pc: 0x400,
            bank: 1,
            ..ArchitecturalState::default()
        };

        assert_eq!(rom.effective_bank(&state), 1);
        assert_eq!(rom.fetch(&state), Ok(0o0420));
    }

    #[test]
    fn hp67_step_forces_bank_zero_in_page_zero_before_fetch() {
        let mut rom = RomImage::new();
        rom.install_word(0, 0, 0o0000)
            .expect("bank zero reset vector must install");
        rom.install_word(1, 0, 0o0420)
            .expect("bank one fixture must install");

        let mut machine = ReferenceMachine::default();
        machine.cpu.bank = 1;
        machine.cpu.pc = 0;

        assert_eq!(rom.step_hp67(&mut machine), Ok(0o0000));
        assert_eq!(machine.cpu.bank, 0);
        assert_eq!(machine.cpu.pc, 1);
        assert!(machine.cpu.decimal);
    }

    #[test]
    fn invalid_words_are_rejected_at_install_time() {
        let mut rom = RomImage::new();
        assert_eq!(
            rom.install_word(0, 0, OPCODE_MASK + 1),
            Err(RomError::OpcodeOutOfRange(OPCODE_MASK + 1))
        );
        assert_eq!(
            rom.install_word(BANK_COUNT as u8, 0, 0),
            Err(RomError::BankOutOfRange(BANK_COUNT as u8))
        );
        assert_eq!(
            rom.install_word(0, PC_MASK + 1, 0),
            Err(RomError::AddressOutOfRange(PC_MASK + 1))
        );
    }
}
