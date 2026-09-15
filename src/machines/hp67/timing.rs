//! HP-67 serial-word timing coordinates layered on the temporary PHI scaffold.
//!
//! The physical Woodstock datapath is a 56-bit serial word: fourteen 4-bit
//! digit times.  This module defines the project's bit numbering convention as
//! `b0..b55` and tracks that position.  HP-67-specific logic-analyser evidence
//! now anchors the ROM address window at b16..b27 and the 10-bit ROM response at
//! b46..b55, both least-significant bit first.  Absolute pulse widths and the
//! exact PHI edge used to sample/launch each serial bit remain deliberately
//! unspecified until they are tied to a reviewed waveform.

/// Number of serial bits in one HP-67 digit/nibble.
pub const BITS_PER_DIGIT: u8 = 4;
/// Number of digit times in one HP-67 machine word.
pub const DIGITS_PER_WORD: u8 = 14;
/// Number of serial bit times in one HP-67 machine word.
pub const BITS_PER_WORD: u8 = BITS_PER_DIGIT * DIGITS_PER_WORD;

/// First HP-67 bit time carrying the 12-bit ROM address on IS/ISA.
pub const ROM_ADDRESS_FIRST_BIT: u8 = 16;
/// Number of serial address bits sent to the ROM, LSB first.
pub const ROM_ADDRESS_BITS: u8 = 12;
/// Last HP-67 bit time carrying the 12-bit ROM address on IS/ISA.
pub const ROM_ADDRESS_LAST_BIT: u8 = ROM_ADDRESS_FIRST_BIT + ROM_ADDRESS_BITS - 1;

/// First HP-67 bit time carrying the fetched 10-bit ROM word on IS/ISA.
pub const ROM_WORD_FIRST_BIT: u8 = 46;
/// Number of serial bits in one Woodstock microinstruction/THEN-GOTO word.
pub const ROM_WORD_BITS: u8 = 10;
/// Last HP-67 bit time carrying the fetched 10-bit ROM word on IS/ISA.
pub const ROM_WORD_LAST_BIT: u8 = ROM_WORD_FIRST_BIT + ROM_WORD_BITS - 1;

// The current generic TwoPhaseClock scaffold is PHI1-high, dead, PHI2-high,
// dead.  Until measured widths are installed, one complete four-slot sequence
// is treated as one serial bit time.  This is a scheduler coordinate, not an
// assertion that every slot has the same physical duration.
const CLOCK_SUBPHASES_PER_BIT: u8 = 4;

/// Meaning of the IS/ISA line at one HP-67 serial bit coordinate.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsaWindow {
    /// ACT-to-ROM 12-bit address transfer. `serial_bit` is 0..11, LSB first.
    RomAddress { serial_bit: u8 },
    /// ROM-to-ACT 10-bit word transfer. `serial_bit` is 0..9, LSB first.
    RomWord { serial_bit: u8 },
    /// No instruction-fetch meaning is assigned here by this timing contract.
    Other,
}

/// Classify one `b0..b55` coordinate by its evidenced HP-67 instruction-fetch
/// role.
pub const fn isa_window_for_bit(bit_index: u8) -> IsaWindow {
    if bit_index >= ROM_ADDRESS_FIRST_BIT && bit_index <= ROM_ADDRESS_LAST_BIT {
        IsaWindow::RomAddress {
            serial_bit: bit_index - ROM_ADDRESS_FIRST_BIT,
        }
    } else if bit_index >= ROM_WORD_FIRST_BIT && bit_index <= ROM_WORD_LAST_BIT {
        IsaWindow::RomWord {
            serial_bit: bit_index - ROM_WORD_FIRST_BIT,
        }
    } else {
        IsaWindow::Other
    }
}

/// SYNC's instruction/THEN-GOTO decision window is exactly the ten returned ROM
/// bit times b46..b55.  For a normal instruction SYNC is asserted across this
/// window; after an IF/test it remains low and the returned 10-bit word is used
/// as a full within-1K destination address instead of being decoded as an
/// instruction.
pub const fn sync_decision_window(bit_index: u8) -> bool {
    bit_index >= ROM_WORD_FIRST_BIT && bit_index <= ROM_WORD_LAST_BIT
}

/// Deterministic coordinate within the HP-67 56-bit serial word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67WordTiming {
    word_index: u64,
    bit_index: u8,
    clock_subphase: u8,
}

impl Default for Hp67WordTiming {
    fn default() -> Self {
        Self {
            word_index: 0,
            bit_index: 0,
            clock_subphase: 0,
        }
    }
}

impl Hp67WordTiming {
    /// Monotonic completed/current machine-word number since timing reset.
    pub const fn word_index(&self) -> u64 {
        self.word_index
    }

    /// Current project bit coordinate, always in `0..56`.
    pub const fn bit_index(&self) -> u8 {
        self.bit_index
    }

    /// Current evidenced IS/ISA instruction-fetch role.
    pub const fn isa_window(&self) -> IsaWindow {
        isa_window_for_bit(self.bit_index)
    }

    /// Whether the current bit belongs to the ten-bit SYNC decision window.
    pub const fn in_sync_decision_window(&self) -> bool {
        sync_decision_window(self.bit_index)
    }

    /// Current 4-bit digit coordinate, always in `0..14`.
    pub const fn digit_index(&self) -> u8 {
        self.bit_index / BITS_PER_DIGIT
    }

    /// Bit position inside the current digit, always in `0..4`.
    pub const fn bit_in_digit(&self) -> u8 {
        self.bit_index % BITS_PER_DIGIT
    }

    /// Current slot inside the temporary four-slot two-phase clock period.
    pub const fn clock_subphase(&self) -> u8 {
        self.clock_subphase
    }

    /// Advance one scheduler clock sub-phase.
    ///
    /// Every complete four-slot scaffold period advances one serial bit.  Bit
    /// 55 wraps to bit 0 and increments `word_index`.
    pub fn advance_clock_subphase(&mut self) {
        self.clock_subphase += 1;
        if self.clock_subphase < CLOCK_SUBPHASES_PER_BIT {
            return;
        }

        self.clock_subphase = 0;
        self.bit_index += 1;
        if self.bit_index == BITS_PER_WORD {
            self.bit_index = 0;
            self.word_index = self.word_index.wrapping_add(1);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn word_geometry_is_fourteen_four_bit_digits() {
        assert_eq!(BITS_PER_DIGIT, 4);
        assert_eq!(DIGITS_PER_WORD, 14);
        assert_eq!(BITS_PER_WORD, 56);
    }

    #[test]
    fn hp67_rom_address_window_is_b16_through_b27_lsb_first() {
        assert_eq!(ROM_ADDRESS_FIRST_BIT, 16);
        assert_eq!(ROM_ADDRESS_LAST_BIT, 27);
        assert_eq!(ROM_ADDRESS_BITS, 12);
        assert_eq!(
            isa_window_for_bit(16),
            IsaWindow::RomAddress { serial_bit: 0 }
        );
        assert_eq!(
            isa_window_for_bit(27),
            IsaWindow::RomAddress { serial_bit: 11 }
        );
        assert_eq!(isa_window_for_bit(15), IsaWindow::Other);
        assert_eq!(isa_window_for_bit(28), IsaWindow::Other);
    }

    #[test]
    fn hp67_rom_word_and_sync_window_is_b46_through_b55_lsb_first() {
        assert_eq!(ROM_WORD_FIRST_BIT, 46);
        assert_eq!(ROM_WORD_LAST_BIT, 55);
        assert_eq!(ROM_WORD_BITS, 10);
        assert_eq!(
            isa_window_for_bit(46),
            IsaWindow::RomWord { serial_bit: 0 }
        );
        assert_eq!(
            isa_window_for_bit(55),
            IsaWindow::RomWord { serial_bit: 9 }
        );
        assert!(!sync_decision_window(45));
        for bit in 46..=55 {
            assert!(sync_decision_window(bit));
        }
    }

    #[test]
    fn address_and_rom_word_windows_are_disjoint() {
        for bit in 0..BITS_PER_WORD {
            let role = isa_window_for_bit(bit);
            assert!(!matches!(
                role,
                IsaWindow::RomAddress { .. } if bit >= ROM_WORD_FIRST_BIT
            ));
            assert!(!matches!(
                role,
                IsaWindow::RomWord { .. } if bit <= ROM_ADDRESS_LAST_BIT
            ));
        }
    }

    #[test]
    fn four_scaffold_subphases_make_one_serial_bit() {
        let mut timing = Hp67WordTiming::default();
        for _ in 0..3 {
            timing.advance_clock_subphase();
            assert_eq!(timing.bit_index(), 0);
        }
        timing.advance_clock_subphase();
        assert_eq!(timing.bit_index(), 1);
        assert_eq!(timing.clock_subphase(), 0);
    }

    #[test]
    fn digit_coordinates_follow_b0_through_b55() {
        let mut timing = Hp67WordTiming::default();
        for expected_bit in 0..BITS_PER_WORD {
            assert_eq!(timing.bit_index(), expected_bit);
            assert_eq!(timing.digit_index(), expected_bit / 4);
            assert_eq!(timing.bit_in_digit(), expected_bit % 4);
            for _ in 0..CLOCK_SUBPHASES_PER_BIT {
                timing.advance_clock_subphase();
            }
        }
        assert_eq!(timing.word_index(), 1);
        assert_eq!(timing.bit_index(), 0);
        assert_eq!(timing.digit_index(), 0);
        assert_eq!(timing.bit_in_digit(), 0);
    }
}
