//! Initial HP-67 electrical backplane shell.
//!
//! It currently owns explicit named nets, a deterministic two-phase timing
//! source and the canonical 56-bit serial-word coordinate.  The timing source is
//! still a scaffold, not yet the final ACT oscillator; later milestones will
//! replace its durations/edge placement from hardware evidence while retaining
//! the same observable net and word-coordinate boundaries.

use crate::emulation::{Drive, LogicLevel, Tick};

use super::{
    electrical::{
        Hp67ElectricalError, Hp67ElectricalFabric, Hp67ElectricalSnapshot,
        Hp67ElectricalStager,
    },
    timing::{Hp67ClockEdge, Hp67ClockPhase, BITS_PER_DIGIT, BITS_PER_WORD},
    wiring::{Hp67Driver, Hp67Net},
};

/// Pin-level connection fabric for HP-67 devices.
#[derive(Debug, Clone, Default)]
pub struct Hp67ElectricalBackplane {
    fabric: Hp67ElectricalFabric,
}

impl Hp67ElectricalBackplane {
    pub const fn tick(&self) -> Tick {
        self.fabric.tick()
    }

    /// Current HP-67 serial bit coordinate (`b0..b55`).
    pub const fn word_bit(&self) -> u8 {
        ((self.fabric.tick().get() >> 2) % BITS_PER_WORD as u64) as u8
    }

    /// Current named HP-67 clock position inside the serial bit.
    pub const fn clock_phase(&self) -> Hp67ClockPhase {
        match self.fabric.tick().get() & 0b11 {
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
        self.fabric.tick().get() / (BITS_PER_WORD as u64 * 4)
    }

    pub fn level(&self, net: Hp67Net) -> LogicLevel {
        self.fabric.level(net)
    }

    pub fn drive(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        self.fabric.set_drive_immediate(net, driver, drive);
    }

    pub fn begin_evaluation(
        &mut self,
    ) -> Result<(Hp67ElectricalSnapshot<'_>, Hp67ElectricalStager<'_>), Hp67ElectricalError> {
        self.fabric.begin_evaluation()
    }

    pub fn stage_drive(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        self.fabric.stage_drive(net, driver, drive);
    }

    pub fn commit_staged(&mut self) -> Result<Tick, Hp67ElectricalError> {
        self.fabric.commit_staged()
    }

    /// Advance the temporary timing scaffold by one non-overlapping clock
    /// transition and advance the HP-67 56-bit coordinate accordingly.
    ///
    /// Page-70 HP-67 captures show both PHI pins normally high with alternating
    /// low-going pulses. Pulse width/dead time remain uncalibrated; only the pin
    /// polarity and non-overlap relationship are source-backed here.
    pub fn advance_clock_edge(&mut self) -> (Tick, Hp67ClockEdge) {
        let tick = self.fabric.advance_tick();

        // PHI1/PHI2 are ACT-owned outputs, so the dense fabric updates the
        // known ACT driver slot directly rather than passing each of the 224
        // transitions per word through a generic named-driver container.
        let edge = match tick.get() & 0b11 {
            1 => {
                self.drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::Low);
                Hp67ClockEdge::Phi1Falling
            }
            2 => {
                self.drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::High);
                Hp67ClockEdge::Phi1Rising
            }
            3 => {
                self.drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::Low);
                Hp67ClockEdge::Phi2Falling
            }
            _ => {
                self.drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::High);
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

        (tick, edge)
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
