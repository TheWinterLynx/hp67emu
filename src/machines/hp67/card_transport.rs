//! Nominal HP-67 magnetic-card transport timing at the CRC record boundary.
//!
//! The transport carries one complete physical card but exposes only the logical
//! card track selected by the insertion end.  Each logical track is physically
//! encoded by the two parallel self-clocking flux tracks modeled in `card_flux`.
//! The HP Journal documents a nominal 6 cm/s card speed, approximately 1 kbit/s
//! magnetic bit rate, one CRC-visible 28-bit record every 28 ms on average, and
//! reader-speed variation of +/-5% between calculators.  Exact motor acceleration,
//! switch geometry, record-to-bit serialization and sense-amplifier electrical
//! timing remain later M13 work.

use super::{
    crc::{CrcArchitecturalCore, CrcArchitecturalError, CRC_FLAG_WRITE_MODE},
    magnetic_card::{
        CardInsertionEnd, Hp67MagneticCard, Hp67MagneticTrack, HP67_CARD_RECORDS_PER_TRACK,
    },
};

pub const HP67_NOMINAL_CARD_RECORD_US: u64 = 28_000;
pub const HP67_MIN_CARD_SPEED_PERCENT: u8 = 95;
pub const HP67_MAX_CARD_SPEED_PERCENT: u8 = 105;
const HP67_CARD_RECORD_PHASE_UNITS: u64 = HP67_NOMINAL_CARD_RECORD_US * 100;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CardSpeedError {
    OutOfRange {
        percent: u8,
        min_percent: u8,
        max_percent: u8,
    },
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67CardSpeed {
    percent_of_nominal: u8,
}

impl Hp67CardSpeed {
    pub fn from_percent(percent: u8) -> Result<Self, Hp67CardSpeedError> {
        if !(HP67_MIN_CARD_SPEED_PERCENT..=HP67_MAX_CARD_SPEED_PERCENT).contains(&percent) {
            return Err(Hp67CardSpeedError::OutOfRange {
                percent,
                min_percent: HP67_MIN_CARD_SPEED_PERCENT,
                max_percent: HP67_MAX_CARD_SPEED_PERCENT,
            });
        }
        Ok(Self {
            percent_of_nominal: percent,
        })
    }

    pub const fn percent_of_nominal(self) -> u8 {
        self.percent_of_nominal
    }

    pub const fn record_interval_us(self) -> u64 {
        let numerator = HP67_NOMINAL_CARD_RECORD_US * 100;
        (numerator + self.percent_of_nominal as u64 / 2) / self.percent_of_nominal as u64
    }
}

impl Default for Hp67CardSpeed {
    fn default() -> Self {
        Self {
            percent_of_nominal: 100,
        }
    }
}

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
    record_phase_units: u64,
    speed: Hp67CardSpeed,
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
        self.record_phase_units = 0;
        self.startup_ready_pending = false;
        self.waiting_startup_ack = false;
        Ok(())
    }

    pub fn set_head_active(&mut self, active: bool) {
        if self.head_active != active {
            self.head_active = active;
            self.record_phase_units = 0;
            self.startup_ready_pending = active;
            self.waiting_startup_ack = false;
        }
    }

    pub const fn head_active(&self) -> bool {
        self.head_active
    }

    pub const fn speed(&self) -> Hp67CardSpeed {
        self.speed
    }

    pub fn set_speed_percent(&mut self, percent: u8) -> Result<(), Hp67CardSpeedError> {
        self.speed = Hp67CardSpeed::from_percent(percent)?;
        self.record_phase_units = 0;
        Ok(())
    }

    pub const fn record_interval_us(&self) -> u64 {
        self.speed.record_interval_us()
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

    /// Remove the same physical card only after the selected logical track has
    /// completely crossed the head. The caller may then rotate it 180 degrees in
    /// its plane and reinsert the opposite end to expose the other logical track.
    pub fn take_completed_card(&mut self) -> Option<Hp67MagneticCard> {
        if !self.is_complete() {
            return None;
        }

        self.next_record = 0;
        self.record_phase_units = 0;
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
                self.record_phase_units = 0;
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
        self.record_phase_units = self.record_phase_units.saturating_add(
            elapsed_us.saturating_mul(u64::from(self.speed.percent_of_nominal())),
        );
        let mut produced = 0;

        while self.record_phase_units >= HP67_CARD_RECORD_PHASE_UNITS
            && self.next_record < HP67_CARD_RECORDS_PER_TRACK
        {
            self.record_phase_units -= HP67_CARD_RECORD_PHASE_UNITS;
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
            self.record_phase_units = 0;
            return Ok(0);
        }

        if crc.write_buffer_can_accept() {
            crc.signal_write_capacity(false);
        }

        if crc.queued_write_words() == 0 {
            self.record_phase_units = 0;
            return Ok(0);
        }

        self.record_phase_units = self.record_phase_units.saturating_add(
            elapsed_us.saturating_mul(u64::from(self.speed.percent_of_nominal())),
        );
        let mut written = 0;

        while self.record_phase_units >= HP67_CARD_RECORD_PHASE_UNITS
            && self.next_record < HP67_CARD_RECORDS_PER_TRACK
        {
            let Some(word) = crc.take_queued_write_word() else {
                break;
            };
            self.record_phase_units -= HP67_CARD_RECORD_PHASE_UNITS;
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

    fn card_with_track(track_id: Hp67CardTrack, track: Hp67MagneticTrack) -> Hp67MagneticCard {
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
    fn documented_reader_speed_tolerance_changes_record_cadence() {
        let fast = Hp67CardSpeed::from_percent(105).unwrap();
        let nominal = Hp67CardSpeed::default();
        let slow = Hp67CardSpeed::from_percent(95).unwrap();

        assert_eq!(fast.record_interval_us(), 26_667);
        assert_eq!(nominal.record_interval_us(), HP67_NOMINAL_CARD_RECORD_US);
        assert_eq!(slow.record_interval_us(), 29_474);
        assert!(fast.record_interval_us() < nominal.record_interval_us());
        assert!(slow.record_interval_us() > nominal.record_interval_us());
    }

    #[test]
    fn reader_speed_is_limited_to_the_documented_plus_or_minus_five_percent() {
        assert_eq!(
            Hp67CardSpeed::from_percent(94),
            Err(Hp67CardSpeedError::OutOfRange {
                percent: 94,
                min_percent: 95,
                max_percent: 105,
            })
        );
        assert_eq!(
            Hp67CardSpeed::from_percent(106),
            Err(Hp67CardSpeedError::OutOfRange {
                percent: 106,
                min_percent: 95,
                max_percent: 105,
            })
        );
    }

    #[test]
    fn configured_speed_controls_when_the_crc_receives_a_record() {
        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        words[0] = 0x0123_4567;
        let mut transport = Hp67CardTransport::default();
        transport.set_speed_percent(105).unwrap();
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

        assert_eq!(transport.record_interval_us(), 26_667);
        assert_eq!(transport.advance_us(26_666, true, &mut crc).unwrap(), 0);
        assert_eq!(transport.advance_us(1, true, &mut crc).unwrap(), 1);
        assert_eq!(crc.take_read_word().unwrap(), 0x0123_4567);
    }

    #[test]
    fn non_nominal_speed_preserves_fractional_phase_between_records() {
        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        words[0] = 0x0111_1111;
        words[1] = 0x0222_2222;
        let mut transport = Hp67CardTransport::default();
        transport.set_speed_percent(105).unwrap();
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

        assert_eq!(transport.advance_us(26_667, true, &mut crc).unwrap(), 1);
        assert_eq!(crc.take_read_word().unwrap(), 0x0111_1111);

        assert_eq!(transport.advance_us(26_666, true, &mut crc).unwrap(), 0);
        assert_eq!(transport.advance_us(1, true, &mut crc).unwrap(), 1);
        assert_eq!(crc.take_read_word().unwrap(), 0x0222_2222);
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
        transport.insert_card(card, CardInsertionEnd::End2).unwrap();
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
        assert_eq!(
            protected.advance_us(320, true, &mut protected_crc).unwrap(),
            0
        );
        assert_eq!(
            protected_crc.flag(crate::machines::hp67::crc::CRC_FLAG_F7_STATUS),
            Some(true)
        );
        assert!(!protected.active_track().unwrap().dirty());

        let mut writable = Hp67CardTransport::default();
        writable.insert_card(card, CardInsertionEnd::End2).unwrap();
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
        let untouched_track_2 =
            Hp67MagneticTrack::from_words([0x0055_5555; HP67_CARD_RECORDS_PER_TRACK])
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
        write_crc
            .execute_opcode(0o660)
            .expect("write mode must set");
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
