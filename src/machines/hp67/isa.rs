//! HP-67 IS/ISA serial-bus encoding helpers.
//!
//! HP-67 logic-analyser evidence shows that the IS line is passively biased low
//! and active participants pull it high rather than actively driving zero. The
//! ACT emits the ROM0 display byte LSB-first at b0..b7, sends the 12-bit ROM
//! address LSB-first during b16..b27, and the selected ROM returns a 10-bit word
//! LSB-first during b46..b55. These helpers encode those electrical/serial facts
//! without yet choosing the exact PHI launch/sample edge.

use crate::emulation::Drive;

use super::timing::{display_data_serial_bit, isa_window_for_bit, IsaWindow};

/// Mask for the 12-bit HP-67 ROM address space.
pub const ROM_ADDRESS_MASK: u16 = 0x0fff;
/// Mask for one 10-bit Woodstock ROM word.
pub const ROM_WORD_MASK: u16 = 0x03ff;

/// Convert one logical serial bit to the observed HP-67 wired-high IS drive.
///
/// A logic one is actively pulled high. A logic zero releases the line so the
/// passive low bias determines the visible level.
pub const fn wired_high_drive(bit: bool) -> Drive {
    if bit {
        Drive::High
    } else {
        Drive::HighZ
    }
}

/// Extract one LSB-first serial bit from a 12-bit ROM address.
pub const fn rom_address_serial_bit(address: u16, serial_bit: u8) -> bool {
    debug_assert!(serial_bit < 12);
    ((address >> serial_bit) & 1) != 0
}

/// Extract one LSB-first serial bit from a 10-bit Woodstock ROM word.
pub const fn rom_word_serial_bit(word: u16, serial_bit: u8) -> bool {
    debug_assert!(serial_bit < 10);
    ((word >> serial_bit) & 1) != 0
}

/// Drive contribution the ACT display path should make on IS for the eight-bit
/// ROM0 display code at the given `b0..b55` coordinate. Outside b0..b7 this
/// role releases the bus.
pub const fn act_display_drive(display_byte: u8, word_bit: u8) -> Drive {
    match display_data_serial_bit(word_bit) {
        Some(serial_bit) => wired_high_drive(((display_byte >> serial_bit) & 1) != 0),
        None => Drive::HighZ,
    }
}

/// Drive contribution the ACT should make on IS for a ROM address at the given
/// `b0..b55` coordinate. Outside b16..b27 the ACT releases this fetch role.
pub const fn act_address_drive(address: u16, word_bit: u8) -> Drive {
    match isa_window_for_bit(word_bit) {
        IsaWindow::RomAddress { serial_bit } => wired_high_drive(rom_address_serial_bit(
            address & ROM_ADDRESS_MASK,
            serial_bit,
        )),
        IsaWindow::RomWord { .. } | IsaWindow::Other => Drive::HighZ,
    }
}

/// Drive contribution the selected ROM should make on IS for a fetched word at
/// the given `b0..b55` coordinate. Outside b46..b55 the ROM releases this fetch
/// role.
pub const fn rom_word_drive(word: u16, word_bit: u8) -> Drive {
    match isa_window_for_bit(word_bit) {
        IsaWindow::RomWord { serial_bit } => {
            wired_high_drive(rom_word_serial_bit(word & ROM_WORD_MASK, serial_bit))
        }
        IsaWindow::RomAddress { .. } | IsaWindow::Other => Drive::HighZ,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn zero_is_release_and_one_is_active_high() {
        assert_eq!(wired_high_drive(false), Drive::HighZ);
        assert_eq!(wired_high_drive(true), Drive::High);
    }

    #[test]
    fn eight_bit_display_code_is_emitted_lsb_first_on_b0_through_b7() {
        let display_byte = 0x30;
        let expected = [false, false, false, false, true, true, false, false];
        for (serial_bit, expected_bit) in expected.into_iter().enumerate() {
            assert_eq!(
                act_display_drive(display_byte, serial_bit as u8),
                wired_high_drive(expected_bit)
            );
        }
        assert_eq!(act_display_drive(display_byte, 8), Drive::HighZ);
        assert_eq!(act_display_drive(display_byte, 16), Drive::HighZ);
    }

    #[test]
    fn twelve_bit_address_is_emitted_lsb_first_on_b16_through_b27() {
        // 0x07B = binary 0000_0111_1011. The expected serial stream is the
        // address bit order observed in the HP-67 timing documentation.
        let address = 0x07b;
        let expected = [
            true, true, false, true, true, true, true, false, false, false, false, false,
        ];
        for (serial_bit, expected_bit) in expected.into_iter().enumerate() {
            assert_eq!(
                act_address_drive(address, 16 + serial_bit as u8),
                wired_high_drive(expected_bit)
            );
        }
        assert_eq!(act_address_drive(address, 15), Drive::HighZ);
        assert_eq!(act_address_drive(address, 28), Drive::HighZ);
    }

    #[test]
    fn ten_bit_rom_word_is_emitted_lsb_first_on_b46_through_b55() {
        // HP-67 example from the measured key-wait loop: address 0x07B returns
        // 0x04C. 0x04C = 00_0100_1100, so the serial stream is bit 0 first.
        let word = 0x04c;
        let expected = [
            false, false, true, true, false, false, true, false, false, false,
        ];
        for (serial_bit, expected_bit) in expected.into_iter().enumerate() {
            assert_eq!(
                rom_word_drive(word, 46 + serial_bit as u8),
                wired_high_drive(expected_bit)
            );
        }
        assert_eq!(rom_word_drive(word, 45), Drive::HighZ);
        assert_eq!(rom_word_drive(word, 0), Drive::HighZ);
    }

    #[test]
    fn display_address_and_rom_response_roles_do_not_overlap() {
        for bit in 0..56 {
            let display = act_display_drive(0xff, bit);
            let address = act_address_drive(0x0fff, bit);
            let rom = rom_word_drive(0x03ff, bit);
            let active = [display, address, rom]
                .into_iter()
                .filter(|drive| *drive == Drive::High)
                .count();
            assert!(active <= 1, "multiple active IS roles at b{bit}");
        }
    }
}
