//! Instruction-boundary Woodstock reference state and decoder.
//!
//! This module intentionally models architectural facts rather than electrical
//! timing.  It gives the cycle-accurate ACT implementation a small independent
//! oracle to compare against after each completed microinstruction.

use core::ops::{Index, IndexMut, RangeInclusive};

pub const WORD_DIGITS: usize = 14;
pub const STATUS_BITS: usize = 16;
pub const RETURN_STACK_DEPTH: usize = 2;
pub const PAGE_WORDS: usize = 1024;
pub const PAGE_COUNT: usize = 4;
pub const BANK_COUNT: usize = 2;
pub const OPCODE_MASK: u16 = 0x03ff;
pub const PC_MASK: u16 = 0x0fff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DecodeError {
    OpcodeOutOfRange(u16),
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
}

#[cfg(test)]
mod tests {
    use super::*;

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
    fn out_of_range_opcode_is_rejected_instead_of_truncated() {
        assert_eq!(
            decode(OPCODE_MASK + 1),
            Err(DecodeError::OpcodeOutOfRange(OPCODE_MASK + 1))
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
        state.a[0] = 0xff;
        state.return_stack[0] = 0xffff;

        state.normalize_architectural_widths();

        assert_eq!(state.f, 0x0f);
        assert!(state.p < WORD_DIGITS as u8);
        assert_eq!(state.pc, PC_MASK);
        assert!(state.bank < BANK_COUNT as u8);
        assert!(state.stack_pointer < RETURN_STACK_DEPTH as u8);
        assert_eq!(state.a[0], 0x0f);
        assert_eq!(state.return_stack[0], PC_MASK);
    }
}
