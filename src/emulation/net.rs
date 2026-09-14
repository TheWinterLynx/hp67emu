//! Pin-level digital net resolution with high-impedance and contention states.
//!
//! The goal is not a SPICE solver. It is an explicit electrical boundary where
//! multiple emulated IC pins can drive or release a shared wire and where bus
//! conflicts become observable instead of being hidden by call ordering.

use std::collections::BTreeMap;

/// Stable identity of one output driver attached to a net.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DriverId(&'static str);

impl DriverId {
    pub const fn new(name: &'static str) -> Self {
        Self(name)
    }

    pub const fn name(self) -> &'static str {
        self.0
    }
}

/// What one emulated output pin is doing to a wire.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Drive {
    HighZ,
    Low,
    High,
}

/// Passive behavior of an otherwise undriven net.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Bias {
    Floating,
    PullDown,
    PullUp,
}

/// Resolved electrical state visible to input pins.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LogicLevel {
    Floating,
    Low,
    High,
    Contention,
}

/// Shared wire with any number of named drivers.
#[derive(Debug, Clone)]
pub struct Net {
    bias: Bias,
    drives: BTreeMap<DriverId, Drive>,
}

impl Net {
    pub fn new(bias: Bias) -> Self {
        Self {
            bias,
            drives: BTreeMap::new(),
        }
    }

    pub fn bias(&self) -> Bias {
        self.bias
    }

    pub fn set_bias(&mut self, bias: Bias) {
        self.bias = bias;
    }

    /// Drive a net or release it. High-Z drivers are removed from the active map.
    pub fn set_drive(&mut self, driver: DriverId, drive: Drive) {
        if drive == Drive::HighZ {
            self.drives.remove(&driver);
        } else {
            self.drives.insert(driver, drive);
        }
    }

    pub fn release(&mut self, driver: DriverId) {
        self.drives.remove(&driver);
    }

    pub fn active_driver_count(&self) -> usize {
        self.drives.len()
    }

    /// Active drivers in deterministic `DriverId` order.
    pub fn active_drives(&self) -> impl Iterator<Item = (DriverId, Drive)> + '_ {
        self.drives.iter().map(|(driver, drive)| (*driver, *drive))
    }

    pub fn level(&self) -> LogicLevel {
        let mut low = false;
        let mut high = false;

        for drive in self.drives.values().copied() {
            match drive {
                Drive::HighZ => {}
                Drive::Low => low = true,
                Drive::High => high = true,
            }
        }

        match (low, high) {
            (true, true) => LogicLevel::Contention,
            (true, false) => LogicLevel::Low,
            (false, true) => LogicLevel::High,
            (false, false) => match self.bias {
                Bias::Floating => LogicLevel::Floating,
                Bias::PullDown => LogicLevel::Low,
                Bias::PullUp => LogicLevel::High,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    const A: DriverId = DriverId::new("A");
    const B: DriverId = DriverId::new("B");

    #[test]
    fn floating_and_pull_biases_resolve_without_a_driver() {
        assert_eq!(Net::new(Bias::Floating).level(), LogicLevel::Floating);
        assert_eq!(Net::new(Bias::PullDown).level(), LogicLevel::Low);
        assert_eq!(Net::new(Bias::PullUp).level(), LogicLevel::High);
    }

    #[test]
    fn released_driver_does_not_mask_the_bias() {
        let mut net = Net::new(Bias::PullUp);
        net.set_drive(A, Drive::Low);
        assert_eq!(net.level(), LogicLevel::Low);
        net.set_drive(A, Drive::HighZ);
        assert_eq!(net.level(), LogicLevel::High);
        assert_eq!(net.active_driver_count(), 0);
    }

    #[test]
    fn opposite_active_drives_are_visible_as_contention() {
        let mut net = Net::new(Bias::Floating);
        net.set_drive(A, Drive::High);
        net.set_drive(B, Drive::Low);
        assert_eq!(net.level(), LogicLevel::Contention);
    }

    #[test]
    fn active_driver_diagnostics_are_stably_sorted() {
        let mut net = Net::new(Bias::Floating);
        net.set_drive(B, Drive::Low);
        net.set_drive(A, Drive::High);
        let observed: Vec<_> = net.active_drives().collect();
        assert_eq!(observed, vec![(A, Drive::High), (B, Drive::Low)]);
    }
}
