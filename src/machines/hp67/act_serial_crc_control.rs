//! Completed-word CRC control flag and ACT S3 result authority.
//! No F2 pulse edge, sense-amplifier timing or DATA ownership is inferred.

use super::{
    decode_crc_opcode, ActArchitecturalState, ActSerialExecution, ActSerialStateSnapshot,
    ActSerialWordClass, CrcArchitecturalCore, CrcInstruction,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialCrcControlResultImage {
    instruction: CrcInstruction,
    flag: u8,
    previous_flag: bool,
    previous_status3: bool,
    result_flag: bool,
    result_status3: bool,
    condition: Option<bool>,
    complete: bool,
}

impl ActSerialCrcControlResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
        crc: &CrcArchitecturalCore,
    ) -> Result<Option<Self>, String> {
        let ActSerialWordClass::SpecialOrPeripheral { opcode } = execution.class() else {
            return Ok(None);
        };
        let Some(instruction) = decode_crc_opcode(opcode)
            .map_err(|error| format!("CRC structural decode failed: {error:?}"))?
        else {
            return Ok(None);
        };
        let flag = match instruction {
            CrcInstruction::SetFlag { flag } | CrcInstruction::TestFlagAndClear { flag } => flag,
        };
        let index = usize::from(flag);
        let previous_flag = crc
            .flag(index)
            .ok_or_else(|| format!("CRC structural flag {flag} is out of range"))?;
        let external = crc
            .external_flag(index)
            .ok_or_else(|| format!("CRC structural external flag {flag} is out of range"))?;
        let previous_status3 = snapshot.status()[3];
        let (result_flag, condition) = match instruction {
            CrcInstruction::SetFlag { .. } => (true, None),
            CrcInstruction::TestFlagAndClear { .. } => (false, Some(previous_flag || external)),
        };
        Ok(Some(Self {
            instruction,
            flag,
            previous_flag,
            previous_status3,
            result_flag,
            result_status3: previous_status3 || condition == Some(true),
            condition,
            complete: false,
        }))
    }

    pub const fn instruction(&self) -> CrcInstruction {
        self.instruction
    }
    pub const fn flag(&self) -> u8 {
        self.flag
    }
    pub const fn result_flag(&self) -> bool {
        self.result_flag
    }
    pub const fn result_status3(&self) -> bool {
        self.result_status3
    }
    pub const fn condition(&self) -> Option<bool> {
        self.condition
    }
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn complete_word(&mut self) {
        self.complete = true;
    }

    pub fn restore(
        self,
        state: &mut ActArchitecturalState,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<(), String> {
        crc.commit_control_flag(self.flag, self.previous_flag)
            .map_err(|error| format!("CRC structural restore failed: {error:?}"))?;
        state.status[3] = self.previous_status3;
        Ok(())
    }

    pub fn commit(
        self,
        state: &mut ActArchitecturalState,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<(), String> {
        if !self.complete {
            return Err("CRC control authority requires completed b0..b55 execution".into());
        }
        crc.commit_control_flag(self.flag, self.result_flag)
            .map_err(|error| format!("CRC structural commit failed: {error:?}"))?;
        state.status[3] = self.result_status3;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActInstructionState, Hp67ArchitecturalMachine, CRC_FLAG_COUNT};

    #[test]
    fn m14l_all_control_words_match_composed_oracle_for_flags_external_inputs_and_s3() {
        let mut cases = 0;
        for internal in [false, true] {
            for external in [false, true] {
                for status3 in [false, true] {
                    for word in 0..=0x03ff {
                        let mut machine = Hp67ArchitecturalMachine::default();
                        for flag in 0..CRC_FLAG_COUNT as u8 {
                            machine.crc.commit_control_flag(flag, internal).unwrap();
                            machine.crc.set_external_flag(flag, external).unwrap();
                        }
                        machine.act.state.status[3] = status3;
                        let execution =
                            ActSerialExecution::new(word, ActInstructionState::Normal).unwrap();
                        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
                        let Ok(Some(mut image)) = ActSerialCrcControlResultImage::begin(
                            &snapshot,
                            &execution,
                            &machine.crc,
                        ) else {
                            continue;
                        };
                        let before = machine.clone();
                        let oracle = machine.execute_word(word).unwrap();
                        assert_eq!(
                            image.result_flag(),
                            machine.crc.flag(usize::from(image.flag())).unwrap()
                        );
                        assert_eq!(image.result_status3(), machine.act.state.status[3]);
                        let super::super::Hp67ArchitecturalOperation::CrcControl {
                            instruction,
                            condition,
                        } = oracle.operation
                        else {
                            panic!("CRC ownership mismatch");
                        };
                        assert_eq!(image.instruction(), instruction);
                        assert_eq!(image.condition(), condition);
                        let mut actual = before.clone();
                        assert!(image
                            .commit(&mut actual.act.state, &mut actual.crc)
                            .is_err());
                        image.complete_word();
                        image
                            .commit(&mut actual.act.state, &mut actual.crc)
                            .unwrap();
                        assert_eq!(actual.crc, machine.crc);
                        assert_eq!(actual.act.state.status, machine.act.state.status);
                        // Restore must also preserve all unrelated peripheral state.
                        image
                            .restore(&mut actual.act.state, &mut actual.crc)
                            .unwrap();
                        assert_eq!(actual, before);
                        cases += 1;
                    }
                }
            }
        }
        assert!(cases > 100);
    }

    #[test]
    fn m14l_false_test_preserves_s3_and_external_input_survives_clear() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.status[3] = true;
        let execution = ActSerialExecution::new(0o100, ActInstructionState::Normal).unwrap();
        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
        let image = ActSerialCrcControlResultImage::begin(&snapshot, &execution, &machine.crc)
            .unwrap()
            .unwrap();
        assert_eq!(image.condition(), Some(false));
        assert!(image.result_status3());
        machine.crc.set_external_flag(0, true).unwrap();
        let mut image = ActSerialCrcControlResultImage::begin(&snapshot, &execution, &machine.crc)
            .unwrap()
            .unwrap();
        image.complete_word();
        image
            .commit(&mut machine.act.state, &mut machine.crc)
            .unwrap();
        assert_eq!(machine.crc.flag(0), Some(false));
        assert_eq!(machine.crc.external_flag(0), Some(true));
    }

    #[test]
    fn m14l_invalid_structural_flag_commit_preserves_crc_state() {
        let mut crc = CrcArchitecturalCore::default();
        crc.commit_control_flag(0, true).unwrap();
        let before = crc.clone();
        assert!(crc
            .commit_control_flag(CRC_FLAG_COUNT as u8, false)
            .is_err());
        assert_eq!(crc, before);
    }

    #[test]
    fn m14l_implied_goto_never_decodes_crc_control() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        for word in [0o100, 0o1000, 0o560] {
            let execution = ActSerialExecution::new(word, ActInstructionState::ThenGoto).unwrap();
            assert!(ActSerialCrcControlResultImage::begin(
                &snapshot,
                &execution,
                &CrcArchitecturalCore::default()
            )
            .unwrap()
            .is_none());
        }
    }
}
