//! Physical HP-67 magnetic-card media model and host file formats.
//!
//! The physical object is one card with two user-visible logical tracks, matching
//! HP's "side 1 / side 2" terminology.  The printed face remains visible;
//! inserting the opposite end selects the other logical track.  Each logical bit
//! stream is physically recorded by a parallel 0-track/1-track flux pair; that
//! self-clocking encoding is modeled separately in `card_flux`.  This module
//! deliberately remains at the CRC-visible 34 x 28-bit media boundary.

use super::crc::CRC_CARD_WORD_MASK;

pub const HP67_CARD_RECORDS_PER_TRACK: usize = 34;
pub const HP67_CARD_BITS_PER_TRACK: usize = HP67_CARD_RECORDS_PER_TRACK * 28;
pub const HP67_CARD_LOGICAL_BYTES_PER_TRACK: usize = HP67_CARD_BITS_PER_TRACK / 8;
pub const HP67_CARD_LOGICAL_BYTES: usize = HP67_CARD_LOGICAL_BYTES_PER_TRACK * 2;
pub const HP67_CARD_CONTAINER_BYTES: usize = 12 + HP67_CARD_LOGICAL_BYTES;

const HP67_CARD_MAGIC: [u8; 8] = *b"HP67CARD";
const HP67_CARD_VERSION: u8 = 1;
const TRACK_FLAG_RECORDED: u8 = 0x01;
const TRACK_FLAG_WRITE_PROTECTED: u8 = 0x02;
const TRACK_FLAG_MASK: u8 = TRACK_FLAG_RECORDED | TRACK_FLAG_WRITE_PROTECTED;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CardTrack {
    Track1,
    Track2,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CardInsertionEnd {
    End1,
    End2,
}

impl CardInsertionEnd {
    pub const fn track(self) -> Hp67CardTrack {
        match self {
            Self::End1 => Hp67CardTrack::Track1,
            Self::End2 => Hp67CardTrack::Track2,
        }
    }

    pub const fn opposite(self) -> Self {
        match self {
            Self::End1 => Self::End2,
            Self::End2 => Self::End1,
        }
    }

    pub const fn for_track(track: Hp67CardTrack) -> Self {
        match track {
            Hp67CardTrack::Track1 => Self::End1,
            Hp67CardTrack::Track2 => Self::End2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TrackMedia {
    Unrecorded,
    Recorded([u32; HP67_CARD_RECORDS_PER_TRACK]),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67MagneticTrack {
    media: TrackMedia,
    write_protected: bool,
    dirty: bool,
}

impl Default for Hp67MagneticTrack {
    fn default() -> Self {
        Self {
            media: TrackMedia::Unrecorded,
            write_protected: false,
            dirty: false,
        }
    }
}

impl Hp67MagneticTrack {
    pub fn from_words(
        words: [u32; HP67_CARD_RECORDS_PER_TRACK],
    ) -> Result<Self, Hp67MagneticCardError> {
        validate_words(&words)?;
        Ok(Self {
            media: TrackMedia::Recorded(words),
            write_protected: false,
            dirty: false,
        })
    }

    pub fn media(&self) -> &TrackMedia {
        &self.media
    }

    pub fn words(&self) -> Option<&[u32; HP67_CARD_RECORDS_PER_TRACK]> {
        match &self.media {
            TrackMedia::Unrecorded => None,
            TrackMedia::Recorded(words) => Some(words),
        }
    }

    pub fn is_recorded(&self) -> bool {
        matches!(&self.media, TrackMedia::Recorded(_))
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

    pub fn set_write_protected(&mut self, protected: bool) {
        self.write_protected = protected;
    }

    pub fn mark_clean(&mut self) {
        self.dirty = false;
    }

    pub fn word(&self, index: usize) -> Option<u32> {
        match &self.media {
            TrackMedia::Unrecorded => None,
            TrackMedia::Recorded(words) => words.get(index).copied(),
        }
    }

    pub(crate) fn write_word(&mut self, index: usize, word: u32) {
        debug_assert!(index < HP67_CARD_RECORDS_PER_TRACK);
        debug_assert!(word <= CRC_CARD_WORD_MASK);

        if matches!(self.media, TrackMedia::Unrecorded) {
            self.media = TrackMedia::Recorded([0; HP67_CARD_RECORDS_PER_TRACK]);
        }

        let TrackMedia::Recorded(words) = &mut self.media else {
            unreachable!("unrecorded media is materialized before writing");
        };
        words[index] = word;
        self.dirty = true;
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hp67MagneticCard {
    track_1: Hp67MagneticTrack,
    track_2: Hp67MagneticTrack,
}

impl Hp67MagneticCard {
    pub fn new(track_1: Hp67MagneticTrack, track_2: Hp67MagneticTrack) -> Self {
        Self { track_1, track_2 }
    }

    pub fn track(&self, track: Hp67CardTrack) -> &Hp67MagneticTrack {
        match track {
            Hp67CardTrack::Track1 => &self.track_1,
            Hp67CardTrack::Track2 => &self.track_2,
        }
    }

    pub fn track_mut(&mut self, track: Hp67CardTrack) -> &mut Hp67MagneticTrack {
        match track {
            Hp67CardTrack::Track1 => &mut self.track_1,
            Hp67CardTrack::Track2 => &mut self.track_2,
        }
    }

    pub fn set_track(&mut self, track: Hp67CardTrack, media: Hp67MagneticTrack) {
        *self.track_mut(track) = media;
    }

    pub const fn dirty(&self) -> bool {
        self.track_1.dirty() || self.track_2.dirty()
    }

    pub fn mark_clean(&mut self) {
        self.track_1.mark_clean();
        self.track_2.mark_clean();
    }

    pub fn with_track(mut self, track: Hp67CardTrack, media: Hp67MagneticTrack) -> Self {
        self.set_track(track, media);
        self
    }

    pub fn to_hp67raw_bytes(&self) -> Result<[u8; HP67_CARD_LOGICAL_BYTES], Hp67MagneticCardError> {
        let track_1 =
            self.track_1
                .words()
                .ok_or(Hp67MagneticCardError::RawRequiresRecordedTrack(
                    Hp67CardTrack::Track1,
                ))?;
        let track_2 =
            self.track_2
                .words()
                .ok_or(Hp67MagneticCardError::RawRequiresRecordedTrack(
                    Hp67CardTrack::Track2,
                ))?;

        let packed_1 = pack_track(track_1);
        let packed_2 = pack_track(track_2);
        let mut out = [0u8; HP67_CARD_LOGICAL_BYTES];
        out[..HP67_CARD_LOGICAL_BYTES_PER_TRACK].copy_from_slice(&packed_1);
        out[HP67_CARD_LOGICAL_BYTES_PER_TRACK..].copy_from_slice(&packed_2);
        Ok(out)
    }

    pub fn from_hp67raw_bytes(bytes: &[u8]) -> Result<Self, Hp67MagneticCardError> {
        if bytes.len() != HP67_CARD_LOGICAL_BYTES {
            return Err(Hp67MagneticCardError::InvalidRawLength {
                expected: HP67_CARD_LOGICAL_BYTES,
                actual: bytes.len(),
            });
        }

        let track_1 = Hp67MagneticTrack::from_words(unpack_track(
            &bytes[..HP67_CARD_LOGICAL_BYTES_PER_TRACK],
        ))?;
        let track_2 = Hp67MagneticTrack::from_words(unpack_track(
            &bytes[HP67_CARD_LOGICAL_BYTES_PER_TRACK..],
        ))?;
        Ok(Self::new(track_1, track_2))
    }

    pub fn to_hp67card_bytes(&self) -> [u8; HP67_CARD_CONTAINER_BYTES] {
        let mut out = [0u8; HP67_CARD_CONTAINER_BYTES];
        out[..8].copy_from_slice(&HP67_CARD_MAGIC);
        out[8] = HP67_CARD_VERSION;
        out[9] = track_flags(&self.track_1);
        out[10] = track_flags(&self.track_2);
        out[11] = 0;

        if let Some(words) = self.track_1.words() {
            let packed = pack_track(words);
            out[12..12 + HP67_CARD_LOGICAL_BYTES_PER_TRACK].copy_from_slice(&packed);
        }
        if let Some(words) = self.track_2.words() {
            let packed = pack_track(words);
            let start = 12 + HP67_CARD_LOGICAL_BYTES_PER_TRACK;
            out[start..start + HP67_CARD_LOGICAL_BYTES_PER_TRACK].copy_from_slice(&packed);
        }

        out
    }

    pub fn from_hp67card_bytes(bytes: &[u8]) -> Result<Self, Hp67MagneticCardError> {
        if bytes.len() != HP67_CARD_CONTAINER_BYTES {
            return Err(Hp67MagneticCardError::InvalidCardLength {
                expected: HP67_CARD_CONTAINER_BYTES,
                actual: bytes.len(),
            });
        }
        if bytes[..8] != HP67_CARD_MAGIC {
            return Err(Hp67MagneticCardError::InvalidCardMagic);
        }
        if bytes[8] != HP67_CARD_VERSION {
            return Err(Hp67MagneticCardError::UnsupportedCardVersion(bytes[8]));
        }
        if bytes[11] != 0 {
            return Err(Hp67MagneticCardError::NonZeroReservedByte(bytes[11]));
        }

        let track_1 = decode_container_track(
            Hp67CardTrack::Track1,
            bytes[9],
            &bytes[12..12 + HP67_CARD_LOGICAL_BYTES_PER_TRACK],
        )?;
        let start = 12 + HP67_CARD_LOGICAL_BYTES_PER_TRACK;
        let track_2 = decode_container_track(
            Hp67CardTrack::Track2,
            bytes[10],
            &bytes[start..start + HP67_CARD_LOGICAL_BYTES_PER_TRACK],
        )?;

        Ok(Self::new(track_1, track_2))
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hp67MagneticCardError {
    WordOutOfRange { index: usize, word: u32 },
    InvalidRawLength { expected: usize, actual: usize },
    RawRequiresRecordedTrack(Hp67CardTrack),
    InvalidCardLength { expected: usize, actual: usize },
    InvalidCardMagic,
    UnsupportedCardVersion(u8),
    NonZeroReservedByte(u8),
    InvalidTrackFlags { track: Hp67CardTrack, flags: u8 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeenixHppImport {
    pub calculator_id: u8,
    pub bitmap_name: String,
    pub card_name: String,
    pub card_track: Hp67CardTrack,
    pub header_id: u8,
    pub track: Hp67MagneticTrack,
    pub declared_text_len: usize,
    pub actual_text_len: usize,
    pub length_matches: bool,
}

impl TeenixHppImport {
    pub fn from_bytes(encoded: &[u8]) -> Result<Self, TeenixHppError> {
        let decoded = encoded.iter().map(|byte| byte ^ 0x55).collect::<Vec<_>>();
        let decoded = std::str::from_utf8(&decoded).map_err(|_| TeenixHppError::InvalidUtf8)?;

        let (magic, after_magic) = split_hpp_line(decoded, "magic")?;
        if magic.trim() != "NeWe" {
            return Err(TeenixHppError::InvalidMagic(magic.trim().to_owned()));
        }

        let (length_line, raw_body) = split_hpp_line(after_magic, "length")?;
        let declared_text_len = length_line
            .trim()
            .parse::<usize>()
            .map_err(|_| TeenixHppError::InvalidLength(length_line.trim().to_owned()))?;
        let actual_text_len = raw_body.len();
        let normalized_body = raw_body.replace("\r\n", "\n").replace('\r', "\n");

        let (calculator_line, body) = split_hpp_line(&normalized_body, "calculator")?;
        let calculator_id = calculator_line
            .trim()
            .parse::<u8>()
            .map_err(|_| TeenixHppError::InvalidCalculator(calculator_line.trim().to_owned()))?;
        if !matches!(calculator_id, 67 | 97) {
            return Err(TeenixHppError::InvalidCalculator(
                calculator_line.trim().to_owned(),
            ));
        }

        let (bitmap_name, body) = split_hpp_line(body, "bitmap name")?;
        let (card_name, card_data) = split_hpp_line(body, "card name")?;

        let mut nibbles = Vec::new();
        for ch in card_data.chars() {
            if ch.is_ascii_whitespace() {
                continue;
            }
            let Some(value) = ch.to_digit(16) else {
                return Err(TeenixHppError::InvalidDataCharacter(ch));
            };
            nibbles.push(value as u8);
        }

        const DUMMY_NIBBLES: usize = 21;
        const REAL_NIBBLES: usize = HP67_CARD_RECORDS_PER_TRACK * 7;
        const DUPLICATE_CHECKSUM_NIBBLES: usize = 7;
        const EXPECTED_NIBBLES: usize = DUMMY_NIBBLES + REAL_NIBBLES + DUPLICATE_CHECKSUM_NIBBLES;

        if nibbles.len() != EXPECTED_NIBBLES {
            return Err(TeenixHppError::InvalidNibbleCount {
                expected: EXPECTED_NIBBLES,
                actual: nibbles.len(),
            });
        }

        let real = &nibbles[DUMMY_NIBBLES..DUMMY_NIBBLES + REAL_NIBBLES];
        let mirrored_checksum = &nibbles[DUMMY_NIBBLES + REAL_NIBBLES..];
        if &real[REAL_NIBBLES - 7..] != mirrored_checksum {
            return Err(TeenixHppError::ChecksumMirrorMismatch);
        }

        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        for (record_index, word) in words.iter_mut().enumerate() {
            let start = record_index * 7;
            for nibble_index in 0..7 {
                *word = (*word << 4) | u32::from(real[start + nibble_index]);
            }
        }

        let header_id = ((words[0] >> 24) & 0x0f) as u8;
        let card_track = match header_id {
            1 | 3 => Hp67CardTrack::Track1,
            2 | 4 => Hp67CardTrack::Track2,
            other => return Err(TeenixHppError::UnsupportedHeaderId(other)),
        };
        let track = Hp67MagneticTrack::from_words(words).map_err(TeenixHppError::Magnetic)?;

        Ok(Self {
            calculator_id,
            bitmap_name: bitmap_name.trim().to_owned(),
            card_name: card_name.trim().to_owned(),
            card_track,
            header_id,
            track,
            declared_text_len,
            actual_text_len,
            length_matches: declared_text_len == actual_text_len,
        })
    }

    pub fn into_card(self) -> Hp67MagneticCard {
        Hp67MagneticCard::default().with_track(self.card_track, self.track)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeenixHppError {
    InvalidUtf8,
    MissingField(&'static str),
    InvalidMagic(String),
    InvalidLength(String),
    InvalidCalculator(String),
    InvalidDataCharacter(char),
    InvalidNibbleCount { expected: usize, actual: usize },
    ChecksumMirrorMismatch,
    UnsupportedHeaderId(u8),
    Magnetic(Hp67MagneticCardError),
}

fn split_hpp_line<'a>(
    input: &'a str,
    field: &'static str,
) -> Result<(&'a str, &'a str), TeenixHppError> {
    input
        .split_once('\n')
        .ok_or(TeenixHppError::MissingField(field))
}

fn validate_words(words: &[u32; HP67_CARD_RECORDS_PER_TRACK]) -> Result<(), Hp67MagneticCardError> {
    for (index, word) in words.iter().copied().enumerate() {
        if word > CRC_CARD_WORD_MASK {
            return Err(Hp67MagneticCardError::WordOutOfRange { index, word });
        }
    }
    Ok(())
}

fn pack_track(
    words: &[u32; HP67_CARD_RECORDS_PER_TRACK],
) -> [u8; HP67_CARD_LOGICAL_BYTES_PER_TRACK] {
    let mut packed = [0u8; HP67_CARD_LOGICAL_BYTES_PER_TRACK];
    let mut bit_index = 0usize;

    for word in words {
        for bit in (0..28).rev() {
            if word & (1u32 << bit) != 0 {
                packed[bit_index / 8] |= 1u8 << (7 - (bit_index % 8));
            }
            bit_index += 1;
        }
    }

    debug_assert_eq!(bit_index, HP67_CARD_BITS_PER_TRACK);
    packed
}

fn unpack_track(bytes: &[u8]) -> [u32; HP67_CARD_RECORDS_PER_TRACK] {
    debug_assert_eq!(bytes.len(), HP67_CARD_LOGICAL_BYTES_PER_TRACK);
    let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
    let mut bit_index = 0usize;

    for word in &mut words {
        for bit in (0..28).rev() {
            if bytes[bit_index / 8] & (1u8 << (7 - (bit_index % 8))) != 0 {
                *word |= 1u32 << bit;
            }
            bit_index += 1;
        }
    }

    debug_assert_eq!(bit_index, HP67_CARD_BITS_PER_TRACK);
    words
}

fn track_flags(track: &Hp67MagneticTrack) -> u8 {
    let mut flags = 0u8;
    if track.is_recorded() {
        flags |= TRACK_FLAG_RECORDED;
    }
    if track.write_protected() {
        flags |= TRACK_FLAG_WRITE_PROTECTED;
    }
    flags
}

fn decode_container_track(
    track_id: Hp67CardTrack,
    flags: u8,
    payload: &[u8],
) -> Result<Hp67MagneticTrack, Hp67MagneticCardError> {
    if flags & !TRACK_FLAG_MASK != 0 {
        return Err(Hp67MagneticCardError::InvalidTrackFlags {
            track: track_id,
            flags,
        });
    }

    let mut track = if flags & TRACK_FLAG_RECORDED != 0 {
        Hp67MagneticTrack::from_words(unpack_track(payload))?
    } else {
        Hp67MagneticTrack::default()
    };
    track.set_write_protected(flags & TRACK_FLAG_WRITE_PROTECTED != 0);
    Ok(track)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_words(seed: u32) -> [u32; HP67_CARD_RECORDS_PER_TRACK] {
        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        for (index, word) in words.iter_mut().enumerate() {
            *word = (seed.wrapping_add(index as u32 * 0x0012_3456)) & CRC_CARD_WORD_MASK;
        }
        words
    }

    fn encode_teenix(words: [u32; HP67_CARD_RECORDS_PER_TRACK]) -> Vec<u8> {
        let mut nibbles = vec![0u8; 21];
        for word in words {
            for nibble in (0..7).rev() {
                nibbles.push(((word >> (nibble * 4)) & 0x0f) as u8);
            }
        }
        let checksum = nibbles[nibbles.len() - 7..].to_vec();
        nibbles.extend_from_slice(&checksum);

        let data = nibbles
            .into_iter()
            .map(|nibble| char::from_digit(u32::from(nibble), 16).unwrap())
            .collect::<String>();
        let body = format!("67\nmoon.bmp\nMoon Rocket Lander\n{data}\n");
        let decoded = format!("NeWe\n{}\n{body}", body.len());
        decoded.bytes().map(|byte| byte ^ 0x55).collect()
    }

    #[test]
    fn insertion_end_selects_the_opposite_logical_card_track() {
        assert_eq!(CardInsertionEnd::End1.track(), Hp67CardTrack::Track1);
        assert_eq!(CardInsertionEnd::End2.track(), Hp67CardTrack::Track2);
        assert_eq!(CardInsertionEnd::End1.opposite(), CardInsertionEnd::End2);
        assert_eq!(CardInsertionEnd::End2.opposite(), CardInsertionEnd::End1);
    }

    #[test]
    fn raw_logical_image_is_exactly_238_bytes_and_round_trips() {
        let card = Hp67MagneticCard::new(
            Hp67MagneticTrack::from_words(sample_words(0x0010_0000)).unwrap(),
            Hp67MagneticTrack::from_words(sample_words(0x0020_0000)).unwrap(),
        );
        let raw = card.to_hp67raw_bytes().unwrap();
        assert_eq!(raw.len(), 238);
        assert_eq!(Hp67MagneticCard::from_hp67raw_bytes(&raw).unwrap(), card);
    }

    #[test]
    fn card_container_preserves_unrecorded_state_and_per_track_write_protect() {
        let card = Hp67MagneticCard::default().with_track(
            Hp67CardTrack::Track1,
            Hp67MagneticTrack::from_words(sample_words(0x0030_0000))
                .unwrap()
                .with_write_protected(true),
        );
        let encoded = card.to_hp67card_bytes();
        assert_eq!(encoded.len(), HP67_CARD_CONTAINER_BYTES);

        let decoded = Hp67MagneticCard::from_hp67card_bytes(&encoded).unwrap();
        assert!(decoded.track(Hp67CardTrack::Track1).is_recorded());
        assert!(decoded.track(Hp67CardTrack::Track1).write_protected());
        assert!(!decoded.track(Hp67CardTrack::Track2).is_recorded());
        assert!(!decoded.track(Hp67CardTrack::Track2).write_protected());
    }

    #[test]
    fn teenix_record_nibbles_are_high_to_low_like_the_documented_status_word() {
        let mut words = [0u32; HP67_CARD_RECORDS_PER_TRACK];
        words[0] = 0x0222_0013;
        let imported = TeenixHppImport::from_bytes(&encode_teenix(words)).unwrap();
        assert_eq!(imported.track.word(0), Some(0x0222_0013));
        assert_eq!(imported.header_id, 2);
        assert_eq!(imported.card_track, Hp67CardTrack::Track2);
    }

    #[test]
    fn teenix_import_discards_dummy_records_and_duplicate_checksum() {
        let mut words = sample_words(0x0004_2000);
        words[0] = 0x0300_0222;
        let encoded = encode_teenix(words);
        let imported = TeenixHppImport::from_bytes(&encoded).unwrap();

        assert_eq!(imported.calculator_id, 67);
        assert_eq!(imported.card_name, "Moon Rocket Lander");
        assert_eq!(imported.card_track, Hp67CardTrack::Track1);
        assert_eq!(imported.header_id, 3);
        assert!(imported.length_matches);
        assert_eq!(imported.track.words(), Some(&words));
    }

    #[test]
    fn teenix_header_is_authoritative_for_second_track() {
        let mut words = sample_words(0x0003_1000);
        words[0] = 0x0400_0000;
        let imported = TeenixHppImport::from_bytes(&encode_teenix(words)).unwrap();
        assert_eq!(imported.card_track, Hp67CardTrack::Track2);
        assert_eq!(imported.header_id, 4);
    }

    #[test]
    fn whole_card_dirty_state_clears_only_when_explicitly_marked_clean() {
        let mut card = Hp67MagneticCard::default();
        card.track_mut(Hp67CardTrack::Track1)
            .write_word(0, 0x0123_4567);
        assert!(card.dirty());
        assert!(card.track(Hp67CardTrack::Track1).dirty());
        assert!(!card.track(Hp67CardTrack::Track2).dirty());

        card.mark_clean();
        assert!(!card.dirty());
        assert!(!card.track(Hp67CardTrack::Track1).dirty());
        assert!(!card.track(Hp67CardTrack::Track2).dirty());
    }

    #[test]
    fn raw_format_refuses_to_invent_bits_for_an_unrecorded_track() {
        let card = Hp67MagneticCard::default().with_track(
            Hp67CardTrack::Track1,
            Hp67MagneticTrack::from_words(sample_words(0x0040_0000)).unwrap(),
        );
        assert_eq!(
            card.to_hp67raw_bytes(),
            Err(Hp67MagneticCardError::RawRequiresRecordedTrack(
                Hp67CardTrack::Track2
            ))
        );
    }
}
