//! HP-67 instruction-boundary bring-up composition of ACT, architectural RAM and CRC control.
//!
//! This layer exists to run long stretches of real firmware while preserving chip boundaries.
//! It is not the final electrical machine: serial fetch is already external to this module,
//! while physical RAM/DATA timing remains an explicit lower-level boundary.

use super::{
    act::{
        ActArchitecturalCore, ActError, ActExecution, ActInstructionState, ActOperation,
        ActRamImage,
    },
    crc::{
        decode_crc_opcode, CrcArchitecturalCore, CrcArchitecturalError, CrcInstruction,
        CRC_FLAG_CARD_PRESENT, CRC_FLAG_PROGRAM_MODE, CRC_RAM_READ_ADDRESS, CRC_RAM_WRITE_ADDRESS,
    },
    isa::ROM_WORD_MASK,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ArchitecturalOperation {
    Act(ActOperation),
    CrcControl {
        instruction: CrcInstruction,
        condition: Option<bool>,
    },
    CrcDataRead {
        address: u8,
        card_word: u32,
    },
    CrcDataWrite {
        address: u8,
        card_word: u32,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67ArchitecturalExecution {
    pub pc: u16,
    pub word: u16,
    pub next_pc: u16,
    pub operation: Hp67ArchitecturalOperation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ArchitecturalError {
    OpcodeOutOfRange(u16),
    Act(ActError),
    Crc(CrcArchitecturalError),
}

impl From<ActError> for Hp67ArchitecturalError {
    fn from(value: ActError) -> Self {
        Self::Act(value)
    }
}

impl From<CrcArchitecturalError> for Hp67ArchitecturalError {
    fn from(value: CrcArchitecturalError) -> Self {
        Self::Crc(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67ArchitecturalMachine {
    pub act: ActArchitecturalCore,
    pub ram: ActRamImage,
    pub crc: CrcArchitecturalCore,
}

impl Default for Hp67ArchitecturalMachine {
    fn default() -> Self {
        Self {
            act: ActArchitecturalCore::default(),
            ram: ActRamImage::hp67(),
            crc: CrcArchitecturalCore::default(),
        }
    }
}

impl Hp67ArchitecturalMachine {
    pub const fn pc(&self) -> u16 {
        self.act.pc()
    }

    pub const fn bank(&self) -> u8 {
        self.act.bank()
    }

    pub const fn executed_words(&self) -> u64 {
        self.act.executed_words()
    }

    pub fn prepare_hp67_fetch(&mut self) -> u8 {
        self.act.prepare_hp67_fetch()
    }

    pub fn set_program_mode(&mut self, program: bool) -> Result<(), Hp67ArchitecturalError> {
        self.crc
            .set_external_flag(CRC_FLAG_PROGRAM_MODE as u8, program)?;
        Ok(())
    }

    pub fn set_card_present(&mut self, present: bool) -> Result<(), Hp67ArchitecturalError> {
        self.crc
            .set_external_flag(CRC_FLAG_CARD_PRESENT as u8, present)?;
        Ok(())
    }

    /// Execute one already-fetched HP-67 word with ACT/CRC ownership resolved.
    ///
    /// CRC opcodes are external peripheral commands, not ACT specials. The ACT still performs
    /// its universal instruction-boundary work by consuming a NOP-equivalent cycle. A word
    /// following an IF is always branch data and therefore bypasses CRC decode entirely.
    pub fn execute_word(
        &mut self,
        word: u16,
    ) -> Result<Hp67ArchitecturalExecution, Hp67ArchitecturalError> {
        if word > ROM_WORD_MASK {
            return Err(Hp67ArchitecturalError::OpcodeOutOfRange(word));
        }

        let checkpoint = self.clone();
        let result = self.execute_word_inner(word);
        if result.is_err() {
            *self = checkpoint;
        }
        result
    }

    fn execute_word_inner(
        &mut self,
        word: u16,
    ) -> Result<Hp67ArchitecturalExecution, Hp67ArchitecturalError> {
        let pc = self.act.pc();

        if self.act.state.instruction_state == ActInstructionState::ThenGoto {
            return self.execute_act_word(word);
        }

        if let Some(instruction) = decode_crc_opcode(word)? {
            let boundary = self.act.execute_word(&mut self.ram, 0o0000)?;
            let condition = self.crc.execute_opcode(word)?;
            if condition == Some(true) {
                // A true CRC flag test pulses ACT F2; the ACT latches that pulse into S3.
                self.act.state.status[3] = true;
            }
            return Ok(Hp67ArchitecturalExecution {
                pc,
                word,
                next_pc: boundary.next_pc,
                operation: Hp67ArchitecturalOperation::CrcControl {
                    instruction,
                    condition,
                },
            });
        }

        if let Some((address, write)) = self.pending_crc_data_access(word) {
            if write {
                return self.execute_crc_data_write(word, address);
            }
            return self.execute_crc_data_read(word, address);
        }

        self.execute_act_word(word)
    }

    fn execute_crc_data_read(
        &mut self,
        word: u16,
        address: u8,
    ) -> Result<Hp67ArchitecturalExecution, Hp67ArchitecturalError> {
        let boundary = self.act.execute_word(&mut self.ram, word)?;
        let card_word = self.crc.take_read_word()?;

        for digit in 0..7 {
            let nibble = ((card_word >> (digit * 4)) & 0x0f) as u8;
            self.act.state.c[digit] = nibble;
            self.act.state.c[7 + digit] = nibble;
        }

        Ok(Hp67ArchitecturalExecution {
            pc: boundary.pc,
            word,
            next_pc: boundary.next_pc,
            operation: Hp67ArchitecturalOperation::CrcDataRead { address, card_word },
        })
    }

    fn execute_crc_data_write(
        &mut self,
        word: u16,
        address: u8,
    ) -> Result<Hp67ArchitecturalExecution, Hp67ArchitecturalError> {
        let boundary = self.act.execute_word(&mut self.ram, word)?;

        let mut card_word = 0u32;
        for digit in (7..14).rev() {
            card_word = (card_word << 4) | u32::from(self.act.state.c[digit] & 0x0f);
        }
        self.crc.queue_write_word(card_word)?;

        Ok(Hp67ArchitecturalExecution {
            pc: boundary.pc,
            word,
            next_pc: boundary.next_pc,
            operation: Hp67ArchitecturalOperation::CrcDataWrite { address, card_word },
        })
    }

    fn execute_act_word(
        &mut self,
        word: u16,
    ) -> Result<Hp67ArchitecturalExecution, Hp67ArchitecturalError> {
        let ActExecution {
            pc,
            word,
            next_pc,
            operation,
        } = self.act.execute_word(&mut self.ram, word)?;
        Ok(Hp67ArchitecturalExecution {
            pc,
            word,
            next_pc,
            operation: Hp67ArchitecturalOperation::Act(operation),
        })
    }

    fn pending_crc_data_access(&self, word: u16) -> Option<(u8, bool)> {
        let current = self.act.state.ram_address;

        if word == 0o0070 && current == CRC_RAM_READ_ADDRESS {
            return Some((current, false));
        }
        if word == 0o1360 && current == CRC_RAM_WRITE_ADDRESS {
            return Some((current, true));
        }

        let family = word & 0o77;
        let operand = ((word >> 6) & 0x0f) as u8;
        let selected = (current & 0xf0) | operand;

        if word != 0o0070 && family == 0o70 && selected == CRC_RAM_READ_ADDRESS {
            return Some((selected, false));
        }
        if family == 0o50 && selected == CRC_RAM_WRITE_ADDRESS {
            return Some((selected, true));
        }
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn startup_crc_set_flag_is_not_treated_as_unknown_act_special() {
        let mut machine = Hp67ArchitecturalMachine::default();
        let execution = machine
            .execute_word(0o1000)
            .expect("CRC set flag must execute");
        assert_eq!(execution.pc, 0);
        assert_eq!(execution.next_pc, 1);
        assert_eq!(machine.crc.flag(4), Some(true));
        assert_eq!(machine.executed_words(), 1);
    }

    #[test]
    fn crc_program_switch_test_pulses_act_s3() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine
            .set_program_mode(true)
            .expect("program switch must exist");
        machine.execute_word(0o300).expect("CRC test must execute");
        assert!(machine.act.state.status[3]);
        assert_eq!(machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(true));
    }

    #[test]
    fn crc_card_present_switch_pulses_act_s3_without_clearing_external_contact() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine
            .set_card_present(true)
            .expect("card-present switch must exist");
        machine.execute_word(0o560).expect("CRC test must execute");
        assert!(machine.act.state.status[3]);
        assert_eq!(machine.crc.external_flag(CRC_FLAG_CARD_PRESENT), Some(true));
    }

    #[test]
    fn then_goto_data_bypasses_crc_decoder() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.pc = 0x800;
        machine.act.state.instruction_state = ActInstructionState::ThenGoto;
        machine.act.state.carry = false;

        machine
            .execute_word(0o260)
            .expect("branch data must execute");
        assert_eq!(machine.act.state.pc, 0x8b0);
        assert_eq!(machine.crc.flag(9), Some(false));
    }

    #[test]
    fn crc_read_port_consumes_transport_word_and_duplicates_seven_nibbles() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_READ_ADDRESS;
        machine
            .crc
            .present_read_word(0x0765_4321)
            .expect("transport word must latch");

        let execution = machine
            .execute_word(0o0070)
            .expect("CRC read port must execute");
        assert_eq!(
            execution.operation,
            Hp67ArchitecturalOperation::CrcDataRead {
                address: CRC_RAM_READ_ADDRESS,
                card_word: 0x0765_4321,
            }
        );
        assert_eq!(&machine.act.state.c[0..7], &[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(&machine.act.state.c[7..14], &[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(machine.crc.buffered_read_word(), None);
    }

    #[test]
    fn firmware_register_read_11_selects_crc_0x9b_and_consumes_record() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_WRITE_ADDRESS;
        machine
            .crc
            .present_read_word(0x0123_4567)
            .expect("transport word must latch");

        let execution = machine
            .execute_word(0o1370)
            .expect("register -> c 11 must read CRC buffer");
        assert_eq!(machine.act.state.ram_address, CRC_RAM_READ_ADDRESS);
        assert_eq!(
            execution.operation,
            Hp67ArchitecturalOperation::CrcDataRead {
                address: CRC_RAM_READ_ADDRESS,
                card_word: 0x0123_4567,
            }
        );
        assert_eq!(&machine.act.state.c[0..7], &[7, 6, 5, 4, 3, 2, 1]);
        assert_eq!(&machine.act.state.c[7..14], &[7, 6, 5, 4, 3, 2, 1]);
    }

    #[test]
    fn crc_read_port_without_transport_word_is_transactional() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_READ_ADDRESS;
        let before = machine.clone();
        assert_eq!(
            machine.execute_word(0o0070),
            Err(Hp67ArchitecturalError::Crc(
                CrcArchitecturalError::ReadBufferEmpty
            ))
        );
        assert_eq!(machine, before);
    }

    #[test]
    fn crc_write_port_packs_only_high_half_of_c_into_28_bit_buffer() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_WRITE_ADDRESS;
        machine
            .crc
            .execute_opcode(0o660)
            .expect("write mode must set");

        for digit in 0..7 {
            machine.act.state.c[digit] = 0x0f;
            machine.act.state.c[7 + digit] = (digit + 1) as u8;
        }

        let execution = machine
            .execute_word(0o1360)
            .expect("CRC write port must accept high half");
        assert_eq!(
            execution.operation,
            Hp67ArchitecturalOperation::CrcDataWrite {
                address: CRC_RAM_WRITE_ADDRESS,
                card_word: 0x0765_4321,
            }
        );
        assert_eq!(machine.crc.queued_write_words(), 1);
        assert_eq!(machine.crc.take_queued_write_word(), Some(0x0765_4321));
    }

    #[test]
    fn crc_write_port_requires_firmware_write_mode_transactionally() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_WRITE_ADDRESS;
        let before = machine.clone();
        assert_eq!(
            machine.execute_word(0o1360),
            Err(Hp67ArchitecturalError::Crc(
                CrcArchitecturalError::WriteModeInactive
            ))
        );
        assert_eq!(machine, before);
    }
}
