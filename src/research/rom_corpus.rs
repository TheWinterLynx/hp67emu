//! Normalization and comparison of independently sourced HP-67 ROM corpora.
//!
//! The production emulator deliberately does not depend on this module. It is
//! a research tool used to compare external firmware sources such as x11-calc,
//! Teenix ROM-reader dumps and assembled Nonpareil listings before any corpus
//! is accepted as canonical.

use std::{error::Error, fmt};

pub const ROM_BANKS: usize = 2;
pub const ROM_PAGES: usize = 4;
pub const WORDS_PER_PAGE: usize = 1024;
pub const WORDS_PER_BANK: usize = ROM_PAGES * WORDS_PER_PAGE;
pub const PHYSICAL_ROM_WORDS: usize = ROM_BANKS * WORDS_PER_BANK;
pub const WORD_MASK: u16 = 0x03ff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CorpusError {
    X11ArrayNotFound,
    X11ArrayUnterminated,
    WrongWordCount { expected: usize, actual: usize },
    InvalidNumber { token: String, radix: u32 },
    WordOutOfRange { word: u32 },
    BankOutOfRange { bank: usize },
    PcOutOfRange { pc: usize },
    DuplicateLocation { bank: usize, pc: usize },
    InvalidNormalizedLine { line: usize, text: String },
    InvalidPairLine { line: usize, text: String },
    UnsupportedRadix(String),
}

impl fmt::Display for CorpusError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::X11ArrayNotFound => write!(f, "x11-calc int i_rom[...] array was not found"),
            Self::X11ArrayUnterminated => write!(f, "x11-calc i_rom array has no closing brace"),
            Self::WrongWordCount { expected, actual } => {
                write!(
                    f,
                    "ROM contains {actual} words; expected exactly {expected}"
                )
            }
            Self::InvalidNumber { token, radix } => {
                write!(f, "invalid base-{radix} integer literal: {token}")
            }
            Self::WordOutOfRange { word } => {
                write!(f, "ROM word 0x{word:x} exceeds the 10-bit Woodstock range")
            }
            Self::BankOutOfRange { bank } => write!(f, "ROM bank {bank} is outside 0..{ROM_BANKS}"),
            Self::PcOutOfRange { pc } => write!(f, "ROM PC 0x{pc:x} is outside 0x000..0xfff"),
            Self::DuplicateLocation { bank, pc } => {
                write!(f, "duplicate ROM location bank {bank}, PC 0x{pc:03x}")
            }
            Self::InvalidNormalizedLine { line, text } => {
                write!(f, "invalid normalized ROM line {line}: {text}")
            }
            Self::InvalidPairLine { line, text } => {
                write!(f, "invalid address/opcode line {line}: {text}")
            }
            Self::UnsupportedRadix(radix) => {
                write!(f, "unsupported radix {radix}; use auto, 8, 10 or 16")
            }
        }
    }
}

impl Error for CorpusError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NumberRadix {
    Auto,
    Octal,
    Decimal,
    Hexadecimal,
}

impl NumberRadix {
    pub fn parse(value: &str) -> Result<Self, CorpusError> {
        match value.to_ascii_lowercase().as_str() {
            "auto" => Ok(Self::Auto),
            "8" | "oct" | "octal" => Ok(Self::Octal),
            "10" | "dec" | "decimal" => Ok(Self::Decimal),
            "16" | "hex" | "hexadecimal" => Ok(Self::Hexadecimal),
            _ => Err(CorpusError::UnsupportedRadix(value.to_owned())),
        }
    }

    fn value(self) -> Option<u32> {
        match self {
            Self::Auto => None,
            Self::Octal => Some(8),
            Self::Decimal => Some(10),
            Self::Hexadecimal => Some(16),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RomCorpus {
    slots: Vec<Option<u16>>,
}

impl Default for RomCorpus {
    fn default() -> Self {
        Self {
            slots: vec![None; PHYSICAL_ROM_WORDS],
        }
    }
}

impl RomCorpus {
    pub fn from_flat_words(words: &[u16]) -> Result<Self, CorpusError> {
        if words.len() != PHYSICAL_ROM_WORDS {
            return Err(CorpusError::WrongWordCount {
                expected: PHYSICAL_ROM_WORDS,
                actual: words.len(),
            });
        }

        let mut corpus = Self::default();
        for (index, &word) in words.iter().enumerate() {
            validate_word(word as u32)?;
            corpus.slots[index] = Some(word);
        }
        Ok(corpus)
    }

    pub fn set(&mut self, bank: usize, pc: usize, word: u16) -> Result<(), CorpusError> {
        validate_word(word as u32)?;
        let index = flat_index(bank, pc)?;
        if self.slots[index].is_some() {
            return Err(CorpusError::DuplicateLocation { bank, pc });
        }
        self.slots[index] = Some(word);
        Ok(())
    }

    pub fn get(&self, bank: usize, pc: usize) -> Result<Option<u16>, CorpusError> {
        Ok(self.slots[flat_index(bank, pc)?])
    }

    pub fn populated_words(&self) -> usize {
        self.slots.iter().filter(|word| word.is_some()).count()
    }

    pub fn to_normalized_tsv(&self) -> String {
        let mut output = String::from("# hp67emu-rom-corpus-v1\nbank\tpc\tword\n");
        for index in 0..PHYSICAL_ROM_WORDS {
            if let Some(word) = self.slots[index] {
                let location = location_from_index(index);
                output.push_str(&format!(
                    "{}\t0x{:03x}\t0x{:03x}\n",
                    location.bank, location.pc, word
                ));
            }
        }
        output
    }

    pub fn from_normalized_tsv(input: &str) -> Result<Self, CorpusError> {
        let mut corpus = Self::default();
        for (line_index, raw_line) in input.lines().enumerate() {
            let line_number = line_index + 1;
            let line = raw_line.trim();
            if line.is_empty()
                || line.starts_with('#')
                || line.eq_ignore_ascii_case("bank\tpc\tword")
            {
                continue;
            }
            let fields: Vec<_> = line.split_whitespace().collect();
            if fields.len() != 3 {
                return Err(CorpusError::InvalidNormalizedLine {
                    line: line_number,
                    text: raw_line.to_owned(),
                });
            }
            let bank = parse_auto_integer(fields[0])? as usize;
            let pc = parse_auto_integer(fields[1])? as usize;
            let word = parse_auto_integer(fields[2])?;
            validate_word(word)?;
            corpus.set(bank, pc, word as u16)?;
        }
        Ok(corpus)
    }

    fn slot(&self, index: usize) -> Option<u16> {
        self.slots[index]
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RomLocation {
    pub bank: usize,
    pub page: usize,
    pub offset: usize,
    pub pc: usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RomDifference {
    MissingLeft {
        location: RomLocation,
        right: u16,
    },
    MissingRight {
        location: RomLocation,
        left: u16,
    },
    Mismatch {
        location: RomLocation,
        left: u16,
        right: u16,
    },
}

impl RomDifference {
    pub fn location(self) -> RomLocation {
        match self {
            Self::MissingLeft { location, .. }
            | Self::MissingRight { location, .. }
            | Self::Mismatch { location, .. } => location,
        }
    }
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct PageSummary {
    pub left_words: usize,
    pub right_words: usize,
    pub matches: usize,
    pub mismatches: usize,
    pub left_only: usize,
    pub right_only: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comparison {
    pub pages: [[PageSummary; ROM_PAGES]; ROM_BANKS],
    pub differences: Vec<RomDifference>,
}

impl Comparison {
    pub fn is_identical(&self) -> bool {
        self.differences.is_empty()
    }

    pub fn matching_words(&self) -> usize {
        self.pages
            .iter()
            .flatten()
            .map(|summary| summary.matches)
            .sum()
    }
}

pub fn compare(left: &RomCorpus, right: &RomCorpus) -> Comparison {
    let mut comparison = Comparison {
        pages: [[PageSummary::default(); ROM_PAGES]; ROM_BANKS],
        differences: Vec::new(),
    };

    for index in 0..PHYSICAL_ROM_WORDS {
        let location = location_from_index(index);
        let summary = &mut comparison.pages[location.bank][location.page];
        let left_word = left.slot(index);
        let right_word = right.slot(index);
        if left_word.is_some() {
            summary.left_words += 1;
        }
        if right_word.is_some() {
            summary.right_words += 1;
        }

        match (left_word, right_word) {
            (None, None) => {}
            (Some(left_word), Some(right_word)) if left_word == right_word => {
                summary.matches += 1;
            }
            (Some(left_word), Some(right_word)) => {
                summary.mismatches += 1;
                comparison.differences.push(RomDifference::Mismatch {
                    location,
                    left: left_word,
                    right: right_word,
                });
            }
            (Some(left_word), None) => {
                summary.left_only += 1;
                comparison.differences.push(RomDifference::MissingRight {
                    location,
                    left: left_word,
                });
            }
            (None, Some(right_word)) => {
                summary.right_only += 1;
                comparison.differences.push(RomDifference::MissingLeft {
                    location,
                    right: right_word,
                });
            }
        }
    }

    comparison
}

/// Extract the complete 8192-word HP-67 ROM array embedded in x11-calc.
///
/// x11-calc addresses the array as `rom[pc | (bank << 12)]`; therefore the
/// first 4096 entries are bank 0 and the second 4096 entries are bank 1. This
/// is exactly the flat ordering used by `RomCorpus`.
pub fn parse_x11_hp67_c(source: &str) -> Result<RomCorpus, CorpusError> {
    let marker = "int i_rom";
    let marker_start = source.find(marker).ok_or(CorpusError::X11ArrayNotFound)?;
    let after_marker = &source[marker_start..];
    let open_relative = after_marker
        .find('{')
        .ok_or(CorpusError::X11ArrayNotFound)?;
    let open = marker_start + open_relative;
    let close_relative = source[open + 1..]
        .find('}')
        .ok_or(CorpusError::X11ArrayUnterminated)?;
    let close = open + 1 + close_relative;
    let body = strip_c_comments(&source[open + 1..close]);

    let mut words = Vec::with_capacity(PHYSICAL_ROM_WORDS);
    for token in body
        .split(',')
        .map(str::trim)
        .filter(|token| !token.is_empty())
    {
        let value = parse_c_integer(token)?;
        validate_word(value)?;
        words.push(value as u16);
    }

    RomCorpus::from_flat_words(&words)
}

/// Import a simple address/opcode text listing into one bank.
///
/// Each non-comment line must contain exactly two fields: PC and 10-bit word.
/// `#`, `;` and `//` start comments. The caller chooses the numeric radix, so
/// this can normalize assembler listings without guessing their notation.
pub fn parse_address_opcode_pairs(
    input: &str,
    bank: usize,
    radix: NumberRadix,
) -> Result<RomCorpus, CorpusError> {
    if bank >= ROM_BANKS {
        return Err(CorpusError::BankOutOfRange { bank });
    }

    let mut corpus = RomCorpus::default();
    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let line = remove_line_comment(raw_line).trim();
        if line.is_empty() {
            continue;
        }
        let normalized = line.replace(',', " ");
        let fields: Vec<_> = normalized.split_whitespace().collect();
        if fields.len() != 2 {
            return Err(CorpusError::InvalidPairLine {
                line: line_number,
                text: raw_line.to_owned(),
            });
        }
        let pc = parse_with_radix(fields[0], radix)? as usize;
        let word = parse_with_radix(fields[1], radix)?;
        validate_word(word)?;
        corpus.set(bank, pc, word as u16)?;
    }
    Ok(corpus)
}

pub fn location_from_index(index: usize) -> RomLocation {
    assert!(index < PHYSICAL_ROM_WORDS, "ROM flat index out of range");
    let bank = index / WORDS_PER_BANK;
    let pc = index % WORDS_PER_BANK;
    RomLocation {
        bank,
        page: pc / WORDS_PER_PAGE,
        offset: pc % WORDS_PER_PAGE,
        pc,
    }
}

fn flat_index(bank: usize, pc: usize) -> Result<usize, CorpusError> {
    if bank >= ROM_BANKS {
        return Err(CorpusError::BankOutOfRange { bank });
    }
    if pc >= WORDS_PER_BANK {
        return Err(CorpusError::PcOutOfRange { pc });
    }
    Ok(bank * WORDS_PER_BANK + pc)
}

fn validate_word(word: u32) -> Result<(), CorpusError> {
    if word > WORD_MASK as u32 {
        Err(CorpusError::WordOutOfRange { word })
    } else {
        Ok(())
    }
}

fn parse_c_integer(token: &str) -> Result<u32, CorpusError> {
    let token = token.trim();
    let literal = token.trim_end_matches(|ch: char| matches!(ch, 'u' | 'U' | 'l' | 'L'));
    let (digits, radix) = if let Some(rest) = literal
        .strip_prefix("0x")
        .or_else(|| literal.strip_prefix("0X"))
    {
        (rest, 16)
    } else if literal.len() > 1 && literal.starts_with('0') {
        (&literal[1..], 8)
    } else {
        (literal, 10)
    };
    u32::from_str_radix(if digits.is_empty() { "0" } else { digits }, radix).map_err(|_| {
        CorpusError::InvalidNumber {
            token: token.to_owned(),
            radix,
        }
    })
}

fn parse_auto_integer(token: &str) -> Result<u32, CorpusError> {
    let trimmed = token.trim();
    if let Some(rest) = trimmed
        .strip_prefix("0x")
        .or_else(|| trimmed.strip_prefix("0X"))
    {
        parse_digits(rest, 16, trimmed)
    } else if let Some(rest) = trimmed
        .strip_prefix("0o")
        .or_else(|| trimmed.strip_prefix("0O"))
    {
        parse_digits(rest, 8, trimmed)
    } else {
        parse_digits(trimmed, 10, trimmed)
    }
}

fn parse_with_radix(token: &str, radix: NumberRadix) -> Result<u32, CorpusError> {
    match radix.value() {
        None => parse_auto_integer(token),
        Some(value) => {
            let trimmed = token.trim();
            let digits = match value {
                8 => trimmed
                    .strip_prefix("0o")
                    .or_else(|| trimmed.strip_prefix("0O"))
                    .unwrap_or(trimmed),
                16 => trimmed
                    .strip_prefix("0x")
                    .or_else(|| trimmed.strip_prefix("0X"))
                    .unwrap_or(trimmed),
                _ => trimmed,
            };
            parse_digits(digits, value, trimmed)
        }
    }
}

fn parse_digits(digits: &str, radix: u32, original: &str) -> Result<u32, CorpusError> {
    u32::from_str_radix(digits, radix).map_err(|_| CorpusError::InvalidNumber {
        token: original.to_owned(),
        radix,
    })
}

fn strip_c_comments(input: &str) -> String {
    let bytes = input.as_bytes();
    let mut output = String::with_capacity(input.len());
    let mut index = 0;
    while index < bytes.len() {
        if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'/' {
            index += 2;
            while index < bytes.len() && bytes[index] != b'\n' {
                index += 1;
            }
        } else if index + 1 < bytes.len() && bytes[index] == b'/' && bytes[index + 1] == b'*' {
            index += 2;
            while index + 1 < bytes.len() && !(bytes[index] == b'*' && bytes[index + 1] == b'/') {
                if bytes[index] == b'\n' {
                    output.push('\n');
                }
                index += 1;
            }
            index = (index + 2).min(bytes.len());
        } else {
            output.push(bytes[index] as char);
            index += 1;
        }
    }
    output
}

fn remove_line_comment(line: &str) -> &str {
    let mut end = line.len();
    for marker in ["//", "#", ";"] {
        if let Some(position) = line.find(marker) {
            end = end.min(position);
        }
    }
    &line[..end]
}

#[cfg(test)]
mod tests {
    use super::*;

    fn synthetic_x11_source(overrides: &[(usize, u16)]) -> String {
        let mut words: Vec<u16> = (0..PHYSICAL_ROM_WORDS)
            .map(|index| (index as u16) & WORD_MASK)
            .collect();
        for &(index, word) in overrides {
            words[index] = word;
        }
        let body = words
            .iter()
            .map(|word| format!("0{:o}", word))
            .collect::<Vec<_>>()
            .join(", ");
        format!("int unrelated = 1;\nint i_rom[ROM_SIZE] = {{\n{body}\n}};\n")
    }

    #[test]
    fn x11_array_maps_bit_12_to_bank_selection() {
        let source =
            synthetic_x11_source(&[(0, 0o001), (4095, 0o002), (4096, 0o003), (8191, 0o004)]);
        let corpus = parse_x11_hp67_c(&source).expect("synthetic x11 ROM must parse");

        assert_eq!(corpus.populated_words(), PHYSICAL_ROM_WORDS);
        assert_eq!(corpus.get(0, 0x000).unwrap(), Some(0o001));
        assert_eq!(corpus.get(0, 0xfff).unwrap(), Some(0o002));
        assert_eq!(corpus.get(1, 0x000).unwrap(), Some(0o003));
        assert_eq!(corpus.get(1, 0xfff).unwrap(), Some(0o004));
    }

    #[test]
    fn x11_parser_rejects_non_ten_bit_words() {
        let source = synthetic_x11_source(&[(17, 0x400)]);
        assert_eq!(
            parse_x11_hp67_c(&source),
            Err(CorpusError::WordOutOfRange { word: 0x400 })
        );
    }

    #[test]
    fn normalized_tsv_round_trips_sparse_corpus() {
        let mut corpus = RomCorpus::default();
        corpus.set(0, 0x000, 0o1234).unwrap();
        corpus.set(1, 0x456, 0o0765).unwrap();
        let encoded = corpus.to_normalized_tsv();
        let decoded = RomCorpus::from_normalized_tsv(&encoded).unwrap();
        assert_eq!(decoded, corpus);
    }

    #[test]
    fn pair_importer_accepts_explicit_octal_listing() {
        let input = "0000 00000 ; reset\n0001 01743\n1777 00042 # end page\n";
        let corpus = parse_address_opcode_pairs(input, 0, NumberRadix::Octal).unwrap();
        assert_eq!(corpus.get(0, 0o0000).unwrap(), Some(0o00000));
        assert_eq!(corpus.get(0, 0o0001).unwrap(), Some(0o01743));
        assert_eq!(corpus.get(0, 0o1777).unwrap(), Some(0o00042));
    }

    #[test]
    fn comparison_reports_value_and_presence_differences_by_page() {
        let mut left = RomCorpus::default();
        let mut right = RomCorpus::default();
        left.set(0, 0x010, 1).unwrap();
        right.set(0, 0x010, 2).unwrap();
        left.set(1, 0x400, 3).unwrap();
        right.set(1, 0x401, 4).unwrap();

        let result = compare(&left, &right);
        assert!(!result.is_identical());
        assert_eq!(result.differences.len(), 3);
        assert_eq!(result.pages[0][0].mismatches, 1);
        assert_eq!(result.pages[1][1].left_only, 1);
        assert_eq!(result.pages[1][1].right_only, 1);
    }

    #[test]
    fn x11_parser_requires_complete_physical_array() {
        let source = "int i_rom[ROM_SIZE] = { 00000, 01743 };";
        assert_eq!(
            parse_x11_hp67_c(source),
            Err(CorpusError::WrongWordCount {
                expected: PHYSICAL_ROM_WORDS,
                actual: 2,
            })
        );
    }
}
