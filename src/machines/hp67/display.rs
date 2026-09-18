//! Structural HP-67 ROM0/anode and cathode-scan display model.
//!
//! Direct HP-67 logic-analyser evidence places the eight-bit ROM0 display code
//! on IS at b0..b7, LSB first. ROM0 (1818-0268) decodes that byte to LED anode
//! segments and issues STR; the 1820-1749 cathode driver advances the display
//! scan and is reset by RCD from the ACT. This module locks those observed
//! coarse bit/scan facts without inventing final PHI-relative edges, pulse
//! widths, transistor delays or LED current.

use crate::emulation::LogicLevel;

use super::timing::{display_data_serial_bit, DISPLAY_DATA_BITS};

pub const HP67_DISPLAY_SCAN_SLOTS: u8 = 15;
pub const HP67_SHARED_SIGN_SLOT: u8 = 3;
pub const HP67_ROM0_BLANK_CODE: u8 = 0x20;

const COMPLETE_DISPLAY_MASK: u8 = ((1u16 << DISPLAY_DATA_BITS) - 1) as u8;

/// Anode segment mask emitted by the ROM0 display decoder.
///
/// Bits 0..7 are A, B, C, D, E, F, G and decimal point respectively.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67SegmentMask(u8);

impl Hp67SegmentMask {
    pub const A: Self = Self(1 << 0);
    pub const B: Self = Self(1 << 1);
    pub const C: Self = Self(1 << 2);
    pub const D: Self = Self(1 << 3);
    pub const E: Self = Self(1 << 4);
    pub const F: Self = Self(1 << 5);
    pub const G: Self = Self(1 << 6);
    pub const DP: Self = Self(1 << 7);
    pub const BLANK: Self = Self(0);

    pub const fn from_bits(bits: u8) -> Self {
        Self(bits)
    }

    pub const fn bits(self) -> u8 {
        self.0
    }

    pub const fn contains(self, segment: Self) -> bool {
        (self.0 & segment.0) == segment.0
    }
}

/// Source-backed order of the fifteen ROM0/STR display scan slots.
///
/// The hardware has fourteen cathode-driver outputs because the two signs share
/// one cathode position. Slot 15 repeats the exponent-units data observed at
/// slot 1 and is retained as a scan slot rather than invented as a 15th cathode.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67DisplayRole {
    ExponentUnits,
    ExponentTens,
    SharedSigns,
    MantissaDigit(u8),
    ExponentUnitsDuplicate,
}

pub const fn display_role_for_scan_slot(scan_slot: u8) -> Option<Hp67DisplayRole> {
    match scan_slot {
        1 => Some(Hp67DisplayRole::ExponentUnits),
        2 => Some(Hp67DisplayRole::ExponentTens),
        3 => Some(Hp67DisplayRole::SharedSigns),
        4..=14 => Some(Hp67DisplayRole::MantissaDigit(15 - scan_slot)),
        15 => Some(Hp67DisplayRole::ExponentUnitsDuplicate),
        _ => None,
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Rom0DisplayError {
    FloatingIsa { word_bit: u8 },
    IsaContention { word_bit: u8 },
    IncompleteDisplayByte { received_mask: u8 },
    InvalidScanSlot(u8),
    UnknownDisplayCode { scan_slot: u8, code: u8 },
}

/// Falling STR event produced by ROM0 after one complete b0..b7 display byte.
///
/// The event carries the ACT-owned display slot only as an annotation so the
/// downstream cathode model can verify phase. Its existence represents STR;
/// exact PHI-relative edge placement and pulse width remain intentionally open.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Rom0StrEvent {
    pub scan_slot: u8,
}

/// Structural failures in the downstream 1820-1749 scan state.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CathodeScanError {
    PhaseMismatch { cathode_slot: u8, str_slot: u8 },
    UnexpectedRcd { scan_slot: u8 },
    MissingRcdAtFinalSlot,
}

fn sample_isa(level: LogicLevel, word_bit: u8) -> Result<bool, Rom0DisplayError> {
    match level {
        LogicLevel::Low => Ok(false),
        LogicLevel::High => Ok(true),
        LogicLevel::Floating => Err(Rom0DisplayError::FloatingIsa { word_bit }),
        LogicLevel::Contention => Err(Rom0DisplayError::IsaContention { word_bit }),
    }
}

/// ROM0-side receiver for the eight display bits observed at IS b0..b7.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Rom0DisplayEndpoint {
    received_byte: u8,
    received_mask: u8,
}

impl Rom0DisplayEndpoint {
    pub fn begin_word(&mut self) {
        self.received_byte = 0;
        self.received_mask = 0;
    }

    pub fn sample_for_bit(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
    ) -> Result<(), Rom0DisplayError> {
        let Some(serial_bit) = display_data_serial_bit(word_bit) else {
            return Ok(());
        };

        if sample_isa(level, word_bit)? {
            self.received_byte |= 1u8 << serial_bit;
        } else {
            self.received_byte &= !(1u8 << serial_bit);
        }
        self.received_mask |= 1u8 << serial_bit;
        Ok(())
    }

    pub const fn display_byte(&self) -> Result<u8, Rom0DisplayError> {
        if self.received_mask != COMPLETE_DISPLAY_MASK {
            return Err(Rom0DisplayError::IncompleteDisplayByte {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_byte)
    }

    /// Emit the structural falling-STR event for a completed display byte.
    pub fn str_falling_event(&self, scan_slot: u8) -> Result<Rom0StrEvent, Rom0DisplayError> {
        self.display_byte()?;
        if display_role_for_scan_slot(scan_slot).is_none() {
            return Err(Rom0DisplayError::InvalidScanSlot(scan_slot));
        }
        Ok(Rom0StrEvent { scan_slot })
    }

    pub fn decoded_anodes(&self, scan_slot: u8) -> Result<Hp67SegmentMask, Rom0DisplayError> {
        decode_rom0_display_byte(scan_slot, self.display_byte()?)
    }
}

/// Decode the HP-67 ROM0 display byte to anode segments.
///
/// The code assignments follow the direct HP-67 ROM0 logic-analyser table. The
/// seven-segment shapes are corroborated by the reviewed HP-67 character table;
/// where that semantic table disagrees on the `o`/`r` code numbers, the direct
/// HP-67 capture wins here. Unknown codes remain hard failures.
pub fn decode_rom0_display_byte(
    scan_slot: u8,
    code: u8,
) -> Result<Hp67SegmentMask, Rom0DisplayError> {
    let Some(role) = display_role_for_scan_slot(scan_slot) else {
        return Err(Rom0DisplayError::InvalidScanSlot(scan_slot));
    };

    if role == Hp67DisplayRole::SharedSigns {
        let mantissa_negative = code & 0x01 == 0;
        let exponent_negative = code & 0x02 != 0;
        let mut segments = 0u8;
        if mantissa_negative {
            segments |= Hp67SegmentMask::E.bits();
        }
        if exponent_negative {
            segments |= Hp67SegmentMask::G.bits();
        }
        return Ok(Hp67SegmentMask::from_bits(segments));
    }

    let segments = match code {
        0x00 => 0x3f, // 0: A B C D E F
        0x01 => 0x06, // 1: B C
        0x02 => 0x5b, // 2: A B D E G
        0x03 => 0x4f, // 3: A B C D G
        0x04 => 0x66, // 4: B C F G
        0x05 => 0x6d, // 5: A C D F G
        0x06 => 0x7d, // 6: A C D E F G
        0x07 => 0x07, // 7: A B C
        0x08 => 0x7f, // 8: A B C D E F G
        0x09 => 0x6f, // 9: A B C D F G
        0x0a => 0x5c, // o: C D E G
        0x0b => 0x39, // C: A D E F
        0x0c => 0x50, // r: E G
        0x0d => 0x5e, // d: B C D E G
        0x0e => 0x79, // E: A D E F G
        0x0f => 0x00,
        0x20..=0x2f => 0x00,
        0x30..=0x3f => Hp67SegmentMask::DP.bits(),
        0x40..=0x4f => 0x00,
        _ => return Err(Rom0DisplayError::UnknownDisplayCode { scan_slot, code }),
    };
    Ok(Hp67SegmentMask::from_bits(segments))
}

/// Structural 1820-1749 cathode scan state.
///
/// ROM0 supplies STR and ACT supplies RCD. `apply_control_edges()` consumes those
/// source-owned events and validates that the downstream cathode phase agrees;
/// the cathode never feeds a slot selection back into the ACT. Final RCD/STR
/// overlap ordering and PHI-relative propagation remain responsibilities of the
/// future electrical scheduler.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct CathodeDriver1820_1749 {
    scan_slot: u8,
}

impl Default for CathodeDriver1820_1749 {
    fn default() -> Self {
        Self { scan_slot: 1 }
    }
}

impl CathodeDriver1820_1749 {
    pub const fn scan_slot(&self) -> u8 {
        self.scan_slot
    }

    pub fn scan_role(&self) -> Hp67DisplayRole {
        display_role_for_scan_slot(self.scan_slot)
            .expect("cathode structural scan slot is always in 1..=15")
    }

    /// Consume one ROM0 STR event and the coarse ACT RCD state for the same word.
    ///
    /// At slot 15 the real captures show STR/RCD overlap; this method records
    /// their shared word boundary without inventing which electrical edge wins
    /// first inside that overlap.
    pub fn apply_control_edges(
        &mut self,
        str_event: Rom0StrEvent,
        rcd_falling: bool,
    ) -> Result<Hp67DisplayRole, CathodeScanError> {
        if str_event.scan_slot != self.scan_slot {
            return Err(CathodeScanError::PhaseMismatch {
                cathode_slot: self.scan_slot,
                str_slot: str_event.scan_slot,
            });
        }

        let final_slot = self.scan_slot == HP67_DISPLAY_SCAN_SLOTS;
        if rcd_falling && !final_slot {
            return Err(CathodeScanError::UnexpectedRcd {
                scan_slot: self.scan_slot,
            });
        }
        if final_slot && !rcd_falling {
            return Err(CathodeScanError::MissingRcdAtFinalSlot);
        }

        let role = self.scan_role();
        self.scan_slot = if rcd_falling { 1 } else { self.scan_slot + 1 };
        Ok(role)
    }

    pub fn rcd_falling_edge(&mut self) {
        self.scan_slot = 1;
    }

    pub fn str_falling_edge(&mut self) -> Hp67DisplayRole {
        let role = self.scan_role();
        self.scan_slot = if self.scan_slot == HP67_DISPLAY_SCAN_SLOTS {
            1
        } else {
            self.scan_slot + 1
        };
        role
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_byte(endpoint: &mut Rom0DisplayEndpoint, value: u8) {
        endpoint.begin_word();
        for bit in 0..DISPLAY_DATA_BITS {
            let level = if value & (1u8 << bit) != 0 {
                LogicLevel::High
            } else {
                LogicLevel::Low
            };
            endpoint
                .sample_for_bit(bit, level)
                .expect("display bit must sample");
        }
    }

    #[test]
    fn rom0_reconstructs_display_byte_lsb_first_from_b0_through_b7() {
        let mut endpoint = Rom0DisplayEndpoint::default();
        sample_byte(&mut endpoint, 0x20);
        assert_eq!(endpoint.display_byte(), Ok(0x20));
        assert_eq!(endpoint.decoded_anodes(1), Ok(Hp67SegmentMask::BLANK));
        assert_eq!(
            endpoint.str_falling_event(1),
            Ok(Rom0StrEvent { scan_slot: 1 })
        );
    }

    #[test]
    fn direct_hp67_digit_and_modifier_classes_decode_to_segments() {
        assert_eq!(decode_rom0_display_byte(1, 0x00).unwrap().bits(), 0x3f);
        assert_eq!(decode_rom0_display_byte(4, 0x09).unwrap().bits(), 0x6f);
        assert_eq!(
            decode_rom0_display_byte(8, 0x25),
            Ok(Hp67SegmentMask::BLANK)
        );
        assert_eq!(decode_rom0_display_byte(8, 0x30), Ok(Hp67SegmentMask::DP));
        assert_eq!(decode_rom0_display_byte(5, 0x33), Ok(Hp67SegmentMask::DP));
        assert_eq!(
            decode_rom0_display_byte(14, 0x4f),
            Ok(Hp67SegmentMask::BLANK)
        );
    }

    #[test]
    fn direct_hp67_o_and_r_code_assignment_is_preserved() {
        assert_eq!(decode_rom0_display_byte(4, 0x0a).unwrap().bits(), 0x5c);
        assert_eq!(decode_rom0_display_byte(4, 0x0c).unwrap().bits(), 0x50);
    }

    #[test]
    fn shared_sign_slot_uses_only_first_two_display_bits() {
        let both_negative = decode_rom0_display_byte(3, 0b0000_0010).unwrap();
        assert!(both_negative.contains(Hp67SegmentMask::E));
        assert!(both_negative.contains(Hp67SegmentMask::G));

        let both_positive = decode_rom0_display_byte(3, 0b1111_1101).unwrap();
        assert!(!both_positive.contains(Hp67SegmentMask::E));
        assert!(!both_positive.contains(Hp67SegmentMask::G));

        let exponent_negative = decode_rom0_display_byte(3, 0b0000_0011).unwrap();
        assert!(!exponent_negative.contains(Hp67SegmentMask::E));
        assert!(exponent_negative.contains(Hp67SegmentMask::G));
    }

    #[test]
    fn scan_roles_follow_observed_fifteen_slot_order() {
        assert_eq!(
            display_role_for_scan_slot(1),
            Some(Hp67DisplayRole::ExponentUnits)
        );
        assert_eq!(
            display_role_for_scan_slot(2),
            Some(Hp67DisplayRole::ExponentTens)
        );
        assert_eq!(
            display_role_for_scan_slot(3),
            Some(Hp67DisplayRole::SharedSigns)
        );
        assert_eq!(
            display_role_for_scan_slot(4),
            Some(Hp67DisplayRole::MantissaDigit(11))
        );
        assert_eq!(
            display_role_for_scan_slot(14),
            Some(Hp67DisplayRole::MantissaDigit(1))
        );
        assert_eq!(
            display_role_for_scan_slot(15),
            Some(Hp67DisplayRole::ExponentUnitsDuplicate)
        );
    }

    #[test]
    fn cathode_consumes_rom0_str_and_act_rcd_without_driving_upstream_phase() {
        let mut driver = CathodeDriver1820_1749::default();
        let mut roles = Vec::new();
        for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            roles.push(
                driver
                    .apply_control_edges(
                        Rom0StrEvent { scan_slot },
                        scan_slot == HP67_DISPLAY_SCAN_SLOTS,
                    )
                    .unwrap(),
            );
        }
        assert_eq!(roles[0], Hp67DisplayRole::ExponentUnits);
        assert_eq!(roles[2], Hp67DisplayRole::SharedSigns);
        assert_eq!(roles[14], Hp67DisplayRole::ExponentUnitsDuplicate);
        assert_eq!(driver.scan_slot(), 1);
    }

    #[test]
    fn cathode_rejects_phase_or_rcd_mismatches() {
        let mut driver = CathodeDriver1820_1749::default();
        assert_eq!(
            driver.apply_control_edges(Rom0StrEvent { scan_slot: 2 }, false),
            Err(CathodeScanError::PhaseMismatch {
                cathode_slot: 1,
                str_slot: 2,
            })
        );
        assert_eq!(
            driver.apply_control_edges(Rom0StrEvent { scan_slot: 1 }, true),
            Err(CathodeScanError::UnexpectedRcd { scan_slot: 1 })
        );
    }

    #[test]
    fn cathode_structural_scan_wraps_after_fifteen_strobes_and_rcd_resets() {
        let mut driver = CathodeDriver1820_1749::default();
        let mut roles = Vec::new();
        for _ in 0..HP67_DISPLAY_SCAN_SLOTS {
            roles.push(driver.str_falling_edge());
        }
        assert_eq!(roles[0], Hp67DisplayRole::ExponentUnits);
        assert_eq!(roles[2], Hp67DisplayRole::SharedSigns);
        assert_eq!(roles[14], Hp67DisplayRole::ExponentUnitsDuplicate);
        assert_eq!(driver.scan_slot(), 1);

        driver.str_falling_edge();
        driver.str_falling_edge();
        assert_eq!(driver.scan_slot(), 3);
        driver.rcd_falling_edge();
        assert_eq!(driver.scan_slot(), 1);
    }

    #[test]
    fn unknown_rom0_codes_fail_instead_of_being_guessed() {
        assert_eq!(
            decode_rom0_display_byte(1, 0x50),
            Err(Rom0DisplayError::UnknownDisplayCode {
                scan_slot: 1,
                code: 0x50,
            })
        );
    }
}
