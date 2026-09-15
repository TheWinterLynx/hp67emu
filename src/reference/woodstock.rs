//! Instruction-boundary Woodstock semantic reference machine.
//!
//! This module intentionally models architectural behaviour rather than
//! electrical timing. It is an independent oracle for the future serial,
//! cycle-accurate ACT and peripheral implementation.

use core::ops::{Index, IndexMut, RangeInclusive};

pub const WORD_DIGITS: usize = 14;
pub const STATUS_BITS: usize = 16;
pub const RETURN_STACK_DEPTH: usize = 2;
pub const PAGE_WORDS: usize = 1024;
pub const PAGE_COUNT: usize = 4;
pub const BANK_COUNT: usize = 2;
pub const OPCODE_MASK: u16 = 0x03ff;
pub const PC_MASK: u16 = 0x0fff;
pub const RAM_WORDS: usize = 256;

const LOW_PAGE_MASK: u16 = 0x00ff;
const THEN_GOTO_MASK: u16 = 0x03ff;
const P_SET_MAP: [u8; 16] = [14, 4, 7, 8, 11, 2, 10, 12, 1, 3, 13, 6, 0, 9, 5, 14];
const P_TEST_MAP: [u8; 16] = [4, 8, 12, 2, 9, 1, 6, 3, 1, 13, 5, 0, 11, 10, 7, 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    OpcodeOutOfRange(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionError {
    Decode(DecodeError),
    UnknownSpecial(u16),
    UnsupportedRomSelfTest,
}

impl From<DecodeError> for ExecutionError {
    fn from(value: DecodeError) -> Self {
        Self::Decode(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Field {
    P,
    Wp,
    Xs,
    X,
    S,
    M,
    W,
    Ms,
}

impl Field {
    pub const fn from_encoded(value: u8) -> Self {
        match value & 0x07 {
            0 => Self::P,
            1 => Self::Wp,
            2 => Self::Xs,
            3 => Self::X,
            4 => Self::S,
            5 => Self::M,
            6 => Self::W,
            7 => Self::Ms,
            _ => unreachable!(),
        }
    }

    pub fn digit_range(self, p: u8) -> RangeInclusive<usize> {
        let p = usize::from(p);
        match self {
            Self::P if p < WORD_DIGITS => p..=p,
            Self::P => WORD_DIGITS..=(WORD_DIGITS - 1),
            Self::Wp if p < WORD_DIGITS => 0..=p,
            Self::Wp => 0..=(WORD_DIGITS - 1),
            Self::Xs => 2..=2,
            Self::X => 0..=2,
            Self::S => 13..=13,
            Self::M => 3..=12,
            Self::W => 0..=13,
            Self::Ms => 3..=13,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstructionClass {
    Special { opcode: u16 },
    Jsb { page_offset: u8 },
    Arithmetic { operation: u8, field: Field },
    Goto { page_offset: u8 },
}

pub fn decode(opcode: u16) -> Result<InstructionClass, DecodeError> {
    if opcode > OPCODE_MASK {
        return Err(DecodeError::OpcodeOutOfRange(opcode));
    }

    Ok(match opcode & 0x03 {
        0 => InstructionClass::Special { opcode },
        1 => InstructionClass::Jsb {
            page_offset: (opcode >> 2) as u8,
        },
        2 => InstructionClass::Arithmetic {
            operation: (opcode >> 5) as u8,
            field: Field::from_encoded(((opcode >> 2) & 0x07) as u8),
        },
        3 => InstructionClass::Goto {
            page_offset: (opcode >> 2) as u8,
        },
        _ => unreachable!(),
    })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Register {
    digits: [u8; WORD_DIGITS],
}

impl Register {
    pub const fn zero() -> Self {
        Self {
            digits: [0; WORD_DIGITS],
        }
    }

    pub const fn digits(&self) -> &[u8; WORD_DIGITS] {
        &self.digits
    }

    pub fn digits_mut(&mut self) -> &mut [u8; WORD_DIGITS] {
        &mut self.digits
    }
}

impl Index<usize> for Register {
    type Output = u8;

    fn index(&self, index: usize) -> &Self::Output {
        &self.digits[index]
    }
}

impl IndexMut<usize> for Register {
    fn index_mut(&mut self, index: usize) -> &mut Self::Output {
        &mut self.digits[index]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum InstructionState {
    #[default]
    Normal,
    ThenGoto,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ArchitecturalState {
    pub a: Register,
    pub b: Register,
    pub c: Register,
    pub y: Register,
    pub z: Register,
    pub t: Register,
    pub m1: Register,
    pub m2: Register,
    pub f: u8,
    pub p: u8,
    pub p_change: [i8; 3],
    pub decimal: bool,
    pub carry: bool,
    pub previous_carry: bool,
    pub status: [bool; STATUS_BITS],
    pub pc: u16,
    pub delayed_rom: Option<u8>,
    pub bank: u8,
    pub return_stack: [u16; RETURN_STACK_DEPTH],
    pub stack_pointer: u8,
    pub instruction_state: InstructionState,
    pub key_buffer: Option<u8>,
    pub display_enable: bool,
    pub display_14_digit: bool,
    pub ram_address: u8,
}

impl Default for ArchitecturalState {
    fn default() -> Self {
        Self {
            a: Register::zero(),
            b: Register::zero(),
            c: Register::zero(),
            y: Register::zero(),
            z: Register::zero(),
            t: Register::zero(),
            m1: Register::zero(),
            m2: Register::zero(),
            f: 0,
            p: 0,
            p_change: [0; 3],
            decimal: true,
            carry: false,
            previous_carry: false,
            status: [false; STATUS_BITS],
            pc: 0,
            delayed_rom: None,
            bank: 0,
            return_stack: [0; RETURN_STACK_DEPTH],
            stack_pointer: 0,
            instruction_state: InstructionState::Normal,
            key_buffer: None,
            display_enable: false,
            display_14_digit: false,
            ram_address: 0,
        }
    }
}

impl ArchitecturalState {
    pub const fn arithmetic_base(&self) -> u8 {
        if self.decimal { 10 } else { 16 }
    }

    pub fn normalize_architectural_widths(&mut self) {
        self.f &= 0x0f;
        self.p &= 0x0f;
        self.pc &= PC_MASK;
        self.bank &= 0x01;
        self.stack_pointer %= RETURN_STACK_DEPTH as u8;
        if let Some(delayed_rom) = &mut self.delayed_rom {
            *delayed_rom &= 0x0f;
        }
        for address in &mut self.return_stack {
            *address &= PC_MASK;
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
            for digit in register.digits_mut() {
                *digit &= 0x0f;
            }
        }
    }

    fn begin_word(&mut self) -> (InstructionState, Option<u8>) {
        self.p_change[2] = self.p_change[1];
        self.p_change[1] = self.p_change[0];
        self.p_change[0] = 0;

        let prior_instruction_state = self.instruction_state;
        let prior_delayed_rom = self.delayed_rom.take();
        self.previous_carry = self.carry;
        self.carry = false;
        self.pc = self.pc.wrapping_add(1) & PC_MASK;
        (prior_instruction_state, prior_delayed_rom)
    }

    fn finish_word(&mut self, prior_delayed_rom: Option<u8>) {
        if let Some(delayed_rom) = prior_delayed_rom {
            self.pc = (u16::from(delayed_rom & 0x0f) << 8) | (self.pc & LOW_PAGE_MASK);
        }
    }

    fn execute_jsb(&mut self, page_offset: u8) {
        let stack_index = usize::from(self.stack_pointer);
        self.return_stack[stack_index] = self.pc;
        self.stack_pointer = (self.stack_pointer + 1) % RETURN_STACK_DEPTH as u8;
        self.pc = (self.pc & !LOW_PAGE_MASK) | u16::from(page_offset);
    }

    fn execute_return(&mut self) {
        self.stack_pointer = if self.stack_pointer == 0 {
            (RETURN_STACK_DEPTH - 1) as u8
        } else {
            self.stack_pointer - 1
        };
        self.pc = self.return_stack[usize::from(self.stack_pointer)];
    }

    fn execute_goto(&mut self, page_offset: u8) {
        if !self.previous_carry {
            self.pc = (self.pc & !LOW_PAGE_MASK) | u16::from(page_offset);
        }
    }

    fn execute_arithmetic(&mut self, operation: u8, field: Field) {
        let range = field.digit_range(self.p);
        let first = *range.start();
        let last = *range.end();
        let base = self.arithmetic_base();
        let a = self.a;
        let b = self.b;
        let c = self.c;

        match operation {
            0x00 => zero_range(&mut self.a, first, last),
            0x01 => zero_range(&mut self.b, first, last),
            0x02 => exchange_range(&mut self.a, &mut self.b, first, last),
            0x03 => copy_range(&mut self.b, a, first, last),
            0x04 => exchange_range(&mut self.a, &mut self.c, first, last),
            0x05 => copy_range(&mut self.a, c, first, last),
            0x06 => copy_range(&mut self.c, b, first, last),
            0x07 => exchange_range(&mut self.b, &mut self.c, first, last),
            0x08 => zero_range(&mut self.c, first, last),
            0x09 => {
                self.carry = add_range(&mut self.a, a, Some(b), first, last, false, base)
            }
            0x0a => {
                self.carry = add_range(&mut self.a, a, Some(c), first, last, false, base)
            }
            0x0b => {
                self.carry = add_range(&mut self.c, c, Some(c), first, last, false, base)
            }
            0x0c => {
                self.carry = add_range(&mut self.c, a, Some(c), first, last, false, base)
            }
            0x0d => {
                self.carry = add_range(&mut self.a, a, None, first, last, true, base)
            }
            0x0e => shift_left_range(&mut self.a, first, last),
            0x0f => {
                self.carry = add_range(&mut self.c, c, None, first, last, true, base)
            }
            0x10 => {
                self.carry = sub_range(
                    Some(&mut self.a),
                    Some(a),
                    Some(b),
                    first,
                    last,
                    false,
                    base,
                )
            }
            0x11 => {
                self.carry = sub_range(
                    Some(&mut self.c),
                    Some(a),
                    Some(c),
                    first,
                    last,
                    false,
                    base,
                )
            }
            0x12 => {
                self.carry = sub_range(
                    Some(&mut self.a),
                    Some(a),
                    None,
                    first,
                    last,
                    true,
                    base,
                )
            }
            0x13 => {
                self.carry = sub_range(
                    Some(&mut self.c),
                    Some(c),
                    None,
                    first,
                    last,
                    true,
                    base,
                )
            }
            0x14 => {
                self.carry = sub_range(
                    Some(&mut self.c),
                    None,
                    Some(c),
                    first,
                    last,
                    false,
                    base,
                )
            }
            0x15 => {
                self.carry = sub_range(
                    Some(&mut self.c),
                    None,
                    Some(c),
                    first,
                    last,
                    true,
                    base,
                )
            }
            0x16 => {
                self.instruction_state = InstructionState::ThenGoto;
                self.carry = any_nonzero(b, first, last);
            }
            0x17 => {
                self.instruction_state = InstructionState::ThenGoto;
                self.carry = any_nonzero(c, first, last);
            }
            0x18 => {
                self.instruction_state = InstructionState::ThenGoto;
                self.carry = sub_range(
                    None,
                    Some(a),
                    Some(c),
                    first,
                    last,
                    false,
                    base,
                );
            }
            0x19 => {
                self.instruction_state = InstructionState::ThenGoto;
                self.carry = sub_range(
                    None,
                    Some(a),
                    Some(b),
                    first,
                    last,
                    false,
                    base,
                );
            }
            0x1a => {
                self.instruction_state = InstructionState::ThenGoto;
                self.carry = all_zero(a, first, last);
            }
            0x1b => {
                self.instruction_state = InstructionState::ThenGoto;
                self.carry = all_zero(c, first, last);
            }
            0x1c => {
                self.carry = sub_range(
                    Some(&mut self.a),
                    Some(a),
                    Some(c),
                    first,
                    last,
                    false,
                    base,
                )
            }
            0x1d => shift_right_range(&mut self.a, first, last),
            0x1e => shift_right_range(&mut self.b, first, last),
            0x1f => shift_right_range(&mut self.c, first, last),
            _ => unreachable!("five-bit arithmetic operation"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReferenceMachine {
    pub cpu: ArchitecturalState,
    ram: [Option<Register>; RAM_WORDS],
}

impl Default for ReferenceMachine {
    fn default() -> Self {
        Self {
            cpu: ArchitecturalState::default(),
            ram: [None; RAM_WORDS],
        }
    }
}

impl ReferenceMachine {
    pub fn hp67() -> Self {
        let mut machine = Self::default();
        machine.install_ram_range(0x00, 0x40);
        machine
    }

    pub fn install_ram_range(&mut self, start: u8, count: usize) {
        let first = usize::from(start);
        let end = first.saturating_add(count).min(RAM_WORDS);
        for slot in &mut self.ram[first..end] {
            *slot = Some(Register::zero());
        }
    }

    pub fn ram(&self, address: u8) -> Option<Register> {
        self.ram[usize::from(address)]
    }

    pub fn set_ram(&mut self, address: u8, value: Register) -> bool {
        let slot = &mut self.ram[usize::from(address)];
        if slot.is_none() {
            return false;
        }
        *slot = Some(value);
        true
    }

    pub fn step_word(&mut self, opcode: u16) -> Result<(), ExecutionError> {
        if opcode > OPCODE_MASK {
            return Err(DecodeError::OpcodeOutOfRange(opcode).into());
        }

        let (prior_instruction_state, prior_delayed_rom) = self.cpu.begin_word();
        match prior_instruction_state {
            InstructionState::Normal => self.execute_normal_word(opcode)?,
            InstructionState::ThenGoto => {
                self.cpu.instruction_state = InstructionState::Normal;
                if !self.cpu.previous_carry {
                    self.cpu.pc = (self.cpu.pc & !THEN_GOTO_MASK) | opcode;
                }
            }
        }
        self.cpu.finish_word(prior_delayed_rom);
        Ok(())
    }

    fn execute_normal_word(&mut self, opcode: u16) -> Result<(), ExecutionError> {
        match decode(opcode)? {
            InstructionClass::Special { opcode } => self.execute_special(opcode),
            InstructionClass::Jsb { page_offset } => {
                self.cpu.execute_jsb(page_offset);
                Ok(())
            }
            InstructionClass::Arithmetic { operation, field } => {
                self.cpu.execute_arithmetic(operation, field);
                Ok(())
            }
            InstructionClass::Goto { page_offset } => {
                self.cpu.execute_goto(page_offset);
                Ok(())
            }
        }
    }

    fn execute_special(&mut self, opcode: u16) -> Result<(), ExecutionError> {
        match opcode {
            0o0000 | 0o1760 => Ok(()),
            0o0070 => {
                self.data_to_c();
                Ok(())
            }
            0o0010 => {
                self.cpu.a = Register::zero();
                self.cpu.b = Register::zero();
                self.cpu.c = Register::zero();
                self.cpu.y = Register::zero();
                self.cpu.z = Register::zero();
                self.cpu.t = Register::zero();
                Ok(())
            }
            0o0110 => {
                for index in 0..STATUS_BITS {
                    if !matches!(index, 1 | 2 | 5 | 15) {
                        self.cpu.status[index] = false;
                    }
                }
                Ok(())
            }
            0o0210 => {
                self.cpu.display_enable = !self.cpu.display_enable;
                Ok(())
            }
            0o0310 => {
                self.cpu.display_enable = false;
                Ok(())
            }
            0o0410 => {
                core::mem::swap(&mut self.cpu.c, &mut self.cpu.m1);
                Ok(())
            }
            0o0510 => {
                self.cpu.c = self.cpu.m1;
                Ok(())
            }
            0o0610 => {
                core::mem::swap(&mut self.cpu.c, &mut self.cpu.m2);
                Ok(())
            }
            0o0710 => {
                self.cpu.c = self.cpu.m2;
                Ok(())
            }
            0o1010 => {
                self.cpu.a = self.cpu.y;
                self.cpu.y = self.cpu.z;
                self.cpu.z = self.cpu.t;
                Ok(())
            }
            0o1110 => {
                let old_c = self.cpu.c;
                self.cpu.c = self.cpu.y;
                self.cpu.y = self.cpu.z;
                self.cpu.z = self.cpu.t;
                self.cpu.t = old_c;
                Ok(())
            }
            0o1210 => {
                self.cpu.a = self.cpu.y;
                Ok(())
            }
            0o1310 => {
                self.cpu.t = self.cpu.z;
                self.cpu.z = self.cpu.y;
                self.cpu.y = self.cpu.c;
                Ok(())
            }
            0o1410 => {
                self.cpu.decimal = true;
                Ok(())
            }
            0o1610 => {
                self.cpu.a[0] = self.cpu.f;
                Ok(())
            }
            0o1710 => {
                core::mem::swap(&mut self.cpu.a[0], &mut self.cpu.f);
                Ok(())
            }
            0o0020 => {
                self.cpu.pc &= !LOW_PAGE_MASK;
                if let Some(key) = self.cpu.key_buffer {
                    self.cpu.pc |= u16::from(key);
                }
                Ok(())
            }
            0o0120 => {
                if let Some(key) = self.cpu.key_buffer {
                    self.cpu.a[2] = key >> 4;
                    self.cpu.a[1] = key & 0x0f;
                } else {
                    self.cpu.a[2] = 0;
                    self.cpu.a[1] = 0;
                }
                Ok(())
            }
            0o0220 => {
                self.cpu.pc &= !LOW_PAGE_MASK;
                self.cpu.pc |= u16::from((self.cpu.a[2] << 4) | self.cpu.a[1]);
                Ok(())
            }
            0o0320 => {
                self.cpu.display_14_digit = true;
                Ok(())
            }
            0o0420 => {
                self.cpu.decimal = false;
                Ok(())
            }
            0o0520 => {
                let top = self.cpu.a[WORD_DIGITS - 1];
                for index in (1..WORD_DIGITS).rev() {
                    self.cpu.a[index] = self.cpu.a[index - 1];
                }
                self.cpu.a[0] = top;
                Ok(())
            }
            0o0620 => {
                self.cpu.p_change[0] = -1;
                self.cpu.p = if self.cpu.p == 0 {
                    (WORD_DIGITS - 1) as u8
                } else {
                    self.cpu.p - 1
                };
                Ok(())
            }
            0o0720 => {
                self.cpu.p_change[0] = 1;
                self.cpu.p += 1;
                if usize::from(self.cpu.p) >= WORD_DIGITS {
                    self.cpu.p = 0;
                }
                Ok(())
            }
            0o1020 => {
                self.cpu.execute_return();
                Ok(())
            }
            0o1060 => {
                self.cpu.bank ^= 1;
                Ok(())
            }
            0o1160 => {
                self.cpu.ram_address = (self.cpu.c[1] << 4) | self.cpu.c[0];
                Ok(())
            }
            0o1260 => {
                self.clear_ram_block();
                Ok(())
            }
            0o1360 => {
                self.c_to_data();
                Ok(())
            }
            0o1460 => Err(ExecutionError::UnsupportedRomSelfTest),
            _ => self.execute_special_family(opcode),
        }
    }

    fn execute_special_family(&mut self, opcode: u16) -> Result<(), ExecutionError> {
        let operand = (opcode >> 6) as u8;
        match opcode & 0o77 {
            0o04 => {
                self.cpu.status[usize::from(operand)] = true;
                Ok(())
            }
            0o14 => {
                self.cpu.status[usize::from(operand)] = false;
                Ok(())
            }
            0o24 => {
                self.cpu.instruction_state = InstructionState::ThenGoto;
                self.cpu.carry = !self.cpu.status[usize::from(operand)];
                Ok(())
            }
            0o30 => {
                if usize::from(self.cpu.p) < WORD_DIGITS {
                    self.cpu.c[usize::from(self.cpu.p)] = operand;
                }
                self.cpu.p = if self.cpu.p == 0 {
                    (WORD_DIGITS - 1) as u8
                } else {
                    self.cpu.p - 1
                };
                Ok(())
            }
            0o34 => {
                self.cpu.instruction_state = InstructionState::ThenGoto;
                self.cpu.carry = self.cpu.status[usize::from(operand)];
                Ok(())
            }
            0o40 => {
                self.cpu.pc = ((opcode & 0o1700) << 2) | (self.cpu.pc & 0o0377);
                self.cpu.pc &= PC_MASK;
                Ok(())
            }
            0o44 => {
                self.test_p(operand, true);
                Ok(())
            }
            0o50 => {
                self.select_register(operand);
                self.c_to_data();
                Ok(())
            }
            0o54 => {
                self.test_p(operand, false);
                Ok(())
            }
            0o64 => {
                self.cpu.delayed_rom = Some(operand);
                Ok(())
            }
            0o70 => {
                self.select_register(operand);
                self.data_to_c();
                Ok(())
            }
            0o74 => {
                self.cpu.p = P_SET_MAP[usize::from(operand)];
                Ok(())
            }
            _ => Err(ExecutionError::UnknownSpecial(opcode)),
        }
    }

    fn test_p(&mut self, operand: u8, equal: bool) {
        let target = P_TEST_MAP[usize::from(operand)];
        self.cpu.instruction_state = InstructionState::ThenGoto;

        let equal_result = if target == 0 && self.cpu.p_change[1] == 1 && self.cpu.p_change[2] == 1 {
            // Woodstock documentation describes P as disappearing briefly on
            // wrap. HP-67/97 label search depends on that behaviour. Model the
            // condition generically rather than keying it to a firmware PC.
            self.cpu.p == 0 || self.cpu.p == 1
        } else {
            self.cpu.p == target
        };

        self.cpu.carry = if equal { !equal_result } else { equal_result };
    }

    fn select_register(&mut self, operand: u8) {
        self.cpu.ram_address = (self.cpu.ram_address & 0xf0) | (operand & 0x0f);
    }

    fn data_to_c(&mut self) {
        self.cpu.c = self.ram[usize::from(self.cpu.ram_address)].unwrap_or_default();
    }

    fn c_to_data(&mut self) {
        let slot = &mut self.ram[usize::from(self.cpu.ram_address)];
        if slot.is_some() {
            *slot = Some(self.cpu.c);
        }
    }

    fn clear_ram_block(&mut self) {
        let base = usize::from(self.cpu.ram_address & 0xf0);
        for slot in &mut self.ram[base..base + 16] {
            if slot.is_some() {
                *slot = Some(Register::zero());
            }
        }
    }
}

fn zero_range(register: &mut Register, first: usize, last: usize) {
    if first > last {
        return;
    }
    for digit in &mut register.digits_mut()[first..=last] {
        *digit = 0;
    }
}

fn copy_range(destination: &mut Register, source: Register, first: usize, last: usize) {
    for index in first..=last {
        destination[index] = source[index];
    }
}

fn exchange_range(left: &mut Register, right: &mut Register, first: usize, last: usize) {
    for index in first..=last {
        core::mem::swap(&mut left[index], &mut right[index]);
    }
}

fn add_range(
    destination: &mut Register,
    left: Register,
    right: Option<Register>,
    first: usize,
    last: usize,
    mut carry: bool,
    base: u8,
) -> bool {
    for index in first..=last {
        let right_digit = right.map_or(0, |register| register[index]);
        let (digit, next_carry) = add_digit(left[index], right_digit, carry, base);
        destination[index] = digit;
        carry = next_carry;
    }
    carry
}

fn sub_range(
    mut destination: Option<&mut Register>,
    left: Option<Register>,
    right: Option<Register>,
    first: usize,
    last: usize,
    mut borrow: bool,
    base: u8,
) -> bool {
    for index in first..=last {
        let left_digit = left.map_or(0, |register| register[index]);
        let right_digit = right.map_or(0, |register| register[index]);
        let (digit, next_borrow) = sub_digit(left_digit, right_digit, borrow, base);
        if let Some(register) = destination.as_deref_mut() {
            register[index] = digit;
        }
        borrow = next_borrow;
    }
    borrow
}

fn add_digit(left: u8, right: u8, carry: bool, base: u8) -> (u8, bool) {
    let raw = u16::from(left) + u16::from(right) + u16::from(carry);
    let carry_out = if base == 10 { raw > 9 } else { raw > 0x0f };
    let adjusted = if base == 10 && carry_out {
        raw + 6
    } else {
        raw
    };
    ((adjusted & 0x0f) as u8, carry_out)
}

fn sub_digit(left: u8, right: u8, borrow: bool, base: u8) -> (u8, bool) {
    let raw = i16::from(left) - i16::from(right) - i16::from(borrow);
    let borrow_out = raw < 0;
    let adjusted = if borrow_out {
        raw + i16::from(base)
    } else {
        raw
    };
    (((adjusted as i32) & 0x0f) as u8, borrow_out)
}

fn shift_left_range(register: &mut Register, first: usize, last: usize) {
    for index in (first..=last).rev() {
        register[index] = if index == first {
            0
        } else {
            register[index - 1]
        };
    }
}

fn shift_right_range(register: &mut Register, first: usize, last: usize) {
    for index in first..=last {
        register[index] = if index == last {
            0
        } else {
            register[index + 1]
        };
    }
}

fn any_nonzero(register: Register, first: usize, last: usize) -> bool {
    (first..=last).any(|index| register[index] != 0)
}

fn all_zero(register: Register, first: usize, last: usize) -> bool {
    !any_nonzero(register, first, last)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn arithmetic_word(operation: u8, field: Field) -> u16 {
        let encoded_field: u16 = match field {
            Field::P => 0,
            Field::Wp => 1,
            Field::Xs => 2,
            Field::X => 3,
            Field::S => 4,
            Field::M => 5,
            Field::W => 6,
            Field::Ms => 7,
        };
        (u16::from(operation) << 5) | (encoded_field << 2) | 0x02
    }

    fn jsb_word(page_offset: u8) -> u16 {
        (u16::from(page_offset) << 2) | 0x01
    }

    fn goto_word(page_offset: u8) -> u16 {
        (u16::from(page_offset) << 2) | 0x03
    }

    #[test]
    fn complete_opcode_space_has_the_expected_four_way_shape() {
        let mut counts = [0usize; 4];
        for opcode in 0..=OPCODE_MASK {
            match decode(opcode).expect("10-bit opcode must decode") {
                InstructionClass::Special { .. } => counts[0] += 1,
                InstructionClass::Jsb { .. } => counts[1] += 1,
                InstructionClass::Arithmetic { .. } => counts[2] += 1,
                InstructionClass::Goto { .. } => counts[3] += 1,
            }
        }
        assert_eq!(counts, [256, 256, 256, 256]);
    }

    #[test]
    fn field_ranges_include_the_documented_invalid_p_behaviour() {
        assert_eq!(Field::P.digit_range(5), 5..=5);
        assert_eq!(Field::Wp.digit_range(5), 0..=5);
        assert!(Field::P.digit_range(14).is_empty());
        assert_eq!(Field::Wp.digit_range(14), 0..=13);
        assert_eq!(Field::Xs.digit_range(0), 2..=2);
        assert_eq!(Field::M.digit_range(0), 3..=12);
    }

    #[test]
    fn decimal_and_binary_arithmetic_have_distinct_correction() {
        let mut decimal = ReferenceMachine::default();
        decimal.cpu.a[0] = 9;
        decimal.cpu.b[0] = 1;
        decimal
            .step_word(arithmetic_word(0x09, Field::P))
            .expect("BCD add must execute");
        assert_eq!(decimal.cpu.a[0], 0);
        assert!(decimal.cpu.carry);

        let mut binary = ReferenceMachine::default();
        binary.cpu.decimal = false;
        binary.cpu.a[0] = 9;
        binary.cpu.b[0] = 1;
        binary
            .step_word(arithmetic_word(0x09, Field::P))
            .expect("binary add must execute");
        assert_eq!(binary.cpu.a[0], 10);
        assert!(!binary.cpu.carry);
    }

    #[test]
    fn decimal_subtraction_reports_borrow() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.a[0] = 3;
        machine.cpu.b[0] = 5;
        machine
            .step_word(arithmetic_word(0x10, Field::P))
            .expect("subtraction must execute");
        assert_eq!(machine.cpu.a[0], 8);
        assert!(machine.cpu.carry);
    }

    #[test]
    fn comparison_consumes_following_word_as_then_goto_target() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.pc = 0x800;
        machine.cpu.b[0] = 0;
        machine
            .step_word(arithmetic_word(0x16, Field::P))
            .expect("comparison must execute");
        assert_eq!(machine.cpu.instruction_state, InstructionState::ThenGoto);
        machine.step_word(0x02a).expect("target word must execute");
        assert_eq!(machine.cpu.pc, 0x82a);
        assert_eq!(machine.cpu.instruction_state, InstructionState::Normal);
    }

    #[test]
    fn goto_uses_carry_from_the_preceding_word() {
        let mut branch = ReferenceMachine::default();
        branch.cpu.pc = 0x120;
        branch.cpu.carry = false;
        branch.step_word(goto_word(0x55)).expect("goto must execute");
        assert_eq!(branch.cpu.pc, 0x155);

        let mut fall_through = ReferenceMachine::default();
        fall_through.cpu.pc = 0x120;
        fall_through.cpu.carry = true;
        fall_through
            .step_word(goto_word(0x55))
            .expect("goto must execute");
        assert_eq!(fall_through.cpu.pc, 0x121);
    }

    #[test]
    fn jsb_and_return_use_the_two_level_ring_stack() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.pc = 0x345;
        machine
            .step_word(jsb_word(0x67))
            .expect("jsb must execute");
        assert_eq!(machine.cpu.return_stack[0], 0x346);
        assert_eq!(machine.cpu.pc, 0x367);
        machine.step_word(0o1020).expect("return must execute");
        assert_eq!(machine.cpu.pc, 0x346);
        assert_eq!(machine.cpu.stack_pointer, 0);
    }

    #[test]
    fn delayed_rom_selection_takes_effect_after_the_following_word() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.pc = 0x0123;
        machine
            .step_word(0o0664)
            .expect("delayed ROM select must execute");
        assert_eq!(machine.cpu.pc, 0x0124);
        assert_eq!(machine.cpu.delayed_rom, Some(0o6));

        machine.step_word(0o0000).expect("nop must execute");
        assert_eq!(machine.cpu.pc, 0x0625);
        assert_eq!(machine.cpu.delayed_rom, None);
    }

    #[test]
    fn hp67_ram_model_exposes_four_sixteen_register_blocks() {
        let mut machine = ReferenceMachine::hp67();
        machine.cpu.ram_address = 0x20;
        machine.cpu.c[0] = 7;
        machine.step_word(0o1360).expect("C to DATA must execute");
        assert_eq!(machine.ram(0x20).expect("RAM must exist")[0], 7);

        machine.cpu.c = Register::zero();
        machine.step_word(0o0070).expect("DATA to C must execute");
        assert_eq!(machine.cpu.c[0], 7);
        assert!(machine.ram(0x40).is_none());
    }

    #[test]
    fn key_dispatch_uses_the_hardware_keycode() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.key_buffer = Some(0o244);
        machine.step_word(0o0120).expect("keys to A must execute");
        assert_eq!(machine.cpu.a[2], 0x0a);
        assert_eq!(machine.cpu.a[1], 0x04);
    }

    #[test]
    fn stack_special_opcodes_match_documented_operations() {
        let mut down_rotate = ReferenceMachine::default();
        down_rotate.cpu.c[0] = 1;
        down_rotate.cpu.y[0] = 2;
        down_rotate.cpu.z[0] = 3;
        down_rotate.cpu.t[0] = 4;
        down_rotate.step_word(0o1110).expect("down rotate must execute");
        assert_eq!(down_rotate.cpu.c[0], 2);
        assert_eq!(down_rotate.cpu.y[0], 3);
        assert_eq!(down_rotate.cpu.z[0], 4);
        assert_eq!(down_rotate.cpu.t[0], 1);

        let mut push = ReferenceMachine::default();
        push.cpu.c[0] = 1;
        push.cpu.y[0] = 2;
        push.cpu.z[0] = 3;
        push.cpu.t[0] = 4;
        push.step_word(0o1310).expect("C to stack must execute");
        assert_eq!(push.cpu.c[0], 1);
        assert_eq!(push.cpu.y[0], 1);
        assert_eq!(push.cpu.z[0], 2);
        assert_eq!(push.cpu.t[0], 3);
    }

    #[test]
    fn generic_p_wrap_rule_replaces_nonpareil_pc_specific_hack() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.p = 13;
        machine.step_word(0o0720).expect("first inc P must execute");
        machine.step_word(0o0720).expect("second inc P must execute");
        assert_eq!(machine.cpu.p, 1);

        // P-test operand 11 maps to zero. After two consecutive increments,
        // the documented wrap quirk makes the zero test succeed at P=1.
        let test_p_zero = 0o0044 + (11 << 6);
        machine.step_word(test_p_zero).expect("P test must execute");
        assert_eq!(machine.cpu.instruction_state, InstructionState::ThenGoto);
        assert!(!machine.cpu.carry);
    }

    #[test]
    fn clear_status_preserves_the_four_hardware_retained_bits() {
        let mut machine = ReferenceMachine::default();
        machine.cpu.status = [true; STATUS_BITS];
        machine.step_word(0o0110).expect("clear status must execute");
        for index in 0..STATUS_BITS {
            assert_eq!(machine.cpu.status[index], matches!(index, 1 | 2 | 5 | 15));
        }
    }

    #[test]
    fn out_of_range_and_unknown_words_fail_loudly() {
        let mut machine = ReferenceMachine::default();
        assert_eq!(
            machine.step_word(OPCODE_MASK + 1),
            Err(ExecutionError::Decode(DecodeError::OpcodeOutOfRange(
                OPCODE_MASK + 1
            )))
        );
        assert_eq!(
            machine.step_word(0o0060),
            Err(ExecutionError::UnknownSpecial(0o0060))
        );
    }

    #[test]
    fn state_width_normalization_preserves_real_register_widths() {
        let mut state = ArchitecturalState::default();
        state.f = 0xff;
        state.p = 0xff;
        state.pc = 0xffff;
        state.bank = 7;
        state.stack_pointer = 7;
        state.delayed_rom = Some(0xff);
        state.a[0] = 0xff;
        state.return_stack[0] = 0xffff;
        state.normalize_architectural_widths();

        assert_eq!(state.f, 0x0f);
        assert_eq!(state.p, 0x0f);
        assert_eq!(state.pc, PC_MASK);
        assert_eq!(state.bank, 1);
        assert!(state.stack_pointer < RETURN_STACK_DEPTH as u8);
        assert_eq!(state.delayed_rom, Some(0x0f));
        assert_eq!(state.a[0], 0x0f);
        assert_eq!(state.return_stack[0], PC_MASK);
    }
}
