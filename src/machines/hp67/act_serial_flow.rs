//! Structural successor-fetch and completed-word ACT flow authority.
//!
//! The successor is derived from immutable pre-instruction inputs, independently
//! of the architectural oracle. Its early availability is a scheduling bridge,
//! not evidence of an internal PC/stack write edge. Completion requires b55.

use super::{ActArchitecturalState, ActInstructionState, ACT_RETURN_STACK_DEPTH};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialFlowState {
    pub pc: u16,
    pub bank: u8,
    pub delayed_rom: Option<u8>,
    pub return_stack: [u16; ACT_RETURN_STACK_DEPTH],
    pub stack_pointer: u8,
}

impl ActSerialFlowState {
    pub fn capture(state: &ActArchitecturalState) -> Self {
        Self {
            pc: state.pc,
            bank: state.bank,
            delayed_rom: state.delayed_rom,
            return_stack: state.return_stack,
            stack_pointer: state.stack_pointer,
        }
    }

    pub fn apply(self, state: &mut ActArchitecturalState) {
        state.pc = self.pc;
        state.bank = self.bank;
        state.delayed_rom = self.delayed_rom;
        state.return_stack = self.return_stack;
        state.stack_pointer = self.stack_pointer;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialFlowResultImage {
    result: ActSerialFlowState,
    then_goto: bool,
    complete: bool,
}

impl ActSerialFlowResultImage {
    /// Preview only the fetch-visible successor and flow state. No live ACT
    /// state is mutated, and no architectural executor is invoked.
    pub fn begin(state: &ActArchitecturalState, word: u16) -> Self {
        let mut result = ActSerialFlowState::capture(state);
        result.pc = result.pc.wrapping_add(1) & 0x0fff;
        result.delayed_rom = None;
        let then_goto = state.instruction_state == ActInstructionState::ThenGoto;
        if then_goto {
            if !state.carry {
                result.pc = (result.pc & !0x03ff) | word;
            }
        } else {
            match word & 3 {
                1 => {
                    result.return_stack[usize::from(result.stack_pointer)] = result.pc;
                    result.stack_pointer =
                        (result.stack_pointer + 1) % ACT_RETURN_STACK_DEPTH as u8;
                    result.pc = (result.pc & !0x00ff) | (word >> 2);
                }
                3 => {
                    if !state.carry {
                        result.pc = (result.pc & !0x00ff) | (word >> 2);
                    }
                }
                0 => match word {
                    0o0020 => {
                        result.pc =
                            (result.pc & !0x00ff) | u16::from(state.key_buffer.unwrap_or(0));
                    }
                    0o0220 => {
                        result.pc =
                            (result.pc & !0x00ff) | u16::from((state.a[2] << 4) | state.a[1]);
                    }
                    0o1020 => {
                        result.stack_pointer = if result.stack_pointer == 0 {
                            (ACT_RETURN_STACK_DEPTH - 1) as u8
                        } else {
                            result.stack_pointer - 1
                        };
                        result.pc = result.return_stack[usize::from(result.stack_pointer)];
                    }
                    0o1060 => result.bank ^= 1,
                    _ => match word & 0o77 {
                        0o40 => {
                            result.pc = (((word & 0o1700) << 2) | (result.pc & 0o0377)) & 0x0fff
                        }
                        0o64 => result.delayed_rom = Some((word >> 6) as u8),
                        _ => {}
                    },
                },
                _ => {}
            }
        }
        // A previously selected delayed ROM overrides the page after this
        // operation, including a call/return; a newly armed selection survives.
        if let Some(rom) = state.delayed_rom {
            result.pc = (u16::from(rom & 0x0f) << 8) | (result.pc & 0x00ff);
        }
        Self {
            result,
            then_goto,
            complete: false,
        }
    }

    /// Restore oracle flow effects, including the implied-GOTO completion latch.
    /// Other instruction-state owners must not be overwritten.
    pub fn restore(self, before: ActSerialFlowState, state: &mut ActArchitecturalState) {
        before.apply(state);
        if self.then_goto { state.instruction_state = ActInstructionState::ThenGoto; }
    }

    pub const fn result(&self) -> ActSerialFlowState {
        self.result
    }
    pub const fn fetch_bank(&self) -> u8 {
        if self.result.pc < 0x0400 {
            0
        } else {
            self.result.bank & 1
        }
    }
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn complete_word(&mut self) {
        self.complete = true;
    }
    pub fn commit(self, state: &mut ActArchitecturalState) {
        assert!(
            self.complete,
            "flow authority requires completed b0..b55 execution"
        );
        self.result.apply(state);
        state.bank = self.fetch_bank();
        if self.then_goto {
            state.instruction_state = ActInstructionState::Normal;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::Hp67ArchitecturalMachine;

    #[test]
    fn m14i_all_words_match_composed_oracle_flow() {
        for pc in [0, 0x00ff, 0x03ff, 0x0400, 0x0068, 0x0fff] {
            for carry in [false, true] {
                for instruction_state in
                    [ActInstructionState::Normal, ActInstructionState::ThenGoto]
                {
                    for delayed_rom in [None, Some(0), Some(15)] {
                        for stack_pointer in 0..ACT_RETURN_STACK_DEPTH as u8 {
                            for word in 0..=0x03ff {
                                let mut machine = Hp67ArchitecturalMachine::default();
                                machine.act.state.pc = pc;
                                machine.act.state.bank = 1;
                                machine.act.state.carry = carry;
                                machine.act.state.instruction_state = instruction_state;
                                machine.act.state.delayed_rom = delayed_rom;
                                machine.act.state.stack_pointer = stack_pointer;
                                machine.act.state.return_stack = [0x0abc, 0x0321];
                                machine.act.state.key_buffer = Some(0o244);
                                machine.act.state.a[2] = 10;
                                machine.act.state.a[1] = 5;
                                let before = machine.act.state.clone();
                                let mut image = ActSerialFlowResultImage::begin(&before, word);
                                assert!(!image.is_complete());
                                if machine.execute_word(word).is_err() {
                                    continue;
                                }
                                assert_eq!(image.result(), ActSerialFlowState::capture(&machine.act.state),
                                    "pc={pc:03x} word={word:03x} carry={carry} state={instruction_state:?} delayed={delayed_rom:?} sp={stack_pointer}");
                                machine.prepare_hp67_fetch();
                                image.complete_word();
                                let mut actual = before;
                                image.commit(&mut actual);
                                assert_eq!(
                                    ActSerialFlowState::capture(&actual),
                                    ActSerialFlowState::capture(&machine.act.state)
                                );
                                if instruction_state == ActInstructionState::ThenGoto {
                                    assert_eq!(
                                        actual.instruction_state,
                                        ActInstructionState::Normal
                                    );
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn m14n_implied_goto_latch_is_restored_until_completed_word() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.instruction_state = ActInstructionState::ThenGoto;
        let before = machine.act.state.clone();
        let mut image = ActSerialFlowResultImage::begin(&before, 0o1060);
        machine.execute_word(0o1060).unwrap();
        assert_eq!(machine.act.state.instruction_state, ActInstructionState::Normal);
        image.restore(ActSerialFlowState::capture(&before), &mut machine.act.state);
        assert_eq!(machine.act.state.instruction_state, ActInstructionState::ThenGoto);
        assert_eq!(ActSerialFlowState::capture(&machine.act.state), ActSerialFlowState::capture(&before));
        assert!(!image.is_complete());
        image.complete_word();
        image.commit(&mut machine.act.state);
        assert_eq!(machine.act.state.instruction_state, ActInstructionState::Normal);
        // Ordinary flow restore cannot erase an arithmetic/control condition.
        let mut normal = ActArchitecturalState::default();
        let image = ActSerialFlowResultImage::begin(&normal, 0);
        normal.instruction_state = ActInstructionState::ThenGoto;
        image.restore(ActSerialFlowState::capture(&normal), &mut normal);
        assert_eq!(normal.instruction_state, ActInstructionState::ThenGoto);
    }

    #[test]
    fn m14i_delayed_rom_call_uses_incremented_return_address() {
        let state = ActArchitecturalState {
            pc: 0x0068,
            delayed_rom: Some(15),
            ..Default::default()
        };
        let image = ActSerialFlowResultImage::begin(&state, (0xc6 << 2) | 1);
        assert_eq!(image.result().pc, 0x0fc6);
        assert_eq!(image.result().return_stack[0], 0x0069);
        assert_eq!(image.result().delayed_rom, None);
    }

    #[test]
    fn m14i_then_goto_payload_is_not_decoded_as_bank_switch() {
        let state = ActArchitecturalState {
            pc: 0x0450,
            bank: 1,
            instruction_state: ActInstructionState::ThenGoto,
            ..Default::default()
        };
        let image = ActSerialFlowResultImage::begin(&state, 0o1060);
        assert_eq!(image.result().bank, 1);
        assert_eq!(image.result().pc, 0x0400 | 0o1060);
    }
}
