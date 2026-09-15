//! Minimal HP-67 ACT execution core for the first real-microcode power-on trace.
//!
//! This is intentionally not the finished 1820-2530 model. It implements only
//! source-backed architectural effects reached by the growing structural
//! power-on trace while those words are fetched through the serial IS/ISA path.
//! Unsupported opcodes fail loudly instead of falling back to the semantic
//! reference machine.

use super::isa::{ROM_ADDRESS_MASK, ROM_WORD_MASK};

pub const ACT_WORD_DIGITS: usize = 14;
const LOW_PAGE_MASK: u16 = 0x00ff;

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
    ExchangeCAndM2,
    DelayedSelectRom { rom: u8 },
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
/// The program counter and carry path are real Woodstock semantics. C, M1 and
/// M2 are present because the reconciled startup path has reached `0 -> c[w]`,
/// `c exchange m1` and `c exchange m2`. `delayed_rom` models the Woodstock rule
/// where a delayed ROM select takes effect only after the following word has
/// executed. Deterministic reset contents are not claimed as physical power-on
/// evidence; tests seed registers when contents matter to the operation tested.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PowerOnActCore {
    pc: u16,
    c: [u8; ACT_WORD_DIGITS],
    m1: [u8; ACT_WORD_DIGITS],
    m2: [u8; ACT_WORD_DIGITS],
    carry: bool,
    previous_carry: bool,
    delayed_rom: Option<u8>,
    executed_words: u64,
}

impl Default for PowerOnActCore {
    fn default() -> Self {
        Self {
            pc: 0,
            c: [0; ACT_WORD_DIGITS],
            m1: [0; ACT_WORD_DIGITS],
            m2: [0; ACT_WORD_DIGITS],
            carry: false,
            previous_carry: false,
            delayed_rom: None,
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

    pub const fn m2(&self) -> &[u8; ACT_WORD_DIGITS] {
        &self.m2
    }

    pub fn m2_mut(&mut self) -> &mut [u8; ACT_WORD_DIGITS] {
        &mut self.m2
    }

    pub const fn carry(&self) -> bool {
        self.carry
    }

    pub const fn previous_carry(&self) -> bool {
        self.previous_carry
    }

    pub const fn delayed_rom(&self) -> Option<u8> {
        self.delayed_rom
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

        // Keep failures transactional: probing an unsupported opcode must not
        // consume carry or a pending delayed-ROM selection.
        let original = self.clone();
        let execution_pc = self.pc;
        let prior_delayed_rom = self.delayed_rom.take();
        self.previous_carry = self.carry;
        self.carry = false;
        self.pc = self.pc.wrapping_add(1) & ROM_ADDRESS_MASK;

        let operation = if word == 0 {
            Some(PowerOnOperation::Nop)
        } else if word == 0o0410 {
            core::mem::swap(&mut self.c, &mut self.m1);
            Some(PowerOnOperation::ExchangeCAndM1)
        } else if word == 0o0610 {
            core::mem::swap(&mut self.c, &mut self.m2);
            Some(PowerOnOperation::ExchangeCAndM2)
        } else if word & 0x03 == 0 && word & 0o77 == 0o64 {
            let rom = ((word >> 6) & 0x0f) as u8;
            self.delayed_rom = Some(rom);
            Some(PowerOnOperation::DelayedSelectRom { rom })
        } else if word & 0x03 == 0x03 {
            let page_offset = (word >> 2) as u8;
            let target = (self.pc & 0x0f00) | u16::from(page_offset);
            let taken = !self.previous_carry;
            if taken {
                self.pc = target;
            }
            Some(PowerOnOperation::ConditionalGoto { taken, target })
        } else if word & 0x03 == 0x02 {
            let arithmetic_operation = ((word >> 5) & 0x1f) as u8;
            let field = ((word >> 2) & 0x07) as u8;
            if arithmetic_operation == 0x08 && field == 0x06 {
                self.c.fill(0);
                Some(PowerOnOperation::ZeroCWhole)
            } else {
                None
            }
        } else {
            None
        };

        let Some(operation) = operation else {
            *self = original;
            return Err(PowerOnActError::UnsupportedOpcode {
                pc: execution_pc,
                word,
            });
        };

        // A delayed select issued by the *previous* word changes the ROM page
        // only after this word has executed. Any control-flow result contributes
        // its low eight address bits before the delayed page is applied.
        if let Some(rom) = prior_delayed_rom {
            self.pc = (u16::from(rom & 0x0f) << 8) | (self.pc & LOW_PAGE_MASK);
        }

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
    fn c_exchange_m2_swaps_all_fourteen_digits() {
        let mut act = PowerOnActCore::default();
        for index in 0..ACT_WORD_DIGITS {
            act.c_mut()[index] = (index as u8) ^ 0x05;
            act.m2_mut()[index] = 0x0f - index as u8;
        }
        let old_c = *act.c();
        let old_m2 = *act.m2();

        let execution = act
            .execute_word(0o0610)
            .expect("c exchange m2 must execute");

        assert_eq!(execution.pc, 0x000);
        assert_eq!(execution.next_pc, 0x001);
        assert_eq!(execution.operation, PowerOnOperation::ExchangeCAndM2);
        assert_eq!(*act.c(), old_m2);
        assert_eq!(*act.m2(), old_c);
    }

    #[test]
    fn delayed_select_rom_waits_until_the_following_word_finishes() {
        let mut act = PowerOnActCore::default();

        // Follow the real startup control flow as far as the observed 0x0fd word.
        for word in [0x000, 0x3e3, 0x11a, 0o0410, 0x11a, 0o0610, 0x11a] {
            act.execute_word(word).expect("known startup prefix must execute");
        }
        assert_eq!(act.pc(), 0x0fd);

        let select = act
            .execute_word(0o0264)
            .expect("delayed select rom 2 must execute");
        assert_eq!(select.pc, 0x0fd);
        assert_eq!(select.next_pc, 0x0fe);
        assert_eq!(select.operation, PowerOnOperation::DelayedSelectRom { rom: 2 });
        assert_eq!(act.delayed_rom(), Some(2));

        // The next word is still fetched/executed at 0x0fe. Only when it
        // completes does the pending ROM number replace PC[11:8].
        let following = act.execute_word(0x000).expect("fixture NOP must execute");
        assert_eq!(following.pc, 0x0fe);
        assert_eq!(following.next_pc, 0x2ff);
        assert_eq!(act.delayed_rom(), None);
    }

    #[test]
    fn unsupported_word_is_transactional() {
        let mut act = PowerOnActCore::default();
        act.carry = true;
        act.delayed_rom = Some(3);
        let before = act.clone();

        assert_eq!(
            act.execute_word(0o0010),
            Err(PowerOnActError::UnsupportedOpcode {
                pc: 0,
                word: 0o0010,
            })
        );
        assert_eq!(act, before);
    }
}
