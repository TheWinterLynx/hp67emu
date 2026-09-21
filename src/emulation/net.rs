//! Pin-level digital net resolution with high-impedance and contention states.
//!
//! The goal is not a SPICE solver. It is an explicit electrical boundary where
//! multiple emulated IC pins can drive or release a shared wire and where bus
//! conflicts become observable instead of being hidden by call ordering.

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
    drives: Vec<(DriverId, Drive)>,
    low_driver_count: usize,
    high_driver_count: usize,
}

impl Net {
    pub fn new(bias: Bias) -> Self {
        Self {
            bias,
            drives: Vec::new(),
            low_driver_count: 0,
            high_driver_count: 0,
        }
    }

    pub fn bias(&self) -> Bias {
        self.bias
    }

    pub fn set_bias(&mut self, bias: Bias) {
        self.bias = bias;
    }

    fn remove_drive_count(&mut self, drive: Drive) {
        match drive {
            Drive::HighZ => {}
            Drive::Low => self.low_driver_count -= 1,
            Drive::High => self.high_driver_count -= 1,
        }
    }

    fn add_drive_count(&mut self, drive: Drive) {
        match drive {
            Drive::HighZ => {}
            Drive::Low => self.low_driver_count += 1,
            Drive::High => self.high_driver_count += 1,
        }
    }

    /// Drive a net or release it.
    ///
    /// Electrical nets normally have only one or two active output pins. A
    /// sorted compact vector avoids tree allocation/traversal on every PHI edge
    /// while preserving deterministic DriverId ordering for diagnostics.
    pub fn set_drive(&mut self, driver: DriverId, drive: Drive) {
        match self
            .drives
            .binary_search_by_key(&driver, |(candidate, _)| *candidate)
        {
            Ok(index) => {
                let previous = self.drives[index].1;
                if drive == Drive::HighZ {
                    self.remove_drive_count(previous);
                    self.drives.remove(index);
                } else if previous != drive {
                    self.remove_drive_count(previous);
                    self.drives[index].1 = drive;
                    self.add_drive_count(drive);
                }
            }
            Err(index) if drive != Drive::HighZ => {
                self.drives.insert(index, (driver, drive));
                self.add_drive_count(drive);
            }
            Err(_) => {}
        }
    }

    pub fn release(&mut self, driver: DriverId) {
        self.set_drive(driver, Drive::HighZ);
    }

    pub fn active_driver_count(&self) -> usize {
        self.drives.len()
    }

    /// Active drivers in deterministic `DriverId` order.
    pub fn active_drives(&self) -> impl Iterator<Item = (DriverId, Drive)> + '_ {
        self.drives.iter().copied()
    }

    /// Resolved electrical state visible to input pins.
    ///
    /// Counts are maintained when output pins change, so the hot-path resolve
    /// is constant time regardless of the number of attached drivers.
    pub fn level(&self) -> LogicLevel {
        match (self.low_driver_count != 0, self.high_driver_count != 0) {
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
    fn replacing_and_releasing_drives_keeps_cached_resolution_exact() {
        let mut net = Net::new(Bias::Floating);
        net.set_drive(A, Drive::Low);
        assert_eq!(net.level(), LogicLevel::Low);
        net.set_drive(A, Drive::High);
        assert_eq!(net.level(), LogicLevel::High);
        net.set_drive(B, Drive::Low);
        assert_eq!(net.level(), LogicLevel::Contention);
        net.release(A);
        assert_eq!(net.level(), LogicLevel::Low);
        net.release(B);
        assert_eq!(net.level(), LogicLevel::Floating);
        assert_eq!(net.active_driver_count(), 0);
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
