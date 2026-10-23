//! Retained logical CRC data buffers. No magnetic bit or PHI timing is inferred.
use super::crc::{CrcArchitecturalCore, CrcArchitecturalError, CrcDataState, CRC_CARD_WORD_MASK, CRC_READ_BUFFER_COUNT, CRC_WRITE_BUFFER_COUNT};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67CrcOwnedData {
    state: CrcDataState,
}
impl Default for Hp67CrcOwnedData {
    fn default() -> Self {
        Self { state: CrcDataState { read_buffers: [None; CRC_READ_BUFFER_COUNT], read_head: 0, read_len: 0, write_buffers: [None; CRC_WRITE_BUFFER_COUNT], write_head: 0, write_len: 0, ready: false, status_error: false } }
    }
}
impl Hp67CrcOwnedData {
    pub(crate) const fn state(&self) -> CrcDataState { self.state }
    pub const fn buffer_status(&self) -> (bool, bool) { (self.state.ready, self.state.status_error) }
    /// Internal CRC control, rather than the host mirror, owns flag set/clear.
    pub fn sync_control(&mut self, ready: bool, error: bool) { self.state.ready = ready; self.state.status_error = error; }
    pub fn check_mirror(&self, mirror: &CrcArchitecturalCore) -> Result<(), String> {
        if self.state != mirror.data_state() { return Err(format!("CRC owned DATA/mirror mismatch: device={:?} mirror={:?}", self.state, mirror.data_state())); }
        Ok(())
    }
    /// The observer must agree before the owned buffers replace the mirror.
    pub fn commit_mirror(&self, observer: &CrcArchitecturalCore, mirror: &mut CrcArchitecturalCore) -> Result<(), String> {
        self.check_mirror(observer)?; mirror.apply_data_state(self.state); Ok(())
    }
    pub(crate) fn commit_staged(&mut self, previous: CrcDataState, result: CrcDataState) -> Result<(), String> {
        if self.state != previous { return Err("CRC owned DATA pre-state changed during structural execution".into()); }
        self.state = result; Ok(())
    }
    pub(crate) fn stage_data_word(&self, write: Option<u32>, write_mode: bool) -> Result<(u32, CrcDataState), CrcArchitecturalError> {
        let mut next = *self;
        let word = if let Some(word) = write {
            if word > CRC_CARD_WORD_MASK { return Err(CrcArchitecturalError::CardWordOutOfRange(word)); }
            if !write_mode { return Err(CrcArchitecturalError::WriteModeInactive); }
            if next.state.write_len == CRC_WRITE_BUFFER_COUNT { return Err(CrcArchitecturalError::WriteBufferFull); }
            let tail = (next.state.write_head + next.state.write_len) % CRC_WRITE_BUFFER_COUNT;
            next.state.write_buffers[tail] = Some(word); next.state.write_len += 1;
            next.sync_control(false, false); word
        } else {
            if next.state.read_len == 0 { return Err(CrcArchitecturalError::ReadBufferEmpty); }
            let word = next.state.read_buffers[next.state.read_head].take().expect("owned CRC read head");
            next.state.read_head = (next.state.read_head + 1) % CRC_READ_BUFFER_COUNT;
            next.state.read_len -= 1; next.state.ready = next.state.read_len != 0; word
        };
        Ok((word, next.state))
    }
    pub(crate) fn present_read_word(&mut self, word: u32) -> Result<(), CrcArchitecturalError> {
        if word > CRC_CARD_WORD_MASK { return Err(CrcArchitecturalError::CardWordOutOfRange(word)); }
        if self.state.read_len == CRC_READ_BUFFER_COUNT { self.state.status_error = true; return Err(CrcArchitecturalError::ReadBufferFull); }
        let tail = (self.state.read_head + self.state.read_len) % CRC_READ_BUFFER_COUNT;
        self.state.read_buffers[tail] = Some(word); self.state.read_len += 1; self.sync_control(true, false); Ok(())
    }
    pub(crate) const fn write_buffer_can_accept(&self) -> bool { self.state.write_len < CRC_WRITE_BUFFER_COUNT }
    pub(crate) const fn queued_write_words(&self) -> usize { self.state.write_len }
    pub(crate) fn take_queued_write_word(&mut self) -> Option<u32> {
        if self.state.write_len == 0 { return None; }
        let word = self.state.write_buffers[self.state.write_head].take();
        self.state.write_head = (self.state.write_head + 1) % CRC_WRITE_BUFFER_COUNT; self.state.write_len -= 1; word
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActArchitecturalState, ActInstructionState, ActSerialStateSnapshot, ActSerialExecution, ActSerialCrcDataResultImage, CRC_FLAG_WRITE_MODE};

    fn seeded(write_mode: bool) -> (Hp67CrcOwnedData, CrcArchitecturalCore) {
        let mut data = Hp67CrcOwnedData::default(); let mut oracle = CrcArchitecturalCore::default();
        oracle.commit_control_flag(CRC_FLAG_WRITE_MODE as u8, write_mode).unwrap();
        for word in [0x7654321, 0xabcdef0] { data.present_read_word(word).unwrap(); oracle.present_read_word(word).unwrap(); }
        (data, oracle)
    }
    #[test]
    fn m15x_all_crc_port_words_qualifiers_and_addresses_use_owned_storage() {
        let mut accepted = 0;
        for address in [0, 0x40, 0x90, 0x99, 0x9b, 0xff] {
            for qualifier in [ActInstructionState::Normal, ActInstructionState::ThenGoto] {
                for write_mode in [false, true] {
                    for word in 0..=0x3ff {
                        let (mut data, mut oracle) = seeded(write_mode);
                        let mut state = ActArchitecturalState { ram_address: address, instruction_state: qualifier, c: std::array::from_fn(|digit| (digit & 15) as u8), ..Default::default() };
                        let snapshot = ActSerialStateSnapshot::capture(&state);
                        let execution = ActSerialExecution::new(word, qualifier).unwrap();
                        let expected = ActSerialCrcDataResultImage::begin(&snapshot, &execution, &oracle);
                        let actual = ActSerialCrcDataResultImage::begin_owned(&snapshot, &execution, &data, write_mode);
                        assert_eq!(actual, expected, "word={word:03x} address={address:02x} qualifier={qualifier:?}");
                        let Ok(Some(mut image)) = actual else { continue; };
                        let before = data;
                        assert!(image.commit_owned(&mut data).is_err()); assert_eq!(data, before);
                        image.complete_word(); image.commit_owned(&mut data).unwrap(); image.commit(&mut state, &mut oracle).unwrap();
                        data.check_mirror(&oracle).unwrap(); assert_eq!(image.result_c(), state.c); accepted += 1;
                    }
                }
            }
        }
        assert!(accepted > 0);
    }
    #[test]
    fn m15x_fifo_wrap_errors_and_stale_commit_preserve_device_storage() {
        let (mut data, mut oracle) = seeded(true);
        for cycle in 0..100 {
            let (actual, next) = data.stage_data_word(None, true).unwrap();
            let expected = oracle.take_read_word().unwrap(); assert_eq!(actual, expected);
            let previous = data.state(); data.commit_staged(previous, next).unwrap();
            let value = 0x1000000 + cycle;
            data.present_read_word(value).unwrap(); oracle.present_read_word(value).unwrap();
            let (word, next) = data.stage_data_word(Some(value), true).unwrap(); assert_eq!(word, value);
            let previous = data.state(); data.commit_staged(previous, next).unwrap(); oracle.queue_write_word(value).unwrap();
            assert_eq!(data.take_queued_write_word(), oracle.take_queued_write_word());
            data.check_mirror(&oracle).unwrap();
        }
        let before = data;
        assert_eq!(data.stage_data_word(Some(1), false), Err(CrcArchitecturalError::WriteModeInactive));
        assert_eq!(data.stage_data_word(Some(0x10000000), true), Err(CrcArchitecturalError::CardWordOutOfRange(0x10000000)));
        assert_eq!(data, before);
        let (_, next) = data.stage_data_word(None, true).unwrap();
        data.sync_control(!data.buffer_status().0, true); let changed = data;
        assert!(data.commit_staged(before.state(), next).is_err()); assert_eq!(data, changed);
        let mut empty = Hp67CrcOwnedData::default();
        assert_eq!(empty.stage_data_word(None, true), Err(CrcArchitecturalError::ReadBufferEmpty));
        for value in [1, 2] { let previous = empty.state(); let (_, next) = empty.stage_data_word(Some(value), true).unwrap(); empty.commit_staged(previous, next).unwrap(); }
        let full = empty;
        assert_eq!(empty.stage_data_word(Some(3), true), Err(CrcArchitecturalError::WriteBufferFull)); assert_eq!(empty, full);
    }
    #[test]
    fn m15x_poisoned_crc_mirror_cannot_supply_port_payload_or_publish() {
        let (data, mut mirror) = seeded(true);
        mirror.take_read_word().unwrap(); mirror.present_read_word(0x1111111).unwrap();
        assert!(data.check_mirror(&mirror).is_err());
        assert_eq!(data.stage_data_word(None, true).unwrap().0, 0x7654321);
        let mut destination = CrcArchitecturalCore::default(); let before = destination.clone();
        assert!(data.commit_mirror(&mirror, &mut destination).is_err()); assert_eq!(destination, before);
    }
}

#[cfg(test)]
mod transport_tests {
    use super::*;
    use crate::{emulation::Drive, machines::hp67::{Hp67SerialCrcControl, Hp67ElectricalBackplane, Hp67CardTransport, Hp67MagneticCard, Hp67MagneticTrack, Hp67CardTrack, CardInsertionEnd, HP67_CARD_RECORDS_PER_TRACK, CRC_FLAG_WRITE_MODE, CRC_FLAG_BUFFER_READY, isa::{act_address_drive, rom_word_drive}, rom_devices::RomIsDrives}};
    fn control_word(control: &mut Hp67SerialCrcControl, bus: &mut Hp67ElectricalBackplane, word: u16) {
        for bit in 0..56 {
            bus.commit_serial_launch(act_address_drive(0, bit), RomIsDrives::Aggregate(rom_word_drive(word, bit)), if bit >= 46 { Drive::High } else { Drive::Low }, None);
            bus.advance_clock(); bus.advance_clock(); control.sample_at_edge(bus).unwrap(); bus.advance_clock(); bus.advance_clock();
        }
    }
    #[test]
    fn m15x_owned_transport_matches_reference_for_read_write_and_all_speeds() {
        for speed in [95, 100, 105] {
            for write in [false, true] {
                let mut data = Hp67CrcOwnedData::default(); let mut oracle = CrcArchitecturalCore::default(); let mut control = Hp67SerialCrcControl::default();
                if write {
                    let mut bus = Hp67ElectricalBackplane::default(); control_word(&mut control, &mut bus, 0o660); control_word(&mut control, &mut bus, 0);
                    assert!(control.flags()[CRC_FLAG_WRITE_MODE]); oracle.commit_control_flag(CRC_FLAG_WRITE_MODE as u8, true).unwrap();
                }
                let words = std::array::from_fn(|i| 0x1000000 + i as u32);
                let track = if write { Hp67MagneticTrack::default() } else { Hp67MagneticTrack::from_words(words).unwrap() };
                let card = Hp67MagneticCard::default().with_track(Hp67CardTrack::Track1, track);
                let mut actual = Hp67CardTransport::default(); actual.set_speed_percent(speed).unwrap(); actual.insert_card(card, CardInsertionEnd::End1).unwrap(); actual.set_head_active(true);
                let mut expected = actual.clone();
                assert_eq!(actual.advance_us_owned(320, true, &mut data, &mut control), expected.advance_us(320, true, &mut oracle));
                data.check_mirror(&oracle).unwrap();
                control.apply_buffer_status(false, false); oracle.commit_control_flag(CRC_FLAG_BUFFER_READY as u8, false).unwrap();
                assert_eq!(actual.advance_us_owned(320, true, &mut data, &mut control), expected.advance_us(320, true, &mut oracle));
                for word in words {
                    if write {
                        let previous = data.state(); let (_, next) = data.stage_data_word(Some(word), true).unwrap(); data.commit_staged(previous, next).unwrap(); oracle.queue_write_word(word).unwrap(); let (ready, error) = data.buffer_status(); control.apply_buffer_status(ready, error);
                    }
                    let elapsed = actual.record_interval_us();
                    assert_eq!(actual.advance_us_owned(elapsed, true, &mut data, &mut control), expected.advance_us(elapsed, true, &mut oracle));
                    if !write {
                        let previous = data.state(); let (received, next) = data.stage_data_word(None, false).unwrap(); assert_eq!(received, word); data.commit_staged(previous, next).unwrap(); assert_eq!(oracle.take_read_word(), Ok(word)); let (ready, error) = data.buffer_status(); control.apply_buffer_status(ready, error);
                    }
                    data.check_mirror(&oracle).unwrap(); assert_eq!(actual, expected);
                }
                assert_eq!(actual.next_record(), HP67_CARD_RECORDS_PER_TRACK); assert!(actual.is_complete()); assert_eq!(actual.take_completed_card(), expected.take_completed_card());
            }
        }
    }
}
