//! Completed-word structural authority for focused ACT mode/latch specials.
//!
//! M14H moves final decimal/display/RAM-address latch state for a small special
//! family behind the shared b0..b55 structural execution lifetime. The b55
//! handoff is an authority boundary only; exact internal latch timing remains
//! source-blocked.

use super::{
    act::ActInstructionState,
    act_serial_execution::{ActSerialExecution, ActSerialRegister, ActSerialWordClass},
    act_serial_state::ActSerialStateSnapshot,
    timing::BITS_PER_WORD,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActSerialModeLatchAction {
    ToggleDisplay,
    DisplayOff,
    DecimalMode,
    Display14Digit,
    HexMode,
    SelectRamAddressFromC,
}

pub const fn decode_serial_mode_latch_action(
    execution: &ActSerialExecution,
) -> Option<ActSerialModeLatchAction> {
    let ActSerialWordClass::SpecialOrPeripheral { opcode } = execution.class() else {
        return None;
    };

    match opcode {
        0o0210 => Some(ActSerialModeLatchAction::ToggleDisplay),
        0o0310 => Some(ActSerialModeLatchAction::DisplayOff),
        0o1410 => Some(ActSerialModeLatchAction::DecimalMode),
        0o0320 => Some(ActSerialModeLatchAction::Display14Digit),
        0o0420 => Some(ActSerialModeLatchAction::HexMode),
        0o1160 => Some(ActSerialModeLatchAction::SelectRamAddressFromC),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialModeLatchResultImage {
    action: ActSerialModeLatchAction,
    decimal: bool,
    display_enable: bool,
    display_14_digit: bool,
    ram_address: u8,
    complete: bool,
}

impl ActSerialModeLatchResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
    ) -> Option<Self> {
        let action = decode_serial_mode_latch_action(execution)?;
        Some(Self {
            action,
            decimal: snapshot.decimal(),
            display_enable: snapshot.display_enable(),
            display_14_digit: snapshot.display_14_digit(),
            ram_address: snapshot.ram_address(),
            complete: false,
        })
    }

    pub fn evaluate(snapshot: &ActSerialStateSnapshot, word: u16) -> Option<Self> {
        let mut execution = ActSerialExecution::new(word, ActInstructionState::Normal).ok()?;
        let mut image = Self::begin(snapshot, &execution)?;
        for word_bit in 0..BITS_PER_WORD {
            execution.advance_word_bit(word_bit).ok()?;
        }
        image.complete_word(snapshot);
        Some(image)
    }

    pub fn complete_word(&mut self, snapshot: &ActSerialStateSnapshot) {
        if self.complete {
            return;
        }

        match self.action {
            ActSerialModeLatchAction::ToggleDisplay => {
                self.display_enable = !self.display_enable;
            }
            ActSerialModeLatchAction::DisplayOff => {
                self.display_enable = false;
            }
            ActSerialModeLatchAction::DecimalMode => {
                self.decimal = true;
            }
            ActSerialModeLatchAction::Display14Digit => {
                self.display_14_digit = true;
            }
            ActSerialModeLatchAction::HexMode => {
                self.decimal = false;
            }
            ActSerialModeLatchAction::SelectRamAddressFromC => {
                let c0 = snapshot
                    .register_digit(ActSerialRegister::C, 0)
                    .expect("C digit 0 must exist");
                let c1 = snapshot
                    .register_digit(ActSerialRegister::C, 1)
                    .expect("C digit 1 must exist");
                self.ram_address = (c1 << 4) | c0;
            }
        }

        self.complete = true;
    }

    pub const fn action(&self) -> ActSerialModeLatchAction {
        self.action
    }

    pub const fn decimal(&self) -> bool {
        self.decimal
    }

    pub const fn display_enable(&self) -> bool {
        self.display_enable
    }

    pub const fn display_14_digit(&self) -> bool {
        self.display_14_digit
    }

    pub const fn ram_address(&self) -> u8 {
        self.ram_address
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActArchitecturalCore, ActArchitecturalState, ActRamImage};

    fn compare_with_architectural(state: ActArchitecturalState, word: u16) {
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let image = ActSerialModeLatchResultImage::evaluate(&snapshot, word)
            .expect("test word must be in the M14H mode/latch family");

        let mut core = ActArchitecturalCore::default();
        core.state = state.clone();
        let mut ram = ActRamImage::hp67();
        core.execute_word(&mut ram, word)
            .expect("architectural mode/latch execution must succeed");

        assert!(image.is_complete());
        assert_eq!(image.decimal(), core.state.decimal);
        assert_eq!(image.display_enable(), core.state.display_enable);
        assert_eq!(image.display_14_digit(), core.state.display_14_digit);
        assert_eq!(image.ram_address(), core.state.ram_address);
    }

    #[test]
    fn fixed_mode_latch_family_matches_architectural_boundary_state() {
        for word in [0o0210u16, 0o0310, 0o1410, 0o0320, 0o0420, 0o1160] {
            let mut state = ActArchitecturalState::default();
            state.decimal = false;
            state.display_enable = true;
            state.display_14_digit = false;
            state.ram_address = 0xa7;
            state.c[0] = 0x0d;
            state.c[1] = 0x06;
            compare_with_architectural(state, word);
        }
    }

    #[test]
    fn display_toggle_matches_both_input_states() {
        for display_enable in [false, true] {
            let mut state = ActArchitecturalState::default();
            state.display_enable = display_enable;
            compare_with_architectural(state, 0o0210);
        }
    }

    #[test]
    fn ram_address_uses_pre_instruction_c_digits() {
        let mut state = ActArchitecturalState::default();
        state.ram_address = 0x11;
        state.c[0] = 0x0c;
        state.c[1] = 0x03;

        let snapshot = ActSerialStateSnapshot::capture(&state);
        let image = ActSerialModeLatchResultImage::evaluate(&snapshot, 0o1160).unwrap();
        assert_eq!(image.ram_address(), 0x3c);
    }

    #[test]
    fn non_mode_latch_words_have_no_m14h_result_image() {
        let snapshot = ActSerialStateSnapshot::capture(&ActArchitecturalState::default());
        assert_eq!(
            ActSerialModeLatchResultImage::evaluate(&snapshot, 0o0000),
            None
        );
        assert_eq!(
            ActSerialModeLatchResultImage::evaluate(&snapshot, 0o1060),
            None
        );
        assert_eq!(
            ActSerialModeLatchResultImage::evaluate(&snapshot, 0o0064),
            None
        );
    }
}
