//! Initial HP-67 electrical backplane shell.
//!
//! It currently owns explicit named nets and a deterministic two-phase timing
//! source.  The timing source is a scaffold, not yet the final ACT oscillator;
//! later milestones will replace it with ACT-driven, measured timing while
//! retaining the same observable net boundary.

use std::collections::BTreeMap;

use crate::emulation::{Bias, Drive, DriverId, LogicLevel, Net, Tick, TwoPhaseClock};

use super::wiring::Hp67Net;

const CLOCK_DRIVER: DriverId = DriverId::new("hp67-act-clock-scaffold");

/// Pin-level connection fabric for HP-67 devices.
#[derive(Debug, Clone)]
pub struct Hp67ElectricalBackplane {
    clock: TwoPhaseClock,
    nets: BTreeMap<Hp67Net, Net>,
}

impl Default for Hp67ElectricalBackplane {
    fn default() -> Self {
        let nets = Hp67Net::ALL
            .into_iter()
            .map(|net| (net, Net::new(Bias::Floating)))
            .collect();
        Self {
            clock: TwoPhaseClock::default(),
            nets,
        }
    }
}

impl Hp67ElectricalBackplane {
    pub fn tick(&self) -> Tick {
        self.clock.tick()
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

    /// Advance the timing scaffold by one non-overlapping clock sub-phase.
    pub fn advance_clock(&mut self) -> Tick {
        let levels = self.clock.advance();
        self.drive(
            Hp67Net::Phi1,
            CLOCK_DRIVER,
            if levels.phi1 { Drive::High } else { Drive::Low },
        );
        self.drive(
            Hp67Net::Phi2,
            CLOCK_DRIVER,
            if levels.phi2 { Drive::High } else { Drive::Low },
        );
        self.clock.tick()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_declared_nets_exist_and_begin_floating() {
        let backplane = Hp67ElectricalBackplane::default();
        for net in Hp67Net::ALL {
            assert_eq!(backplane.level(net), LogicLevel::Floating);
        }
    }

    #[test]
    fn scaffold_clock_never_drives_both_phases_high() {
        let mut backplane = Hp67ElectricalBackplane::default();
        for _ in 0..64 {
            backplane.advance_clock();
            let phi1 = backplane.level(Hp67Net::Phi1) == LogicLevel::High;
            let phi2 = backplane.level(Hp67Net::Phi2) == LogicLevel::High;
            assert!(!(phi1 && phi2));
        }
    }
}
