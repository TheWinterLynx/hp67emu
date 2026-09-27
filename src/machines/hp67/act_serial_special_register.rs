//! Completed-word structural result image for ACT special register-transfer instructions.
//!
//! M14G moves the final data-register state for a focused non-arithmetic special
//! family behind the structural b0..b55 execution lifetime. Exact internal
//! register-write and PHI timing remains source-blocked.

use super::{
    act::{ActInstructionState, ActRegister, ACT_WORD_DIGITS},
    act_serial_execution::{ActSerialExecution, ActSerialRegister, ActSerialWordClass},
    act_serial_state::ActSerialStateSnapshot,
    timing::BITS_PER_WORD,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActSerialSpecialRegisterAction {
    ClearWorkingStack,
    ExchangeCM1,
    CopyM1ToC,
    ExchangeCM2,
    CopyM2ToC,
    CopyStackToAAndDrop,
    RotateStackDown,
    CopyYToA,
    PushCIntoStack,
    CopyFToA0,
    ExchangeA0F,
    CopyKeyToA,
    RotateARight,
    LoadConstant { constant: u8 },
}

pub const fn decode_serial_special_register_action(
    execution: &ActSerialExecution,
) -> Option<ActSerialSpecialRegisterAction> {
    let ActSerialWordClass::SpecialOrPeripheral { opcode } = execution.class() else {
        return None;
    };

    match opcode {
        0o0010 => return Some(ActSerialSpecialRegisterAction::ClearWorkingStack),
        0o0410 => return Some(ActSerialSpecialRegisterAction::ExchangeCM1),
        0o0510 => return Some(ActSerialSpecialRegisterAction::CopyM1ToC),
        0o0610 => return Some(ActSerialSpecialRegisterAction::ExchangeCM2),
        0o0710 => return Some(ActSerialSpecialRegisterAction::CopyM2ToC),
        0o1010 => return Some(ActSerialSpecialRegisterAction::CopyStackToAAndDrop),
        0o1110 => return Some(ActSerialSpecialRegisterAction::RotateStackDown),
        0o1210 => return Some(ActSerialSpecialRegisterAction::CopyYToA),
        0o1310 => return Some(ActSerialSpecialRegisterAction::PushCIntoStack),
        0o1610 => return Some(ActSerialSpecialRegisterAction::CopyFToA0),
        0o1710 => return Some(ActSerialSpecialRegisterAction::ExchangeA0F),
        0o0120 => return Some(ActSerialSpecialRegisterAction::CopyKeyToA),
        0o0520 => return Some(ActSerialSpecialRegisterAction::RotateARight),
        _ => {}
    }

    let operand = (opcode >> 6) as u8;
    match opcode & 0o77 {
        0o30 => Some(ActSerialSpecialRegisterAction::LoadConstant { constant: operand }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialSpecialRegisterResultImage {
    action: ActSerialSpecialRegisterAction,
    a: ActRegister,
    b: ActRegister,
    c: ActRegister,
    y: ActRegister,
    z: ActRegister,
    t: ActRegister,
    m1: ActRegister,
    m2: ActRegister,
    f: u8,
    complete: bool,
}

impl ActSerialSpecialRegisterResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
    ) -> Option<Self> {
        let action = decode_serial_special_register_action(execution)?;
        Some(Self {
            action,
            a: *snapshot.register(ActSerialRegister::A),
            b: *snapshot.register(ActSerialRegister::B),
            c: *snapshot.register(ActSerialRegister::C),
            y: *snapshot.y(),
            z: *snapshot.z(),
            t: *snapshot.t(),
            m1: *snapshot.m1(),
            m2: *snapshot.m2(),
            f: snapshot.f(),
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
            ActSerialSpecialRegisterAction::ClearWorkingStack => {
                self.a = [0; ACT_WORD_DIGITS];
                self.b = [0; ACT_WORD_DIGITS];
                self.c = [0; ACT_WORD_DIGITS];
                self.y = [0; ACT_WORD_DIGITS];
                self.z = [0; ACT_WORD_DIGITS];
                self.t = [0; ACT_WORD_DIGITS];
            }
            ActSerialSpecialRegisterAction::ExchangeCM1 => {
                core::mem::swap(&mut self.c, &mut self.m1);
            }
            ActSerialSpecialRegisterAction::CopyM1ToC => {
                self.c = self.m1;
            }
            ActSerialSpecialRegisterAction::ExchangeCM2 => {
                core::mem::swap(&mut self.c, &mut self.m2);
            }
            ActSerialSpecialRegisterAction::CopyM2ToC => {
                self.c = self.m2;
            }
            ActSerialSpecialRegisterAction::CopyStackToAAndDrop => {
                self.a = self.y;
                self.y = self.z;
                self.z = self.t;
            }
            ActSerialSpecialRegisterAction::RotateStackDown => {
                let old_c = self.c;
                self.c = self.y;
                self.y = self.z;
                self.z = self.t;
                self.t = old_c;
            }
            ActSerialSpecialRegisterAction::CopyYToA => {
                self.a = self.y;
            }
            ActSerialSpecialRegisterAction::PushCIntoStack => {
                self.t = self.z;
                self.z = self.y;
                self.y = self.c;
            }
            ActSerialSpecialRegisterAction::CopyFToA0 => {
                self.a[0] = self.f & 0x0f;
            }
            ActSerialSpecialRegisterAction::ExchangeA0F => {
                core::mem::swap(&mut self.a[0], &mut self.f);
                self.a[0] &= 0x0f;
                self.f &= 0x0f;
            }
            ActSerialSpecialRegisterAction::CopyKeyToA => {
                let key = snapshot.key_buffer().unwrap_or(0);
                self.a[2] = (key >> 4) & 0x0f;
                self.a[1] = key & 0x0f;
            }
            ActSerialSpecialRegisterAction::RotateARight => {
                self.a.rotate_right(1);
            }
            ActSerialSpecialRegisterAction::LoadConstant { constant } => {
                let p = usize::from(snapshot.p());
                if p < ACT_WORD_DIGITS {
                    self.c[p] = constant & 0x0f;
                }
            }
        }

        self.complete = true;
    }

    pub const fn action(&self) -> ActSerialSpecialRegisterAction {
        self.action
    }

    pub const fn a(&self) -> &ActRegister {
        &self.a
    }

    pub const fn b(&self) -> &ActRegister {
        &self.b
    }

    pub const fn c(&self) -> &ActRegister {
        &self.c
    }

    pub const fn y(&self) -> &ActRegister {
        &self.y
    }

    pub const fn z(&self) -> &ActRegister {
        &self.z
    }

    pub const fn t(&self) -> &ActRegister {
        &self.t
    }

    pub const fn m1(&self) -> &ActRegister {
        &self.m1
    }

    pub const fn m2(&self) -> &ActRegister {
        &self.m2
    }

    pub const fn f(&self) -> u8 {
        self.f
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActArchitecturalCore, ActArchitecturalState, ActRamImage};

    fn patterned_register(seed: u8) -> ActRegister {
        std::array::from_fn(|digit| (seed.wrapping_add((digit as u8).wrapping_mul(3))) & 0x0f)
    }

    fn compare_with_architectural(state: ActArchitecturalState, word: u16) {
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let image = ActSerialSpecialRegisterResultImage::evaluate(&snapshot, word)
            .expect("test word must be in the M14G special-register family");

        let mut core = ActArchitecturalCore::default();
        core.state = state;
        let mut ram = ActRamImage::hp67();
        core.execute_word(&mut ram, word)
            .expect("architectural ACT special-register execution must succeed");

        assert!(image.is_complete());
        assert_eq!(image.a(), &core.state.a);
        assert_eq!(image.b(), &core.state.b);
        assert_eq!(image.c(), &core.state.c);
        assert_eq!(image.y(), &core.state.y);
        assert_eq!(image.z(), &core.state.z);
        assert_eq!(image.t(), &core.state.t);
        assert_eq!(image.m1(), &core.state.m1);
        assert_eq!(image.m2(), &core.state.m2);
        assert_eq!(image.f(), core.state.f);
    }

    fn nontrivial_state() -> ActArchitecturalState {
        let mut state = ActArchitecturalState::default();
        state.a = patterned_register(1);
        state.b = patterned_register(2);
        state.c = patterned_register(3);
        state.y = patterned_register(4);
        state.z = patterned_register(5);
        state.t = patterned_register(6);
        state.m1 = patterned_register(7);
        state.m2 = patterned_register(8);
        state.f = 0x0b;
        state.key_buffer = Some(0x6d);
        state.p = 5;
        state
    }

    #[test]
    fn fixed_special_register_families_match_architectural_boundary_state() {
        for word in [
            0o0010u16, 0o0410, 0o0510, 0o0610, 0o0710, 0o1010, 0o1110, 0o1210, 0o1310, 0o1610,
            0o1710, 0o0120, 0o0520,
        ] {
            compare_with_architectural(nontrivial_state(), word);
        }
    }

    #[test]
    fn key_to_a_matches_none_and_present_key_cases() {
        let mut no_key = nontrivial_state();
        no_key.key_buffer = None;
        compare_with_architectural(no_key, 0o0120);

        let mut key = nontrivial_state();
        key.key_buffer = Some(0xa5);
        compare_with_architectural(key, 0o0120);
    }

    #[test]
    fn load_constant_matches_all_operands_and_representative_p_values() {
        for operand in 0u16..=15 {
            for p in [0u8, 1, 7, 13, 15] {
                let mut state = nontrivial_state();
                state.p = p;
                compare_with_architectural(state, (operand << 6) | 0o30);
            }
        }
    }

    #[test]
    fn non_special_register_words_have_no_m14g_result_image() {
        let snapshot = ActSerialStateSnapshot::capture(&ActArchitecturalState::default());
        assert_eq!(
            ActSerialSpecialRegisterResultImage::evaluate(&snapshot, 0o0000),
            None
        );
        assert_eq!(
            ActSerialSpecialRegisterResultImage::evaluate(&snapshot, 0x122),
            None
        );
        assert_eq!(
            ActSerialSpecialRegisterResultImage::evaluate(&snapshot, 0o0210),
            None
        );
    }
}
