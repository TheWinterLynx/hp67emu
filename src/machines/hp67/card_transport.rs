//! Nominal HP-67 magnetic-card transport timing at the CRC record boundary.
//!
//! The HP Journal (November 1976) documents a nominal 6 cm/s card speed,
//! approximately 1 kbit/s magnetic bit rate, and one CRC-visible 28-bit record
//! every 28 ms on average.  This module models only that record cadence and the
//! head-active gate.  Exact motor acceleration, switch geometry, flux-transition
//! phase and sense-amplifier electrical timing remain later M13 work.

use super::crc::{
    CrcArchitecturalCore, CrcArchitecturalError, CRC_CARD_WORD_MASK, CRC_FLAG_WRITE_MODE,
};

pub const HP67_CARD_RECORDS_PER_SIDE: usize = 34;
pub const HP67_NOMINAL_CARD_RECORD_US: u64 = 28_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CardTransportError {
    CardAlreadyLoaded,
    WordOutOfRange { index: usize, word: u32 },
    Crc(CrcArchitecturalError),
}

impl From<CrcArchitecturalError> for Hp67CardTransportError {
    fn from(value: CrcArchitecturalError) -> Self {
        Self::Crc(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67CardSide {
    words: [u32; HP67_CARD_RECORDS_PER_SIDE],
    write_protected: bool,
    dirty: bool,
}

impl Default for Hp67CardSide {
    fn default() -> Self {
        Self {
            words: [0; HP67_CARD_RECORDS_PER_SIDE],
            write_protected: false,
            dirty: false,
        }
    }
}

impl Hp67CardSide {
    pub fn from_words(
        words: [u32; HP67_CARD_RECORDS_PER_SIDE],
    ) -> Result<Self, Hp67CardTransportError> {
        for (index, word) in words.iter().copied().enumerate() {
            if word > CRC_CARD_WORD_MASK {
                return Err(Hp67CardTransportError::WordOutOfRange { index, word });
            }
        }
        Ok(Self {
            words,
            write_protected: false,
            dirty: false,
        })
    }

    pub const fn words(&self) -> &[u32; HP67_CARD_RECORDS_PER_SIDE] {
        &self.words
    }

    pub const fn write_protected(&self) -> bool {
        self.write_protected
    }

    pub const fn dirty(&self) -> bool {
        self.dirty
    }

    pub fn with_write_protected(mut self, protected: bool) -> Self {
        self.write_protected = protected;
        self
    }

    fn write_word(&mut self, index: usize, word: u32) {
        debug_assert!(index < HP67_CARD_RECORDS_PER_SIDE);
        debug_assert!(word <= CRC_CARD_WORD_MASK);
        self.words[index] = word;
        self.dirty = true;
    }

    pub const fn word(&self, index: usize) -> Option<u32> {
        if index < HP67_CARD_RECORDS_PER_SIDE {
            Some(self.words[index])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hp67CardTransport {
    side: Option<Hp67CardSide>,
    next_record: usize,
    record_elapsed_us: u64,
    head_active: bool,
}

impl Hp67CardTransport {
    pub fn insert_side(&mut self, side: Hp67CardSide) -> Result<(), Hp67CardTransportError> {
        if self.side.is_some() {
            return Err(Hp67CardTransportError::CardAlreadyLoaded);
        }
        self.side = Some(side);
        self.next_record = 0;
        self.record_elapsed_us = 0;
        Ok(())
    }

    pub fn set_head_active(&mut self, active: bool) {
        if self.head_active != active {
            self.head_active = active;
            self.record_elapsed_us = 0;
        }
    }

    pub const fn head_active(&self) -> bool {
        self.head_active
    }

    pub const fn side(&self) -> Option<&Hp67CardSide> {
        self.side.as_ref()
    }

    pub const fn next_record(&self) -> usize {
        self.next_record
    }

    pub const fn is_complete(&self) -> bool {
        self.side.is_some() && self.next_record >= HP67_CARD_RECORDS_PER_SIDE
    }

    pub fn advance_us(
        &mut self,
        elapsed_us: u64,
        motor_on: bool,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<usize, Hp67CardTransportError> {
        if !motor_on || !self.head_active || self.side.is_none() || self.is_complete() {
            return Ok(0);
        }

        if crc.flag(CRC_FLAG_WRITE_MODE) == Some(true) {
            return self.advance_write_us(elapsed_us, crc);
        }

        self.advance_read_us(elapsed_us, crc)
    }

    fn advance_read_us(
        &mut self,
        elapsed_us: u64,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<usize, Hp67CardTransportError> {
        self.record_elapsed_us = self.record_elapsed_us.saturating_add(elapsed_us);
        let mut produced = 0;

        while self.record_elapsed_us >= HP67_NOMINAL_CARD_RECORD_US
            && self.next_record < HP67_CARD_RECORDS_PER_SIDE
        {
            self.record_elapsed_us -= HP67_NOMINAL_CARD_RECORD_US;
            let word = self
                .side
                .as_ref()
                .and_then(|side| side.word(self.next_record))
                .expect("record index is bounded by HP67_CARD_RECORDS_PER_SIDE");
            crc.present_read_word(word)?;
            self.next_record += 1;
            produced += 1;
        }

        Ok(produced)
    }

    fn advance_write_us(
        &mut self,
        elapsed_us: u64,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<usize, Hp67CardTransportError> {
        let write_protected = self
            .side
            .as_ref()
            .expect("side presence checked by advance_us")
            .write_protected();

        if write_protected {
            crc.signal_write_capacity(true);
            self.record_elapsed_us = 0;
            return Ok(0);
        }

        if crc.write_buffer_can_accept() {
            crc.signal_write_capacity(false);
        }

        if crc.queued_write_words() == 0 {
            self.record_elapsed_us = 0;
            return Ok(0);
        }

        self.record_elapsed_us = self.record_elapsed_us.saturating_add(elapsed_us);
        let mut written = 0;

        while self.record_elapsed_us >= HP67_NOMINAL_CARD_RECORD_US
            && self.next_record < HP67_CARD_RECORDS_PER_SIDE
        {
            let Some(word) = crc.take_queued_write_word() else {
                break;
            };
            self.record_elapsed_us -= HP67_NOMINAL_CARD_RECORD_US;
            self.side
                .as_mut()
                .expect("side presence checked by advance_us")
                .write_word(self.next_record, word);
            self.next_record += 1;
            written += 1;

            if crc.write_buffer_can_accept() {
                crc.signal_write_capacity(false);
            }
        }

        Ok(written)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::crc::CRC_FLAG_BUFFER_READY;

    #[test]
    fn record_cadence_requires_motor_and_head_switch() {
        let mut transport = Hp67CardTransport::default();
        let mut words = [0u32; HP67_CARD_RECORDS_PER_SIDE];
        words[0] = 0x0123_4567;
        transport
            .insert_side(Hp67CardSide::from_words(words).unwrap())
            .unwrap();
        let mut crc = CrcArchitecturalCore::default();

        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US, false, &mut crc)
                .unwrap(),
            0
        );
        transport.set_head_active(true);
        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US - 1, true, &mut crc)
                .unwrap(),
            0
        );
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(false));

        assert_eq!(transport.advance_us(1, true, &mut crc).unwrap(), 1);
        assert_eq!(transport.next_record(), 1);
        assert_eq!(crc.buffered_read_word(), Some(0x0123_4567));
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
    }

    #[test]
    fn consecutive_record_boundaries_replace_unread_crc_buffer() {
        let mut transport = Hp67CardTransport::default();
        let mut words = [0u32; HP67_CARD_RECORDS_PER_SIDE];
        words[0] = 0x0111_1111;
        words[1] = 0x0222_2222;
        transport
            .insert_side(Hp67CardSide::from_words(words).unwrap())
            .unwrap();
        transport.set_head_active(true);
        let mut crc = CrcArchitecturalCore::default();

        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US * 2, true, &mut crc)
                .unwrap(),
            2
        );
        assert_eq!(transport.next_record(), 2);
        assert_eq!(crc.buffered_read_word(), Some(0x0222_2222));
    }

    #[test]
    fn write_mode_drains_crc_buffers_at_record_cadence() {
        let mut transport = Hp67CardTransport::default();
        transport
            .insert_side(Hp67CardSide::default())
            .expect("blank side must insert");
        transport.set_head_active(true);

        let mut crc = CrcArchitecturalCore::default();
        crc.execute_opcode(0o660).expect("write mode must set");
        crc.queue_write_word(0x0111_1111).unwrap();
        crc.queue_write_word(0x0222_2222).unwrap();

        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US - 1, true, &mut crc)
                .unwrap(),
            0
        );
        assert_eq!(transport.next_record(), 0);

        assert_eq!(transport.advance_us(1, true, &mut crc).unwrap(), 1);
        assert_eq!(transport.next_record(), 1);
        assert_eq!(crc.queued_write_words(), 1);
        assert_eq!(
            transport
                .side
                .as_ref()
                .and_then(|side| side.word(0)),
            Some(0x0111_1111)
        );
        assert!(
            transport
                .side
                .as_ref()
                .expect("side remains inserted")
                .dirty()
        );
    }

    #[test]
    fn write_protected_side_sets_crc_f7_and_remains_unmodified() {
        let mut transport = Hp67CardTransport::default();
        transport
            .insert_side(Hp67CardSide::default().with_write_protected(true))
            .expect("protected side must insert");
        transport.set_head_active(true);

        let mut crc = CrcArchitecturalCore::default();
        crc.execute_opcode(0o660).expect("write mode must set");

        assert_eq!(transport.advance_us(320, true, &mut crc).unwrap(), 0);
        assert_eq!(crc.flag(crate::machines::hp67::crc::CRC_FLAG_BUFFER_READY), Some(true));
        assert_eq!(crc.flag(crate::machines::hp67::crc::CRC_FLAG_F7_STATUS), Some(true));
        assert_eq!(transport.next_record(), 0);
        assert!(
            !transport
                .side
                .as_ref()
                .expect("side remains inserted")
                .dirty()
        );
    }

    #[test]
    fn one_side_contains_exactly_thirty_four_records() {
        let mut transport = Hp67CardTransport::default();
        transport
            .insert_side(Hp67CardSide::default())
            .expect("empty fixture side must insert");
        transport.set_head_active(true);
        let mut crc = CrcArchitecturalCore::default();

        assert_eq!(
            transport
                .advance_us(
                    HP67_NOMINAL_CARD_RECORD_US * HP67_CARD_RECORDS_PER_SIDE as u64,
                    true,
                    &mut crc,
                )
                .unwrap(),
            HP67_CARD_RECORDS_PER_SIDE
        );
        assert!(transport.is_complete());
        assert_eq!(transport.next_record(), HP67_CARD_RECORDS_PER_SIDE);
    }
}
