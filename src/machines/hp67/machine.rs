//! Initial HP-67 electrical backplane shell.
//!
//! It currently owns explicit named nets, a deterministic two-phase timing
//! source and the canonical 56-bit serial-word coordinate.  The timing source is
//! still a scaffold, not yet the final ACT oscillator; later milestones will
//! replace its durations/edge placement from hardware evidence while retaining
//! the same observable net and word-coordinate boundaries.

use crate::emulation::{Bias, Drive, DriverId, LogicLevel, Net, Tick};

use super::{
    timing::{Hp67ClockEdge, Hp67ClockPhase, BITS_PER_DIGIT, BITS_PER_WORD},
    wiring::Hp67Net,
};

const CLOCK_DRIVER: DriverId = DriverId::new("hp67-act-clock-scaffold");

/// Pin-level connection fabric for HP-67 devices.
#[derive(Debug, Clone)]
pub struct Hp67ElectricalBackplane {
    tick: u64,
    nets: [Net; Hp67Net::COUNT],
}

impl Default for Hp67ElectricalBackplane {
    fn default() -> Self {
        let mut nets = std::array::from_fn(|index| {
            let net = Hp67Net::ALL[index];
            // HP-67 hardware probing shows the shared IS line resting low
            // through a weak internal path while active participants pull
            // it high and otherwise release it. Other non-clock nets remain
            // floating until their own passive behavior is evidenced.
            let bias = if net == Hp67Net::Isa {
                Bias::PullDown
            } else {
                Bias::Floating
            };
            Net::new(bias)
        });

        // Direct HP-67 captures show PHI1/PHI2 normally high between their
        // alternating low-going pulses. Seed those ACT clock outputs in the
        // actual interphase level instead of starting the named phase with
        // electrically floating pins.
        nets[Hp67Net::Phi1.index()].set_drive(CLOCK_DRIVER, Drive::High);
        nets[Hp67Net::Phi2.index()].set_drive(CLOCK_DRIVER, Drive::High);

        Self { tick: 0, nets }
    }
}

impl Hp67ElectricalBackplane {
    pub const fn tick(&self) -> Tick {
        Tick::new(self.tick)
    }

    /// Current HP-67 serial bit coordinate (`b0..b55`).
    pub const fn word_bit(&self) -> u8 {
        ((self.tick >> 2) % BITS_PER_WORD as u64) as u8
    }

    /// Current named HP-67 clock position inside the serial bit.
    pub const fn clock_phase(&self) -> Hp67ClockPhase {
        match self.tick & 0b11 {
            1 => Hp67ClockPhase::Phi1Low,
            2 => Hp67ClockPhase::InterphaseAfterPhi1,
            3 => Hp67ClockPhase::Phi2Low,
            _ => Hp67ClockPhase::InterphaseAfterPhi2,
        }
    }

    /// Current 4-bit digit coordinate (`0..13`).
    pub const fn word_digit(&self) -> u8 {
        self.word_bit() / BITS_PER_DIGIT
    }

    /// Monotonic machine-word coordinate since timing reset.
    pub const fn word_index(&self) -> u64 {
        self.tick / (BITS_PER_WORD as u64 * 4)
    }

    pub fn level(&self, net: Hp67Net) -> LogicLevel {
        self.nets[net.index()].level()
    }

    pub fn drive(&mut self, net: Hp67Net, driver: DriverId, drive: Drive) {
        self.nets[net.index()].set_drive(driver, drive);
    }

    /// Advance the temporary timing scaffold by one non-overlapping clock
    /// sub-phase and advance the HP-67 56-bit coordinate accordingly.
    ///
    /// Page-70 HP-67 captures show both PHI pins normally high with alternating
    /// low-going pulses. Pulse width/dead time remain uncalibrated; only the pin
    /// polarity and non-overlap relationship are source-backed here.
    pub fn advance_clock_edge(&mut self) -> (Tick, Hp67ClockEdge) {
        self.tick = self.tick.wrapping_add(1);

        // One monotonic transition counter is sufficient to derive PHI phase,
        // bit, digit and word coordinates exactly. Keeping one canonical clock
        // coordinate avoids advancing three redundant state machines per edge.
        let edge = match self.tick & 0b11 {
            1 => {
                self.drive(Hp67Net::Phi1, CLOCK_DRIVER, Drive::Low);
                Hp67ClockEdge::Phi1Falling
            }
            2 => {
                self.drive(Hp67Net::Phi1, CLOCK_DRIVER, Drive::High);
                Hp67ClockEdge::Phi1Rising
            }
            3 => {
                self.drive(Hp67Net::Phi2, CLOCK_DRIVER, Drive::Low);
                Hp67ClockEdge::Phi2Falling
            }
            _ => {
                self.drive(Hp67Net::Phi2, CLOCK_DRIVER, Drive::High);
                Hp67ClockEdge::Phi2Rising
            }
        };

        debug_assert_eq!(
            self.level(Hp67Net::Phi1) == LogicLevel::Low,
            self.clock_phase().phi1_low()
        );
        debug_assert_eq!(
            self.level(Hp67Net::Phi2) == LogicLevel::Low,
            self.clock_phase().phi2_low()
        );

        (Tick::new(self.tick), edge)
    }

    pub fn advance_clock(&mut self) -> Tick {
        self.advance_clock_edge().0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::timing::BITS_PER_WORD;

    #[test]
    fn declared_nets_start_at_their_evidenced_idle_levels() {
        let backplane = Hp67ElectricalBackplane::default();
        for net in Hp67Net::ALL {
            let expected = match net {
                Hp67Net::Phi1 | Hp67Net::Phi2 => LogicLevel::High,
                Hp67Net::Isa => LogicLevel::Low,
                _ => LogicLevel::Floating,
            };
            assert_eq!(
                backplane.level(net),
                expected,
                "unexpected initial level for {net:?}"
            );
        }
        assert_eq!(backplane.clock_phase(), Hp67ClockPhase::InterphaseAfterPhi2);
    }

    #[test]
    fn hp67_clock_pulses_are_active_low_and_never_overlap() {
        let mut backplane = Hp67ElectricalBackplane::default();
        assert_eq!(backplane.clock_phase(), Hp67ClockPhase::InterphaseAfterPhi2);
        let mut observed = Vec::new();
        let mut edges = Vec::new();
        for _ in 0..8 {
            let (_, edge) = backplane.advance_clock_edge();
            let phi1 = backplane.level(Hp67Net::Phi1);
            let phi2 = backplane.level(Hp67Net::Phi2);
            assert!(!(phi1 == LogicLevel::Low && phi2 == LogicLevel::Low));
            assert_eq!(phi1 == LogicLevel::Low, backplane.clock_phase().phi1_low());
            assert_eq!(phi2 == LogicLevel::Low, backplane.clock_phase().phi2_low());
            observed.push((phi1, phi2));
            edges.push(edge);
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
        assert_eq!(
            edges,
            vec![
                Hp67ClockEdge::Phi1Falling,
                Hp67ClockEdge::Phi1Rising,
                Hp67ClockEdge::Phi2Falling,
                Hp67ClockEdge::Phi2Rising,
                Hp67ClockEdge::Phi1Falling,
                Hp67ClockEdge::Phi1Rising,
                Hp67ClockEdge::Phi2Falling,
                Hp67ClockEdge::Phi2Rising,
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
