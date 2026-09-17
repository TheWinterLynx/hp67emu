//! Immutable pre-instruction ACT state used by the serial execution path.
//!
//! The instruction-boundary architectural core currently applies a complete
//! Woodstock operation before the structural b0..b55 word has finished. Serial
//! execution therefore needs its own pre-instruction view of A/B/C, P and radix
//! so later bit-level work cannot accidentally observe post-instruction state.

use super::{
    act::{ActArchitecturalState, ActRegister, ACT_WORD_DIGITS},
    act_serial_execution::{
        ActSerialArithmeticAction, ActSerialArithmeticCoordinate, ActSerialExecution,
        ActSerialOperand, ActSerialRegister,
    },
    timing::BITS_PER_DIGIT,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialAluInputs {
    pub coordinate: ActSerialArithmeticCoordinate,
    pub left_bit: bool,
    pub right_bit: bool,
    pub radix: u8,
    pub initial_carry: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialStateSnapshot {
    a: ActRegister,
    b: ActRegister,
    c: ActRegister,
    p: u8,
    decimal: bool,
}

impl ActSerialStateSnapshot {
    pub fn capture(state: &ActArchitecturalState) -> Self {
        Self {
            a: state.a,
            b: state.b,
            c: state.c,
            p: state.p,
            decimal: state.decimal,
        }
    }

    pub const fn p(&self) -> u8 {
        self.p
    }

    pub const fn decimal(&self) -> bool {
        self.decimal
    }

    pub const fn radix(&self) -> u8 {
        if self.decimal {
            10
        } else {
            16
        }
    }

    pub const fn register(&self, register: ActSerialRegister) -> &ActRegister {
        match register {
            ActSerialRegister::A => &self.a,
            ActSerialRegister::B => &self.b,
            ActSerialRegister::C => &self.c,
        }
    }

    pub fn register_digit(&self, register: ActSerialRegister, digit: u8) -> Option<u8> {
        let digit = usize::from(digit);
        if digit >= ACT_WORD_DIGITS {
            return None;
        }
        Some(self.register(register)[digit] & 0x0f)
    }

    pub fn register_bit(
        &self,
        register: ActSerialRegister,
        digit: u8,
        bit_in_digit: u8,
    ) -> Option<bool> {
        if bit_in_digit >= BITS_PER_DIGIT {
            return None;
        }
        let digit = self.register_digit(register, digit)?;
        Some(((digit >> bit_in_digit) & 1) != 0)
    }

    pub fn operand_bit(
        &self,
        operand: ActSerialOperand,
        digit: u8,
        bit_in_digit: u8,
    ) -> Option<bool> {
        match operand {
            ActSerialOperand::Zero => (bit_in_digit < BITS_PER_DIGIT).then_some(false),
            ActSerialOperand::Register(register) => {
                self.register_bit(register, digit, bit_in_digit)
            }
        }
    }

    pub fn arithmetic_coordinate(
        &self,
        execution: &ActSerialExecution,
    ) -> Option<ActSerialArithmeticCoordinate> {
        execution.arithmetic_coordinate(self.p)
    }

    /// Resolve the two serial operand bits required by an ADD/SUB bit cell.
    ///
    /// This exposes combinational ALU inputs only. It deliberately does not
    /// compute BCD correction, propagate carry, or write a destination register;
    /// those transitions require the still-unresolved physical ACT timing.
    pub fn alu_inputs(&self, execution: &ActSerialExecution) -> Option<ActSerialAluInputs> {
        let coordinate = self.arithmetic_coordinate(execution)?;
        if !coordinate.selected {
            return None;
        }
        let (left, right, initial_carry) = match coordinate.action {
            ActSerialArithmeticAction::Add {
                left,
                right,
                initial_carry,
                ..
            }
            | ActSerialArithmeticAction::Subtract {
                left,
                right,
                initial_carry,
                ..
            } => (left, right, initial_carry),
            _ => return None,
        };

        Some(ActSerialAluInputs {
            coordinate,
            left_bit: self.operand_bit(left, coordinate.digit, coordinate.bit_in_digit)?,
            right_bit: self.operand_bit(right, coordinate.digit, coordinate.bit_in_digit)?,
            radix: self.radix(),
            initial_carry,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::ActInstructionState;

    #[test]
    fn snapshot_is_not_affected_by_later_architectural_mutation() {
        let mut state = ActArchitecturalState::default();
        state.a[3] = 0x0d;
        state.b[3] = 0x06;
        state.c[3] = 0x09;
        state.p = 7;
        state.decimal = true;

        let snapshot = ActSerialStateSnapshot::capture(&state);
        state.a[3] = 0;
        state.b[3] = 0;
        state.c[3] = 0;
        state.p = 1;
        state.decimal = false;

        assert_eq!(snapshot.register_digit(ActSerialRegister::A, 3), Some(0x0d));
        assert_eq!(snapshot.register_digit(ActSerialRegister::B, 3), Some(0x06));
        assert_eq!(snapshot.register_digit(ActSerialRegister::C, 3), Some(0x09));
        assert_eq!(snapshot.p(), 7);
        assert!(snapshot.decimal());
        assert_eq!(snapshot.radix(), 10);
    }

    #[test]
    fn register_and_operand_bits_are_lsb_first_within_each_digit() {
        let mut state = ActArchitecturalState::default();
        state.a[5] = 0b1010;
        let snapshot = ActSerialStateSnapshot::capture(&state);

        assert_eq!(snapshot.register_bit(ActSerialRegister::A, 5, 0), Some(false));
        assert_eq!(snapshot.register_bit(ActSerialRegister::A, 5, 1), Some(true));
        assert_eq!(snapshot.register_bit(ActSerialRegister::A, 5, 2), Some(false));
        assert_eq!(snapshot.register_bit(ActSerialRegister::A, 5, 3), Some(true));
        assert_eq!(snapshot.operand_bit(ActSerialOperand::Zero, 5, 2), Some(false));
    }

    #[test]
    fn add_inputs_follow_the_current_serial_coordinate_and_snapshot_radix() {
        let mut state = ActArchitecturalState::default();
        state.a[0] = 0x01;
        state.b[0] = 0x01;
        state.decimal = true;
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let execution = ActSerialExecution::new(0x13a, ActInstructionState::Normal).unwrap();

        let inputs = snapshot.alu_inputs(&execution).unwrap();
        assert_eq!(inputs.coordinate.operation, 0x09);
        assert!(inputs.coordinate.selected);
        assert!(inputs.left_bit);
        assert!(inputs.right_bit);
        assert_eq!(inputs.radix, 10);
        assert!(!inputs.initial_carry);
    }

    #[test]
    fn increment_uses_zero_rhs_and_source_backed_initial_carry() {
        let mut state = ActArchitecturalState::default();
        state.a[0] = 0x01;
        state.decimal = false;
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let execution = ActSerialExecution::new(0x1ba, ActInstructionState::Normal).unwrap();

        let inputs = snapshot.alu_inputs(&execution).unwrap();
        assert_eq!(inputs.coordinate.operation, 0x0d);
        assert!(inputs.left_bit);
        assert!(!inputs.right_bit);
        assert_eq!(inputs.radix, 16);
        assert!(inputs.initial_carry);
    }

    #[test]
    fn non_selected_or_non_additive_cells_do_not_expose_alu_inputs() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let mantissa_add = ActSerialExecution::new(0x136, ActInstructionState::Normal).unwrap();
        assert_eq!(snapshot.alu_inputs(&mantissa_add), None);

        let clear_c = ActSerialExecution::new(0x11a, ActInstructionState::Normal).unwrap();
        assert_eq!(snapshot.alu_inputs(&clear_c), None);
    }

    #[test]
    fn invalid_digit_or_bit_coordinates_are_rejected() {
        let snapshot = ActSerialStateSnapshot::capture(&ActArchitecturalState::default());
        assert_eq!(
            snapshot.register_digit(ActSerialRegister::A, ACT_WORD_DIGITS as u8),
            None
        );
        assert_eq!(
            snapshot.register_bit(ActSerialRegister::A, 0, BITS_PER_DIGIT),
            None
        );
        assert_eq!(
            snapshot.operand_bit(ActSerialOperand::Zero, 0, BITS_PER_DIGIT),
            None
        );
    }

    #[test]
    fn radix_tracks_the_pre_instruction_decimal_mode() {
        let mut state = ActArchitecturalState::default();
        state.decimal = false;
        let snapshot = ActSerialStateSnapshot::capture(&state);
        assert!(!snapshot.decimal());
        assert_eq!(snapshot.radix(), 16);
    }
}
