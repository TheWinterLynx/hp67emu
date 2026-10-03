//! Completed-word authority for ACT boundary state not already owned by M14F.
//! Exact internal latch edges remain source-blocked.

use super::{
    act_serial_control::decode_serial_control_action,
    ActArchitecturalState, ActInstructionState, ActSerialExecution,
    ActSerialStateSnapshot, ActSerialWordClass,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialBoundaryState {
    pub p_change: [i8; 3],
    pub previous_carry: bool,
    pub carry: bool,
    pub instruction_state: ActInstructionState,
}

impl ActSerialBoundaryState {
    pub fn capture(state: &ActArchitecturalState) -> Self {
        Self { p_change: state.p_change, previous_carry: state.previous_carry,
            carry: state.carry, instruction_state: state.instruction_state }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialBoundaryResultImage {
    result: ActSerialBoundaryState,
    owns_carry: bool,
    owns_instruction_state: bool,
    complete: bool,
}

impl ActSerialBoundaryResultImage {
    pub fn begin(snapshot: &ActSerialStateSnapshot, execution: &ActSerialExecution) -> Option<Self> {
        // M14F already owns these prelude effects for its selected family.
        if decode_serial_control_action(execution).is_some() { return None; }
        let arithmetic = match execution.class() {
            ActSerialWordClass::Arithmetic { operation, .. } => Some(operation),
            _ => None,
        };
        let changes = snapshot.p_change();
        Some(Self {
            result: ActSerialBoundaryState {
                p_change: [0, changes[0], changes[1]],
                previous_carry: snapshot.carry(),
                carry: false,
                instruction_state: if arithmetic.is_some_and(|operation| (0x16..=0x1b).contains(&operation)) {
                    ActInstructionState::ThenGoto
                } else { ActInstructionState::Normal },
            },
            // M14E owns every arithmetic carry result, including clear/shift.
            owns_carry: arithmetic.is_none(),
            // M14I retains implied-GOTO completion. Only arithmetic condition
            // latches migrate here; peripheral/status effects keep their owner.
            owns_instruction_state: arithmetic.is_some(),
            complete: false,
        })
    }

    pub const fn result(&self) -> ActSerialBoundaryState { self.result }
    pub const fn owns_carry(&self) -> bool { self.owns_carry }
    pub const fn owns_instruction_state(&self) -> bool { self.owns_instruction_state }
    pub const fn is_complete(&self) -> bool { self.complete }
    pub fn complete_word(&mut self) { self.complete = true; }

    /// Restore only migrated fields; never overwrite another family's state.
    pub fn restore(self, before: ActSerialBoundaryState, state: &mut ActArchitecturalState) {
        state.p_change = before.p_change;
        state.previous_carry = before.previous_carry;
        if self.owns_carry { state.carry = before.carry; }
        if self.owns_instruction_state { state.instruction_state = before.instruction_state; }
    }

    pub fn matches(self, expected: ActSerialBoundaryState) -> bool {
        self.result.p_change == expected.p_change
            && self.result.previous_carry == expected.previous_carry
            && (!self.owns_carry || self.result.carry == expected.carry)
            && (!self.owns_instruction_state || self.result.instruction_state == expected.instruction_state)
    }

    pub fn commit(self, state: &mut ActArchitecturalState) {
        assert!(self.complete, "boundary authority requires completed b0..b55 execution");
        self.restore(self.result, state);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::Hp67ArchitecturalMachine;

    #[test]
    fn m14j_all_words_match_composed_oracle_owned_boundary_state() {
        for carry in [false, true] {
            for instruction_state in [ActInstructionState::Normal, ActInstructionState::ThenGoto] {
                for p_change in [[0,0,0], [1,1,-1], [-1,0,1]] {
                    for word in 0..=0x03ff {
                        let mut machine = Hp67ArchitecturalMachine::default();
                        machine.act.state.carry = carry;
                        machine.act.state.previous_carry = !carry;
                        machine.act.state.instruction_state = instruction_state;
                        machine.act.state.p_change = p_change;
                        let before = machine.act.state.clone();
                        let execution = ActSerialExecution::new(word, instruction_state).unwrap();
                        let image = ActSerialBoundaryResultImage::begin(&ActSerialStateSnapshot::capture(&before), &execution);
                        if let Some(mut image) = image {
                            assert!(!image.is_complete());
                            if machine.execute_word(word).is_err() { continue; }
                            assert!(image.matches(ActSerialBoundaryState::capture(&machine.act.state)), "word={word:03x} state={instruction_state:?} carry={carry} p_change={p_change:?}");
                            image.complete_word();
                            let mut actual = before;
                            let unrelated = (actual.status, actual.p, actual.pc);
                            image.commit(&mut actual);
                            assert!(image.matches(ActSerialBoundaryState::capture(&actual)));
                            assert_eq!((actual.status, actual.p, actual.pc), unrelated);
                        }
                    }
                }
            }
        }
    }

    #[test]
    fn m14j_control_family_keeps_exclusive_m14f_ownership() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        for word in [0o0110, 0o0620, 0o0720, 0o0024, 0o0030, 0o0074] {
            let execution = ActSerialExecution::new(word, ActInstructionState::Normal).unwrap();
            assert!(ActSerialBoundaryResultImage::begin(&snapshot, &execution).is_none());
        }
    }

    #[test]
    fn m14j_arithmetic_conditions_do_not_overwrite_arithmetic_carry() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        for operation in 0u16..32 {
            let execution = ActSerialExecution::new((operation << 5) | 2, ActInstructionState::Normal).unwrap();
            let mut image = ActSerialBoundaryResultImage::begin(&snapshot, &execution).unwrap();
            assert!(!image.owns_carry());
            assert!(image.owns_instruction_state());
            image.complete_word();
            let mut actual = state.clone();
            actual.carry = true;
            image.commit(&mut actual);
            assert!(actual.carry);
            assert_eq!(actual.instruction_state == ActInstructionState::ThenGoto, (0x16..=0x1b).contains(&operation));
        }
    }
}
