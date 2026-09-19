//! HP-67 instruction-boundary bring-up composition of ACT, architectural RAM and CRC control.
//!
//! This layer exists to run long stretches of real firmware while preserving chip boundaries.
//! It is not the final electrical machine: serial fetch is already external to this module,
//! while CRC card-data ports and physical RAM/DATA timing remain explicit stop conditions.

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
    CrcDataPortNotModeled { pc: u16, address: u8, write: bool },
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
            return Err(Hp67ArchitecturalError::CrcDataPortNotModeled { pc, address, write });
        }

        self.execute_act_word(word)
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
    fn crc_data_port_is_an_explicit_hardware_boundary() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_READ_ADDRESS;
        let before = machine.clone();
        assert_eq!(
            machine.execute_word(0o0070),
            Err(Hp67ArchitecturalError::CrcDataPortNotModeled {
                pc: 0,
                address: CRC_RAM_READ_ADDRESS,
                write: false,
            })
        );
        assert_eq!(machine, before);
    }
}
