//! HP-67 serial-word timing coordinates layered on the temporary PHI scaffold.
//!
//! The physical Woodstock datapath is a 56-bit serial word: fourteen 4-bit
//! digit times.  This module defines the project's bit numbering convention as
//! `b0..b55` and tracks that position without assigning unverified ISA/DATA or
//! SYNC edge placement.  Absolute pulse widths remain deliberately unspecified.

/// Number of serial bits in one HP-67 digit/nibble.
pub const BITS_PER_DIGIT: u8 = 4;
/// Number of digit times in one HP-67 machine word.
pub const DIGITS_PER_WORD: u8 = 14;
/// Number of serial bit times in one HP-67 machine word.
pub const BITS_PER_WORD: u8 = BITS_PER_DIGIT * DIGITS_PER_WORD;

// The current generic TwoPhaseClock scaffold is PHI1-high, dead, PHI2-high,
// dead.  Until measured widths are installed, one complete four-slot sequence
// is treated as one serial bit time.  This is a scheduler coordinate, not an
// assertion that every slot has the same physical duration.
const CLOCK_SUBPHASES_PER_BIT: u8 = 4;

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
