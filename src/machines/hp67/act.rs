//! Independent HP-67 ACT architectural bring-up core.
//!
//! This module is the fast correctness bridge between the already-validated
//! serial ACT<->ROM transport and the future pin/timing-accurate 1820-2530.
//! Unlike the semantic reference model, production HP-67 code imports no
//! reference implementation. The core implements the complete Woodstock
//! instruction-boundary behavior needed by HP-67 firmware so long power-on runs
//! can proceed now; electrical PHI edges, bit-serial ALU timing, DATA timing and
//! physical RAM devices remain separate later milestones.

use super::isa::{ROM_ADDRESS_MASK, ROM_WORD_MASK};

pub const ACT_WORD_DIGITS: usize = 14;
pub const ACT_STATUS_BITS: usize = 16;
pub const ACT_RETURN_STACK_DEPTH: usize = 2;
pub const ACT_RAM_WORDS: usize = 256;

const LOW_ROM_MASK: u16 = 0x00ff;
const THEN_GOTO_MASK: u16 = 0x03ff;
const P_SET_MAP: [u8; 16] = [14, 4, 7, 8, 11, 2, 10, 12, 1, 3, 13, 6, 0, 9, 5, 14];
const P_TEST_MAP: [u8; 16] = [4, 8, 12, 2, 9, 1, 6, 3, 1, 13, 5, 0, 11, 10, 7, 4];

pub type ActRegister = [u8; ACT_WORD_DIGITS];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum ActInstructionState {
    #[default]
    Normal,
    ThenGoto,
}

/// Instruction-boundary ACT state used by the low-level bring-up path.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActArchitecturalState {
    pub a: ActRegister,
    pub b: ActRegister,
    pub c: ActRegister,
    pub y: ActRegister,
    pub z: ActRegister,
    pub t: ActRegister,
    pub m1: ActRegister,
    pub m2: ActRegister,
    pub f: u8,
    pub p: u8,
    pub p_change: [i8; 3],
    pub decimal: bool,
    pub carry: bool,
    pub previous_carry: bool,
    pub status: [bool; ACT_STATUS_BITS],
    pub pc: u16,
    pub delayed_rom: Option<u8>,
    pub bank: u8,
    pub return_stack: [u16; ACT_RETURN_STACK_DEPTH],
    pub stack_pointer: u8,
    pub instruction_state: ActInstructionState,
    pub key_buffer: Option<u8>,
    pub display_enable: bool,
    pub display_14_digit: bool,
    pub ram_address: u8,
}

impl Default for ActArchitecturalState {
    fn default() -> Self {
        Self {
            a: [0; ACT_WORD_DIGITS],
            b: [0; ACT_WORD_DIGITS],
            c: [0; ACT_WORD_DIGITS],
            y: [0; ACT_WORD_DIGITS],
            z: [0; ACT_WORD_DIGITS],
            t: [0; ACT_WORD_DIGITS],
            m1: [0; ACT_WORD_DIGITS],
            m2: [0; ACT_WORD_DIGITS],
            f: 0,
            p: 0,
            p_change: [0; 3],
            decimal: true,
            carry: false,
            previous_carry: false,
            status: [false; ACT_STATUS_BITS],
            pc: 0,
            delayed_rom: None,
            bank: 0,
            return_stack: [0; ACT_RETURN_STACK_DEPTH],
            stack_pointer: 0,
            instruction_state: ActInstructionState::Normal,
            key_buffer: None,
            display_enable: false,
            display_14_digit: false,
            ram_address: 0,
        }
    }
}

impl ActArchitecturalState {
    pub fn normalize_widths(&mut self) {
        self.f &= 0x0f;
        self.p &= 0x0f;
        self.pc &= ROM_ADDRESS_MASK;
        self.bank &= 1;
        self.stack_pointer %= ACT_RETURN_STACK_DEPTH as u8;
        if let Some(rom) = &mut self.delayed_rom {
            *rom &= 0x0f;
        }
        for return_address in &mut self.return_stack {
            *return_address &= ROM_ADDRESS_MASK;
        }
        for register in [
            &mut self.a,
            &mut self.b,
            &mut self.c,
            &mut self.y,
            &mut self.z,
            &mut self.t,
            &mut self.m1,
            &mut self.m2,
        ] {
            for digit in register {
                *digit &= 0x0f;
            }
        }
    }
}

/// Architectural RAM image used only while physical 1818-* RAM timing is not
/// connected. Keeping it outside the ACT state preserves the chip boundary.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ActRamImage {
    slots: [Option<ActRegister>; ACT_RAM_WORDS],
}

impl Default for ActRamImage {
    fn default() -> Self {
        Self {
            slots: [None; ACT_RAM_WORDS],
        }
    }
}

impl ActRamImage {
    pub fn hp67() -> Self {
        let mut ram = Self::default();
        ram.install_range(0x00, 0x40);
        ram
    }

    pub fn install_range(&mut self, start: u8, count: usize) {
        let first = usize::from(start);
        let end = first.saturating_add(count).min(ACT_RAM_WORDS);
        for slot in &mut self.slots[first..end] {
            *slot = Some([0; ACT_WORD_DIGITS]);
        }
    }

    pub fn read(&self, address: u8) -> Option<ActRegister> {
        self.slots[usize::from(address)]
    }

    pub fn write(&mut self, address: u8, value: ActRegister) -> bool {
        let slot = &mut self.slots[usize::from(address)];
        if slot.is_none() {
            return false;
        }
        *slot = Some(value);
        true
    }

    pub fn clear_block(&mut self, base: u8) {
        let first = usize::from(base & 0xf0);
        for slot in &mut self.slots[first..first + 16] {
            if slot.is_some() {
                *slot = Some([0; ACT_WORD_DIGITS]);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActError {
    OpcodeOutOfRange(u16),
    UnknownSpecial { pc: u16, word: u16 },
    UnsupportedRomSelfTest { pc: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActOperation {
    ThenGoto { taken: bool, target: u16 },
    Special { opcode: u16 },
    Jsb { target: u16 },
    Arithmetic { operation: u8, field: u8 },
    Goto { taken: bool, target: u16 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActExecution {
    pub pc: u16,
    pub word: u16,
    pub next_pc: u16,
    pub operation: ActOperation,
}

/// Complete instruction-boundary Woodstock ACT semantics used by HP-67 bring-up.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ActArchitecturalCore {
    pub state: ActArchitecturalState,
    executed_words: u64,
}

impl ActArchitecturalCore {
    pub const fn pc(&self) -> u16 {
        self.state.pc
    }

    pub const fn bank(&self) -> u8 {
        self.state.bank
    }

    pub const fn executed_words(&self) -> u64 {
        self.executed_words
    }

    /// Apply the Hawkeye/HP-67 rule that bank 1 cannot remain selected in the
    /// first 1K page, then return the bank to use for the pending ROM fetch.
    pub fn prepare_hp67_fetch(&mut self) -> u8 {
        if self.state.pc < 0x0400 {
            self.state.bank = 0;
        }
        self.state.bank & 1
    }

    /// Execute one already reconstructed 10-bit ROM word.
    ///
    /// Unsupported specials are transactional: architectural state is restored
    /// before returning an error, which keeps long probe runs deterministic.
    pub fn execute_word(
        &mut self,
        ram: &mut ActRamImage,
        word: u16,
    ) -> Result<ActExecution, ActError> {
        if word > ROM_WORD_MASK {
            return Err(ActError::OpcodeOutOfRange(word));
        }

        let checkpoint = self.state.clone();
        let execution_pc = self.state.pc;
        let result = self.execute_word_inner(ram, word, execution_pc);
        match result {
            Ok(operation) => {
                self.executed_words = self.executed_words.wrapping_add(1);
                Ok(ActExecution {
                    pc: execution_pc,
                    word,
                    next_pc: self.state.pc,
                    operation,
                })
            }
            Err(error) => {
                self.state = checkpoint;
                Err(error)
            }
        }
    }

    fn execute_word_inner(
        &mut self,
        ram: &mut ActRamImage,
        word: u16,
        execution_pc: u16,
    ) -> Result<ActOperation, ActError> {
        self.state.p_change[2] = self.state.p_change[1];
        self.state.p_change[1] = self.state.p_change[0];
        self.state.p_change[0] = 0;

        let prior_instruction_state = self.state.instruction_state;
        let prior_delayed_rom = self.state.delayed_rom.take();
        self.state.previous_carry = self.state.carry;
        self.state.carry = false;
        self.state.pc = self.state.pc.wrapping_add(1) & ROM_ADDRESS_MASK;

        let operation = match prior_instruction_state {
            ActInstructionState::ThenGoto => {
                self.state.instruction_state = ActInstructionState::Normal;
                let target = (self.state.pc & !THEN_GOTO_MASK) | word;
                let taken = !self.state.previous_carry;
                if taken {
                    self.state.pc = target;
                }
                ActOperation::ThenGoto { taken, target }
            }
            ActInstructionState::Normal => self.execute_normal_word(ram, word, execution_pc)?,
        };

        if let Some(rom) = prior_delayed_rom {
            self.state.pc = (u16::from(rom & 0x0f) << 8) | (self.state.pc & LOW_ROM_MASK);
        }

        Ok(operation)
    }

    fn execute_normal_word(
        &mut self,
        ram: &mut ActRamImage,
        word: u16,
        execution_pc: u16,
    ) -> Result<ActOperation, ActError> {
        match word & 0x03 {
            0 => {
                self.execute_special(ram, word, execution_pc)?;
                Ok(ActOperation::Special { opcode: word })
            }
            1 => {
                let offset = (word >> 2) as u8;
                let return_slot = usize::from(self.state.stack_pointer);
                self.state.return_stack[return_slot] = self.state.pc;
                self.state.stack_pointer =
                    (self.state.stack_pointer + 1) % ACT_RETURN_STACK_DEPTH as u8;
                let target = (self.state.pc & !LOW_ROM_MASK) | u16::from(offset);
                self.state.pc = target;
                Ok(ActOperation::Jsb { target })
            }
            2 => {
                let operation = ((word >> 5) & 0x1f) as u8;
                let field = ((word >> 2) & 0x07) as u8;
                self.execute_arithmetic(operation, field);
                Ok(ActOperation::Arithmetic { operation, field })
            }
            3 => {
                let offset = (word >> 2) as u8;
                let target = (self.state.pc & !LOW_ROM_MASK) | u16::from(offset);
                let taken = !self.state.previous_carry;
                if taken {
                    self.state.pc = target;
                }
                Ok(ActOperation::Goto { taken, target })
            }
            _ => unreachable!(),
        }
    }

    fn execute_special(
        &mut self,
        ram: &mut ActRamImage,
        word: u16,
        execution_pc: u16,
    ) -> Result<(), ActError> {
        match word {
            0o0000 | 0o1760 => return Ok(()),
            0o0070 => {
                self.state.c = ram.read(self.state.ram_address).unwrap_or([0; ACT_WORD_DIGITS]);
                return Ok(());
            }
            0o0010 => {
                self.state.a = [0; ACT_WORD_DIGITS];
                self.state.b = [0; ACT_WORD_DIGITS];
                self.state.c = [0; ACT_WORD_DIGITS];
                self.state.y = [0; ACT_WORD_DIGITS];
                self.state.z = [0; ACT_WORD_DIGITS];
                self.state.t = [0; ACT_WORD_DIGITS];
                return Ok(());
            }
            0o0110 => {
                for index in 0..ACT_STATUS_BITS {
                    if !matches!(index, 1 | 2 | 5 | 15) {
                        self.state.status[index] = false;
                    }
                }
                return Ok(());
            }
            0o0210 => {
                self.state.display_enable = !self.state.display_enable;
                return Ok(());
            }
            0o0310 => {
                self.state.display_enable = false;
                return Ok(());
            }
            0o0410 => {
                core::mem::swap(&mut self.state.c, &mut self.state.m1);
                return Ok(());
            }
            0o0510 => {
                self.state.c = self.state.m1;
                return Ok(());
            }
            0o0610 => {
                core::mem::swap(&mut self.state.c, &mut self.state.m2);
                return Ok(());
            }
            0o0710 => {
                self.state.c = self.state.m2;
                return Ok(());
            }
            0o1010 => {
                self.state.a = self.state.y;
                self.state.y = self.state.z;
                self.state.z = self.state.t;
                return Ok(());
            }
            0o1110 => {
                let old_c = self.state.c;
                self.state.c = self.state.y;
                self.state.y = self.state.z;
                self.state.z = self.state.t;
                self.state.t = old_c;
                return Ok(());
            }
            0o1210 => {
                self.state.a = self.state.y;
                return Ok(());
            }
            0o1310 => {
                self.state.t = self.state.z;
                self.state.z = self.state.y;
                self.state.y = self.state.c;
                return Ok(());
            }
            0o1410 => {
                self.state.decimal = true;
                return Ok(());
            }
            0o1610 => {
                self.state.a[0] = self.state.f;
                return Ok(());
            }
            0o1710 => {
                core::mem::swap(&mut self.state.a[0], &mut self.state.f);
                return Ok(());
            }
            0o0020 => {
                self.state.pc &= !LOW_ROM_MASK;
                if let Some(key) = self.state.key_buffer {
                    self.state.pc |= u16::from(key);
                }
                return Ok(());
            }
            0o0120 => {
                let key = self.state.key_buffer.unwrap_or(0);
                self.state.a[2] = key >> 4;
                self.state.a[1] = key & 0x0f;
                return Ok(());
            }
            0o0220 => {
                self.state.pc = (self.state.pc & !LOW_ROM_MASK)
                    | u16::from((self.state.a[2] << 4) | self.state.a[1]);
                return Ok(());
            }
            0o0320 => {
                self.state.display_14_digit = true;
                return Ok(());
            }
            0o0420 => {
                self.state.decimal = false;
                return Ok(());
            }
            0o0520 => {
                self.state.a.rotate_right(1);
                return Ok(());
            }
            0o0620 => {
                self.state.p_change[0] = -1;
                self.state.p = if self.state.p == 0 {
                    (ACT_WORD_DIGITS - 1) as u8
                } else {
                    self.state.p - 1
                };
                return Ok(());
            }
            0o0720 => {
                self.state.p_change[0] = 1;
                self.state.p = self.state.p.wrapping_add(1);
                if usize::from(self.state.p) >= ACT_WORD_DIGITS {
                    self.state.p = 0;
                }
                return Ok(());
            }
            0o1020 => {
                self.state.stack_pointer = if self.state.stack_pointer == 0 {
                    (ACT_RETURN_STACK_DEPTH - 1) as u8
                } else {
                    self.state.stack_pointer - 1
                };
                self.state.pc = self.state.return_stack[usize::from(self.state.stack_pointer)];
                return Ok(());
            }
            0o1060 => {
                self.state.bank ^= 1;
                return Ok(());
            }
            0o1160 => {
                self.state.ram_address = (self.state.c[1] << 4) | self.state.c[0];
                return Ok(());
            }
            0o1260 => {
                ram.clear_block(self.state.ram_address);
                return Ok(());
            }
            0o1360 => {
                ram.write(self.state.ram_address, self.state.c);
                return Ok(());
            }
            0o1460 => return Err(ActError::UnsupportedRomSelfTest { pc: execution_pc }),
            _ => {}
        }

        let operand = (word >> 6) as u8;
        match word & 0o77 {
            0o04 => self.state.status[usize::from(operand)] = true,
            0o14 => self.state.status[usize::from(operand)] = false,
            0o24 => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = !self.state.status[usize::from(operand)];
            }
            0o30 => {
                if usize::from(self.state.p) < ACT_WORD_DIGITS {
                    self.state.c[usize::from(self.state.p)] = operand;
                }
                self.state.p = if self.state.p == 0 {
                    (ACT_WORD_DIGITS - 1) as u8
                } else {
                    self.state.p - 1
                };
            }
            0o34 => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = self.state.status[usize::from(operand)];
            }
            0o40 => {
                self.state.pc = ((word & 0o1700) << 2) | (self.state.pc & 0o0377);
                self.state.pc &= ROM_ADDRESS_MASK;
            }
            0o44 => self.test_p(operand, true),
            0o50 => {
                self.select_register(operand);
                ram.write(self.state.ram_address, self.state.c);
            }
            0o54 => self.test_p(operand, false),
            0o64 => self.state.delayed_rom = Some(operand),
            0o70 => {
                self.select_register(operand);
                self.state.c = ram.read(self.state.ram_address).unwrap_or([0; ACT_WORD_DIGITS]);
            }
            0o74 => self.state.p = P_SET_MAP[usize::from(operand)],
            _ => {
                return Err(ActError::UnknownSpecial {
                    pc: execution_pc,
                    word,
                })
            }
        }
        Ok(())
    }

    fn execute_arithmetic(&mut self, operation: u8, field: u8) {
        let Some((first, last)) = field_bounds(field, self.state.p) else {
            return;
        };
        let base = if self.state.decimal { 10 } else { 16 };
        let a = self.state.a;
        let b = self.state.b;
        let c = self.state.c;

        match operation {
            0x00 => zero_range(&mut self.state.a, first, last),
            0x01 => zero_range(&mut self.state.b, first, last),
            0x02 => exchange_range(&mut self.state.a, &mut self.state.b, first, last),
            0x03 => copy_range(&mut self.state.b, a, first, last),
            0x04 => exchange_range(&mut self.state.a, &mut self.state.c, first, last),
            0x05 => copy_range(&mut self.state.a, c, first, last),
            0x06 => copy_range(&mut self.state.c, b, first, last),
            0x07 => exchange_range(&mut self.state.b, &mut self.state.c, first, last),
            0x08 => zero_range(&mut self.state.c, first, last),
            0x09 => self.state.carry = add_range(&mut self.state.a, a, Some(b), first, last, false, base),
            0x0a => self.state.carry = add_range(&mut self.state.a, a, Some(c), first, last, false, base),
            0x0b => self.state.carry = add_range(&mut self.state.c, c, Some(c), first, last, false, base),
            0x0c => self.state.carry = add_range(&mut self.state.c, a, Some(c), first, last, false, base),
            0x0d => self.state.carry = add_range(&mut self.state.a, a, None, first, last, true, base),
            0x0e => shift_left_range(&mut self.state.a, first, last),
            0x0f => self.state.carry = add_range(&mut self.state.c, c, None, first, last, true, base),
            0x10 => self.state.carry = sub_range(Some(&mut self.state.a), Some(a), Some(b), first, last, false, base),
            0x11 => self.state.carry = sub_range(Some(&mut self.state.c), Some(a), Some(c), first, last, false, base),
            0x12 => self.state.carry = sub_range(Some(&mut self.state.a), Some(a), None, first, last, true, base),
            0x13 => self.state.carry = sub_range(Some(&mut self.state.c), Some(c), None, first, last, true, base),
            0x14 => self.state.carry = sub_range(Some(&mut self.state.c), None, Some(c), first, last, false, base),
            0x15 => self.state.carry = sub_range(Some(&mut self.state.c), None, Some(c), first, last, true, base),
            0x16 => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = any_nonzero(b, first, last);
            }
            0x17 => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = any_nonzero(c, first, last);
            }
            0x18 => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = sub_range(None, Some(a), Some(c), first, last, false, base);
            }
            0x19 => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = sub_range(None, Some(a), Some(b), first, last, false, base);
            }
            0x1a => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = all_zero(a, first, last);
            }
            0x1b => {
                self.state.instruction_state = ActInstructionState::ThenGoto;
                self.state.carry = all_zero(c, first, last);
            }
            0x1c => self.state.carry = sub_range(Some(&mut self.state.a), Some(a), Some(c), first, last, false, base),
            0x1d => shift_right_range(&mut self.state.a, first, last),
            0x1e => shift_right_range(&mut self.state.b, first, last),
            0x1f => shift_right_range(&mut self.state.c, first, last),
            _ => unreachable!("five-bit arithmetic operation"),
        }
    }

    fn test_p(&mut self, operand: u8, equal: bool) {
        let target = P_TEST_MAP[usize::from(operand)];
        self.state.instruction_state = ActInstructionState::ThenGoto;
        let equal_result = if target == 0
            && self.state.p_change[1] == 1
            && self.state.p_change[2] == 1
        {
            self.state.p == 0 || self.state.p == 1
        } else {
            self.state.p == target
        };
        self.state.carry = if equal { !equal_result } else { equal_result };
    }

    fn select_register(&mut self, operand: u8) {
        self.state.ram_address = (self.state.ram_address & 0xf0) | (operand & 0x0f);
    }
}

fn field_bounds(field: u8, p: u8) -> Option<(usize, usize)> {
    let p = usize::from(p);
    match field & 7 {
        0 if p < ACT_WORD_DIGITS => Some((p, p)),
        0 => None,
        1 if p < ACT_WORD_DIGITS => Some((0, p)),
        1 => Some((0, ACT_WORD_DIGITS - 1)),
        2 => Some((2, 2)),
        3 => Some((0, 2)),
        4 => Some((13, 13)),
        5 => Some((3, 12)),
        6 => Some((0, 13)),
        7 => Some((3, 13)),
        _ => unreachable!(),
    }
}

fn zero_range(register: &mut ActRegister, first: usize, last: usize) {
    for digit in &mut register[first..=last] {
        *digit = 0;
    }
}

fn copy_range(destination: &mut ActRegister, source: ActRegister, first: usize, last: usize) {
    destination[first..=last].copy_from_slice(&source[first..=last]);
}

fn exchange_range(left: &mut ActRegister, right: &mut ActRegister, first: usize, last: usize) {
    for index in first..=last {
        core::mem::swap(&mut left[index], &mut right[index]);
    }
}

fn add_range(
    destination: &mut ActRegister,
    left: ActRegister,
    right: Option<ActRegister>,
    first: usize,
    last: usize,
    mut carry: bool,
    base: u8,
) -> bool {
    for index in first..=last {
        let rhs = right.map_or(0, |register| register[index]);
        let raw = u16::from(left[index]) + u16::from(rhs) + u16::from(carry);
        let next_carry = if base == 10 { raw > 9 } else { raw > 15 };
        let adjusted = if base == 10 && next_carry { raw + 6 } else { raw };
        destination[index] = (adjusted & 0x0f) as u8;
        carry = next_carry;
    }
    carry
}

fn sub_range(
    mut destination: Option<&mut ActRegister>,
    left: Option<ActRegister>,
    right: Option<ActRegister>,
    first: usize,
    last: usize,
    mut borrow: bool,
    base: u8,
) -> bool {
    for index in first..=last {
        let lhs = left.map_or(0, |register| register[index]);
        let rhs = right.map_or(0, |register| register[index]);
        let raw = i16::from(lhs) - i16::from(rhs) - i16::from(borrow);
        let next_borrow = raw < 0;
        let adjusted = if next_borrow { raw + i16::from(base) } else { raw };
        if let Some(register) = destination.as_deref_mut() {
            register[index] = ((adjusted as i32) & 0x0f) as u8;
        }
        borrow = next_borrow;
    }
    borrow
}

fn shift_left_range(register: &mut ActRegister, first: usize, last: usize) {
    for index in (first..=last).rev() {
        register[index] = if index == first { 0 } else { register[index - 1] };
    }
}

fn shift_right_range(register: &mut ActRegister, first: usize, last: usize) {
    for index in first..=last {
        register[index] = if index == last { 0 } else { register[index + 1] };
    }
}

fn any_nonzero(register: ActRegister, first: usize, last: usize) -> bool {
    register[first..=last].iter().any(|digit| *digit != 0)
}

fn all_zero(register: ActRegister, first: usize, last: usize) -> bool {
    register[first..=last].iter().all(|digit| *digit == 0)
}

// Compatibility aliases keep external bring-up tools source-compatible while
// making the broader role of this module explicit.
pub type PowerOnActCore = ActArchitecturalCore;
pub type PowerOnActError = ActError;
pub type PowerOnExecution = ActExecution;
pub type PowerOnOperation = ActOperation;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn known_power_on_prefix_runs_without_special_cases() {
        let mut act = ActArchitecturalCore::default();
        let mut ram = ActRamImage::hp67();
        for word in [0x000, 0x3e3, 0x11a, 0x108, 0x11a, 0x188, 0x11a, 0x0b4] {
            act.execute_word(&mut ram, word)
                .expect("known startup prefix must execute");
        }
        assert_eq!(act.pc(), 0x0fe);
        assert_eq!(act.state.delayed_rom, Some(2));
    }

    #[test]
    fn delayed_rom_is_applied_after_the_following_word() {
        let mut act = ActArchitecturalCore::default();
        let mut ram = ActRamImage::hp67();
        act.state.pc = 0x0fd;

        act.execute_word(&mut ram, 0o0264)
            .expect("delayed select ROM 2 must execute");
        assert_eq!(act.pc(), 0x0fe);
        assert_eq!(act.state.delayed_rom, Some(2));

        act.execute_word(&mut ram, 0o0000)
            .expect("following word must execute before delayed ROM applies");
        assert_eq!(act.pc(), 0x2ff);
        assert_eq!(act.state.delayed_rom, None);
    }

    #[test]
    fn unsupported_special_is_transactional() {
        let mut act = ActArchitecturalCore::default();
        let mut ram = ActRamImage::hp67();
        act.state.pc = 0x222;
        act.state.carry = true;
        act.state.delayed_rom = Some(7);
        let before = act.state.clone();

        assert_eq!(
            act.execute_word(&mut ram, 0o1460),
            Err(ActError::UnsupportedRomSelfTest { pc: 0x222 })
        );
        assert_eq!(act.state, before);
        assert_eq!(act.executed_words(), 0);
    }

    #[test]
    fn hp67_page_zero_forces_bank_zero_before_fetch() {
        let mut act = ActArchitecturalCore::default();
        act.state.pc = 0x123;
        act.state.bank = 1;
        assert_eq!(act.prepare_hp67_fetch(), 0);
        assert_eq!(act.bank(), 0);
    }
}
