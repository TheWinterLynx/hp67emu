//! Completed-word logical CRC data-port authority, not electrical DATA timing.
use super::{
    crc::CrcDataState, ActArchitecturalState, ActRegister, ActSerialExecution, ActSerialRegister,
    ActSerialStateSnapshot, ActSerialWordClass, CrcArchitecturalCore, Hp67ArchitecturalOperation,
    CRC_RAM_READ_ADDRESS, CRC_RAM_WRITE_ADDRESS,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialCrcDataResultImage {
    address: u8,
    write: bool,
    word: u32,
    previous_c: ActRegister,
    result_c: ActRegister,
    previous_crc: CrcDataState,
    result_crc: CrcDataState,
    complete: bool,
}

impl ActSerialCrcDataResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
        crc: &CrcArchitecturalCore,
    ) -> Result<Option<Self>, String> {
        Self::begin_from(snapshot, execution, |payload| {
            crc.stage_data_word(payload)
                .map(|(word, next)| (word, crc.data_state(), next))
        })
    }

    pub fn begin_owned(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
        data: &super::crc_owned_data::Hp67CrcOwnedData,
        write_mode: bool,
    ) -> Result<Option<Self>, String> {
        Self::begin_from(snapshot, execution, |payload| {
            data.stage_data_word(payload, write_mode)
                .map(|(word, next)| (word, data.state(), next))
        })
    }

    fn begin_from(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
        stage: impl FnOnce(
            Option<u32>,
        )
            -> Result<(u32, CrcDataState, CrcDataState), super::CrcArchitecturalError>,
    ) -> Result<Option<Self>, String> {
        let ActSerialWordClass::SpecialOrPeripheral { opcode } = execution.class() else {
            return Ok(None);
        };
        let current = snapshot.ram_address();
        let selected = (current & 0xf0) | ((opcode >> 6) & 0xf) as u8;
        let access = if opcode == 0o0070 && current == CRC_RAM_READ_ADDRESS {
            Some((current, false))
        } else if opcode == 0o1360 && current == CRC_RAM_WRITE_ADDRESS {
            Some((current, true))
        } else if opcode != 0o0070 && opcode & 0o77 == 0o70 && selected == CRC_RAM_READ_ADDRESS {
            Some((selected, false))
        } else if opcode & 0o77 == 0o50 && selected == CRC_RAM_WRITE_ADDRESS {
            Some((selected, true))
        } else {
            None
        };
        let Some((address, write)) = access else {
            return Ok(None);
        };
        let previous_c = *snapshot.register(ActSerialRegister::C);
        let payload = write.then(|| {
            previous_c[7..]
                .iter()
                .enumerate()
                .fold(0u32, |word, (digit, nibble)| {
                    word | (u32::from(nibble & 0xf) << (digit * 4))
                })
        });
        let (word, previous_crc, result_crc) = stage(payload)
            .map_err(|error| format!("CRC data staging at 0x{address:02x} failed: {error:?}"))?;
        let result_c = if write {
            previous_c
        } else {
            std::array::from_fn(|digit| ((word >> ((digit % 7) * 4)) & 0xf) as u8)
        };
        Ok(Some(Self {
            address,
            write,
            word,
            previous_c,
            result_c,
            previous_crc,
            result_crc,
            complete: false,
        }))
    }

    /// Frozen logical-port C result; authority still requires completed b55.
    pub const fn result_c(&self) -> ActRegister {
        self.result_c
    }

    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn complete_word(&mut self) {
        self.complete = true;
    }
    pub const fn operation(&self) -> Hp67ArchitecturalOperation {
        if self.write {
            Hp67ArchitecturalOperation::CrcDataWrite {
                address: self.address,
                card_word: self.word,
            }
        } else {
            Hp67ArchitecturalOperation::CrcDataRead {
                address: self.address,
                card_word: self.word,
            }
        }
    }
    pub fn matches_result(
        &self,
        state: &ActArchitecturalState,
        crc: &CrcArchitecturalCore,
    ) -> bool {
        state.c == self.result_c && crc.data_state() == self.result_crc
    }
    pub fn restore(&self, state: &mut ActArchitecturalState, crc: &mut CrcArchitecturalCore) {
        if !self.write {
            state.c = self.previous_c;
        }
        crc.apply_data_state(self.previous_crc);
    }
    pub fn commit_owned(
        &self,
        data: &mut super::crc_owned_data::Hp67CrcOwnedData,
    ) -> Result<(), String> {
        if !self.complete {
            return Err("CRC owned DATA authority requires completed b0..b55 execution".into());
        }
        data.commit_staged(self.previous_crc, self.result_crc)
    }
    pub fn commit(
        &self,
        state: &mut ActArchitecturalState,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<(), String> {
        if !self.complete {
            return Err("CRC data authority requires completed b0..b55 execution".into());
        }
        if state.c != self.previous_c || crc.data_state() != self.previous_crc {
            return Err("CRC data pre-state changed during structural execution".into());
        }
        crc.apply_data_state(self.result_crc);
        if !self.write {
            state.c = self.result_c;
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{
        ActInstructionState, Hp67ArchitecturalMachine, CRC_FLAG_F7_STATUS, CRC_FLAG_WRITE_MODE,
    };

    #[test]
    fn m14m_crc_data_matches_oracle_across_addresses_opcodes_and_fifo_wrap() {
        let mut cases = 0;
        for address in 0u8..=255 {
            for opcode in 0u16..=0x3ff {
                let execution =
                    ActSerialExecution::new(opcode, ActInstructionState::Normal).unwrap();
                let mut machine = Hp67ArchitecturalMachine::default();
                machine.act.state.ram_address = address;
                machine.act.state.c = std::array::from_fn(|digit| ((digit * 3 + 1) & 15) as u8);
                machine
                    .crc
                    .commit_control_flag(CRC_FLAG_WRITE_MODE as u8, true)
                    .unwrap();
                // Move both FIFO heads away from zero and retain a queued record.
                machine.crc.present_read_word(0x1111111).unwrap();
                machine.crc.take_read_word().unwrap();
                machine.crc.present_read_word(0x7654321).unwrap();
                machine.crc.present_read_word(0xabcdef0).unwrap();
                machine.crc.queue_write_word(0x2222222).unwrap();
                machine.crc.take_queued_write_word();
                machine.crc.queue_write_word(0x3333333).unwrap();
                machine
                    .crc
                    .commit_control_flag(CRC_FLAG_F7_STATUS as u8, true)
                    .unwrap();
                let before = machine.clone();
                let Some(mut image) = ActSerialCrcDataResultImage::begin(
                    &ActSerialStateSnapshot::capture(&machine.act.state),
                    &execution,
                    &machine.crc,
                )
                .unwrap() else {
                    continue;
                };
                assert_eq!(machine, before);
                let oracle = machine.execute_word(opcode).unwrap();
                assert_eq!(image.operation(), oracle.operation);
                assert!(image.matches_result(&machine.act.state, &machine.crc));
                let expected = machine.clone();
                image.restore(&mut machine.act.state, &mut machine.crc);
                assert_eq!(machine.crc, before.crc);
                assert_eq!(machine.act.state.c, before.act.state.c);
                assert!(image
                    .commit(&mut machine.act.state, &mut machine.crc)
                    .is_err());
                image.complete_word();
                image
                    .commit(&mut machine.act.state, &mut machine.crc)
                    .unwrap();
                assert_eq!(machine, expected);
                cases += 1;
            }
        }
        assert_eq!(cases, 34);
    }

    #[test]
    fn m14m_crc_data_failure_and_stale_commit_are_transactional() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_READ_ADDRESS;
        let read = ActSerialExecution::new(0o0070, ActInstructionState::Normal).unwrap();
        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
        let before = machine.clone();
        assert!(
            ActSerialCrcDataResultImage::begin(&snapshot, &read, &machine.crc)
                .unwrap_err()
                .contains("ReadBufferEmpty")
        );
        assert_eq!(machine, before);
        machine.act.state.ram_address = CRC_RAM_WRITE_ADDRESS;
        let write = ActSerialExecution::new(0o1360, ActInstructionState::Normal).unwrap();
        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
        assert!(
            ActSerialCrcDataResultImage::begin(&snapshot, &write, &machine.crc)
                .unwrap_err()
                .contains("WriteModeInactive")
        );
        machine
            .crc
            .commit_control_flag(CRC_FLAG_WRITE_MODE as u8, true)
            .unwrap();
        let mut image = ActSerialCrcDataResultImage::begin(&snapshot, &write, &machine.crc)
            .unwrap()
            .unwrap();
        machine.crc.queue_write_word(1).unwrap();
        let changed = machine.clone();
        image.complete_word();
        assert!(image
            .commit(&mut machine.act.state, &mut machine.crc)
            .is_err());
        assert_eq!(machine, changed);
        machine.crc.queue_write_word(2).unwrap();
        let full = machine.clone();
        assert!(
            ActSerialCrcDataResultImage::begin(&snapshot, &write, &machine.crc)
                .unwrap_err()
                .contains("WriteBufferFull")
        );
        assert_eq!(machine, full);
    }

    #[test]
    fn m14m_implied_goto_is_not_crc_data_and_commit_preserves_external_inputs() {
        let mut machine = Hp67ArchitecturalMachine::default();
        machine.act.state.ram_address = CRC_RAM_READ_ADDRESS;
        machine.crc.present_read_word(0xfffffff).unwrap();
        let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
        let implied = ActSerialExecution::new(0o0070, ActInstructionState::ThenGoto).unwrap();
        assert!(
            ActSerialCrcDataResultImage::begin(&snapshot, &implied, &machine.crc)
                .unwrap()
                .is_none()
        );
        let read = ActSerialExecution::new(0o0070, ActInstructionState::Normal).unwrap();
        let mut image = ActSerialCrcDataResultImage::begin(&snapshot, &read, &machine.crc)
            .unwrap()
            .unwrap();
        machine.crc.set_external_flag(10, true).unwrap();
        machine.crc.commit_control_flag(9, true).unwrap();
        image.complete_word();
        image
            .commit(&mut machine.act.state, &mut machine.crc)
            .unwrap();
        assert_eq!(machine.crc.external_flag(10), Some(true));
        assert_eq!(machine.crc.flag(9), Some(true));
        assert_eq!(machine.act.state.c, [15; 14]);
    }
}
