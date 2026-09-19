//! Nominal HP-67 magnetic-card transport timing at the CRC record boundary.
//!
//! The transport carries one complete physical card but exposes only the track
//! selected by the insertion end to the magnetic head.  The HP Journal documents
//! a nominal 6 cm/s card speed, approximately 1 kbit/s magnetic bit rate, and one
//! CRC-visible 28-bit record every 28 ms on average.  Exact motor acceleration,
//! switch geometry, flux-transition phase and sense-amplifier electrical timing
//! remain later M13 work.

use super::{
    crc::{CrcArchitecturalCore, CrcArchitecturalError, CRC_FLAG_WRITE_MODE},
    magnetic_card::{
        CardInsertionEnd, Hp67MagneticCard, Hp67MagneticTrack, HP67_CARD_RECORDS_PER_TRACK,
    },
};

pub const HP67_NOMINAL_CARD_RECORD_US: u64 = 28_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CardTransportError {
    CardAlreadyLoaded,
    Crc(CrcArchitecturalError),
}

impl From<CrcArchitecturalError> for Hp67CardTransportError {
    fn from(value: CrcArchitecturalError) -> Self {
        Self::Crc(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hp67CardTransport {
    card: Option<Hp67MagneticCard>,
    insertion_end: Option<CardInsertionEnd>,
    next_record: usize,
    record_elapsed_us: u64,
    head_active: bool,
    startup_ready_pending: bool,
    waiting_startup_ack: bool,
}

impl Hp67CardTransport {
    pub fn insert_card(
        &mut self,
        card: Hp67MagneticCard,
        insertion_end: CardInsertionEnd,
    ) -> Result<(), Hp67CardTransportError> {
        if self.card.is_some() {
            return Err(Hp67CardTransportError::CardAlreadyLoaded);
        }
        self.card = Some(card);
        self.insertion_end = Some(insertion_end);
        self.next_record = 0;
        self.record_elapsed_us = 0;
        self.startup_ready_pending = false;
        self.waiting_startup_ack = false;
        Ok(())
    }

    pub fn set_head_active(&mut self, active: bool) {
        if self.head_active != active {
            self.head_active = active;
            self.record_elapsed_us = 0;
            self.startup_ready_pending = active;
            self.waiting_startup_ack = false;
        }
    }

    pub const fn head_active(&self) -> bool {
        self.head_active
    }

    pub const fn record_stream_active(&self) -> bool {
        self.head_active && !self.startup_ready_pending && !self.waiting_startup_ack
    }

    pub fn card(&self) -> Option<&Hp67MagneticCard> {
        self.card.as_ref()
    }

    pub const fn insertion_end(&self) -> Option<CardInsertionEnd> {
        self.insertion_end
    }

    pub fn active_track(&self) -> Option<&Hp67MagneticTrack> {
        let card = self.card.as_ref()?;
        let insertion_end = self.insertion_end?;
        Some(card.track(insertion_end.track()))
    }

    fn active_track_mut(&mut self) -> Option<&mut Hp67MagneticTrack> {
        let insertion_end = self.insertion_end?;
        self.card
            .as_mut()
            .map(|card| card.track_mut(insertion_end.track()))
    }

    pub const fn next_record(&self) -> usize {
        self.next_record
    }

    pub const fn is_complete(&self) -> bool {
        self.card.is_some() && self.next_record >= HP67_CARD_RECORDS_PER_TRACK
    }

    /// Remove the same physical card only after the selected track has completely
    /// crossed the head.  The caller may then rotate it 180 degrees in its plane
    /// and reinsert the opposite end to expose the other longitudinal track.
    pub fn take_completed_card(&mut self) -> Option<Hp67MagneticCard> {
        if !self.is_complete() {
            return None;
        }

        self.next_record = 0;
        self.record_elapsed_us = 0;
        self.head_active = false;
        self.startup_ready_pending = false;
        self.waiting_startup_ack = false;
        self.insertion_end = None;
        self.card.take()
    }

    pub fn advance_us(
        &mut self,
        elapsed_us: u64,
        motor_on: bool,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<usize, Hp67CardTransportError> {
        if !motor_on || !self.head_active || self.card.is_none() || self.is_complete() {
            return Ok(0);
        }

        if self.startup_ready_pending {
            crc.signal_transport_ready(false);
            self.startup_ready_pending = false;
            self.waiting_startup_ack = true;
            return Ok(0);
        }

        if self.waiting_startup_ack {
            if crc.flag(super::crc::CRC_FLAG_BUFFER_READY) == Some(false) {
                self.waiting_startup_ack = false;
                self.record_elapsed_us = 0;
            }
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
            && self.next_record < HP67_CARD_RECORDS_PER_TRACK
        {
            self.record_elapsed_us -= HP67_NOMINAL_CARD_RECORD_US;
            let word = self
                .active_track()
                .and_then(|track| track.word(self.next_record));

            if let Some(word) = word {
                crc.present_read_word(word)?;
                produced += 1;
            }

            self.next_record += 1;
        }

        Ok(produced)
    }

    fn advance_write_us(
        &mut self,
        elapsed_us: u64,
        crc: &mut CrcArchitecturalCore,
    ) -> Result<usize, Hp67CardTransportError> {
        let write_protected = self
            .active_track()
            .expect("card and insertion end presence checked by advance_us")
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
            && self.next_record < HP67_CARD_RECORDS_PER_TRACK
        {
            let Some(word) = crc.take_queued_write_word() else {
                break;
            };
            self.record_elapsed_us -= HP67_NOMINAL_CARD_RECORD_US;
            let record_index = self.next_record;
            self.active_track_mut()
                .expect("card and insertion end presence checked by advance_us")
                .write_word(record_index, word);
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
    use crate::machines::hp67::{
        crc::CRC_FLAG_BUFFER_READY, Hp67CardTrack, Hp67MagneticTrack, TrackMedia,
        CRC_CARD_WORD_MASK,
    };

    fn card_with_track(
        track_id: Hp67CardTrack,
        track: Hp67MagneticTrack,
    ) -> Hp67MagneticCard {
        Hp67MagneticCard::default().with_track(track_id, track)
    }

    fn complete_startup_handshake(
        transport: &mut Hp67CardTransport,
        crc: &mut CrcArchitecturalCore,
    ) {
        assert_eq!(transport.advance_us(0, true, crc).unwrap(), 0);
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
        assert_eq!(crc.execute_opcode(0o100), Ok(Some(true)));
        assert_eq!(transport.advance_us(0, true, crc).unwrap(), 0);
        assert_eq!(crc.flag(CRC_FLAG_BUFFER_READY), Some(false));
        assert!(transport.record_stream_active());
    }

    #[test]
    fn record_cadence_requires_motor_and_head_switch() {
        let mut transport = Hp67CardTransport::default();
        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        words[0] = 0x0123_4567;
        transport
            .insert_card(
                card_with_track(
                    Hp67CardTrack::Track1,
                    Hp67MagneticTrack::from_words(words).unwrap(),
                ),
                CardInsertionEnd::End1,
            )
            .unwrap();
        let mut crc = CrcArchitecturalCore::default();

        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US, false, &mut crc)
                .unwrap(),
            0
        );
        transport.set_head_active(true);
        complete_startup_handshake(&mut transport, &mut crc);
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
    fn opposite_insertion_end_reads_the_other_track_of_the_same_card() {
        let mut track_1_words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        let mut track_2_words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        track_1_words[0] = 0x0111_1111;
        track_2_words[0] = 0x0222_2222;
        let card = Hp67MagneticCard::new(
            Hp67MagneticTrack::from_words(track_1_words).unwrap(),
            Hp67MagneticTrack::from_words(track_2_words).unwrap(),
        );

        let mut transport = Hp67CardTransport::default();
        transport
            .insert_card(card, CardInsertionEnd::End2)
            .unwrap();
        transport.set_head_active(true);
        let mut crc = CrcArchitecturalCore::default();
        complete_startup_handshake(&mut transport, &mut crc);

        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US, true, &mut crc)
                .unwrap(),
            1
        );
        assert_eq!(crc.take_read_word().unwrap(), 0x0222_2222);
        assert_eq!(transport.insertion_end(), Some(CardInsertionEnd::End2));
    }

    #[test]
    fn unrecorded_track_crosses_the_head_without_inventing_zero_records() {
        let mut transport = Hp67CardTransport::default();
        transport
            .insert_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .unwrap();
        transport.set_head_active(true);
        let mut crc = CrcArchitecturalCore::default();
        complete_startup_handshake(&mut transport, &mut crc);

        assert_eq!(
            transport
                .advance_us(
                    HP67_NOMINAL_CARD_RECORD_US * HP67_CARD_RECORDS_PER_TRACK as u64,
                    true,
                    &mut crc,
                )
                .unwrap(),
            0
        );
        assert!(transport.is_complete());
        assert_eq!(crc.queued_read_words(), 0);
    }

    #[test]
    fn consecutive_record_boundaries_fill_both_crc_read_buffers_in_fifo_order() {
        let mut transport = Hp67CardTransport::default();
        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        words[0] = 0x0111_1111;
        words[1] = 0x0222_2222;
        transport
            .insert_card(
                card_with_track(
                    Hp67CardTrack::Track1,
                    Hp67MagneticTrack::from_words(words).unwrap(),
                ),
                CardInsertionEnd::End1,
            )
            .unwrap();
        transport.set_head_active(true);
        let mut crc = CrcArchitecturalCore::default();
        complete_startup_handshake(&mut transport, &mut crc);

        assert_eq!(
            transport
                .advance_us(HP67_NOMINAL_CARD_RECORD_US * 2, true, &mut crc)
                .unwrap(),
            2
        );
        assert_eq!(transport.next_record(), 2);
        assert_eq!(crc.queued_read_words(), 2);
        assert_eq!(crc.buffered_read_word(), Some(0x0111_1111));
        assert_eq!(crc.take_read_word(), Ok(0x0111_1111));
        assert_eq!(crc.take_read_word(), Ok(0x0222_2222));
    }

    #[test]
    fn write_mode_records_an_initially_unrecorded_selected_track() {
        let mut transport = Hp67CardTransport::default();
        transport
            .insert_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .unwrap();
        transport.set_head_active(true);

        let mut crc = CrcArchitecturalCore::default();
        crc.execute_opcode(0o660).expect("write mode must set");
        complete_startup_handshake(&mut transport, &mut crc);
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
            transport.active_track().and_then(|track| track.word(0)),
            Some(0x0111_1111)
        );
        assert!(transport.active_track().unwrap().dirty());
        assert!(matches!(
            transport.active_track().unwrap().media(),
            TrackMedia::Recorded(_)
        ));
    }

    #[test]
    fn write_protection_is_independent_per_track() {
        let card = Hp67MagneticCard::new(
            Hp67MagneticTrack::default().with_write_protected(true),
            Hp67MagneticTrack::default(),
        );

        let mut protected = Hp67CardTransport::default();
        protected
            .insert_card(card.clone(), CardInsertionEnd::End1)
            .unwrap();
        protected.set_head_active(true);
        let mut protected_crc = CrcArchitecturalCore::default();
        protected_crc.execute_opcode(0o660).unwrap();
        complete_startup_handshake(&mut protected, &mut protected_crc);
        assert_eq!(protected.advance_us(320, true, &mut protected_crc).unwrap(), 0);
        assert_eq!(
            protected_crc.flag(crate::machines::hp67::crc::CRC_FLAG_F7_STATUS),
            Some(true)
        );
        assert!(!protected.active_track().unwrap().dirty());

        let mut writable = Hp67CardTransport::default();
        writable
            .insert_card(card, CardInsertionEnd::End2)
            .unwrap();
        writable.set_head_active(true);
        let mut writable_crc = CrcArchitecturalCore::default();
        writable_crc.execute_opcode(0o660).unwrap();
        complete_startup_handshake(&mut writable, &mut writable_crc);
        writable_crc.queue_write_word(0x0123_4567).unwrap();
        assert_eq!(
            writable
                .advance_us(HP67_NOMINAL_CARD_RECORD_US, true, &mut writable_crc)
                .unwrap(),
            1
        );
        assert_eq!(writable.active_track().unwrap().word(0), Some(0x0123_4567));
    }

    #[test]
    fn full_track_write_eject_rotate_state_preserves_both_tracks() {
        let untouched_track_2 = Hp67MagneticTrack::from_words([0x0055_5555; HP67_CARD_RECORDS_PER_TRACK])
            .unwrap()
            .with_write_protected(true);
        let card = Hp67MagneticCard::new(Hp67MagneticTrack::default(), untouched_track_2.clone());

        let mut expected = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        for (index, word) in expected.iter_mut().enumerate() {
            *word = ((index as u32 + 1) * 0x0001_2345) & CRC_CARD_WORD_MASK;
        }

        let mut writer = Hp67CardTransport::default();
        writer
            .insert_card(card, CardInsertionEnd::End1)
            .expect("card must insert for writing");
        writer.set_head_active(true);
        let mut write_crc = CrcArchitecturalCore::default();
        write_crc.execute_opcode(0o660).expect("write mode must set");
        complete_startup_handshake(&mut writer, &mut write_crc);

        for expected_word in expected {
            write_crc.queue_write_word(expected_word).unwrap();
            assert_eq!(
                writer
                    .advance_us(HP67_NOMINAL_CARD_RECORD_US, true, &mut write_crc)
                    .unwrap(),
                1
            );
        }

        assert!(writer.is_complete());
        let completed = writer.take_completed_card().unwrap();
        assert_eq!(
            completed.track(Hp67CardTrack::Track1).words(),
            Some(&expected)
        );
        assert_eq!(completed.track(Hp67CardTrack::Track2), &untouched_track_2);

        let mut reader = Hp67CardTransport::default();
        reader
            .insert_card(completed, CardInsertionEnd::End1)
            .expect("same physical card must reinsert");
        reader.set_head_active(true);
        let mut read_crc = CrcArchitecturalCore::default();
        complete_startup_handshake(&mut reader, &mut read_crc);

        for expected_word in expected {
            assert_eq!(
                reader
                    .advance_us(HP67_NOMINAL_CARD_RECORD_US, true, &mut read_crc)
                    .unwrap(),
                1
            );
            assert_eq!(read_crc.take_read_word().unwrap(), expected_word);
        }

        assert!(reader.is_complete());
        assert!(reader.take_completed_card().is_some());
    }
}
