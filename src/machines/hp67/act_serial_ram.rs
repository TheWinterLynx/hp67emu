//! Completed-word RAM selector and clear-block authority.
//! This is logical installed-RAM authority, not a physical DATA transaction.

use super::{
    ActArchitecturalState, ActRamImage, ActRegister, ActSerialExecution, ActSerialStateSnapshot,
    ActSerialWordClass, ACT_WORD_DIGITS,
};

pub type ActSerialRamBlock = [Option<ActRegister>; 16];

pub fn capture_ram_block(ram: &ActRamImage, base: u8) -> ActSerialRamBlock {
    std::array::from_fn(|offset| ram.read((base & 0xf0) + offset as u8))
}

pub fn restore_ram_block(
    ram: &mut ActRamImage,
    base: u8,
    block: ActSerialRamBlock,
) -> Result<(), String> {
    for (offset, value) in block.into_iter().enumerate() {
        let address = (base & 0xf0) + offset as u8;
        if ram.read(address).is_some() != value.is_some() {
            return Err(format!("RAM topology changed at 0x{address:02x}"));
        }
    }
    for (offset, value) in block.into_iter().enumerate() {
        let address = (base & 0xf0) + offset as u8;
        if let Some(value) = value {
            if !ram.write(address, value) {
                return Err(format!("RAM restore failed at 0x{address:02x}"));
            }
        }
    }
    Ok(())
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialRamResultImage {
    ram_address: u8,
    clear_base: Option<u8>,
    complete: bool,
}

impl ActSerialRamResultImage {
    pub fn begin(
        snapshot: &ActSerialStateSnapshot,
        execution: &ActSerialExecution,
    ) -> Option<Self> {
        let ActSerialWordClass::SpecialOrPeripheral { opcode } = execution.class() else {
            return None;
        };
        let before = snapshot.ram_address();
        if opcode == 0o1260 {
            Some(Self {
                ram_address: before,
                clear_base: Some(before & 0xf0),
                complete: false,
            })
        } else if opcode != 0o0070 && matches!(opcode & 0o77, 0o50 | 0o70) {
            Some(Self {
                ram_address: (before & 0xf0) | ((opcode >> 6) as u8 & 0x0f),
                clear_base: None,
                complete: false,
            })
        } else {
            None
        }
    }

    pub const fn ram_address(&self) -> u8 {
        self.ram_address
    }
    pub const fn clear_base(&self) -> Option<u8> {
        self.clear_base
    }
    pub const fn is_complete(&self) -> bool {
        self.complete
    }
    pub fn complete_word(&mut self) {
        self.complete = true;
    }

    /// Installed slots become zero; absent slots stay absent. No physical-chip
    /// mapping or inferred 16-register bus sequence is introduced here.
    pub fn cleared_block(before: ActSerialRamBlock) -> ActSerialRamBlock {
        before.map(|slot| slot.map(|_| [0; ACT_WORD_DIGITS]))
    }

    pub fn commit(
        self,
        state: &mut ActArchitecturalState,
        ram: &mut ActRamImage,
    ) -> Result<(), String> {
        if !self.complete {
            return Err("RAM authority requires completed b0..b55 execution".into());
        }
        if let Some(base) = self.clear_base {
            let before = capture_ram_block(ram, base);
            restore_ram_block(ram, base, Self::cleared_block(before))?;
        }
        state.ram_address = self.ram_address;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActInstructionState, Hp67ArchitecturalMachine};

    #[test]
    fn m14k_all_selectors_match_composed_oracle_and_keep_unrelated_state() {
        for before_address in 0u8..=255 {
            for operand in 0u16..16 {
                for family in [0o50, 0o70] {
                    let word = (operand << 6) | family;
                    let mut machine = Hp67ArchitecturalMachine::default();
                    machine.act.state.ram_address = before_address;
                    machine.act.state.c = [7; ACT_WORD_DIGITS];
                    let snapshot = ActSerialStateSnapshot::capture(&machine.act.state);
                    let execution =
                        ActSerialExecution::new(word, ActInstructionState::Normal).unwrap();
                    let image = ActSerialRamResultImage::begin(&snapshot, &execution);
                    // Fixed C <- DATA does not select a new low address nibble.
                    if word == 0o0070 {
                        assert!(image.is_none());
                        continue;
                    }
                    let mut image = image.unwrap();
                    assert!(!image.is_complete());
                    let before = machine.clone();
                    if machine.execute_word(word).is_err() {
                        continue;
                    }
                    assert_eq!(image.ram_address(), machine.act.state.ram_address);
                    let mut actual = before;
                    image.complete_word();
                    image
                        .commit(&mut actual.act.state, &mut actual.ram)
                        .unwrap();
                    assert_eq!(actual.act.state.ram_address, machine.act.state.ram_address);
                    assert_eq!(actual.act.state.c, [7; ACT_WORD_DIGITS]);
                    assert_eq!(actual.ram, Hp67ArchitecturalMachine::default().ram);
                }
            }
        }
    }

    #[test]
    fn m14k_clear_block_matches_oracle_and_preserves_sparse_topology() {
        for base in 0u8..=255 {
            let mut machine = Hp67ArchitecturalMachine::default();
            machine.ram = ActRamImage::default();
            for start in [0u8, 15, 32, 127, 240, 255] {
                machine.ram.install_range(start, 1);
            }
            for address in 0u8..=255 {
                machine.ram.write(address, [9; ACT_WORD_DIGITS]);
            }
            machine.act.state.ram_address = base;
            let before = machine.clone();
            let execution = ActSerialExecution::new(0o1260, ActInstructionState::Normal).unwrap();
            let mut image = ActSerialRamResultImage::begin(
                &ActSerialStateSnapshot::capture(&machine.act.state),
                &execution,
            )
            .unwrap();
            machine.execute_word(0o1260).unwrap();
            let mut actual = before;
            assert!(image
                .commit(&mut actual.act.state, &mut actual.ram)
                .is_err());
            image.complete_word();
            image
                .commit(&mut actual.act.state, &mut actual.ram)
                .unwrap();
            assert_eq!(actual.ram, machine.ram, "base={base:02x}");
            assert_eq!(actual.act.state.ram_address, base);
        }
    }

    #[test]
    fn m14k_implied_goto_payload_and_m14h_c_selector_have_no_ram_control_image() {
        let state = ActArchitecturalState::default();
        let snapshot = ActSerialStateSnapshot::capture(&state);
        for word in [0o1260, 0o0050, 0o0170, 0o1160] {
            let execution = ActSerialExecution::new(word, ActInstructionState::ThenGoto).unwrap();
            assert!(ActSerialRamResultImage::begin(&snapshot, &execution).is_none());
        }
        let execution = ActSerialExecution::new(0o1160, ActInstructionState::Normal).unwrap();
        assert!(ActSerialRamResultImage::begin(&snapshot, &execution).is_none());
    }
}
