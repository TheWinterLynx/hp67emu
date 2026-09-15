//! Parser for Nonpareil uasm Woodstock object files.
//!
//! This is research tooling only. It consumes the textual object format emitted
//! by Nonpareil's official `uasm` so HP-67 sources can be assembled upstream and
//! normalized into hp67emu's sparse `(bank, pc, word)` corpus without copying
//! Nonpareil's assembler implementation.

use std::{error::Error, fmt};

use super::rom_corpus::{CorpusError, RomCorpus, ROM_BANKS, WORD_MASK, WORDS_PER_BANK};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum NonpareilObjError {
    InvalidLine { line: usize, text: String },
    InvalidBankSpec { line: usize, text: String },
    InvalidOctal { line: usize, token: String },
    BankOutOfRange { line: usize, bank: usize },
    AddressOutOfRange { line: usize, address: usize },
    OpcodeOutOfRange { line: usize, opcode: u32 },
    Corpus(CorpusError),
}

impl fmt::Display for NonpareilObjError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidLine { line, text } => {
                write!(f, "invalid Nonpareil object line {line}: {text}")
            }
            Self::InvalidBankSpec { line, text } => {
                write!(f, "invalid Nonpareil bank prefix on line {line}: {text}")
            }
            Self::InvalidOctal { line, token } => {
                write!(f, "invalid octal token on Nonpareil object line {line}: {token}")
            }
            Self::BankOutOfRange { line, bank } => {
                write!(f, "Nonpareil object line {line} names unsupported bank {bank}")
            }
            Self::AddressOutOfRange { line, address } => write!(
                f,
                "Nonpareil object line {line} address {address:04o} exceeds HP-67 4K PC space"
            ),
            Self::OpcodeOutOfRange { line, opcode } => write!(
                f,
                "Nonpareil object line {line} opcode {opcode:04o} exceeds 10-bit Woodstock range"
            ),
            Self::Corpus(error) => error.fmt(f),
        }
    }
}

impl Error for NonpareilObjError {}

impl From<CorpusError> for NonpareilObjError {
    fn from(value: CorpusError) -> Self {
        Self::Corpus(value)
    }
}

/// Parse the current Nonpareil `uasm` Woodstock object format.
///
/// Current `uasm` writes Woodstock words as optional bank mask plus octal
/// address/opcode, for example `[0]0001:1743` or `[1]2000:0432`. A prefix such
/// as `[01]` emits the same word into both banks. Historical prefix-less object
/// lines are accepted as bank 0. Blank lines and `#` metadata/comments are
/// ignored.
pub fn parse_nonpareil_woodstock_obj(input: &str) -> Result<RomCorpus, NonpareilObjError> {
    let mut corpus = RomCorpus::default();

    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let line = raw_line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }

        let (banks, payload) = parse_bank_prefix(line_number, line)?;
        let Some((address_text, opcode_text)) = payload.split_once(':') else {
            return Err(NonpareilObjError::InvalidLine {
                line: line_number,
                text: raw_line.to_owned(),
            });
        };
        if opcode_text.contains(':') {
            return Err(NonpareilObjError::InvalidLine {
                line: line_number,
                text: raw_line.to_owned(),
            });
        }

        let address = parse_octal(line_number, address_text)? as usize;
        let opcode = parse_octal(line_number, opcode_text)?;
        if address >= WORDS_PER_BANK {
            return Err(NonpareilObjError::AddressOutOfRange {
                line: line_number,
                address,
            });
        }
        if opcode > u32::from(WORD_MASK) {
            return Err(NonpareilObjError::OpcodeOutOfRange {
                line: line_number,
                opcode,
            });
        }

        for bank in banks {
            corpus.set(bank, address, opcode as u16)?;
        }
    }

    Ok(corpus)
}

fn parse_bank_prefix(line_number: usize, line: &str) -> Result<(Vec<usize>, &str), NonpareilObjError> {
    if !line.starts_with('[') {
        return Ok((vec![0], line));
    }

    let Some(end) = line.find(']') else {
        return Err(NonpareilObjError::InvalidBankSpec {
            line: line_number,
            text: line.to_owned(),
        });
    };
    let bank_text = &line[1..end];
    if bank_text.is_empty() {
        return Err(NonpareilObjError::InvalidBankSpec {
            line: line_number,
            text: line.to_owned(),
        });
    }

    let mut banks = Vec::new();
    for byte in bank_text.bytes() {
        if !byte.is_ascii_digit() {
            return Err(NonpareilObjError::InvalidBankSpec {
                line: line_number,
                text: line.to_owned(),
            });
        }
        let bank = usize::from(byte - b'0');
        if bank >= ROM_BANKS {
            return Err(NonpareilObjError::BankOutOfRange {
                line: line_number,
                bank,
            });
        }
        if banks.contains(&bank) {
            return Err(NonpareilObjError::InvalidBankSpec {
                line: line_number,
                text: line.to_owned(),
            });
        }
        banks.push(bank);
    }

    let payload = line[end + 1..].trim();
    if payload.is_empty() {
        return Err(NonpareilObjError::InvalidLine {
            line: line_number,
            text: line.to_owned(),
        });
    }
    Ok((banks, payload))
}

fn parse_octal(line_number: usize, token: &str) -> Result<u32, NonpareilObjError> {
    let token = token.trim();
    u32::from_str_radix(token, 8).map_err(|_| NonpareilObjError::InvalidOctal {
        line: line_number,
        token: token.to_owned(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_current_uasm_bank_prefixed_format() {
        let input = "# SPDX-License-Identifier: GPL-3.0\n[0]0000:0000\n[0]0001:1743\n[1]2000:0432\n";
        let corpus = parse_nonpareil_woodstock_obj(input).unwrap();
        assert_eq!(corpus.populated_words(), 3);
        assert_eq!(corpus.get(0, 0x000).unwrap(), Some(0x000));
        assert_eq!(corpus.get(0, 0x001).unwrap(), Some(0x3e3));
        assert_eq!(corpus.get(1, 0x400).unwrap(), Some(0x11a));
    }

    #[test]
    fn expands_multi_bank_masks() {
        let corpus = parse_nonpareil_woodstock_obj("[01]0370:0432\n").unwrap();
        assert_eq!(corpus.populated_words(), 2);
        assert_eq!(corpus.get(0, 0x0f8).unwrap(), Some(0x11a));
        assert_eq!(corpus.get(1, 0x0f8).unwrap(), Some(0x11a));
    }

    #[test]
    fn accepts_historical_prefixless_lines_as_bank_zero() {
        let corpus = parse_nonpareil_woodstock_obj("0001:1743\n").unwrap();
        assert_eq!(corpus.get(0, 0x001).unwrap(), Some(0x3e3));
    }

    #[test]
    fn rejects_non_octal_and_out_of_range_words() {
        assert!(matches!(
            parse_nonpareil_woodstock_obj("[0]0008:0000\n"),
            Err(NonpareilObjError::InvalidOctal { .. })
        ));
        assert!(matches!(
            parse_nonpareil_woodstock_obj("[0]0000:2000\n"),
            Err(NonpareilObjError::OpcodeOutOfRange { .. })
        ));
    }

    #[test]
    fn rejects_unsupported_banks_and_duplicate_locations() {
        assert!(matches!(
            parse_nonpareil_woodstock_obj("[2]0000:0000\n"),
            Err(NonpareilObjError::BankOutOfRange { .. })
        ));
        assert!(matches!(
            parse_nonpareil_woodstock_obj("[0]0000:0000\n[0]0000:0001\n"),
            Err(NonpareilObjError::Corpus(CorpusError::DuplicateLocation { .. }))
        ));
    }
}
