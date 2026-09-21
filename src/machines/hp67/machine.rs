//! Initial HP-67 electrical backplane shell.
//!
//! It currently owns explicit named nets, a deterministic two-phase timing
//! source and the canonical 56-bit serial-word coordinate.  The timing source is
//! still a scaffold, not yet the final ACT oscillator; later milestones will
//! replace its durations/edge placement from hardware evidence while retaining
//! the same observable net and word-coordinate boundaries.

use std::collections::BTreeMap;

use crate::emulation::{Bias, Drive, DriverId, LogicLevel, Net, Tick, TwoPhaseClock};

use super::{timing::Hp67WordTiming, wiring::Hp67Net};

const CLOCK_DRIVER: DriverId = DriverId::new("hp67-act-clock-scaffold");

const fn hp67_clock_drive(active: bool) -> Drive {
    // Direct HP-67 captures show PHI1/PHI2 as alternating low-going pulses
    // from an otherwise high level. `TwoPhaseClock` expresses which phase is
    // active; the machine-specific backplane maps that abstract activity to
    // the evidenced HP-67 pin polarity.
    if active {
        Drive::Low
    } else {
        Drive::High
    }
}

/// Pin-level connection fabric for HP-67 devices.
#[derive(Debug, Clone)]
pub struct Hp67ElectricalBackplane {
    clock: TwoPhaseClock,
    word_timing: Hp67WordTiming,
    nets: BTreeMap<Hp67Net, Net>,
}

impl Default for Hp67ElectricalBackplane {
    fn default() -> Self {
        let nets = Hp67Net::ALL
            .into_iter()
            .map(|net| {
                // HP-67 hardware probing shows the shared IS line resting low
                // through a weak internal path while active participants pull
                // it high and otherwise release it.  Other nets remain
                // floating until their own passive behavior is evidenced.
                let bias = if net == Hp67Net::Isa {
                    Bias::PullDown
                } else {
                    Bias::Floating
                };
                (net, Net::new(bias))
            })
            .collect();
        Self {
            clock: TwoPhaseClock::default(),
            word_timing: Hp67WordTiming::default(),
            nets,
        }
    }
}

impl Hp67ElectricalBackplane {
    pub fn tick(&self) -> Tick {
        self.clock.tick()
    }

    /// Current HP-67 serial bit coordinate (`b0..b55`).
    pub const fn word_bit(&self) -> u8 {
        self.word_timing.bit_index()
    }

    /// Current 4-bit digit coordinate (`0..13`).
    pub const fn word_digit(&self) -> u8 {
        self.word_timing.digit_index()
    }

    /// Monotonic machine-word coordinate since timing reset.
    pub const fn word_index(&self) -> u64 {
        self.word_timing.word_index()
    }

    pub fn level(&self, net: Hp67Net) -> LogicLevel {
        self.nets
            .get(&net)
            .expect("all Hp67Net values are installed in the backplane")
            .level()
    }

    pub fn drive(&mut self, net: Hp67Net, driver: DriverId, drive: Drive) {
        self.nets
            .get_mut(&net)
            .expect("all Hp67Net values are installed in the backplane")
            .set_drive(driver, drive);
    }

    /// Advance the temporary timing scaffold by one non-overlapping clock
    /// sub-phase and advance the HP-67 56-bit coordinate accordingly.
    ///
    /// Page-70 HP-67 captures show both PHI pins normally high with alternating
    /// low-going pulses. Pulse width/dead time remain uncalibrated; only the pin
    /// polarity and non-overlap relationship are source-backed here.
    pub fn advance_clock(&mut self) -> Tick {
        let levels = self.clock.advance();
        self.drive(
            Hp67Net::Phi1,
            CLOCK_DRIVER,
            hp67_clock_drive(levels.phi1),
        );
        self.drive(
            Hp67Net::Phi2,
            CLOCK_DRIVER,
            hp67_clock_drive(levels.phi2),
        );
        self.word_timing.advance_clock_subphase();
        self.clock.tick()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::timing::BITS_PER_WORD;

    #[test]
    fn declared_nets_exist_and_only_is_has_an_evidenced_passive_low_bias() {
        let backplane = Hp67ElectricalBackplane::default();
        for net in Hp67Net::ALL {
            let expected = if net == Hp67Net::Isa {
                LogicLevel::Low
            } else {
                LogicLevel::Floating
            };
            assert_eq!(
                backplane.level(net),
                expected,
                "unexpected initial level for {net:?}"
            );
        }
    }

    #[test]
    fn hp67_clock_pulses_are_active_low_and_never_overlap() {
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut observed = Vec::new();
        for _ in 0..8 {
            backplane.advance_clock();
            let phi1 = backplane.level(Hp67Net::Phi1);
            let phi2 = backplane.level(Hp67Net::Phi2);
            assert!(!(phi1 == LogicLevel::Low && phi2 == LogicLevel::Low));
            observed.push((phi1, phi2));
        }
        assert_eq!(
            observed,
            vec![
                (LogicLevel::Low, LogicLevel::High),
                (LogicLevel::High, LogicLevel::High),
                (LogicLevel::High, LogicLevel::Low),
                (LogicLevel::High, LogicLevel::High),
                (LogicLevel::Low, LogicLevel::High),
                (LogicLevel::High, LogicLevel::High),
                (LogicLevel::High, LogicLevel::Low),
                (LogicLevel::High, LogicLevel::High),
            ]
        );
    }

    #[test]
    fn one_scaffold_phase_period_advances_one_serial_bit() {
        let mut backplane = Hp67ElectricalBackplane::default();
        assert_eq!(backplane.word_index(), 0);
        assert_eq!(backplane.word_bit(), 0);
        assert_eq!(backplane.word_digit(), 0);

        for _ in 0..4 {
            backplane.advance_clock();
        }
        assert_eq!(backplane.word_bit(), 1);
        assert_eq!(backplane.word_digit(), 0);

        for _ in 1..BITS_PER_WORD {
            for _ in 0..4 {
                backplane.advance_clock();
            }
        }
        assert_eq!(backplane.word_index(), 1);
        assert_eq!(backplane.word_bit(), 0);
        assert_eq!(backplane.word_digit(), 0);
    }
}
