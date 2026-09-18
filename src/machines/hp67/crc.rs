//! Independent architectural bring-up model for the HP-67 card-reader controller control interface.
//!
//! This module models only the firmware-visible CRC flag/control instructions needed during
//! power-on and idle firmware execution. Card transport, magnetic data timing and DATA-bus
//! read/write transfers remain separate physical-device milestones.

use super::isa::ROM_WORD_MASK;

pub const CRC_FLAG_COUNT: usize = 12;
pub const CRC_RAM_WRITE_ADDRESS: u8 = 0x99;
pub const CRC_RAM_READ_ADDRESS: u8 = 0x9b;
pub const CRC_FLAG_BUFFER_READY: usize = 0;
pub const CRC_FLAG_PROGRAM_MODE: usize = 1;
pub const CRC_FLAG_MOTOR_ON: usize = 9;
pub const CRC_FLAG_CARD_PRESENT: usize = 10;
pub const CRC_FLAG_WRITE_MODE: usize = 11;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcInstruction {
    SetFlag { flag: u8 },
    TestFlagAndClear { flag: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcArchitecturalError {
    OpcodeOutOfRange(u16),
    FlagOutOfRange(u8),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrcArchitecturalCore {
    flags: [bool; CRC_FLAG_COUNT],
    external_flags: [bool; CRC_FLAG_COUNT],
}

impl Default for CrcArchitecturalCore {
    fn default() -> Self {
        Self {
            flags: [false; CRC_FLAG_COUNT],
            external_flags: [false; CRC_FLAG_COUNT],
        }
    }
}

impl CrcArchitecturalCore {
    pub fn reset_control_flags(&mut self) {
        self.flags = [false; CRC_FLAG_COUNT];
    }

    pub fn flag(&self, flag: usize) -> Option<bool> {
        self.flags.get(flag).copied()
    }

    pub fn external_flag(&self, flag: usize) -> Option<bool> {
        self.external_flags.get(flag).copied()
    }

    pub fn set_external_flag(
        &mut self,
        flag: u8,
        value: bool,
    ) -> Result<(), CrcArchitecturalError> {
        let index = usize::from(flag);
        if index >= CRC_FLAG_COUNT {
            return Err(CrcArchitecturalError::FlagOutOfRange(flag));
        }
        self.external_flags[index] = value;
        Ok(())
    }

    /// Execute one CRC control opcode. Returns `Some(condition)` for test-and-clear.
    pub fn execute_opcode(&mut self, opcode: u16) -> Result<Option<bool>, CrcArchitecturalError> {
        let Some(instruction) = decode_crc_opcode(opcode)? else {
            return Ok(None);
        };

        match instruction {
            CrcInstruction::SetFlag { flag } => {
                let index = usize::from(flag);
                if index >= CRC_FLAG_COUNT {
                    return Err(CrcArchitecturalError::FlagOutOfRange(flag));
                }
                self.flags[index] = true;
                Ok(None)
            }
            CrcInstruction::TestFlagAndClear { flag } => {
                let index = usize::from(flag);
                if index >= CRC_FLAG_COUNT {
                    return Err(CrcArchitecturalError::FlagOutOfRange(flag));
                }
                let condition = self.flags[index] || self.external_flags[index];
                self.flags[index] = false;
                Ok(Some(condition))
            }
        }
    }
}

pub fn decode_crc_opcode(opcode: u16) -> Result<Option<CrcInstruction>, CrcArchitecturalError> {
    if opcode > ROM_WORD_MASK {
        return Err(CrcArchitecturalError::OpcodeOutOfRange(opcode));
    }

    for selector in 0u16..=7 {
        if selector != 0 && opcode == (selector << 7) {
            return Ok(Some(CrcInstruction::SetFlag {
                flag: crc_flag_number(opcode),
            }));
        }
        if opcode == (selector << 7) + 0o100 {
            return Ok(Some(CrcInstruction::TestFlagAndClear {
                flag: crc_flag_number(opcode),
            }));
        }
    }

    for selector in 0u16..=3 {
        if opcode == (selector << 7) + 0o060 {
            return Ok(Some(CrcInstruction::SetFlag {
                flag: crc_flag_number(opcode),
            }));
        }
        if opcode == (selector << 7) + 0o160 {
            return Ok(Some(CrcInstruction::TestFlagAndClear {
                flag: crc_flag_number(opcode),
            }));
        }
    }

    Ok(None)
}

fn crc_flag_number(opcode: u16) -> u8 {
    ((opcode >> 7) + ((opcode & 0o020) >> 1)) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn crc_opcode_families_cover_all_twelve_flags() {
        assert_eq!(
            decode_crc_opcode(0o100),
            Ok(Some(CrcInstruction::TestFlagAndClear { flag: 0 }))
        );
        assert_eq!(
            decode_crc_opcode(0o1000),
            Ok(Some(CrcInstruction::SetFlag { flag: 4 }))
        );
        assert_eq!(
            decode_crc_opcode(0o760),
            Ok(Some(CrcInstruction::TestFlagAndClear { flag: 11 }))
        );
    }

    #[test]
    fn external_program_switch_participates_in_test_without_being_cleared() {
        let mut crc = CrcArchitecturalCore::default();
        crc.set_external_flag(CRC_FLAG_PROGRAM_MODE as u8, true)
            .expect("program-mode flag exists");
        assert_eq!(crc.execute_opcode(0o300), Ok(Some(true)));
        assert_eq!(crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(true));
    }

    #[test]
    fn external_card_present_participates_without_being_cleared() {
        let mut crc = CrcArchitecturalCore::default();
        crc.set_external_flag(CRC_FLAG_CARD_PRESENT as u8, true)
            .expect("card-present flag exists");
        assert_eq!(crc.execute_opcode(0o560), Ok(Some(true)));
        assert_eq!(crc.external_flag(CRC_FLAG_CARD_PRESENT), Some(true));
    }
}
