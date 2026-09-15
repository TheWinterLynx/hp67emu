//! Instruction-boundary bridge from ACT display registers to the structural HP-67 display path.
//!
//! This module is deliberately a bring-up aid, not the final electrical ACT display serializer.
//! It converts already-latched A/B architectural nibbles into the eight-bit display code observed
//! at ROM0, then sends that byte through the resolved IS net at b0..b7 and decodes it with the
//! source-backed ROM0/cathode model. Exact PHI launch/sample edges and final STR/RCD pulse timing
//! remain outside this bridge.

use crate::emulation::{Drive, DriverId, LogicLevel};

use super::{
    act::{ActRegister, ACT_WORD_DIGITS},
    display::{
        decode_rom0_display_byte, display_role_for_scan_slot, CathodeDriver1820_1749,
        Hp67DisplayRole, Hp67SegmentMask, Rom0DisplayEndpoint, Rom0DisplayError,
        HP67_DISPLAY_SCAN_SLOTS,
    },
    machine::Hp67ElectricalBackplane,
    timing::{display_data_serial_bit, BITS_PER_WORD},
    wiring::Hp67Net,
};

const ACT_DISPLAY_IS_DRIVER: DriverId = DriverId::new("hp67-act-display-snapshot-is");

/// One decoded display slot captured from architectural A/B state through the structural IS path.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralDisplaySlot {
    pub scan_slot: u8,
    pub role: Hp67DisplayRole,
    pub register_index: usize,
    pub code: u8,
    pub segments: Hp67SegmentMask,
}

/// Map the observed HP-67 display scan order back to the ACT's 14-nibble register layout.
///
/// Register digits 0/1 are exponent units/tens, digit 2 is the sign position, and digits
/// 3..13 are mantissa digits 1..11. The fifteenth scan slot repeats exponent units.
pub const fn display_register_index_for_scan_slot(scan_slot: u8) -> Option<usize> {
    match display_role_for_scan_slot(scan_slot) {
        Some(Hp67DisplayRole::ExponentUnits | Hp67DisplayRole::ExponentUnitsDuplicate) => Some(0),
        Some(Hp67DisplayRole::ExponentTens) => Some(1),
        Some(Hp67DisplayRole::SharedSigns) => Some(2),
        Some(Hp67DisplayRole::MantissaDigit(digit)) => Some((digit + 2) as usize),
        None => None,
    }
}

/// Compose the ROM0 display byte from one A/B register position.
///
/// This is an instruction-boundary diagnostic composition: A supplies the low nibble and B the
/// display-control nibble. It is kept separate from the future bit-serial ACT implementation so
/// no whole-nibble shortcut can accidentally become the production electrical path.
pub fn display_byte_from_act_registers(
    scan_slot: u8,
    a: &ActRegister,
    b: &ActRegister,
) -> Result<u8, Rom0DisplayError> {
    let register_index = display_register_index_for_scan_slot(scan_slot)
        .ok_or(Rom0DisplayError::InvalidScanSlot(scan_slot))?;
    debug_assert!(register_index < ACT_WORD_DIGITS);
    Ok(((b[register_index] & 0x0f) << 4) | (a[register_index] & 0x0f))
}

fn drive_for_display_bit(code: u8, word_bit: u8) -> Drive {
    match display_data_serial_bit(word_bit) {
        Some(serial_bit) if code & (1u8 << serial_bit) != 0 => Drive::High,
        _ => Drive::HighZ,
    }
}

/// Run all fifteen source-backed display slots through a resolved IS net.
///
/// Each slot consumes one structural 56-bit word. Only b0..b7 are driven by this bridge; zero is
/// represented by release against the already-evidenced weak-low IS bias. The cathode scan is
/// advanced structurally after each reconstructed/decoded byte. This intentionally does not claim
/// the final PHI-relative STR/RCD edge ordering.
pub fn structural_display_scan_from_act_registers(
    a: &ActRegister,
    b: &ActRegister,
) -> Result<Vec<StructuralDisplaySlot>, Rom0DisplayError> {
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut cathode = CathodeDriver1820_1749::default();
    let mut slots = Vec::with_capacity(usize::from(HP67_DISPLAY_SCAN_SLOTS));

    for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
        let code = display_byte_from_act_registers(scan_slot, a, b)?;
        let register_index = display_register_index_for_scan_slot(scan_slot)
            .expect("validated display scan slot has a register index");
        rom0.begin_word();

        for expected_bit in 0..BITS_PER_WORD {
            debug_assert_eq!(backplane.word_bit(), expected_bit);
            backplane.drive(
                Hp67Net::Isa,
                ACT_DISPLAY_IS_DRIVER,
                drive_for_display_bit(code, expected_bit),
            );

            if display_data_serial_bit(expected_bit).is_some() {
                let level = backplane.level(Hp67Net::Isa);
                if level == LogicLevel::Contention {
                    return Err(Rom0DisplayError::IsaContention {
                        word_bit: expected_bit,
                    });
                }
                rom0.sample_for_bit(expected_bit, level)?;
            }

            for _ in 0..4 {
                backplane.advance_clock();
            }
        }

        let received = rom0.display_byte()?;
        debug_assert_eq!(received, code);
        let segments = decode_rom0_display_byte(scan_slot, received)?;
        let role = cathode.str_falling_edge();
        debug_assert_eq!(role, display_role_for_scan_slot(scan_slot).unwrap());
        slots.push(StructuralDisplaySlot {
            scan_slot,
            role,
            register_index,
            code: received,
            segments,
        });
    }

    Ok(slots)
}

#[cfg(test)]
mod tests {
    use super::*;

    // Real-firmware idle state independently observed by the power-on smoke and by published
    // HP-67 microcode tracing. Arrays are stored least-significant register digit first.
    const POWER_ON_IDLE_A: ActRegister = [
        0x0, 0x0, 0x1, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0xf, 0x0, 0x0, 0x0, 0x0,
    ];
    const POWER_ON_IDLE_B: ActRegister = [
        0x2, 0x2, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x0, 0x3, 0x0,
    ];

    #[test]
    fn scan_slots_map_to_exponent_sign_and_mantissa_register_positions() {
        assert_eq!(display_register_index_for_scan_slot(1), Some(0));
        assert_eq!(display_register_index_for_scan_slot(2), Some(1));
        assert_eq!(display_register_index_for_scan_slot(3), Some(2));
        assert_eq!(display_register_index_for_scan_slot(4), Some(13));
        assert_eq!(display_register_index_for_scan_slot(14), Some(3));
        assert_eq!(display_register_index_for_scan_slot(15), Some(0));
        assert_eq!(display_register_index_for_scan_slot(0), None);
        assert_eq!(display_register_index_for_scan_slot(16), None);
    }

    #[test]
    fn power_on_idle_registers_generate_source_backed_rom0_scan() {
        let scan = structural_display_scan_from_act_registers(&POWER_ON_IDLE_A, &POWER_ON_IDLE_B)
            .expect("known power-on display state must decode");

        let codes: Vec<u8> = scan.iter().map(|slot| slot.code).collect();
        assert_eq!(
            codes,
            vec![
                0x20, 0x20, 0x01, 0x00, 0x30, 0x00, 0x00, 0x0f, 0x0f, 0x0f, 0x0f, 0x0f,
                0x0f, 0x0f, 0x20,
            ]
        );

        let segment_bits: Vec<u8> = scan.iter().map(|slot| slot.segments.bits()).collect();
        assert_eq!(
            segment_bits,
            vec![
                0x00,
                0x00,
                0x00,
                Hp67SegmentMask::A.bits()
                    | Hp67SegmentMask::B.bits()
                    | Hp67SegmentMask::C.bits()
                    | Hp67SegmentMask::D.bits()
                    | Hp67SegmentMask::E.bits()
                    | Hp67SegmentMask::F.bits(),
                Hp67SegmentMask::DP.bits(),
                0x3f,
                0x3f,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
                0x00,
            ]
        );
    }

    #[test]
    fn structural_scan_reconstructs_every_byte_through_resolved_is() {
        let scan = structural_display_scan_from_act_registers(&POWER_ON_IDLE_A, &POWER_ON_IDLE_B)
            .expect("resolved IS scan must complete");
        assert_eq!(scan.len(), usize::from(HP67_DISPLAY_SCAN_SLOTS));
        assert_eq!(scan.first().unwrap().code, 0x20);
        assert_eq!(scan.last().unwrap().code, 0x20);
    }
}
