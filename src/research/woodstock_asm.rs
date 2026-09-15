//! Strict assembler for the textual Woodstock mnemonics used by Teenix listings.
//!
//! This module is research tooling, not the production ACT implementation. It
//! converts documented Woodstock instruction mnemonics into 10-bit words so a
//! current Teenix HP-67 `.pfl` listing can be normalized and compared against
//! independent ROM corpora. Unknown text is rejected rather than guessed.

use std::{error::Error, fmt};

const WORD_MASK: u16 = 0x03ff;
const PC_MASK: u16 = 0x0fff;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WoodstockAsmError {
    EmptyMnemonic,
    UnknownMnemonic(String),
    InvalidOperand { mnemonic: String, operand: String },
    OperandOutOfRange { mnemonic: String, value: u16, maximum: u16 },
    MultipleFields(String),
}

impl fmt::Display for WoodstockAsmError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMnemonic => write!(f, "empty Woodstock mnemonic"),
            Self::UnknownMnemonic(text) => write!(f, "unknown Woodstock mnemonic: {text}"),
            Self::InvalidOperand { mnemonic, operand } => {
                write!(f, "invalid operand {operand:?} in Woodstock mnemonic {mnemonic:?}")
            }
            Self::OperandOutOfRange {
                mnemonic,
                value,
                maximum,
            } => write!(
                f,
                "operand {value} in Woodstock mnemonic {mnemonic:?} exceeds maximum {maximum}"
            ),
            Self::MultipleFields(text) => {
                write!(f, "multiple Woodstock arithmetic fields in mnemonic: {text}")
            }
        }
    }
}

impl Error for WoodstockAsmError {}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Field {
    P,
    Wp,
    Xs,
    X,
    S,
    M,
    W,
    Ms,
}

impl Field {
    const ALL: [Self; 8] = [
        Self::P,
        Self::Wp,
        Self::Xs,
        Self::X,
        Self::S,
        Self::M,
        Self::W,
        Self::Ms,
    ];

    const fn token(self) -> &'static str {
        match self {
            Self::P => "[p]",
            Self::Wp => "[wp]",
            Self::Xs => "[xs]",
            Self::X => "[x]",
            Self::S => "[s]",
            Self::M => "[m]",
            Self::W => "[w]",
            Self::Ms => "[ms]",
        }
    }

    const fn encoded(self) -> u16 {
        match self {
            Self::P => 0,
            Self::Wp => 1,
            Self::Xs => 2,
            Self::X => 3,
            Self::S => 4,
            Self::M => 5,
            Self::W => 6,
            Self::Ms => 7,
        }
    }
}

pub fn assemble_mnemonic(input: &str) -> Result<u16, WoodstockAsmError> {
    let mnemonic = normalize_spaces(input);
    if mnemonic.is_empty() {
        return Err(WoodstockAsmError::EmptyMnemonic);
    }
    if let Some(word) = assemble_control_flow(&mnemonic)? {
        return Ok(word);
    }
    if let Some(word) = assemble_parameterized_special(&mnemonic)? {
        return Ok(word);
    }
    if let Some(word) = assemble_arithmetic(&mnemonic)? {
        return Ok(word);
    }
    if let Some(word) = assemble_fixed_special(&mnemonic) {
        return Ok(word);
    }
    Err(WoodstockAsmError::UnknownMnemonic(input.trim().to_owned()))
}

fn normalize_spaces(input: &str) -> String {
    input
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .to_ascii_lowercase()
}

fn assemble_control_flow(mnemonic: &str) -> Result<Option<u16>, WoodstockAsmError> {
    if let Some(target) = mnemonic.strip_prefix("jsb ") {
        let target = parse_hex_target(mnemonic, target, PC_MASK)?;
        return Ok(Some((((target & 0x00ff) << 2) | 0x01) & WORD_MASK));
    }
    if let Some(target) = mnemonic.strip_prefix("if no carry go to ") {
        let target = parse_hex_target(mnemonic, target, PC_MASK)?;
        return Ok(Some((((target & 0x00ff) << 2) | 0x03) & WORD_MASK));
    }
    if let Some(target) = mnemonic.strip_prefix("go to ") {
        let target = parse_hex_target(mnemonic, target, PC_MASK)?;
        return Ok(Some((((target & 0x00ff) << 2) | 0x03) & WORD_MASK));
    }
    if let Some(target) = mnemonic.strip_prefix("then go to ") {
        // After an IF-class instruction Woodstock suppresses normal SYNC and
        // consumes the next complete 10-bit ROM word as the destination. The
        // upper two logical-PC bits therefore come from the current 1 Kiword
        // group while the literal 12-bit listing address contributes only its
        // low ten bits. Teenix prints the full logical address (e.g. $61C), so
        // accept a 12-bit target and emit the hardware-visible 10-bit word.
        let target = parse_hex_target(mnemonic, target, PC_MASK)?;
        return Ok(Some(target & WORD_MASK));
    }
    Ok(None)
}

fn assemble_parameterized_special(mnemonic: &str) -> Result<Option<u16>, WoodstockAsmError> {
    if let Some(value) = mnemonic.strip_prefix("load constant ") {
        return family_operand(mnemonic, value, 15, 0o30).map(Some);
    }
    if let Some(value) = mnemonic.strip_prefix("select rom ") {
        return family_operand(mnemonic, value, 15, 0o40).map(Some);
    }
    if let Some(value) = mnemonic.strip_prefix("delayed select rom ") {
        return family_operand(mnemonic, value, 15, 0o64).map(Some);
    }
    if let Some(value) = mnemonic
        .strip_prefix("c -> data register ")
        .or_else(|| mnemonic.strip_prefix("c -> register "))
    {
        return family_operand(mnemonic, value, 15, 0o50).map(Some);
    }
    if let Some(value) = mnemonic
        .strip_prefix("data register ")
        .and_then(|rest| rest.strip_suffix(" -> c"))
    {
        return family_operand(mnemonic, value, 15, 0o70).map(Some);
    }
    if let Some(value) = mnemonic.strip_prefix("crc ") {
        let opcode = parse_u16_radix(mnemonic, value, 8)?;
        if opcode > WORD_MASK {
            return Err(WoodstockAsmError::OperandOutOfRange {
                mnemonic: mnemonic.to_owned(),
                value: opcode,
                maximum: WORD_MASK,
            });
        }
        return Ok(Some(opcode));
    }
    if let Some(value) = mnemonic.strip_prefix("1 -> s").filter(|value| !value.is_empty()) {
        return family_operand(mnemonic, value, 15, 0o04).map(Some);
    }
    if let Some(value) = mnemonic.strip_prefix("0 -> s").filter(|value| !value.is_empty()) {
        return family_operand(mnemonic, value, 15, 0o14).map(Some);
    }
    if let Some(value) = mnemonic
        .strip_prefix("if s")
        .and_then(|rest| rest.strip_suffix(" = 1"))
    {
        return family_operand(mnemonic, value, 15, 0o24).map(Some);
    }
    if let Some(value) = mnemonic
        .strip_prefix("if s")
        .and_then(|rest| rest.strip_suffix(" = 0"))
    {
        return family_operand(mnemonic, value, 15, 0o34).map(Some);
    }
    if let Some(value) = mnemonic.strip_suffix(" -> p") {
        if let Ok(target) = value.parse::<u16>() {
            if target <= 13 {
                let operand = p_set_operand(target as u8);
                return Ok(Some((u16::from(operand) << 6) | 0o74));
            }
        }
    }
    if let Some(value) = mnemonic
        .strip_prefix("if p = ")
        .or_else(|| mnemonic.strip_prefix("if p == "))
    {
        let target = parse_u16_radix(mnemonic, value, 10)?;
        if target > 13 {
            return Err(WoodstockAsmError::OperandOutOfRange {
                mnemonic: mnemonic.to_owned(),
                value: target,
                maximum: 13,
            });
        }
        let operand = p_test_operand(target as u8);
        return Ok(Some((u16::from(operand) << 6) | 0o44));
    }
    if let Some(value) = mnemonic
        .strip_prefix("if p # ")
        .or_else(|| mnemonic.strip_prefix("if p != "))
    {
        let target = parse_u16_radix(mnemonic, value, 10)?;
        if target > 13 {
            return Err(WoodstockAsmError::OperandOutOfRange {
                mnemonic: mnemonic.to_owned(),
                value: target,
                maximum: 13,
            });
        }
        let operand = p_test_operand(target as u8);
        return Ok(Some((u16::from(operand) << 6) | 0o54));
    }
    Ok(None)
}

fn assemble_arithmetic(mnemonic: &str) -> Result<Option<u16>, WoodstockAsmError> {
    let mut found = None;
    let mut generic = mnemonic.to_owned();
    for field in Field::ALL {
        if generic.contains(field.token()) {
            if found.is_some() {
                return Err(WoodstockAsmError::MultipleFields(mnemonic.to_owned()));
            }
            found = Some(field);
            generic = generic.replace(field.token(), "[fs]");
        }
    }
    let Some(field) = found else {
        return Ok(None);
    };

    let operation: u8 = match generic.as_str() {
        "0 -> a[fs]" => 0x00,
        "0 -> b[fs]" => 0x01,
        "a exchange b[fs]" => 0x02,
        "a -> b[fs]" => 0x03,
        "a exchange c[fs]" => 0x04,
        "c -> a[fs]" => 0x05,
        "b -> c[fs]" => 0x06,
        "b exchange c[fs]" => 0x07,
        "0 -> c[fs]" => 0x08,
        "a + b -> a[fs]" => 0x09,
        "a + c -> a[fs]" => 0x0a,
        "c + c -> c[fs]" => 0x0b,
        "a + c -> c[fs]" => 0x0c,
        "a + 1 -> a[fs]" => 0x0d,
        "shift left a[fs]" => 0x0e,
        "c + 1 -> c[fs]" => 0x0f,
        "a - b -> a[fs]" => 0x10,
        "a - c -> c[fs]" => 0x11,
        "a - 1 -> a[fs]" => 0x12,
        "c - 1 -> c[fs]" => 0x13,
        "0 - c -> c[fs]" => 0x14,
        "0 - c - 1 -> c[fs]" => 0x15,
        "if b[fs] = 0" => 0x16,
        "if c[fs] = 0" => 0x17,
        "if a >= c[fs]" => 0x18,
        "if a >= b[fs]" => 0x19,
        "if a[fs] # 0" | "if a[fs] != 0" => 0x1a,
        "if c[fs] # 0" | "if c[fs] != 0" => 0x1b,
        "a - c -> a[fs]" => 0x1c,
        "shift right a[fs]" => 0x1d,
        "shift right b[fs]" => 0x1e,
        "shift right c[fs]" => 0x1f,
        _ => return Ok(None),
    };

    Ok(Some((u16::from(operation) << 5) | (field.encoded() << 2) | 0x02))
}

fn assemble_fixed_special(mnemonic: &str) -> Option<u16> {
    Some(match mnemonic {
        "no operation" => 0o0000,
        "data -> c" => 0o0070,
        "clear registers" => 0o0010,
        "clear status" => 0o0110,
        "display toggle" => 0o0210,
        "display off" => 0o0310,
        "m1 exchange c" => 0o0410,
        "m1 -> c" => 0o0510,
        "m2 exchange c" => 0o0610,
        "m2 -> c" => 0o0710,
        "stack -> a" => 0o1010,
        "down rotate" => 0o1110,
        "y -> a" => 0o1210,
        "c -> stack" => 0o1310,
        "decimal" => 0o1410,
        "f -> a" => 0o1610,
        "f exchange a" => 0o1710,
        "keys -> rom address" => 0o0020,
        "keys -> a" => 0o0120,
        "a -> rom address" => 0o0220,
        "display reset kmf" | "display reset twf" | "reset twf" => 0o0320,
        "binary" => 0o0420,
        "rotate a left" => 0o0520,
        "p - 1 -> p" => 0o0620,
        "p + 1 -> p" => 0o0720,
        "return" => 0o1020,
        "bank switch" => 0o1060,
        "c -> data address" => 0o1160,
        "clear data registers" => 0o1260,
        "c -> data" => 0o1360,
        "rom self test" | "rom checksum" => 0o1460,
        "hi i'm woodstock" => 0o1760,
        _ => return None,
    })
}

fn family_operand(
    mnemonic: &str,
    text: &str,
    maximum: u16,
    low_bits: u16,
) -> Result<u16, WoodstockAsmError> {
    let value = parse_u16_radix(mnemonic, text, 10)?;
    if value > maximum {
        return Err(WoodstockAsmError::OperandOutOfRange {
            mnemonic: mnemonic.to_owned(),
            value,
            maximum,
        });
    }
    Ok((value << 6) | low_bits)
}

fn parse_hex_target(
    mnemonic: &str,
    text: &str,
    maximum: u16,
) -> Result<u16, WoodstockAsmError> {
    let value = text.strip_prefix('$').unwrap_or(text);
    let value = parse_u16_radix(mnemonic, value, 16)?;
    if value > maximum {
        return Err(WoodstockAsmError::OperandOutOfRange {
            mnemonic: mnemonic.to_owned(),
            value,
            maximum,
        });
    }
    Ok(value)
}

fn parse_u16_radix(
    mnemonic: &str,
    text: &str,
    radix: u32,
) -> Result<u16, WoodstockAsmError> {
    u16::from_str_radix(text.trim(), radix).map_err(|_| WoodstockAsmError::InvalidOperand {
        mnemonic: mnemonic.to_owned(),
        operand: text.trim().to_owned(),
    })
}

fn p_set_operand(target: u8) -> u8 {
    const OPERANDS: [u8; 14] = [12, 8, 5, 9, 1, 14, 11, 2, 3, 13, 6, 4, 7, 10];
    OPERANDS[usize::from(target)]
}

fn p_test_operand(target: u8) -> u8 {
    const OPERANDS: [u8; 14] = [11, 5, 3, 7, 0, 10, 6, 14, 1, 4, 13, 12, 2, 9];
    OPERANDS[usize::from(target)]
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn assembles_physical_startup_words() {
        assert_eq!(assemble_mnemonic("no operation").unwrap(), 0x000);
        assert_eq!(assemble_mnemonic("if no carry go to $0F8").unwrap(), 0x3e3);
        assert_eq!(assemble_mnemonic("0 -> c[w]").unwrap(), 0x11a);
    }

    #[test]
    fn assembles_previewed_teenix_hp67_instructions() {
        assert_eq!(assemble_mnemonic("delayed select rom 2").unwrap(), 0o0264);
        assert_eq!(assemble_mnemonic("1 -> p").unwrap(), 0o1074);
        assert_eq!(assemble_mnemonic("load constant 3").unwrap(), 0o0330);
        assert_eq!(assemble_mnemonic("c -> data address").unwrap(), 0o1160);
        assert_eq!(assemble_mnemonic("data register 13 -> c").unwrap(), 0o1570);
        assert_eq!(assemble_mnemonic("return").unwrap(), 0o1020);
        assert_eq!(assemble_mnemonic("c -> a[x]").unwrap(), 0o0256);
        assert_eq!(assemble_mnemonic("then go to $023").unwrap(), 0x023);
        assert_eq!(assemble_mnemonic("if s11 = 0").unwrap(), 0o1334);
        assert_eq!(assemble_mnemonic("CRC 1500").unwrap(), 0o1500);
        assert_eq!(assemble_mnemonic("hi i'm woodstock").unwrap(), 0o1760);
    }

    #[test]
    fn then_goto_uses_the_low_ten_bits_of_the_full_logical_target() {
        assert_eq!(assemble_mnemonic("then go to $400").unwrap(), 0x000);
        assert_eq!(assemble_mnemonic("then go to $61C").unwrap(), 0x21c);
        assert_eq!(assemble_mnemonic("then go to $FFF").unwrap(), 0x3ff);
    }

    #[test]
    fn accepts_documented_twf_display_reset_spelling() {
        assert_eq!(assemble_mnemonic("display reset twf").unwrap(), 0o0320);
        assert_eq!(assemble_mnemonic("reset twf").unwrap(), 0o0320);
    }

    #[test]
    fn stack_specials_match_documented_encodings() {
        assert_eq!(assemble_mnemonic("down rotate").unwrap(), 0o1110);
        assert_eq!(assemble_mnemonic("c -> stack").unwrap(), 0o1310);
    }

    #[test]
    fn covers_all_arithmetic_fields() {
        let expected_fields = [0u16, 1, 2, 3, 4, 5, 6, 7];
        let names = ["p", "wp", "xs", "x", "s", "m", "w", "ms"];
        for (encoded, name) in expected_fields.into_iter().zip(names) {
            let text = format!("0 -> a[{name}]");
            assert_eq!(assemble_mnemonic(&text).unwrap(), (encoded << 2) | 0x02);
        }
    }

    #[test]
    fn status_and_p_families_match_documented_encodings() {
        assert_eq!(assemble_mnemonic("1 -> s9").unwrap(), (9 << 6) | 0o04);
        assert_eq!(assemble_mnemonic("0 -> s12").unwrap(), (12 << 6) | 0o14);
        assert_eq!(assemble_mnemonic("if s3 = 1").unwrap(), (3 << 6) | 0o24);
        assert_eq!(assemble_mnemonic("if s3 = 0").unwrap(), (3 << 6) | 0o34);
        assert_eq!(assemble_mnemonic("0 -> p").unwrap(), 0o1474);
        assert_eq!(assemble_mnemonic("if p = 0").unwrap(), 0o1344);
        assert_eq!(assemble_mnemonic("if p # 0").unwrap(), 0o1354);
    }

    #[test]
    fn rejects_unknown_text() {
        assert!(matches!(
            assemble_mnemonic("invented opcode"),
            Err(WoodstockAsmError::UnknownMnemonic(_))
        ));
    }
}
