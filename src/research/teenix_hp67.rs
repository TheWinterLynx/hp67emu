//! Parser and normalizer for the current Teenix HP-67 textual microcode listing.
//!
//! The decoded `cal67.pfl` payload is a source-style listing: one Woodstock
//! microinstruction per non-empty line, with optional `Lxxxx:` hexadecimal
//! address anchors. The HP-67 firmware topology used here is the independently
//! documented 4096-word bank 0 plus the populated 1024-word bank-1 window at
//! logical PC `0x400..0x7ff`, for 5120 physical microinstructions total.
//!
//! Unknown mnemonics, label/address disagreements and unexpected word counts
//! are never silently accepted. The analysis API reports them first; only a
//! completely understood listing can be normalized into a [`RomCorpus`].

use std::{error::Error, fmt};

use super::{
    rom_corpus::{CorpusError, RomCorpus, WORDS_PER_BANK},
    woodstock_asm::{assemble_mnemonic, WoodstockAsmError},
};

pub const HP67_BANK0_WORDS: usize = WORDS_PER_BANK;
pub const HP67_BANK1_START_PC: usize = 0x400;
pub const HP67_BANK1_WORDS: usize = 0x400;
pub const HP67_PHYSICAL_WORDS: usize = HP67_BANK0_WORDS + HP67_BANK1_WORDS;

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
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct OverflowInstruction {
    pub line: usize,
    pub ordinal: usize,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Hp67ListingReport {
    pub total_lines: usize,
    pub blank_lines: usize,
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
                "Teenix HP-67 listing is not yet fully understood: {} instruction lines, {} unknown, {} label mismatches, {} overflow (expected {} instructions)",
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

/// Analyse a decoded Teenix HP-67 listing without accepting unknown text.
///
/// Every non-empty payload line is treated as a candidate microinstruction.
/// This matches the observed `cal67.pfl` source layout and lets optional labels
/// validate the independently known physical ROM ordering. Non-empty records
/// after the 5120 known physical word slots are preserved verbatim in
/// `overflow_details` so source metadata/directives can be identified rather
/// than silently discarded.
pub fn analyze_hp67_listing(text: &str) -> Hp67ListingReport {
    let mut report = Hp67ListingReport::default();
    let mut instruction_index = 0usize;

    for (line_index, raw_line) in text.lines().enumerate() {
        report.total_lines += 1;
        let line_number = line_index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            report.blank_lines += 1;
            continue;
        }

        report.instruction_lines += 1;
        let ordinal = instruction_index;
        let location = location_for_instruction(instruction_index);
        instruction_index += 1;

        let (label, mnemonic) = split_optional_label(trimmed);
        if let Some((bank, pc)) = location {
            if let Some(label_pc) = label {
                if label_pc != pc {
                    report.label_mismatches.push(LabelMismatch {
                        line: line_number,
                        bank,
                        expected_pc: pc,
                        label_pc,
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
                text: raw_line.to_owned(),
            });
        }
    }

    report
}

/// Assemble and normalize a decoded current Teenix HP-67 listing.
///
/// Extraction is deliberately all-or-nothing. The listing must contain exactly
/// 5120 understood microinstructions and all present labels must agree with the
/// expected bank/address topology.
pub fn normalize_hp67_listing(text: &str) -> Result<RomCorpus, Hp67ListingError> {
    let report = analyze_hp67_listing(text);
    if !report.is_extractable() {
        return Err(Hp67ListingError::NotExtractable(report));
    }

    let mut corpus = RomCorpus::default();
    let mut instruction_index = 0usize;
    for raw_line in text.lines() {
        let trimmed = raw_line.trim();
        if trimmed.is_empty() {
            continue;
        }
        let (_, mnemonic) = split_optional_label(trimmed);
        let word = assemble_mnemonic(mnemonic)
            .expect("analysis guaranteed every Teenix HP-67 mnemonic is understood");
        let (bank, pc) = location_for_instruction(instruction_index)
            .expect("analysis guaranteed the exact HP-67 physical word count");
        corpus.set(bank, pc, word)?;
        instruction_index += 1;
    }
    Ok(corpus)
}

fn location_for_instruction(index: usize) -> Option<(usize, usize)> {
    if index < HP67_BANK0_WORDS {
        return Some((0, index));
    }
    let bank1_offset = index - HP67_BANK0_WORDS;
    if bank1_offset < HP67_BANK1_WORDS {
        return Some((1, HP67_BANK1_START_PC + bank1_offset));
    }
    None
}

fn split_optional_label(line: &str) -> (Option<usize>, &str) {
    let bytes = line.as_bytes();
    if bytes.len() >= 6 && matches!(bytes[0], b'L' | b'l') && bytes[5] == b':' {
        if let Ok(pc) = usize::from_str_radix(&line[1..5], 16) {
            return (Some(pc), line[6..].trim());
        }
    }
    (None, line)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_hp67_physical_word_order() {
        assert_eq!(location_for_instruction(0), Some((0, 0x000)));
        assert_eq!(location_for_instruction(0xfff), Some((0, 0xfff)));
        assert_eq!(location_for_instruction(0x1000), Some((1, 0x400)));
        assert_eq!(location_for_instruction(0x13ff), Some((1, 0x7ff)));
        assert_eq!(location_for_instruction(0x1400), None);
    }

    #[test]
    fn optional_labels_are_hex_address_anchors() {
        assert_eq!(
            split_optional_label("L00F8:   0 -> c[w]"),
            (Some(0x0f8), "0 -> c[w]")
        );
        assert_eq!(split_optional_label("return"), (None, "return"));
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
    fn trailing_nonblank_records_are_preserved_for_analysis() {
        let mut listing = String::new();
        for _ in 0..HP67_PHYSICAL_WORDS {
            listing.push_str("no operation\n");
        }
        listing.push_str("MODULE END\n");

        let report = analyze_hp67_listing(&listing);
        assert_eq!(report.overflow_instructions, 1);
        assert_eq!(report.overflow_details.len(), 1);
        assert_eq!(report.overflow_details[0].ordinal, HP67_PHYSICAL_WORDS);
        assert_eq!(report.overflow_details[0].text, "MODULE END");
    }
}
