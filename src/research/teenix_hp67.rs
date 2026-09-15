//! Parser and normalizer for the current Teenix HP-67 textual microcode listing.
//!
//! The decoded `cal67.pfl` payload is a source-style Woodstock listing with
//! comments, `org` directives and optional hexadecimal address anchors. The
//! current HP-67 file contains bank 0 at combined address `$0000` and the
//! populated bank-1 window beginning at combined address `$1400`.
//!
//! Unknown mnemonics, label/address disagreements and unexpected locations are
//! never silently accepted. Analysis reports every issue first; normalization
//! only succeeds when all 5120 physical microinstructions are understood.

use std::{error::Error, fmt};

use super::{
    rom_corpus::{CorpusError, RomCorpus, WORDS_PER_BANK},
    woodstock_asm::{assemble_mnemonic, WoodstockAsmError},
};

pub const HP67_BANK0_WORDS: usize = WORDS_PER_BANK;
pub const HP67_BANK1_START_PC: usize = 0x400;
pub const HP67_BANK1_WORDS: usize = 0x400;
pub const HP67_PHYSICAL_WORDS: usize = HP67_BANK0_WORDS + HP67_BANK1_WORDS;

const BANK_BIT: usize = 0x1000;
const PC_MASK: usize = 0x0fff;
const BANK1_START_COMBINED: usize = BANK_BIT | HP67_BANK1_START_PC;
const BANK1_END_COMBINED: usize = BANK_BIT | (HP67_BANK1_START_PC + HP67_BANK1_WORDS - 1);

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownInstruction {
    pub line: usize,
    pub bank: usize,
    pub pc: usize,
    pub text: String,
    pub error: WoodstockAsmError,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LabelMismatch {
    pub line: usize,
    pub bank: usize,
    pub expected_pc: usize,
    pub label_pc: usize,
    pub label_prefix: char,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverflowInstruction {
    pub line: usize,
    pub ordinal: usize,
    pub combined_address: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hp67ListingReport {
    pub total_lines: usize,
    pub blank_lines: usize,
    pub comment_lines: usize,
    pub org_directives: usize,
    pub instruction_lines: usize,
    pub unknown_instructions: Vec<UnknownInstruction>,
    pub label_mismatches: Vec<LabelMismatch>,
    pub overflow_instructions: usize,
    pub overflow_details: Vec<OverflowInstruction>,
}

impl Hp67ListingReport {
    pub fn is_extractable(&self) -> bool {
        self.instruction_lines == HP67_PHYSICAL_WORDS
            && self.unknown_instructions.is_empty()
            && self.label_mismatches.is_empty()
            && self.overflow_instructions == 0
    }
}

#[derive(Debug)]
pub enum Hp67ListingError {
    NotExtractable(Hp67ListingReport),
    Corpus(CorpusError),
}

impl fmt::Display for Hp67ListingError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotExtractable(report) => write!(
                f,
                "Teenix HP-67 listing is not fully understood: {} instructions, {} unknown, {} label mismatches, {} unsupported locations (expected {} instructions)",
                report.instruction_lines,
                report.unknown_instructions.len(),
                report.label_mismatches.len(),
                report.overflow_instructions,
                HP67_PHYSICAL_WORDS
            ),
            Self::Corpus(error) => error.fmt(f),
        }
    }
}

impl Error for Hp67ListingError {}

impl From<CorpusError> for Hp67ListingError {
    fn from(value: CorpusError) -> Self {
        Self::Corpus(value)
    }
}

/// Analyse the decoded Teenix HP-67 listing without guessing source records.
///
/// Blank lines and `//` comments consume no ROM location. `org $xxxx` changes
/// the combined source address; bit 12 selects bank 1 and the low 12 bits are
/// the logical Woodstock PC. Both observed `Lxxxx:` and `Hxxxx:` anchors are
/// accepted as address labels and checked against the current PC. The prefix is
/// preserved for diagnostics but is not itself used to infer a bank.
pub fn analyze_hp67_listing(text: &str) -> Hp67ListingReport {
    let mut report = Hp67ListingReport::default();
    let mut combined_address = 0usize;

    for (line_index, raw_line) in text.lines().enumerate() {
        report.total_lines += 1;
        let line_number = line_index + 1;
        let trimmed = raw_line.trim();

        if trimmed.is_empty() {
            report.blank_lines += 1;
            continue;
        }
        if trimmed.starts_with("//") {
            report.comment_lines += 1;
            continue;
        }
        if let Some(address) = parse_org_directive(trimmed) {
            report.org_directives += 1;
            combined_address = address;
            continue;
        }

        let ordinal = report.instruction_lines;
        report.instruction_lines += 1;
        let (label, mnemonic) = split_optional_label(trimmed);

        if let Some((bank, pc)) = supported_location(combined_address) {
            if let Some((prefix, label_pc)) = label {
                if label_pc != pc {
                    report.label_mismatches.push(LabelMismatch {
                        line: line_number,
                        bank,
                        expected_pc: pc,
                        label_pc,
                        label_prefix: prefix,
                        text: raw_line.to_owned(),
                    });
                }
            }

            if let Err(error) = assemble_mnemonic(mnemonic) {
                report.unknown_instructions.push(UnknownInstruction {
                    line: line_number,
                    bank,
                    pc,
                    text: mnemonic.to_owned(),
                    error,
                });
            }
        } else {
            report.overflow_instructions += 1;
            report.overflow_details.push(OverflowInstruction {
                line: line_number,
                ordinal,
                combined_address,
                text: raw_line.to_owned(),
            });
        }

        combined_address = combined_address.saturating_add(1);
    }

    report
}

/// Assemble and normalize a completely understood current Teenix HP-67 listing.
pub fn normalize_hp67_listing(text: &str) -> Result<RomCorpus, Hp67ListingError> {
    let report = analyze_hp67_listing(text);
    if !report.is_extractable() {
        return Err(Hp67ListingError::NotExtractable(report));
    }

    let mut corpus = RomCorpus::default();
    let mut combined_address = 0usize;

    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with("//") {
            continue;
        }
        if let Some(address) = parse_org_directive(trimmed) {
            combined_address = address;
            continue;
        }

        let (_, mnemonic) = split_optional_label(trimmed);
        let word = assemble_mnemonic(mnemonic)
            .expect("analysis guaranteed every Teenix HP-67 mnemonic is understood");
        let (bank, pc) = supported_location(combined_address)
            .expect("analysis guaranteed every instruction is in the supported HP-67 ROM topology");
        corpus.set(bank, pc, word)?;
        combined_address += 1;
    }

    debug_assert_eq!(corpus.populated_words(), HP67_PHYSICAL_WORDS);
    Ok(corpus)
}

fn supported_location(combined_address: usize) -> Option<(usize, usize)> {
    if combined_address < HP67_BANK0_WORDS {
        return Some((0, combined_address));
    }
    if (BANK1_START_COMBINED..=BANK1_END_COMBINED).contains(&combined_address) {
        return Some((1, combined_address & PC_MASK));
    }
    None
}

fn parse_org_directive(line: &str) -> Option<usize> {
    let mut parts = line.split_whitespace();
    let directive = parts.next()?;
    if !directive.eq_ignore_ascii_case("org") {
        return None;
    }
    let address = parts.next()?;
    if parts.next().is_some() {
        return None;
    }
    let address = address.strip_prefix('$').unwrap_or(address);
    usize::from_str_radix(address, 16).ok()
}

fn split_optional_label(line: &str) -> (Option<(char, usize)>, &str) {
    let bytes = line.as_bytes();
    if bytes.len() >= 6 && matches!(bytes[0], b'L' | b'l' | b'H' | b'h') && bytes[5] == b':' {
        if let Ok(pc) = usize::from_str_radix(&line[1..5], 16) {
            return (Some((bytes[0] as char, pc)), line[6..].trim());
        }
    }
    (None, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn combined_addresses_map_to_hp67_physical_rom() {
        assert_eq!(supported_location(0x0000), Some((0, 0x000)));
        assert_eq!(supported_location(0x0fff), Some((0, 0xfff)));
        assert_eq!(supported_location(0x1000), None);
        assert_eq!(supported_location(0x13ff), None);
        assert_eq!(supported_location(0x1400), Some((1, 0x400)));
        assert_eq!(supported_location(0x17ff), Some((1, 0x7ff)));
        assert_eq!(supported_location(0x1800), None);
    }

    #[test]
    fn optional_l_and_h_labels_are_hex_address_anchors() {
        assert_eq!(
            split_optional_label("L00F8:   0 -> c[w]"),
            (Some(('L', 0x0f8)), "0 -> c[w]")
        );
        assert_eq!(
            split_optional_label("H0404:\tif no carry go to $4A7"),
            (Some(('H', 0x404)), "if no carry go to $4A7")
        );
        assert_eq!(split_optional_label("return"), (None, "return"));
    }

    #[test]
    fn comments_and_org_do_not_consume_rom_words() {
        let fragment = [
            "L0000: no operation",
            "// **************************",
            "org $1400",
            "// **************************",
            "H0400: no operation",
            "H0401: return",
        ]
        .join("\n");

        let report = analyze_hp67_listing(&fragment);
        assert_eq!(report.instruction_lines, 3);
        assert_eq!(report.comment_lines, 2);
        assert_eq!(report.org_directives, 1);
        assert!(report.unknown_instructions.is_empty());
        assert!(report.label_mismatches.is_empty());
        assert_eq!(report.overflow_instructions, 0);
    }

    #[test]
    fn preview_fragment_has_correct_addresses_and_words() {
        let fragment = [
            "L0000:   no operation",
            "L0001:   if no carry go to $0F8",
            "L0002:   delayed select rom 2",
            " if no carry go to $023",
            "L0004:   1 -> p",
            "L0005:   load constant 3",
            "L0006:   c -> data address",
            "L0007:   data register 13 -> c",
            " return",
        ]
        .join("\n");

        let report = analyze_hp67_listing(&fragment);
        assert_eq!(report.instruction_lines, 9);
        assert!(report.unknown_instructions.is_empty());
        assert!(report.label_mismatches.is_empty());
    }

    #[test]
    fn bad_anchor_is_reported_not_ignored() {
        let report = analyze_hp67_listing("L0001: no operation\n");
        assert_eq!(report.label_mismatches.len(), 1);
        assert_eq!(report.label_mismatches[0].expected_pc, 0);
        assert_eq!(report.label_mismatches[0].label_pc, 1);
    }

    #[test]
    fn unknown_instruction_is_reported_with_location() {
        let report = analyze_hp67_listing("L0000: mystery operation\n");
        assert_eq!(report.unknown_instructions.len(), 1);
        assert_eq!(report.unknown_instructions[0].bank, 0);
        assert_eq!(report.unknown_instructions[0].pc, 0);
    }

    #[test]
    fn unsupported_combined_address_is_preserved_for_analysis() {
        let report = analyze_hp67_listing("org $1000\nno operation\n");
        assert_eq!(report.org_directives, 1);
        assert_eq!(report.overflow_instructions, 1);
        assert_eq!(report.overflow_details[0].ordinal, 0);
        assert_eq!(report.overflow_details[0].combined_address, 0x1000);
        assert_eq!(report.overflow_details[0].text, "no operation");
    }
}
