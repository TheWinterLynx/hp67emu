//! Deterministic master time and a non-overlapping two-phase clock primitive.
//!
//! The clock is deliberately expressed in abstract sub-phase ticks.  Hardware
//! calibration will map those ticks to measured HP-67 timing later; no guessed
//! host-time frequency is baked into the emulation core.

/// Monotonic simulation time measured in the smallest committed scheduler tick.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Tick(u64);

impl Tick {
    pub const ZERO: Self = Self(0);

    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    pub const fn get(self) -> u64 {
        self.0
    }
}

/// Logic levels of the two non-overlapping clock phases after a scheduler tick.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ClockLevels {
    pub phi1: bool,
    pub phi2: bool,
}

impl ClockLevels {
    pub const BOTH_LOW: Self = Self {
        phi1: false,
        phi2: false,
    };
}

/// Four-slot non-overlapping clock: PHI1 high, dead time, PHI2 high, dead time.
///
/// This is the timing skeleton only.  Real HP-67 pulse widths and edge placement
/// must be calibrated from documented/scope timing before the ACT implementation
/// depends on absolute durations.
#[derive(Debug, Clone)]
pub struct TwoPhaseClock {
    tick: Tick,
    slot: u8,
}

impl Default for TwoPhaseClock {
    fn default() -> Self {
        Self {
            tick: Tick::ZERO,
            // Slot 3 is dead time so the first advance enters PHI1 high.
            slot: 3,
        }
    }
}

impl TwoPhaseClock {
    pub fn tick(&self) -> Tick {
        self.tick
    }

    pub fn levels(&self) -> ClockLevels {
        levels_for_slot(self.slot)
    }

    /// Advance exactly one scheduler sub-phase and return the new line levels.
    pub fn advance(&mut self) -> ClockLevels {
        self.slot = (self.slot + 1) & 0b11;
        self.tick = Tick::new(self.tick.get() + 1);
        self.levels()
    }
}

const fn levels_for_slot(slot: u8) -> ClockLevels {
    match slot & 0b11 {
        0 => ClockLevels {
            phi1: true,
            phi2: false,
        },
        2 => ClockLevels {
            phi1: false,
            phi2: true,
        },
        _ => ClockLevels::BOTH_LOW,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn phases_never_overlap() {
        let mut clock = TwoPhaseClock::default();
        for _ in 0..128 {
            let levels = clock.advance();
            assert!(!(levels.phi1 && levels.phi2));
        }
    }

    #[test]
    fn phase_sequence_is_stable_and_repeating() {
        let mut clock = TwoPhaseClock::default();
        let observed: Vec<_> = (0..8).map(|_| clock.advance()).collect();
        assert_eq!(
            observed,
            vec![
                ClockLevels { phi1: true, phi2: false },
                ClockLevels::BOTH_LOW,
                ClockLevels { phi1: false, phi2: true },
                ClockLevels::BOTH_LOW,
                ClockLevels { phi1: true, phi2: false },
                ClockLevels::BOTH_LOW,
                ClockLevels { phi1: false, phi2: true },
                ClockLevels::BOTH_LOW,
            ]
        );
        assert_eq!(clock.tick().get(), 8);
    }
}
