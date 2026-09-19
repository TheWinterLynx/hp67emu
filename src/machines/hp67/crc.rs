//! Independent architectural bring-up model for the HP-67 card-reader controller control interface.
//!
//! This module models the firmware-visible CRC flag/control interface plus the
//! instruction-boundary 28-bit read/write buffers. Magnetic flux, sense-amplifier
//! timing and electrical DATA-bus transfer remain separate physical-device milestones.

use super::isa::ROM_WORD_MASK;

pub const CRC_FLAG_COUNT: usize = 12;
pub const CRC_RAM_WRITE_ADDRESS: u8 = 0x99;
pub const CRC_RAM_READ_ADDRESS: u8 = 0x9b;
pub const CRC_CARD_WORD_BITS: u8 = 28;
pub const CRC_CARD_WORD_MASK: u32 = (1u32 << CRC_CARD_WORD_BITS) - 1;
pub const CRC_FLAG_BUFFER_READY: usize = 0;
pub const CRC_FLAG_PROGRAM_MODE: usize = 1;
pub const CRC_FLAG_F7_STATUS: usize = 7;
pub const CRC_FLAG_MOTOR_ON: usize = 9;
pub const CRC_FLAG_CARD_PRESENT: usize = 10;
pub const CRC_FLAG_WRITE_MODE: usize = 11;
pub const CRC_WRITE_BUFFER_COUNT: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcInstruction {
    SetFlag { flag: u8 },
    TestFlagAndClear { flag: u8 },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CrcArchitecturalError {
    OpcodeOutOfRange(u16),
    FlagOutOfRange(u8),
    CardWordOutOfRange(u32),
    ReadBufferEmpty,
    WriteModeInactive,
    WriteBufferFull,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CrcArchitecturalCore {
    flags: [bool; CRC_FLAG_COUNT],
    external_flags: [bool; CRC_FLAG_COUNT],
    read_buffer: Option<u32>,
    write_buffers: [Option<u32>; CRC_WRITE_BUFFER_COUNT],
    write_head: usize,
    write_len: usize,
}

impl Default for CrcArchitecturalCore {
    fn default() -> Self {
        Self {
            flags: [false; CRC_FLAG_COUNT],
            external_flags: [false; CRC_FLAG_COUNT],
            read_buffer: None,
            write_buffers: [None; CRC_WRITE_BUFFER_COUNT],
            write_head: 0,
            write_len: 0,
        }
    }
}

impl CrcArchitecturalCore {
    pub fn reset_control_flags(&mut self) {
        self.flags = [false; CRC_FLAG_COUNT];
        self.read_buffer = None;
        self.write_buffers = [None; CRC_WRITE_BUFFER_COUNT];
        self.write_head = 0;
        self.write_len = 0;
    }

    pub fn flag(&self, flag: usize) -> Option<bool> {
        self.flags.get(flag).copied()
    }

    pub fn external_flag(&self, flag: usize) -> Option<bool> {
        self.external_flags.get(flag).copied()
    }

    pub const fn buffered_read_word(&self) -> Option<u32> {
        self.read_buffer
    }

    pub const fn queued_write_words(&self) -> usize {
        self.write_len
    }

    pub const fn write_buffer_can_accept(&self) -> bool {
        self.write_len < CRC_WRITE_BUFFER_COUNT
    }

    pub fn signal_write_capacity(&mut self, status_error: bool) {
        self.flags[CRC_FLAG_BUFFER_READY] = true;
        self.flags[CRC_FLAG_F7_STATUS] = status_error;
    }

    pub fn queue_write_word(&mut self, word: u32) -> Result<(), CrcArchitecturalError> {
        if word > CRC_CARD_WORD_MASK {
            return Err(CrcArchitecturalError::CardWordOutOfRange(word));
        }
        if !self.flags[CRC_FLAG_WRITE_MODE] {
            return Err(CrcArchitecturalError::WriteModeInactive);
        }
        if self.write_len >= CRC_WRITE_BUFFER_COUNT {
            self.flags[CRC_FLAG_F7_STATUS] = true;
            return Err(CrcArchitecturalError::WriteBufferFull);
        }

        let tail = (self.write_head + self.write_len) % CRC_WRITE_BUFFER_COUNT;
        self.write_buffers[tail] = Some(word);
        self.write_len += 1;
        self.flags[CRC_FLAG_BUFFER_READY] = false;
        self.flags[CRC_FLAG_F7_STATUS] = false;
        Ok(())
    }

    pub fn take_queued_write_word(&mut self) -> Option<u32> {
        if self.write_len == 0 {
            return None;
        }
        let word = self.write_buffers[self.write_head].take();
        self.write_head = (self.write_head + 1) % CRC_WRITE_BUFFER_COUNT;
        self.write_len -= 1;
        word
    }

    /// Present one complete 28-bit word from the magnetic transport / sense path.
    /// BUFFER_READY is a latch: firmware testing clears the flag, while the word
    /// remains buffered until the CRC data port consumes it.
    pub fn present_read_word(&mut self, word: u32) -> Result<(), CrcArchitecturalError> {
        if word > CRC_CARD_WORD_MASK {
            return Err(CrcArchitecturalError::CardWordOutOfRange(word));
        }
        // The CRC owns a single 28-bit read buffer. If firmware has not consumed
        // the previous record before the next transport record arrives, the new
        // record replaces it; exact overrun/error signalling is not yet modeled.
        self.read_buffer = Some(word);
        self.flags[CRC_FLAG_BUFFER_READY] = true;
        self.flags[CRC_FLAG_F7_STATUS] = false;
        Ok(())
    }

    pub fn take_read_word(&mut self) -> Result<u32, CrcArchitecturalError> {
        let word = self
            .read_buffer
            .take()
            .ok_or(CrcArchitecturalError::ReadBufferEmpty)?;
        self.flags[CRC_FLAG_BUFFER_READY] = false;
        Ok(word)
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
    fn transport_word_latches_buffer_ready_until_data_port_consumes_it() {
        let mut crc = CrcArchitecturalCore::default();
        crc.present_read_word(0x0765_4321)
            .expect("28-bit card word must fit");
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
        assert_eq!(crc.buffered_read_word(), Some(0x0765_4321));

        assert_eq!(crc.execute_opcode(0o100), Ok(Some(true)));
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(false));
        assert_eq!(crc.buffered_read_word(), Some(0x0765_4321));

        assert_eq!(crc.take_read_word(), Ok(0x0765_4321));
        assert_eq!(crc.buffered_read_word(), None);
    }

    #[test]
    fn transport_rejects_overwide_words_and_replaces_unread_record() {
        let mut crc = CrcArchitecturalCore::default();
        assert_eq!(
            crc.present_read_word(CRC_CARD_WORD_MASK + 1),
            Err(CrcArchitecturalError::CardWordOutOfRange(
                CRC_CARD_WORD_MASK + 1
            ))
        );
        crc.present_read_word(0x0123_4567)
            .expect("first word must latch");
        crc.present_read_word(0x0000_0001)
            .expect("next physical record replaces the one-word buffer");
        assert_eq!(crc.buffered_read_word(), Some(0x0000_0001));
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
    }

    #[test]
    fn write_mode_uses_two_crc_buffers_and_preserves_fifo_order() {
        let mut crc = CrcArchitecturalCore::default();
        crc.execute_opcode(0o660).expect("write mode must set");

        crc.queue_write_word(0x0111_1111)
            .expect("first buffer must accept");
        crc.queue_write_word(0x0222_2222)
            .expect("second buffer must accept");
        assert_eq!(crc.queued_write_words(), 2);
        assert!(!crc.write_buffer_can_accept());

        assert_eq!(
            crc.queue_write_word(0x0333_3333),
            Err(CrcArchitecturalError::WriteBufferFull)
        );
        assert_eq!(crc.flag(CRC_FLAG_F7_STATUS), Some(true));

        assert_eq!(crc.take_queued_write_word(), Some(0x0111_1111));
        assert_eq!(crc.take_queued_write_word(), Some(0x0222_2222));
        assert_eq!(crc.take_queued_write_word(), None);
    }

    #[test]
    fn write_capacity_event_drives_buffer_ready_and_f7_status_separately() {
        let mut crc = CrcArchitecturalCore::default();
        crc.signal_write_capacity(false);
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
        assert_eq!(crc.flag(CRC_FLAG_F7_STATUS), Some(false));

        crc.signal_write_capacity(true);
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
        assert_eq!(crc.flag(CRC_FLAG_F7_STATUS), Some(true));
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
