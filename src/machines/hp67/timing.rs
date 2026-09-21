//! HP-67 serial-word timing coordinates layered on the temporary PHI scaffold.
//!
//! The physical Woodstock datapath is a 56-bit serial word: fourteen 4-bit
//! digit times. This module defines the project's bit numbering convention as
//! `b0..b55` and tracks that position. HP-67-specific logic-analyser evidence
//! anchors ROM0 display data at b0..b7, the ROM address window at b16..b27,
//! the 10-bit ROM response at b46..b55, and the 56-bit DATA stream with its
//! serial bit 0 appearing at machine-word bit b2. Display scope captures also
//! bound STR/LED timing at the microsecond level. Exact PHI launch/sample edges
//! remain deliberately unspecified until the waveform edge relationships are
//! converted into a reviewed scheduler contract.

/// Number of serial bits in one HP-67 digit/nibble.
pub const BITS_PER_DIGIT: u8 = 4;
/// Number of digit times in one HP-67 machine word.
pub const DIGITS_PER_WORD: u8 = 14;
/// Number of serial bit times in one HP-67 machine word.
pub const BITS_PER_WORD: u8 = BITS_PER_DIGIT * DIGITS_PER_WORD;

/// Approximate whole-word duration measured on a physical HP-67 logic-analyser trace.
///
/// Tony Nixon's HP-67 capture reports 320 us for the complete 56-bit instruction
/// cycle. This is a coarse observed duration, not a claim about exact PHI pulse
/// widths or launch/sample edges.
pub const HP67_OBSERVED_WORD_TIME_US: u64 = 320;
/// Approximate full 15-STR display refresh measured on the same physical HP-67.
pub const HP67_OBSERVED_DISPLAY_REFRESH_US: u64 = 4_800;
/// Approximate delay from switch-on until SYNC becomes active on the measured HP-67.
pub const HP67_OBSERVED_POWER_ON_SYNC_DELAY_US: u64 = 35_000;
/// Approximate delay after switch-on at which the measured power-on signals start stabilizing.
pub const HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US: u64 = 330;

/// Approximate on-time of a normal LED segment in the measured HP-67 display scan.
pub const HP67_OBSERVED_DISPLAY_SEGMENT_ON_US: u64 = 40;
/// Approximate on-time of the decimal-point LED in the measured HP-67 display scan.
pub const HP67_OBSERVED_DISPLAY_DP_ON_US: u64 = 30;
/// Approximate quiet gap between the decimal-point LED ending and the STR pulse.
pub const HP67_OBSERVED_DISPLAY_DP_TO_STR_GAP_US: u64 = 5;
/// Approximate STR pulse width measured on the physical HP-67.
pub const HP67_OBSERVED_DISPLAY_STR_PULSE_US: u64 = 5;

/// First HP-67 bit time carrying the eight-bit ROM0 display code on IS.
pub const DISPLAY_DATA_FIRST_BIT: u8 = 0;
/// Number of serial bits in one ROM0 display code, LSB first.
pub const DISPLAY_DATA_BITS: u8 = 8;
/// Last HP-67 bit time carrying the ROM0 display code on IS.
pub const DISPLAY_DATA_LAST_BIT: u8 = DISPLAY_DATA_FIRST_BIT + DISPLAY_DATA_BITS - 1;
/// Bit coordinate in which the ROM0 STR pulse occurs.
///
/// The HP-67 scope capture explicitly places STR on display-data bit 7 and
/// shows the low-going STR edge starting the segment interval. The exact
/// PHI-relative edge/propagation relationship is still not encoded here.
pub const DISPLAY_STR_BIT: u8 = DISPLAY_DATA_LAST_BIT;

/// HP-67 machine-word bit where serial DATA bit 0 appears.
///
/// Direct HP-67 captures show the 56-bit DATA register stream LSB first, with
/// serial bit 0 at machine-word cycle b2. Consequently serial bits 54 and 55
/// cross the word boundary and appear at b0 and b1 of the following word.
pub const DATA_STREAM_FIRST_WORD_BIT: u8 = 2;
/// Number of serial bits in one complete DATA register stream.
pub const DATA_STREAM_BITS: u8 = BITS_PER_WORD;

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

// The generic TwoPhaseClock scaffold marks abstract active-PHI1, interphase,
// active-PHI2, interphase slots. The HP-67 backplane maps an active phase to
// the directly observed low-going PHI pin pulse. Until measured widths are
// installed, one complete four-slot sequence is treated as one serial bit time.
// This is a scheduler coordinate, not an assertion that every slot has the same
// physical duration.
const CLOCK_SUBPHASES_PER_BIT: u8 = 4;

/// HP-67-specific name for the four currently represented positions inside one
/// serial bit.
///
/// Direct HP-67 page-70 captures establish the pin polarity/order: PHI1 is a
/// low-going pulse, both clocks return high, PHI2 is a low-going pulse, then
/// both clocks return high again. The enum deliberately names topology rather
/// than duration; the physical widths/dead times remain uncalibrated.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ClockPhase {
    Phi1Low,
    InterphaseAfterPhi1,
    Phi2Low,
    InterphaseAfterPhi2,
}

impl Hp67ClockPhase {
    pub const fn from_subphase(subphase: u8) -> Self {
        match subphase & 0b11 {
            0 => Self::Phi1Low,
            1 => Self::InterphaseAfterPhi1,
            2 => Self::Phi2Low,
            _ => Self::InterphaseAfterPhi2,
        }
    }

    pub const fn phi1_low(self) -> bool {
        matches!(self, Self::Phi1Low)
    }

    pub const fn phi2_low(self) -> bool {
        matches!(self, Self::Phi2Low)
    }
}

/// Named HP-67 clock transitions used by source-backed timing contracts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ClockEdge {
    Phi1Falling,
    Phi1Rising,
    Phi2Falling,
    Phi2Rising,
}

/// Direct page-70 HP-67 captures show both the rising and falling transitions
/// of SYNC aligned with the rising edge of PHI2.
///
/// This is an observed edge relationship, not a propagation-delay claim. The
/// current scheduler still lacks calibrated sub-microsecond transition timing.
pub const HP67_SYNC_TRANSITION_EDGE: Hp67ClockEdge = Hp67ClockEdge::Phi2Rising;

/// Meaning of the IS/ISA line at one HP-67 serial bit coordinate for fetch.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IsaWindow {
    /// ACT-to-ROM 12-bit address transfer. `serial_bit` is 0..11, LSB first.
    RomAddress { serial_bit: u8 },
    /// ROM-to-ACT 10-bit word transfer. `serial_bit` is 0..9, LSB first.
    RomWord { serial_bit: u8 },
    /// No instruction-fetch meaning is assigned here by this timing contract.
    Other,
}

/// Return the ROM0 display-code bit carried at one `b0..b55` coordinate.
///
/// The eight-bit value is observed LSB first at b0..b7. Outside that window
/// there is no ROM0 display-code bit assigned by this contract.
pub const fn display_data_serial_bit(bit_index: u8) -> Option<u8> {
    if bit_index >= DISPLAY_DATA_FIRST_BIT && bit_index <= DISPLAY_DATA_LAST_BIT {
        Some(bit_index - DISPLAY_DATA_FIRST_BIT)
    } else {
        None
    }
}

/// Map an HP-67 machine-word coordinate to the serial bit number visible on DATA.
///
/// The stream is continuous across instruction-word boundaries: b2 carries DATA
/// bit 0, b55 carries bit 53, and the next word's b0/b1 carry bits 54/55.
pub const fn data_serial_bit_for_word_bit(bit_index: u8) -> u8 {
    (bit_index + (BITS_PER_WORD - DATA_STREAM_FIRST_WORD_BIT)) % DATA_STREAM_BITS
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
/// bit times b46..b55. For a normal instruction SYNC is asserted across this
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

    /// Current ROM0 display-code serial bit, if this is b0..b7.
    pub const fn display_data_serial_bit(&self) -> Option<u8> {
        display_data_serial_bit(self.bit_index)
    }

    /// Current DATA serial bit according to the measured HP-67 b2 phase offset.
    pub const fn data_serial_bit(&self) -> u8 {
        data_serial_bit_for_word_bit(self.bit_index)
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

    /// HP-67-specific named phase for the current scheduler coordinate.
    ///
    /// This exposes source-backed pin polarity/order without claiming physical
    /// phase durations.
    pub const fn clock_phase(&self) -> Hp67ClockPhase {
        Hp67ClockPhase::from_subphase(self.clock_subphase)
    }

    /// Advance one scheduler clock sub-phase.
    ///
    /// Every complete four-slot scaffold period advances one serial bit. Bit 55
    /// wraps to bit 0 and increments `word_index`.
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
    fn coarse_hp67_power_on_and_refresh_times_match_measured_trace() {
        assert_eq!(HP67_OBSERVED_WORD_TIME_US, 320);
        assert_eq!(HP67_OBSERVED_DISPLAY_REFRESH_US, 4_800);
        assert_eq!(HP67_OBSERVED_POWER_ON_SYNC_DELAY_US, 35_000);
        assert_eq!(HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US, 330);
        assert_eq!(HP67_OBSERVED_DISPLAY_SEGMENT_ON_US, 40);
        assert_eq!(HP67_OBSERVED_DISPLAY_DP_ON_US, 30);
        assert_eq!(HP67_OBSERVED_DISPLAY_DP_TO_STR_GAP_US, 5);
        assert_eq!(HP67_OBSERVED_DISPLAY_STR_PULSE_US, 5);
        assert_eq!(
            HP67_OBSERVED_DISPLAY_REFRESH_US,
            HP67_OBSERVED_WORD_TIME_US * 15
        );
    }

    #[test]
    fn hp67_rom0_display_window_is_b0_through_b7_lsb_first() {
        assert_eq!(DISPLAY_DATA_FIRST_BIT, 0);
        assert_eq!(DISPLAY_DATA_LAST_BIT, 7);
        assert_eq!(DISPLAY_DATA_BITS, 8);
        assert_eq!(DISPLAY_STR_BIT, 7);
        for bit in 0..8 {
            assert_eq!(display_data_serial_bit(bit), Some(bit));
        }
        assert_eq!(display_data_serial_bit(8), None);
        assert_eq!(display_data_serial_bit(55), None);
    }

    #[test]
    fn hp67_data_stream_is_lsb_first_with_bit_zero_at_machine_bit_two() {
        assert_eq!(DATA_STREAM_FIRST_WORD_BIT, 2);
        assert_eq!(DATA_STREAM_BITS, 56);
        assert_eq!(data_serial_bit_for_word_bit(2), 0);
        assert_eq!(data_serial_bit_for_word_bit(3), 1);
        assert_eq!(data_serial_bit_for_word_bit(55), 53);
        assert_eq!(data_serial_bit_for_word_bit(0), 54);
        assert_eq!(data_serial_bit_for_word_bit(1), 55);

        let mut seen = [false; BITS_PER_WORD as usize];
        for word_bit in 0..BITS_PER_WORD {
            let serial_bit = data_serial_bit_for_word_bit(word_bit);
            assert!(serial_bit < BITS_PER_WORD);
            assert!(!seen[serial_bit as usize]);
            seen[serial_bit as usize] = true;
        }
        assert!(seen.into_iter().all(|value| value));
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
        assert_eq!(isa_window_for_bit(46), IsaWindow::RomWord { serial_bit: 0 });
        assert_eq!(isa_window_for_bit(55), IsaWindow::RomWord { serial_bit: 9 });
        assert!(!sync_decision_window(45));
        for bit in 46..=55 {
            assert!(sync_decision_window(bit));
        }
    }

    #[test]
    fn evidenced_is_windows_are_disjoint() {
        for bit in 0..BITS_PER_WORD {
            let display = display_data_serial_bit(bit).is_some();
            let fetch = !matches!(isa_window_for_bit(bit), IsaWindow::Other);
            assert!(!(display && fetch));
        }
    }

    #[test]
    fn hp67_sync_transitions_are_anchored_to_phi2_rising() {
        assert_eq!(
            HP67_SYNC_TRANSITION_EDGE,
            Hp67ClockEdge::Phi2Rising
        );
    }

    #[test]
    fn hp67_clock_phase_names_lock_active_low_order_without_durations() {
        assert_eq!(Hp67ClockPhase::from_subphase(0), Hp67ClockPhase::Phi1Low);
        assert_eq!(
            Hp67ClockPhase::from_subphase(1),
            Hp67ClockPhase::InterphaseAfterPhi1
        );
        assert_eq!(Hp67ClockPhase::from_subphase(2), Hp67ClockPhase::Phi2Low);
        assert_eq!(
            Hp67ClockPhase::from_subphase(3),
            Hp67ClockPhase::InterphaseAfterPhi2
        );
        assert!(Hp67ClockPhase::Phi1Low.phi1_low());
        assert!(!Hp67ClockPhase::Phi1Low.phi2_low());
        assert!(!Hp67ClockPhase::Phi2Low.phi1_low());
        assert!(Hp67ClockPhase::Phi2Low.phi2_low());
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
