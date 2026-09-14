//! Instruction-boundary Woodstock reference state, decoder and executor.
//!
//! This module intentionally models architectural behaviour rather than
//! electrical timing. It gives the cycle-accurate ACT implementation a small,
//! independent oracle to compare against after each completed microinstruction.

use core::ops::{Index, IndexMut, RangeInclusive};

pub const WORD_DIGITS: usize = 14;
pub const STATUS_BITS: usize = 16;
pub const RETURN_STACK_DEPTH: usize = 2;
pub const PAGE_WORDS: usize = 1024;
pub const PAGE_COUNT: usize = 4;
pub const BANK_COUNT: usize = 2;
pub const OPCODE_MASK: u16 = 0x03ff;
pub const PC_MASK: u16 = 0x0fff;
const LOW_PAGE_MASK: u16 = 0x00ff;
const THEN_GOTO_MASK: u16 = 0x03ff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    OpcodeOutOfRange(u16),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ExecutionError {
    Decode(DecodeError),
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
        let p = usize::from(p.min((WORD_DIGITS - 1) as u8));
        match self {
            Self::P => p..=p,
            Self::Wp => 0..=p,
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
        }
    }
}

impl ArchitecturalState {
    pub const fn arithmetic_base(&self) -> u8 {
        if self.decimal { 10 } else { 16 }
    }

    pub fn normalize_architectural_widths(&mut self) {
        self.f &= 0x0f;
        self.p %= WORD_DIGITS as u8;
        self.pc &= PC_MASK;
        self.bank %= BANK_COUNT as u8;
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

    /// Executes one 10-bit ROM word at the architectural boundary.
    ///
    /// This is intentionally not a timing model. The real electrical ACT will
    /// eventually reach the same resulting state through serial bit-times and
    /// external bus activity.
    pub fn step_word(&mut self, opcode: u16) -> Result<(), ExecutionError> {
        if opcode > OPCODE_MASK {
            return Err(DecodeError::OpcodeOutOfRange(opcode).into());
        }

        let prior_instruction_state = self.instruction_state;
        let prior_delayed_rom = self.delayed_rom.take();

        self.previous_carry = self.carry;
        self.carry = false;
        self.pc = self.pc.wrapping_add(1) & PC_MASK;

        match prior_instruction_state {
            InstructionState::Normal => self.execute_normal_word(opcode)?,
            InstructionState::ThenGoto => {
                self.instruction_state = InstructionState::Normal;
                if !self.previous_carry {
                    self.pc = (self.pc & !THEN_GOTO_MASK) | opcode;
                }
            }
        }

        if let Some(delayed_rom) = prior_delayed_rom {
            self.pc = (u16::from(delayed_rom & 0x0f) << 8) | (self.pc & LOW_PAGE_MASK);
        }

        Ok(())
    }

    fn execute_normal_word(&mut self, opcode: u16) -> Result<(), ExecutionError> {
        match decode(opcode)? {
            InstructionClass::Special { .. } => {
                // Special-op execution is added separately so arithmetic/control
                // flow can be validated without inventing peripheral behaviour.
            }
            InstructionClass::Jsb { page_offset } => self.execute_jsb(page_offset),
            InstructionClass::Arithmetic { operation, field } => {
                self.execute_arithmetic(operation, field)
            }
            InstructionClass::Goto { page_offset } => self.execute_goto(page_offset),
        }
        Ok(())
    }

    fn execute_jsb(&mut self, page_offset: u8) {
        let stack_index = usize::from(self.stack_pointer);
        self.return_stack[stack_index] = self.pc;
        self.stack_pointer = (self.stack_pointer + 1) % RETURN_STACK_DEPTH as u8;
        self.pc = (self.pc & !LOW_PAGE_MASK) | u16::from(page_offset);
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
                self.carry = add_range(&mut self.a, a, Some(b), first, last, false, base);
            }
            0x0a => {
                self.carry = add_range(&mut self.a, a, Some(c), first, last, false, base);
            }
            0x0b => {
                self.carry = add_range(&mut self.c, c, Some(c), first, last, false, base);
            }
            0x0c => {
                self.carry = add_range(&mut self.c, a, Some(c), first, last, false, base);
            }
            0x0d => {
                self.carry = add_range(&mut self.a, a, None, first, last, true, base);
            }
            0x0e => shift_left_range(&mut self.a, first, last),
            0x0f => {
                self.carry = add_range(&mut self.c, c, None, first, last, true, base);
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
                );
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
                );
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
                );
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
                );
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
                );
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
                );
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
                );
            }
            0x1d => shift_right_range(&mut self.a, first, last),
            0x1e => shift_right_range(&mut self.b, first, last),
            0x1f => shift_right_range(&mut self.c, first, last),
            _ => unreachable!("five-bit arithmetic operation"),
        }
    }
}

fn zero_range(register: &mut Register, first: usize, last: usize) {
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
    fn arithmetic_encoding_separates_operation_and_field() {
        let opcode = (0x1d_u16 << 5) | (6_u16 << 2) | 0x02;
        assert_eq!(
            decode(opcode),
            Ok(InstructionClass::Arithmetic {
                operation: 0x1d,
                field: Field::W,
            })
        );
    }

    #[test]
    fn field_ranges_match_the_fourteen_digit_architecture() {
        assert_eq!(Field::P.digit_range(5), 5..=5);
        assert_eq!(Field::Wp.digit_range(5), 0..=5);
        assert_eq!(Field::Xs.digit_range(0), 2..=2);
        assert_eq!(Field::X.digit_range(0), 0..=2);
        assert_eq!(Field::S.digit_range(0), 13..=13);
        assert_eq!(Field::M.digit_range(0), 3..=12);
        assert_eq!(Field::W.digit_range(0), 0..=13);
        assert_eq!(Field::Ms.digit_range(0), 3..=13);
    }

    #[test]
    fn decimal_addition_uses_bcd_correction_and_carry() {
        let mut state = ArchitecturalState::default();
        state.a[0] = 9;
        state.b[0] = 1;

        state
            .step_word(arithmetic_word(0x09, Field::P))
            .expect("arithmetic word must execute");

        assert_eq!(state.a[0], 0);
        assert!(state.carry);
    }

    #[test]
    fn binary_mode_uses_hexadecimal_nibbles() {
        let mut state = ArchitecturalState {
            decimal: false,
            ..ArchitecturalState::default()
        };
        state.a[0] = 0x0f;
        state.b[0] = 1;

        state
            .step_word(arithmetic_word(0x09, Field::P))
            .expect("arithmetic word must execute");

        assert_eq!(state.a[0], 0);
        assert!(state.carry);
    }

    #[test]
    fn decimal_subtraction_reports_borrow() {
        let mut state = ArchitecturalState::default();
        state.a[0] = 3;
        state.b[0] = 5;

        state
            .step_word(arithmetic_word(0x10, Field::P))
            .expect("subtraction must execute");

        assert_eq!(state.a[0], 8);
        assert!(state.carry);
    }

    #[test]
    fn arithmetic_field_limits_changes_to_selected_digits() {
        let mut state = ArchitecturalState::default();
        state.p = 2;
        state.a[0] = 1;
        state.a[1] = 2;
        state.a[2] = 3;
        state.a[3] = 4;

        state
            .step_word(arithmetic_word(0x00, Field::Wp))
            .expect("arithmetic word must execute");

        assert_eq!(&state.a.digits()[0..4], &[0, 0, 0, 4]);
    }

    #[test]
    fn comparison_consumes_following_word_as_then_goto_target() {
        let mut state = ArchitecturalState::default();
        state.pc = 0x800;
        state.b[0] = 0;

        state
            .step_word(arithmetic_word(0x16, Field::P))
            .expect("comparison must execute");
        assert_eq!(state.instruction_state, InstructionState::ThenGoto);
        assert!(!state.carry);

        state.step_word(0x02a).expect("branch target must execute");

        assert_eq!(state.instruction_state, InstructionState::Normal);
        assert_eq!(state.pc, 0x82a);
    }

    #[test]
    fn failed_comparison_skips_then_goto_target_word() {
        let mut state = ArchitecturalState::default();
        state.pc = 0x800;
        state.b[0] = 1;

        state
            .step_word(arithmetic_word(0x16, Field::P))
            .expect("comparison must execute");
        assert!(state.carry);

        state.step_word(0x02a).expect("branch target must execute");

        assert_eq!(state.instruction_state, InstructionState::Normal);
        assert_eq!(state.pc, 0x802);
    }

    #[test]
    fn goto_uses_carry_from_the_preceding_word() {
        let mut branch = ArchitecturalState::default();
        branch.pc = 0x120;
        branch.carry = false;
        branch
            .step_word(goto_word(0x55))
            .expect("goto must execute");
        assert_eq!(branch.pc, 0x155);

        let mut fall_through = ArchitecturalState::default();
        fall_through.pc = 0x120;
        fall_through.carry = true;
        fall_through
            .step_word(goto_word(0x55))
            .expect("goto must execute");
        assert_eq!(fall_through.pc, 0x121);
    }

    #[test]
    fn jsb_pushes_incremented_pc_and_wraps_two_level_stack() {
        let mut state = ArchitecturalState::default();
        state.pc = 0x345;

        state
            .step_word(jsb_word(0x67))
            .expect("first jsb must execute");
        assert_eq!(state.return_stack[0], 0x346);
        assert_eq!(state.stack_pointer, 1);
        assert_eq!(state.pc, 0x367);

        state
            .step_word(jsb_word(0x11))
            .expect("second jsb must execute");
        assert_eq!(state.return_stack[1], 0x368);
        assert_eq!(state.stack_pointer, 0);
        assert_eq!(state.pc, 0x311);
    }

    #[test]
    fn out_of_range_opcode_is_rejected_instead_of_truncated() {
        assert_eq!(
            decode(OPCODE_MASK + 1),
            Err(DecodeError::OpcodeOutOfRange(OPCODE_MASK + 1))
        );

        let mut state = ArchitecturalState::default();
        assert_eq!(
            state.step_word(OPCODE_MASK + 1),
            Err(ExecutionError::Decode(DecodeError::OpcodeOutOfRange(
                OPCODE_MASK + 1
            )))
        );
    }

    #[test]
    fn state_width_normalization_preserves_hardware_widths() {
        let mut state = ArchitecturalState::default();
        state.f = 0xff;
        state.p = 31;
        state.pc = 0xffff;
        state.bank = 7;
        state.stack_pointer = 7;
        state.delayed_rom = Some(0xff);
        state.a[0] = 0xff;
        state.return_stack[0] = 0xffff;

        state.normalize_architectural_widths();

        assert_eq!(state.f, 0x0f);
        assert!(state.p < WORD_DIGITS as u8);
        assert_eq!(state.pc, PC_MASK);
        assert!(state.bank < BANK_COUNT as u8);
        assert!(state.stack_pointer < RETURN_STACK_DEPTH as u8);
        assert_eq!(state.delayed_rom, Some(0x0f));
        assert_eq!(state.a[0], 0x0f);
        assert_eq!(state.return_stack[0], PC_MASK);
    }
}
