//! Source-backed arithmetic result image for one serially executing ACT word.
//!
//! This module materializes the register/carry result implied by the already
//! modeled b0..b55 arithmetic traversal without assigning a physical PHI write
//! edge. M14E extends the structural authority image from ADD/SUB to every
//! Woodstock arithmetic opcode while keeping exact internal ACT mutation timing
//! explicitly source-blocked.

use super::{
    act::{ActInstructionState, ActRegister, ACT_WORD_DIGITS},
    act_serial_execution::{ActSerialArithmeticAction, ActSerialExecution, ActSerialRegister},
    act_serial_state::{ActSerialDigitAluResult, ActSerialStateSnapshot},
    timing::{BITS_PER_DIGIT, BITS_PER_WORD},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialArithmeticResultImage {
    a: ActRegister,
    b: ActRegister,
    c: ActRegister,
    final_chain: bool,
    processed_digits: u8,
}

impl ActSerialArithmeticResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
    ) -> Option<Self> {
        let coordinate = snapshot.arithmetic_coordinate(execution)?;
        let final_chain = match coordinate.action {
            ActSerialArithmeticAction::Add { initial_carry, .. }
            | ActSerialArithmeticAction::Subtract { initial_carry, .. } => initial_carry,
            ActSerialArithmeticAction::TestZero { .. } => true,
            ActSerialArithmeticAction::Clear { .. }
            | ActSerialArithmeticAction::Copy { .. }
            | ActSerialArithmeticAction::Exchange { .. }
            | ActSerialArithmeticAction::ShiftLeft { .. }
            | ActSerialArithmeticAction::ShiftRight { .. }
            | ActSerialArithmeticAction::TestNonzero { .. } => false,
        };

        Some(Self {
            a: *snapshot.register(ActSerialRegister::A),
            b: *snapshot.register(ActSerialRegister::B),
            c: *snapshot.register(ActSerialRegister::C),
            final_chain,
            processed_digits: 0,
        })
    }

    pub fn evaluate(snapshot: &ActSerialStateSnapshot, word: u16) -> Option<Self> {
        let mut execution = ActSerialExecution::new(word, ActInstructionState::Normal).ok()?;
        let mut image = Self::begin(snapshot, &execution)?;
        let mut chain = None;

        for word_bit in 0..BITS_PER_WORD {
            let coordinate = snapshot.arithmetic_coordinate(&execution)?;
            let digit_result =
                if coordinate.selected && coordinate.bit_in_digit == BITS_PER_DIGIT - 1 {
                    match coordinate.action {
                        ActSerialArithmeticAction::Add { initial_carry, .. }
                        | ActSerialArithmeticAction::Subtract { initial_carry, .. } => {
                            let result = snapshot
                                .alu_digit_result(&execution, chain.unwrap_or(initial_carry))?;
                            chain = Some(result.chain_out);
                            Some(result)
                        }
                        _ => None,
                    }
                } else {
                    None
                };

            if coordinate.bit_in_digit == BITS_PER_DIGIT - 1 {
                image.record_digit_checkpoint(snapshot, &execution, digit_result)?;
            }

            execution.advance_word_bit(word_bit).ok()?;
        }

        Some(image)
    }

    pub fn record_digit_checkpoint(
        &mut self,
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
        alu_result: Option<ActSerialDigitAluResult>,
    ) -> Option<()> {
        let coordinate = snapshot.arithmetic_coordinate(execution)?;
        if coordinate.bit_in_digit != BITS_PER_DIGIT - 1 {
            return None;
        }
        if !coordinate.selected {
            return Some(());
        }

        match coordinate.action {
            ActSerialArithmeticAction::Clear { destination } => {
                self.write_digit(destination, coordinate.digit, 0)?;
            }
            ActSerialArithmeticAction::Copy {
                source,
                destination,
            } => {
                let value = snapshot.register_digit(source, coordinate.digit)?;
                self.write_digit(destination, coordinate.digit, value)?;
            }
            ActSerialArithmeticAction::Exchange { left, right } => {
                let left_value = snapshot.register_digit(left, coordinate.digit)?;
                let right_value = snapshot.register_digit(right, coordinate.digit)?;
                self.write_digit(left, coordinate.digit, right_value)?;
                self.write_digit(right, coordinate.digit, left_value)?;
            }
            ActSerialArithmeticAction::Add { destination, .. } => {
                let result = alu_result?;
                debug_assert_eq!(result.coordinate, coordinate);
                self.write_digit(destination, coordinate.digit, result.result_digit)?;
                self.final_chain = result.chain_out;
            }
            ActSerialArithmeticAction::Subtract { destination, .. } => {
                let result = alu_result?;
                debug_assert_eq!(result.coordinate, coordinate);
                if let Some(destination) = destination {
                    self.write_digit(destination, coordinate.digit, result.result_digit)?;
                }
                self.final_chain = result.chain_out;
            }
            ActSerialArithmeticAction::ShiftLeft { register } => {
                let source_digit = coordinate.digit.checked_sub(1).filter(|source_digit| {
                    execution.arithmetic_field_selects_digit(snapshot.p(), *source_digit)
                        == Some(true)
                });
                let value = source_digit
                    .and_then(|source_digit| snapshot.register_digit(register, source_digit))
                    .unwrap_or(0);
                self.write_digit(register, coordinate.digit, value)?;
            }
            ActSerialArithmeticAction::ShiftRight { register } => {
                let source_digit = coordinate.digit.checked_add(1).filter(|source_digit| {
                    execution.arithmetic_field_selects_digit(snapshot.p(), *source_digit)
                        == Some(true)
                });
                let value = source_digit
                    .and_then(|source_digit| snapshot.register_digit(register, source_digit))
                    .unwrap_or(0);
                self.write_digit(register, coordinate.digit, value)?;
            }
            ActSerialArithmeticAction::TestNonzero { register } => {
                self.final_chain |= snapshot.register_digit(register, coordinate.digit)? != 0;
            }
            ActSerialArithmeticAction::TestZero { register } => {
                self.final_chain &= snapshot.register_digit(register, coordinate.digit)? == 0;
            }
        }

        self.processed_digits = self.processed_digits.saturating_add(1);
        Some(())
    }

    pub const fn register(&self, register: ActSerialRegister) -> &ActRegister {
        match register {
            ActSerialRegister::A => &self.a,
            ActSerialRegister::B => &self.b,
            ActSerialRegister::C => &self.c,
        }
    }

    pub const fn final_chain(&self) -> bool {
        self.final_chain
    }

    pub const fn processed_digits(&self) -> u8 {
        self.processed_digits
    }

    fn write_digit(&mut self, register: ActSerialRegister, digit: u8, value: u8) -> Option<()> {
        let digit = usize::from(digit);
        if digit >= ACT_WORD_DIGITS {
            return None;
        }

        let destination = match register {
            ActSerialRegister::A => &mut self.a,
            ActSerialRegister::B => &mut self.b,
            ActSerialRegister::C => &mut self.c,
        };
        destination[digit] = value & 0x0f;
        Some(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActArchitecturalState, Hp67ArchitecturalMachine};

    fn compare_with_architectural(mut machine: Hp67ArchitecturalMachine, word: u16) {
        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
        let image = ActSerialArithmeticResultImage::evaluate(&snapshot, word)
            .expect("test word must be an arithmetic operation");

        machine
            .execute_word(word)
            .expect("architectural arithmetic execution must succeed");

        assert_eq!(image.register(ActSerialRegister::A), &machine.act.state.a);
        assert_eq!(image.register(ActSerialRegister::B), &machine.act.state.b);
        assert_eq!(image.register(ActSerialRegister::C), &machine.act.state.c);
        assert_eq!(image.final_chain(), machine.act.state.carry);
    }

    #[test]
    fn decimal_add_image_matches_architectural_carry_chain() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.a[0] = 9;
        machine.act.state.a[1] = 9;
        machine.act.state.b[0] = 1;
        machine.act.state.b[1] = 0;
        machine.act.state.p = 1;
        machine.act.state.decimal = true;

        compare_with_architectural(machine, 0x126);
    }

    #[test]
    fn hexadecimal_add_image_matches_architectural_wrap() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.a[0] = 0x0f;
        machine.act.state.b[0] = 0x01;
        machine.act.state.p = 0;
        machine.act.state.decimal = false;

        compare_with_architectural(machine, 0x122);
    }

    #[test]
    fn decimal_subtract_image_matches_architectural_borrow_chain() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.a[0] = 0;
        machine.act.state.a[1] = 1;
        machine.act.state.b[0] = 1;
        machine.act.state.b[1] = 0;
        machine.act.state.p = 1;
        machine.act.state.decimal = true;

        let word = (0x10u16 << 5) | (1u16 << 2) | 0x02;
        compare_with_architectural(machine, word);
    }

    #[test]
    fn compare_only_subtract_keeps_registers_and_matches_carry() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.a[0] = 2;
        machine.act.state.c[0] = 3;
        machine.act.state.p = 0;
        machine.act.state.decimal = true;

        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
        let word = (0x18u16 << 5) | 0x02;
        let image = ActSerialArithmeticResultImage::evaluate(&snapshot, word)
            .expect("A-C compare must produce a serial result image");

        machine
            .execute_word(word)
            .expect("architectural compare execution must succeed");

        assert_eq!(
            image.register(ActSerialRegister::A),
            snapshot.register(ActSerialRegister::A)
        );
        assert_eq!(
            image.register(ActSerialRegister::B),
            snapshot.register(ActSerialRegister::B)
        );
        assert_eq!(
            image.register(ActSerialRegister::C),
            snapshot.register(ActSerialRegister::C)
        );
        assert_eq!(image.final_chain(), machine.act.state.carry);
    }

    #[test]
    fn empty_selected_field_preserves_opcode_specific_carry_result() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.a[0] = 9;
        machine.act.state.p = 0x0f;
        machine.act.state.decimal = true;

        for operation in [0x0d, 0x16, 0x1a] {
            let mut candidate = machine.clone();
            let snapshot = ActSerialStateSnapshot::capture(&candidate.act.state);
            let word = (operation << 5) | 0x02;
            let image = ActSerialArithmeticResultImage::evaluate(&snapshot, word)
                .expect("arithmetic word must produce a serial result image");

            candidate
                .execute_word(word)
                .expect("architectural empty-field arithmetic must execute");

            assert_eq!(image.processed_digits(), 0);
            assert_eq!(
                image.register(ActSerialRegister::A),
                snapshot.register(ActSerialRegister::A)
            );
            assert_eq!(image.final_chain(), candidate.act.state.carry);
        }
    }

    #[test]
    fn clear_copy_exchange_shift_and_tests_match_architectural_results() {
        for operation in [
            0x00u16, 0x01, 0x02, 0x03, 0x04, 0x05, 0x06, 0x07, 0x08, 0x0e, 0x16, 0x17, 0x1a, 0x1b,
            0x1d, 0x1e, 0x1f,
        ] {
            let mut machine = Hp67ArchitecturalMachine::default();
            machine.act.state.a = std::array::from_fn(|digit| ((digit * 3 + 1) & 0x0f) as u8);
            machine.act.state.b = std::array::from_fn(|digit| ((digit * 5 + 2) & 0x0f) as u8);
            machine.act.state.c = std::array::from_fn(|digit| ((digit * 7 + 3) & 0x0f) as u8);
            machine.act.state.p = 7;
            machine.act.state.carry = true;
            let word = (operation << 5) | (6u16 << 2) | 0x02;
            compare_with_architectural(machine, word);
        }
    }

    #[test]
    fn every_arithmetic_operation_field_and_p_case_matches_architectural_oracle() {
        for decimal in [false, true] {
            for p in [0u8, 2, 7, 13, 15] {
                for operation in 0u16..=0x1f {
                    for field in 0u16..=7 {
                        let mut machine = Hp67ArchitecturalMachine::default();
                        let modulus = if decimal { 10 } else { 16 };
                        machine.act.state.a =
                            std::array::from_fn(|digit| ((digit * 3 + 1) % modulus) as u8);
                        machine.act.state.b =
                            std::array::from_fn(|digit| ((digit * 5 + 2) % modulus) as u8);
                        machine.act.state.c =
                            std::array::from_fn(|digit| ((digit * 7 + 3) % modulus) as u8);
                        machine.act.state.p = p;
                        machine.act.state.decimal = decimal;
                        machine.act.state.carry = true;

                        let word = (operation << 5) | (field << 2) | 0x02;
                        compare_with_architectural(machine, word);
                    }
                }
            }
        }
    }

    #[test]
    fn non_arithmetic_word_has_no_arithmetic_result_image() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        assert_eq!(
            ActSerialArithmeticResultImage::evaluate(&snapshot, 0x3e3),
            None
        );
    }
}
