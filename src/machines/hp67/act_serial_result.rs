//! Source-backed arithmetic result image for one serially executing ACT word.
//!
//! This module materializes the register result implied by the already modeled
//! b0..b55 arithmetic traversal without assigning a physical PHI write edge.
//! It is an independent serial-path oracle used to compare the evolving M3
//! implementation against the instruction-boundary architectural core.

use super::{
    act::{ActInstructionState, ActRegister, ACT_WORD_DIGITS},
    act_serial_execution::{ActSerialArithmeticAction, ActSerialExecution, ActSerialRegister},
    act_serial_state::ActSerialStateSnapshot,
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
    pub fn evaluate(snapshot: &ActSerialStateSnapshot, word: u16) -> Option<Self> {
        let mut execution = ActSerialExecution::new(word, ActInstructionState::Normal).ok()?;
        let mut image = Self {
            a: *snapshot.register(ActSerialRegister::A),
            b: *snapshot.register(ActSerialRegister::B),
            c: *snapshot.register(ActSerialRegister::C),
            final_chain: false,
            processed_digits: 0,
        };
        let mut chain = None;

        for word_bit in 0..BITS_PER_WORD {
            let coordinate = snapshot.arithmetic_coordinate(&execution)?;
            let initial_chain = match coordinate.action {
                ActSerialArithmeticAction::Add { initial_carry, .. }
                | ActSerialArithmeticAction::Subtract { initial_carry, .. } => initial_carry,
                _ => return None,
            };

            if coordinate.selected && coordinate.bit_in_digit == BITS_PER_DIGIT - 1 {
                let result = snapshot
                    .alu_digit_result(&execution, chain.unwrap_or(initial_chain))?;

                match coordinate.action {
                    ActSerialArithmeticAction::Add { destination, .. } => {
                        image.write_digit(destination, coordinate.digit, result.result_digit)?;
                    }
                    ActSerialArithmeticAction::Subtract {
                        destination: Some(destination),
                        ..
                    } => {
                        image.write_digit(destination, coordinate.digit, result.result_digit)?;
                    }
                    ActSerialArithmeticAction::Subtract {
                        destination: None, ..
                    } => {}
                    _ => unreachable!("ADD/SUB action checked above"),
                }

                chain = Some(result.chain_out);
                image.final_chain = result.chain_out;
                image.processed_digits = image.processed_digits.saturating_add(1);
            }

            execution.advance_word_bit(word_bit).ok()?;
        }

        (image.processed_digits != 0).then_some(image)
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

    fn write_digit(
        &mut self,
        register: ActSerialRegister,
        digit: u8,
        value: u8,
    ) -> Option<()> {
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
            .expect("test word must be an ADD/SUB arithmetic operation");

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

        assert_eq!(image.register(ActSerialRegister::A), snapshot.register(ActSerialRegister::A));
        assert_eq!(image.register(ActSerialRegister::B), snapshot.register(ActSerialRegister::B));
        assert_eq!(image.register(ActSerialRegister::C), snapshot.register(ActSerialRegister::C));
        assert_eq!(image.final_chain(), machine.act.state.carry);
    }

    #[test]
    fn non_additive_arithmetic_has_no_add_sub_result_image() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        assert_eq!(ActSerialArithmeticResultImage::evaluate(&snapshot, 0x11a), None);
    }
}
