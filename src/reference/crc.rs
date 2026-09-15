//! Semantic reference model for the HP-67/97 card-reader controller (CRC).
//!
//! This is an instruction-boundary behavioural oracle. It does not model motor
//! inertia, magnetic flux, sense amplifiers or electrical propagation timing.

use super::woodstock::{Register, OPCODE_MASK};

pub const CRC_FLAG_COUNT: usize = 12;
pub const CRC_CARD_WORD_BITS: u8 = 28;
pub const CRC_CARD_WORDS: usize = 34;
pub const CRC_RAM_WRITE_ADDRESS: u8 = 0x99;
pub const CRC_RAM_READ_ADDRESS: u8 = 0x9b;

pub const FLAG_BUFFER_READY: usize = 0;
pub const FLAG_PROGRAM_MODE: usize = 1;
pub const FLAG_DEFAULT_FUNCTION_ENABLE: usize = 4;
pub const FLAG_MERGE: usize = 5;
pub const FLAG_PAUSE: usize = 6;
pub const FLAG_MOTOR_ENABLE: usize = 9;
pub const FLAG_CARD_INSERTED: usize = 10;
pub const FLAG_WRITE_MODE: usize = 11;

const CARD_WORD_MASK: u32 = 0x0fff_ffff;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcInstruction {
    SetFlag { flag: u8 },
    TestFlagAndClear { flag: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcError {
    OpcodeOutOfRange(u16),
    FlagOutOfRange(u8),
    CardAlreadyInserted,
    CardInsertedWhileMotorRunning,
    NoCard,
    ReadWhileWriteMode,
    WriteWhileReadMode,
    MotorOff,
    EndOfCard,
    WriteProtected,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CardSide {
    pub write_protect: bool,
    pub dirty: bool,
    words: [u32; CRC_CARD_WORDS],
}

impl Default for CardSide {
    fn default() -> Self {
        Self {
            write_protect: false,
            dirty: false,
            words: [0; CRC_CARD_WORDS],
        }
    }
}

impl CardSide {
    pub fn word(&self, index: usize) -> Option<u32> {
        self.words.get(index).copied()
    }

    pub fn set_word(&mut self, index: usize, word: u32) -> bool {
        let Some(slot) = self.words.get_mut(index) else {
            return false;
        };
        *slot = word & CARD_WORD_MASK;
        true
    }

    pub const fn words(&self) -> &[u32; CRC_CARD_WORDS] {
        &self.words
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrcReference {
    flags: [bool; CRC_FLAG_COUNT],
    external_flags: [bool; CRC_FLAG_COUNT],
    card: Option<CardSide>,
    completed_card: Option<CardSide>,
    head_position: usize,
}

impl Default for CrcReference {
    fn default() -> Self {
        Self {
            flags: [false; CRC_FLAG_COUNT],
            external_flags: [false; CRC_FLAG_COUNT],
            card: None,
            completed_card: None,
            head_position: 0,
        }
    }
}

impl CrcReference {
    pub fn reset(&mut self) {
        // The CRC reset clears its internal flags but does not eject a card,
        // clear the externally driven switch inputs, or rewind the transport.
        self.flags = [false; CRC_FLAG_COUNT];
    }

    pub fn flag(&self, flag: usize) -> Option<bool> {
        self.flags.get(flag).copied()
    }

    pub fn external_flag(&self, flag: usize) -> Option<bool> {
        self.external_flags.get(flag).copied()
    }

    pub fn set_external_flag(&mut self, flag: u8, value: bool) -> Result<(), CrcError> {
        let index = usize::from(flag);
        if index >= CRC_FLAG_COUNT {
            return Err(CrcError::FlagOutOfRange(flag));
        }
        self.external_flags[index] = value;
        Ok(())
    }

    pub const fn head_position(&self) -> usize {
        self.head_position
    }

    pub const fn card_inserted(&self) -> bool {
        self.card.is_some()
    }

    pub fn insert_card(&mut self, card: CardSide) -> Result<(), CrcError> {
        if self.card.is_some() {
            return Err(CrcError::CardAlreadyInserted);
        }
        if self.flags[FLAG_MOTOR_ENABLE] {
            return Err(CrcError::CardInsertedWhileMotorRunning);
        }

        self.card = Some(card);
        self.head_position = 0;
        self.flags[FLAG_CARD_INSERTED] = true;
        self.update_flags();
        Ok(())
    }

    pub fn take_completed_card(&mut self) -> Option<CardSide> {
        self.completed_card.take()
    }

    pub fn execute_opcode(&mut self, opcode: u16) -> Result<Option<bool>, CrcError> {
        let instruction = decode_crc_opcode(opcode)?;
        match instruction {
            Some(CrcInstruction::SetFlag { flag }) => {
                self.set_internal_flag(flag, true)?;
                Ok(None)
            }
            Some(CrcInstruction::TestFlagAndClear { flag }) => {
                let index = usize::from(flag);
                let condition = self.flags[index] || self.external_flags[index];
                self.flags[index] = false;
                self.update_flags();
                Ok(Some(condition))
            }
            None => Ok(None),
        }
    }

    pub fn read_into_c(&mut self, c: &mut Register) -> Result<(), CrcError> {
        if self.card.is_none() {
            return Err(CrcError::NoCard);
        }
        if self.flags[FLAG_WRITE_MODE] {
            return Err(CrcError::ReadWhileWriteMode);
        }
        if !self.flags[FLAG_MOTOR_ENABLE] {
            return Err(CrcError::MotorOff);
        }
        if self.head_position >= CRC_CARD_WORDS {
            return Err(CrcError::EndOfCard);
        }

        let mut word = self.card.as_ref().expect("card checked above").words[self.head_position];
        self.head_position += 1;
        for index in 0..7 {
            let digit = (word & 0x0f) as u8;
            word >>= 4;
            c[index] = digit;
            c[7 + index] = digit;
        }
        self.update_flags();
        Ok(())
    }

    pub fn write_from_c(&mut self, c: &Register) -> Result<(), CrcError> {
        let Some(card) = self.card.as_mut() else {
            return Err(CrcError::NoCard);
        };
        if !self.flags[FLAG_WRITE_MODE] {
            return Err(CrcError::WriteWhileReadMode);
        }
        if card.write_protect {
            return Err(CrcError::WriteProtected);
        }
        if !self.flags[FLAG_MOTOR_ENABLE] {
            return Err(CrcError::MotorOff);
        }
        if self.head_position >= CRC_CARD_WORDS {
            return Err(CrcError::EndOfCard);
        }

        let mut word = 0u32;
        for index in (7..14).rev() {
            word = (word << 4) | u32::from(c[index] & 0x0f);
        }
        card.words[self.head_position] = word & CARD_WORD_MASK;
        card.dirty = true;
        self.head_position += 1;
        self.update_flags();
        Ok(())
    }

    fn set_internal_flag(&mut self, flag: u8, value: bool) -> Result<(), CrcError> {
        let index = usize::from(flag);
        if index >= CRC_FLAG_COUNT {
            return Err(CrcError::FlagOutOfRange(flag));
        }
        self.flags[index] = value;
        self.update_flags();
        Ok(())
    }

    fn update_flags(&mut self) {
        self.flags[FLAG_BUFFER_READY] = self.card.is_some() && self.flags[FLAG_MOTOR_ENABLE];

        if self.card.is_some() && self.head_position >= CRC_CARD_WORDS {
            self.flags[FLAG_CARD_INSERTED] = false;
            self.completed_card = self.card.take();
            self.head_position = 0;
            // Keep BUFFER_READY as calculated above until firmware tests and
            // clears it. This preserves the final-ready behaviour of the
            // instruction-level reference implementation.
        }
    }
}

pub fn decode_crc_opcode(opcode: u16) -> Result<Option<CrcInstruction>, CrcError> {
    if opcode > OPCODE_MASK {
        return Err(CrcError::OpcodeOutOfRange(opcode));
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
    fn opcode_families_cover_crc_flags_zero_through_eleven() {
        assert_eq!(
            decode_crc_opcode(0o100),
            Ok(Some(CrcInstruction::TestFlagAndClear { flag: 0 }))
        );
        assert_eq!(
            decode_crc_opcode(0o200),
            Ok(Some(CrcInstruction::SetFlag { flag: 1 }))
        );
        assert_eq!(
            decode_crc_opcode(0o060),
            Ok(Some(CrcInstruction::SetFlag { flag: 8 }))
        );
        assert_eq!(
            decode_crc_opcode(0o660),
            Ok(Some(CrcInstruction::SetFlag { flag: 11 }))
        );
        assert_eq!(
            decode_crc_opcode(0o760),
            Ok(Some(CrcInstruction::TestFlagAndClear { flag: 11 }))
        );
    }

    #[test]
    fn program_switch_can_be_supplied_as_an_external_flag() {
        let mut crc = CrcReference::default();
        crc.set_external_flag(FLAG_PROGRAM_MODE as u8, true)
            .expect("flag must exist");

        let result = crc.execute_opcode(0o300).expect("test/clear must execute");
        assert_eq!(result, Some(true));
        assert_eq!(crc.external_flag(FLAG_PROGRAM_MODE), Some(true));
    }

    #[test]
    fn motor_and_inserted_card_raise_buffer_ready() {
        let mut crc = CrcReference::default();
        crc.insert_card(CardSide::default())
            .expect("card insertion must succeed");
        assert_eq!(crc.flag(FLAG_BUFFER_READY), Some(false));

        crc.execute_opcode(0o260)
            .expect("motor-enable flag must set");
        assert_eq!(crc.flag(FLAG_MOTOR_ENABLE), Some(true));
        assert_eq!(crc.flag(FLAG_BUFFER_READY), Some(true));
    }

    #[test]
    fn card_read_duplicates_seven_nibbles_into_both_halves_of_c() {
        let mut side = CardSide::default();
        side.set_word(0, 0x0765_4321);
        let mut crc = CrcReference::default();
        crc.insert_card(side).expect("card must insert");
        crc.execute_opcode(0o260).expect("motor must start");

        let mut c = Register::zero();
        crc.read_into_c(&mut c).expect("card word must read");
        assert_eq!(&c.digits()[0..7], &[1, 2, 3, 4, 5, 6, 7]);
        assert_eq!(&c.digits()[7..14], &[1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn card_write_packs_high_half_of_c_and_marks_card_dirty() {
        let mut crc = CrcReference::default();
        crc.insert_card(CardSide::default())
            .expect("card must insert");
        crc.execute_opcode(0o260).expect("motor must start");
        crc.execute_opcode(0o660).expect("write-mode flag must set");

        let mut c = Register::zero();
        for index in 0..7 {
            c[7 + index] = (index + 1) as u8;
        }
        crc.write_from_c(&c).expect("card word must write");

        for _ in 1..CRC_CARD_WORDS {
            crc.write_from_c(&Register::zero())
                .expect("remaining card words must write");
        }
        let completed = crc
            .take_completed_card()
            .expect("card must complete after 34 words");
        assert!(completed.dirty);
        assert_eq!(completed.word(0), Some(0x0765_4321));
    }

    #[test]
    fn write_protect_is_enforced() {
        let mut side = CardSide {
            write_protect: true,
            ..CardSide::default()
        };
        side.set_word(0, 0x1234);
        let mut crc = CrcReference::default();
        crc.insert_card(side).expect("card must insert");
        crc.execute_opcode(0o260).expect("motor must start");
        crc.execute_opcode(0o660).expect("write-mode flag must set");

        assert_eq!(
            crc.write_from_c(&Register::zero()),
            Err(CrcError::WriteProtected)
        );
    }

    #[test]
    fn reset_preserves_inserted_card_and_external_switch_inputs() {
        let mut crc = CrcReference::default();
        crc.set_external_flag(FLAG_PROGRAM_MODE as u8, true)
            .expect("flag must exist");
        crc.insert_card(CardSide::default())
            .expect("card must insert");
        crc.reset();

        assert!(crc.card_inserted());
        assert_eq!(crc.external_flag(FLAG_PROGRAM_MODE), Some(true));
        assert_eq!(crc.flag(FLAG_CARD_INSERTED), Some(false));
    }
}
