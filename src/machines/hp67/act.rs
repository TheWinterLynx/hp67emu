//! Minimal HP-67 ACT execution core for the first real-microcode power-on trace.
//!
//! This is intentionally not the finished 1820-2530 model. It implements only
//! source-backed architectural effects reached by the growing structural
//! power-on trace while those words are fetched through the serial IS/ISA path.
//! Unsupported opcodes fail loudly instead of falling back to the semantic
//! reference machine.

use super::isa::{ROM_ADDRESS_MASK, ROM_WORD_MASK};

pub const ACT_WORD_DIGITS: usize = 14;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerOnActError {
    OpcodeOutOfRange(u16),
    UnsupportedOpcode { pc: u16, word: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PowerOnOperation {
    Nop,
    ConditionalGoto { taken: bool, target: u16 },
    ZeroCWhole,
    ExchangeCAndM1,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PowerOnExecution {
    pub pc: u16,
    pub word: u16,
    pub next_pc: u16,
    pub operation: PowerOnOperation,
}

/// Small architectural ACT state used to grow the real-microcode startup trace.
///
/// The program counter and carry path are real Woodstock semantics. C and M1
/// are present because the reconciled startup path has now reached both
/// `0 -> c[w]` and `c exchange m1`. Their deterministic reset contents are not
/// claimed as physical power-on evidence; tests seed them when register contents
/// matter to the operation being verified.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerOnActCore {
    pc: u16,
    c: [u8; ACT_WORD_DIGITS],
    m1: [u8; ACT_WORD_DIGITS],
    carry: bool,
    previous_carry: bool,
    executed_words: u64,
}

impl Default for PowerOnActCore {
    fn default() -> Self {
        Self {
            pc: 0,
            c: [0; ACT_WORD_DIGITS],
            m1: [0; ACT_WORD_DIGITS],
            carry: false,
            previous_carry: false,
            executed_words: 0,
        }
    }
}

impl PowerOnActCore {
    pub const fn pc(&self) -> u16 {
        self.pc
    }

    pub const fn c(&self) -> &[u8; ACT_WORD_DIGITS] {
        &self.c
    }

    pub fn c_mut(&mut self) -> &mut [u8; ACT_WORD_DIGITS] {
        &mut self.c
    }

    pub const fn m1(&self) -> &[u8; ACT_WORD_DIGITS] {
        &self.m1
    }

    pub fn m1_mut(&mut self) -> &mut [u8; ACT_WORD_DIGITS] {
        &mut self.m1
    }

    pub const fn carry(&self) -> bool {
        self.carry
    }

    pub const fn previous_carry(&self) -> bool {
        self.previous_carry
    }

    pub const fn executed_words(&self) -> u64 {
        self.executed_words
    }

    /// Execute one already-fetched 10-bit word at the current architectural PC.
    ///
    /// Only source-backed forms already reached by the structural power-on path
    /// are accepted. This keeps unknown startup behavior visible while the ACT
    /// implementation grows one verified opcode family at a time.
    pub fn execute_word(&mut self, word: u16) -> Result<PowerOnExecution, PowerOnActError> {
        if word > ROM_WORD_MASK {
            return Err(PowerOnActError::OpcodeOutOfRange(word));
        }

        let execution_pc = self.pc;
        self.previous_carry = self.carry;
        self.carry = false;
        self.pc = self.pc.wrapping_add(1) & ROM_ADDRESS_MASK;

        let operation = if word == 0 {
            PowerOnOperation::Nop
        } else if word == 0o0410 {
            core::mem::swap(&mut self.c, &mut self.m1);
            PowerOnOperation::ExchangeCAndM1
        } else if word & 0x03 == 0x03 {
            let page_offset = (word >> 2) as u8;
            let target = (self.pc & 0x0f00) | u16::from(page_offset);
            let taken = !self.previous_carry;
            if taken {
                self.pc = target;
            }
            PowerOnOperation::ConditionalGoto { taken, target }
        } else if word & 0x03 == 0x02 {
            let arithmetic_operation = ((word >> 5) & 0x1f) as u8;
            let field = ((word >> 2) & 0x07) as u8;
            if arithmetic_operation == 0x08 && field == 0x06 {
                self.c.fill(0);
                PowerOnOperation::ZeroCWhole
            } else {
                self.pc = execution_pc;
                return Err(PowerOnActError::UnsupportedOpcode {
                    pc: execution_pc,
                    word,
                });
            }
        } else {
            self.pc = execution_pc;
            return Err(PowerOnActError::UnsupportedOpcode {
                pc: execution_pc,
                word,
            });
        };

        self.executed_words = self.executed_words.wrapping_add(1);
        Ok(PowerOnExecution {
            pc: execution_pc,
            word,
            next_pc: self.pc,
            operation,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn physical_startup_words_reach_0f8_and_clear_c() {
        let mut act = PowerOnActCore::default();
        act.c_mut().fill(9);

        let nop = act.execute_word(0x000).expect("startup NOP must execute");
        assert_eq!(nop.pc, 0x000);
        assert_eq!(nop.next_pc, 0x001);
        assert_eq!(nop.operation, PowerOnOperation::Nop);

        let branch = act
            .execute_word(0x3e3)
            .expect("startup conditional GOTO must execute");
        assert_eq!(branch.pc, 0x001);
        assert_eq!(
            branch.operation,
            PowerOnOperation::ConditionalGoto {
                taken: true,
                target: 0x0f8,
            }
        );
        assert_eq!(branch.next_pc, 0x0f8);

        let clear = act
            .execute_word(0x11a)
            .expect("startup 0 -> c[w] must execute");
        assert_eq!(clear.pc, 0x0f8);
        assert_eq!(clear.operation, PowerOnOperation::ZeroCWhole);
        assert_eq!(clear.next_pc, 0x0f9);
        assert!(act.c().iter().all(|digit| *digit == 0));
        assert_eq!(act.executed_words(), 3);
    }

    #[test]
    fn c_exchange_m1_swaps_all_fourteen_digits() {
        let mut act = PowerOnActCore::default();
        for index in 0..ACT_WORD_DIGITS {
            act.c_mut()[index] = index as u8;
            act.m1_mut()[index] = 0x0f - index as u8;
        }
        let old_c = *act.c();
        let old_m1 = *act.m1();

        let execution = act
            .execute_word(0o0410)
            .expect("c exchange m1 must execute");

        assert_eq!(execution.pc, 0x000);
        assert_eq!(execution.next_pc, 0x001);
        assert_eq!(execution.operation, PowerOnOperation::ExchangeCAndM1);
        assert_eq!(*act.c(), old_m1);
        assert_eq!(*act.m1(), old_c);
    }

    #[test]
    fn unsupported_word_fails_without_advancing_pc() {
        let mut act = PowerOnActCore::default();
        assert_eq!(
            act.execute_word(0o0010),
            Err(PowerOnActError::UnsupportedOpcode {
                pc: 0,
                word: 0o0010,
            })
        );
        assert_eq!(act.pc(), 0);
        assert_eq!(act.executed_words(), 0);
    }
}
