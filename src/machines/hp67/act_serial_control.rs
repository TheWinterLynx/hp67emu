//! Completed-word control-state image for selected ACT P/status instructions.
//!
//! M14F moves final P/status/condition authority for a focused special-opcode
//! family onto the structural execution lifetime without inventing an internal
//! bit or PHI write edge. The image becomes complete only when the bound
//! b0..b55 execution reaches the word boundary.

use super::{
    act::{ActInstructionState, ACT_STATUS_BITS, ACT_WORD_DIGITS},
    act_serial_execution::{ActSerialExecution, ActSerialWordClass},
    act_serial_state::ActSerialStateSnapshot,
    timing::BITS_PER_WORD,
};

const P_SET_MAP: [u8; 16] = [14, 4, 7, 8, 11, 2, 10, 12, 1, 3, 13, 6, 0, 9, 5, 14];
const P_TEST_MAP: [u8; 16] = [4, 8, 12, 2, 9, 1, 6, 3, 1, 13, 5, 0, 11, 10, 7, 4];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ActSerialControlAction {
    ClearStatus,
    SetStatus { index: u8 },
    ClearStatusBit { index: u8 },
    TestStatusClear { index: u8 },
    LoadConstantAndDecrementP { constant: u8 },
    TestStatusSet { index: u8 },
    TestPEqual { operand: u8 },
    TestPNotEqual { operand: u8 },
    SetP { operand: u8 },
    DecrementP,
    IncrementP,
}

pub const fn decode_serial_control_action(
    execution: &ActSerialExecution,
) -> Option<ActSerialControlAction> {
    let ActSerialWordClass::SpecialOrPeripheral { opcode } = execution.class() else {
        return None;
    };

    match opcode {
        0o0110 => return Some(ActSerialControlAction::ClearStatus),
        0o0620 => return Some(ActSerialControlAction::DecrementP),
        0o0720 => return Some(ActSerialControlAction::IncrementP),
        _ => {}
    }

    let operand = (opcode >> 6) as u8;
    match opcode & 0o77 {
        0o04 => Some(ActSerialControlAction::SetStatus { index: operand }),
        0o14 => Some(ActSerialControlAction::ClearStatusBit { index: operand }),
        0o24 => Some(ActSerialControlAction::TestStatusClear { index: operand }),
        0o30 => Some(ActSerialControlAction::LoadConstantAndDecrementP {
            constant: operand,
        }),
        0o34 => Some(ActSerialControlAction::TestStatusSet { index: operand }),
        0o44 => Some(ActSerialControlAction::TestPEqual { operand }),
        0o54 => Some(ActSerialControlAction::TestPNotEqual { operand }),
        0o74 => Some(ActSerialControlAction::SetP { operand }),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialControlResultImage {
    action: ActSerialControlAction,
    p: u8,
    p_change: [i8; 3],
    status: [bool; ACT_STATUS_BITS],
    carry: bool,
    previous_carry: bool,
    instruction_state: ActInstructionState,
    pre_carry: bool,
    complete: bool,
}

impl ActSerialControlResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
    ) -> Option<Self> {
        let action = decode_serial_control_action(execution)?;
        Some(Self {
            action,
            p: snapshot.p(),
            p_change: snapshot.p_change(),
            status: *snapshot.status(),
            carry: snapshot.carry(),
            previous_carry: false,
            instruction_state: snapshot.instruction_state(),
            pre_carry: snapshot.carry(),
            complete: false,
        })
    }

    pub fn evaluate(snapshot: &ActSerialStateSnapshot, word: u16) -> Option<Self> {
        let mut execution =
            ActSerialExecution::new(word, ActInstructionState::Normal).ok()?;
        let mut image = Self::begin(snapshot, &execution)?;

        for word_bit in 0..BITS_PER_WORD {
            execution.advance_word_bit(word_bit).ok()?;
        }
        image.complete_word();
        Some(image)
    }

    pub fn complete_word(&mut self) {
        if self.complete {
            return;
        }

        let old_change = self.p_change;
        self.p_change = [0, old_change[0], old_change[1]];
        self.previous_carry = self.pre_carry;
        self.carry = false;
        self.instruction_state = ActInstructionState::Normal;

        match self.action {
            ActSerialControlAction::ClearStatus => {
                for index in 0..ACT_STATUS_BITS {
                    if !matches!(index, 1 | 2 | 5 | 15) {
                        self.status[index] = false;
                    }
                }
            }
            ActSerialControlAction::SetStatus { index } => {
                self.status[usize::from(index)] = true;
            }
            ActSerialControlAction::ClearStatusBit { index } => {
                self.status[usize::from(index)] = false;
            }
            ActSerialControlAction::TestStatusClear { index } => {
                self.instruction_state = ActInstructionState::ThenGoto;
                self.carry = !self.status[usize::from(index)];
            }
            ActSerialControlAction::LoadConstantAndDecrementP { .. } => {
                self.p = if self.p == 0 {
                    (ACT_WORD_DIGITS - 1) as u8
                } else {
                    self.p - 1
                };
            }
            ActSerialControlAction::TestStatusSet { index } => {
                self.instruction_state = ActInstructionState::ThenGoto;
                self.carry = self.status[usize::from(index)];
            }
            ActSerialControlAction::TestPEqual { operand } => {
                self.apply_p_test(operand, true);
            }
            ActSerialControlAction::TestPNotEqual { operand } => {
                self.apply_p_test(operand, false);
            }
            ActSerialControlAction::SetP { operand } => {
                self.p = P_SET_MAP[usize::from(operand)];
            }
            ActSerialControlAction::DecrementP => {
                self.p_change[0] = -1;
                self.p = if self.p == 0 {
                    (ACT_WORD_DIGITS - 1) as u8
                } else {
                    self.p - 1
                };
            }
            ActSerialControlAction::IncrementP => {
                self.p_change[0] = 1;
                self.p = self.p.wrapping_add(1);
                if usize::from(self.p) >= ACT_WORD_DIGITS {
                    self.p = 0;
                }
            }
        }

        self.complete = true;
    }

    pub const fn action(&self) -> ActSerialControlAction {
        self.action
    }

    pub const fn p(&self) -> u8 {
        self.p
    }

    pub const fn p_change(&self) -> [i8; 3] {
        self.p_change
    }

    pub const fn status(&self) -> &[bool; ACT_STATUS_BITS] {
        &self.status
    }

    pub const fn carry(&self) -> bool {
        self.carry
    }

    pub const fn previous_carry(&self) -> bool {
        self.previous_carry
    }

    pub const fn instruction_state(&self) -> ActInstructionState {
        self.instruction_state
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }

    fn apply_p_test(&mut self, operand: u8, equal: bool) {
        let target = P_TEST_MAP[usize::from(operand)];
        self.instruction_state = ActInstructionState::ThenGoto;

        let equal_result = if target == 0 && self.p_change[1] == 1 && self.p_change[2] == 1 {
            self.p == 0 || self.p == 1
        } else {
            self.p == target
        };
        self.carry = if equal { !equal_result } else { equal_result };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActArchitecturalCore, ActArchitecturalState, ActRamImage};

    fn compare_with_architectural(state: ActArchitecturalState, word: u16) {
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let image = ActSerialControlResultImage::evaluate(&snapshot, word)
            .expect("test word must be in the M14F control family");
        let mut core = ActArchitecturalCore::default();
        core.state = state;
        let mut ram = ActRamImage::hp67();
        core.execute_word(&mut ram, word)
            .expect("architectural ACT control execution must succeed");

        assert!(image.is_complete());
        assert_eq!(image.p(), core.state.p);
        assert_eq!(image.p_change(), core.state.p_change);
        assert_eq!(image.status(), &core.state.status);
        assert_eq!(image.carry(), core.state.carry);
        assert_eq!(image.previous_carry(), core.state.previous_carry);
        assert_eq!(image.instruction_state(), core.state.instruction_state);
    }

    #[test]
    fn p_and_status_families_match_architectural_boundary_state() {
        let words = [
            0o0110u16, 0o0620, 0o0720, 0o0004, 0o0014, 0o0024, 0o0030, 0o0034, 0o0044,
            0o0054, 0o0074, 0o1704, 0o1714, 0o1724, 0o1730, 0o1734, 0o1744, 0o1754,
            0o1774,
        ];

        for word in words {
            let mut state = ActArchitecturalState::default();
            state.p = 7;
            state.p_change = [1, 1, 0];
            state.status = std::array::from_fn(|index| index % 3 == 0);
            state.carry = true;
            compare_with_architectural(state, word);
        }
    }

    #[test]
    fn p_wrap_history_special_case_matches_architectural_oracle() {
        let mut state = ActArchitecturalState::default();
        state.p = 1;
        state.p_change = [1, 1, 0];
        state.carry = false;

        // Operand 11 maps to P target zero. After begin-word history aging,
        // p_change[1] and [2] are both +1, matching the documented wrap case.
        compare_with_architectural(state, (0o13u16 << 6) | 0o44);
    }

    #[test]
    fn constant_load_moves_p_but_does_not_claim_c_authority() {
        let mut state = ActArchitecturalState::default();
        state.p = 4;
        state.c[4] = 9;
        let snapshot = ActSerialStateSnapshot::capture(&state);
        let word = (0o12u16 << 6) | 0o30;
        let image = ActSerialControlResultImage::evaluate(&snapshot, word).unwrap();

        assert_eq!(
            image.action(),
            ActSerialControlAction::LoadConstantAndDecrementP { constant: 0o12 }
        );
        assert_eq!(image.p(), 3);
        assert_eq!(state.c[4], 9);
    }

    #[test]
    fn non_control_words_have_no_m14f_result_image() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        assert_eq!(ActSerialControlResultImage::evaluate(&snapshot, 0o0000), None);
        assert_eq!(ActSerialControlResultImage::evaluate(&snapshot, 0x122), None);
    }
}
