//! Intra-word ACT execution lifetime for the HP-67.
//!
//! This module gives one executing Woodstock word an explicit lifetime across
//! the canonical b0..b55 machine-word coordinate. It intentionally does not
//! invent internal register-write or PHI-edge timing that is not yet supported
//! by HP-67/1820-2530 evidence.

use super::act::ActInstructionState;
use super::isa::ROM_WORD_MASK;
use super::timing::{BITS_PER_DIGIT, BITS_PER_WORD};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActSerialWordClass {
    ThenGotoData,
    SpecialOrPeripheral { opcode: u16 },
    Jsb { offset: u8 },
    Arithmetic { operation: u8, field: u8 },
    Goto { offset: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActSerialExecutionError {
    OpcodeOutOfRange(u16),
    UnexpectedWordBit { expected: u8, actual: u8 },
    ExecutionAlreadyComplete,
    ExecutionAlreadyActive { word: u16, next_word_bit: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialExecution {
    word: u16,
    class: ActSerialWordClass,
    next_word_bit: u8,
}

impl ActSerialExecution {
    pub fn new(
        word: u16,
        instruction_state: ActInstructionState,
    ) -> Result<Self, ActSerialExecutionError> {
        if word > ROM_WORD_MASK {
            return Err(ActSerialExecutionError::OpcodeOutOfRange(word));
        }

        let class = if instruction_state == ActInstructionState::ThenGoto {
            ActSerialWordClass::ThenGotoData
        } else {
            match word & 0x03 {
                0 => ActSerialWordClass::SpecialOrPeripheral { opcode: word },
                1 => ActSerialWordClass::Jsb {
                    offset: (word >> 2) as u8,
                },
                2 => ActSerialWordClass::Arithmetic {
                    operation: ((word >> 5) & 0x1f) as u8,
                    field: ((word >> 2) & 0x07) as u8,
                },
                3 => ActSerialWordClass::Goto {
                    offset: (word >> 2) as u8,
                },
                _ => unreachable!("two-bit Woodstock opcode class"),
            }
        };

        Ok(Self {
            word,
            class,
            next_word_bit: 0,
        })
    }

    pub const fn word(&self) -> u16 {
        self.word
    }

    pub const fn class(&self) -> ActSerialWordClass {
        self.class
    }

    pub const fn is_complete(&self) -> bool {
        self.next_word_bit == BITS_PER_WORD
    }

    pub const fn next_word_bit(&self) -> Option<u8> {
        if self.is_complete() {
            None
        } else {
            Some(self.next_word_bit)
        }
    }

    pub const fn digit_coordinate(&self) -> Option<u8> {
        match self.next_word_bit() {
            Some(word_bit) => Some(word_bit / BITS_PER_DIGIT),
            None => None,
        }
    }

    pub const fn bit_in_digit(&self) -> Option<u8> {
        match self.next_word_bit() {
            Some(word_bit) => Some(word_bit % BITS_PER_DIGIT),
            None => None,
        }
    }

    pub fn advance_word_bit(&mut self, actual: u8) -> Result<(), ActSerialExecutionError> {
        if self.is_complete() {
            return Err(ActSerialExecutionError::ExecutionAlreadyComplete);
        }
        if actual != self.next_word_bit {
            return Err(ActSerialExecutionError::UnexpectedWordBit {
                expected: self.next_word_bit,
                actual,
            });
        }

        self.next_word_bit += 1;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn arithmetic_word_has_one_full_56_bit_execution_lifetime() {
        let mut execution = ActSerialExecution::new(0x11a, ActInstructionState::Normal)
            .expect("known arithmetic word must decode");

        assert_eq!(
            execution.class(),
            ActSerialWordClass::Arithmetic {
                operation: 8,
                field: 6,
            }
        );
        for expected_bit in 0..BITS_PER_WORD {
            assert_eq!(execution.next_word_bit(), Some(expected_bit));
            assert_eq!(
                execution.digit_coordinate(),
                Some(expected_bit / BITS_PER_DIGIT)
            );
            assert_eq!(
                execution.bit_in_digit(),
                Some(expected_bit % BITS_PER_DIGIT)
            );
            execution
                .advance_word_bit(expected_bit)
                .expect("sequential bit coordinate must advance");
        }

        assert!(execution.is_complete());
        assert_eq!(execution.next_word_bit(), None);
        assert_eq!(execution.digit_coordinate(), None);
        assert_eq!(execution.bit_in_digit(), None);
    }

    #[test]
    fn then_goto_state_treats_the_returned_word_as_branch_data() {
        let execution = ActSerialExecution::new(0x260, ActInstructionState::ThenGoto)
            .expect("ten-bit branch data must be accepted");
        assert_eq!(execution.class(), ActSerialWordClass::ThenGotoData);
    }

    #[test]
    fn execution_rejects_out_of_order_bit_coordinates() {
        let mut execution = ActSerialExecution::new(0, ActInstructionState::Normal).unwrap();
        assert_eq!(
            execution.advance_word_bit(1),
            Err(ActSerialExecutionError::UnexpectedWordBit {
                expected: 0,
                actual: 1,
            })
        );
        assert_eq!(execution.next_word_bit(), Some(0));
    }

    #[test]
    fn execution_rejects_words_wider_than_the_physical_rom_word() {
        let result = ActSerialExecution::new(ROM_WORD_MASK + 1, ActInstructionState::Normal);
        assert_eq!(
            result,
            Err(ActSerialExecutionError::OpcodeOutOfRange(ROM_WORD_MASK + 1))
        );
    }
}
