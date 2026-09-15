//! Decoder for Teenix's lightweight `NeWe` file container.
//!
//! Recent Teenix documentation describes calculator card files as text whose
//! bytes are XORed with `0x55`, with the decoded first line `NeWe` and the
//! decoded second line holding the byte count of the following payload. The
//! current HP-67 `.pfl` files exhibit the same signature: their first four raw
//! bytes XOR to `NeWe`, and their decoded count matches the remaining payload
//! length exactly. This module only decodes that outer container; it does not
//! assume that every payload has the same internal grammar.

use std::{error::Error, fmt, str};

pub const XOR_KEY: u8 = 0x55;
pub const MAGIC: &str = "NeWe";

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TeenixContainerError {
    MissingMagicLine,
    WrongMagic(String),
    MissingLengthLine,
    InvalidLength(String),
    LengthMismatch { declared: usize, actual: usize },
    PayloadNotUtf8,
}

impl fmt::Display for TeenixContainerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::MissingMagicLine => write!(f, "missing Teenix container magic line"),
            Self::WrongMagic(actual) => {
                write!(f, "decoded Teenix magic is {actual:?}; expected {MAGIC:?}")
            }
            Self::MissingLengthLine => write!(f, "missing Teenix container payload-length line"),
            Self::InvalidLength(value) => {
                write!(f, "invalid Teenix payload length {value:?}")
            }
            Self::LengthMismatch { declared, actual } => write!(
                f,
                "Teenix payload length mismatch: header declares {declared} bytes, file contains {actual}"
            ),
            Self::PayloadNotUtf8 => write!(f, "decoded Teenix payload is not valid UTF-8"),
        }
    }
}

impl Error for TeenixContainerError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TeenixContainer {
    pub declared_payload_len: usize,
    payload: Vec<u8>,
}

impl TeenixContainer {
    pub fn payload(&self) -> &[u8] {
        &self.payload
    }

    pub fn payload_text(&self) -> Result<&str, TeenixContainerError> {
        str::from_utf8(&self.payload).map_err(|_| TeenixContainerError::PayloadNotUtf8)
    }
}

/// Returns true when the first four decoded bytes are exactly `NeWe`.
///
/// This is intentionally only a signature probe. Full validation, including
/// the declared payload length, is performed by [`decode_container`].
pub fn has_newe_signature(bytes: &[u8]) -> bool {
    bytes.len() >= MAGIC.len()
        && bytes[..MAGIC.len()]
            .iter()
            .copied()
            .map(|byte| byte ^ XOR_KEY)
            .eq(MAGIC.bytes())
}

/// Decode and validate a Teenix `NeWe` container.
///
/// The entire file is XOR-decoded with `0x55`. Line endings may be CR, LF or
/// CRLF. The first decoded line must be `NeWe`; the second must be a decimal
/// byte count that exactly equals the remaining payload length.
pub fn decode_container(bytes: &[u8]) -> Result<TeenixContainer, TeenixContainerError> {
    let decoded = bytes.iter().map(|byte| byte ^ XOR_KEY).collect::<Vec<_>>();

    let (magic, after_magic) =
        split_line(&decoded, 0).ok_or(TeenixContainerError::MissingMagicLine)?;
    let magic = str::from_utf8(magic)
        .map_err(|_| TeenixContainerError::WrongMagic("<non-UTF8>".to_owned()))?;
    if magic != MAGIC {
        return Err(TeenixContainerError::WrongMagic(magic.to_owned()));
    }

    let (length_text, payload_start) =
        split_line(&decoded, after_magic).ok_or(TeenixContainerError::MissingLengthLine)?;
    let length_text = str::from_utf8(length_text)
        .map_err(|_| TeenixContainerError::InvalidLength("<non-UTF8>".to_owned()))?;
    let declared_payload_len = length_text
        .parse::<usize>()
        .map_err(|_| TeenixContainerError::InvalidLength(length_text.to_owned()))?;

    let payload = decoded[payload_start..].to_vec();
    if payload.len() != declared_payload_len {
        return Err(TeenixContainerError::LengthMismatch {
            declared: declared_payload_len,
            actual: payload.len(),
        });
    }

    Ok(TeenixContainer {
        declared_payload_len,
        payload,
    })
}

fn split_line(bytes: &[u8], start: usize) -> Option<(&[u8], usize)> {
    if start >= bytes.len() {
        return None;
    }

    let relative_end = bytes[start..]
        .iter()
        .position(|byte| matches!(*byte, b'\r' | b'\n'))?;
    let end = start + relative_end;
    let mut next = end + 1;
    if bytes[end] == b'\r' && bytes.get(next) == Some(&b'\n') {
        next += 1;
    }
    Some((&bytes[start..end], next))
}

#[cfg(test)]
mod tests {
    use super::*;

    fn encode(decoded: &[u8]) -> Vec<u8> {
        decoded.iter().map(|byte| byte ^ XOR_KEY).collect()
    }

    #[test]
    fn signature_matches_observed_teenix_prefix() {
        let observed_prefix = [0x1b, 0x30, 0x02, 0x30];
        assert!(has_newe_signature(&observed_prefix));
    }

    #[test]
    fn decodes_cr_container_and_validates_count() {
        let payload = b"L0000:\tno operation\r\n";
        let decoded = format!("NeWe\r{}\r", payload.len()).into_bytes();
        let mut file = decoded;
        file.extend_from_slice(payload);
        let file = encode(&file);

        let container = decode_container(&file).expect("valid container");
        assert_eq!(container.declared_payload_len, payload.len());
        assert_eq!(container.payload(), payload);
        assert_eq!(
            container.payload_text().unwrap(),
            "L0000:\tno operation\r\n"
        );
    }

    #[test]
    fn accepts_crlf_header_lines() {
        let payload = b"abc\r\n";
        let mut decoded = format!("NeWe\r\n{}\r\n", payload.len()).into_bytes();
        decoded.extend_from_slice(payload);

        let container = decode_container(&encode(&decoded)).expect("valid CRLF container");
        assert_eq!(container.payload(), payload);
    }

    #[test]
    fn rejects_declared_length_mismatch() {
        let decoded = encode(b"NeWe\r99\rabc");
        assert_eq!(
            decode_container(&decoded),
            Err(TeenixContainerError::LengthMismatch {
                declared: 99,
                actual: 3,
            })
        );
    }

    #[test]
    fn rejects_wrong_magic() {
        let decoded = encode(b"Nope\r0\r");
        assert_eq!(
            decode_container(&decoded),
            Err(TeenixContainerError::WrongMagic("Nope".to_owned()))
        );
    }
}
