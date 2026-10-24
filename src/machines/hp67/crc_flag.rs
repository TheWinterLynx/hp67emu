//! Resolved CRC FLAG -> ACT F2 logical-level bridge.
//!
//! Wiring and active-low barred FLAG polarity follow the HP-67 schematic.
//! Boundary publication and push-pull
//! levels are an explicit working bridge, not measured pulse width/output stage.
use super::{Hp67Driver, Hp67ElectricalBackplane, Hp67Net, Hp67SerialCrcControl};
use crate::emulation::{Drive, LogicLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CrcFlagError {
    Phase { tick: u64, bit: u8 },
    InvalidPin { tick: u64, level: LogicLevel, active_drivers: u8, crc_drive: Drive },
    ChangedSource { expected: bool, actual: bool },
}

/// Retains only the last published source state, never ACT S3 or host flags.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67CrcFlagBridge { asserted: bool }

/// ACT-facing sampler: consume the resolved pin, not the CRC result parameter.
/// An agreeing foreign source is invalid even though the level is binary.
pub fn sample_crc_f2(bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67CrcFlagError> {
    let level = bus.level(Hp67Net::F2);
    let active_drivers = bus.active_driver_count(Hp67Net::F2);
    let crc_drive = bus.driver_drive(Hp67Net::F2, Hp67Driver::CardReaderController1820_1751);
    if active_drivers != 1 || crc_drive == Drive::HighZ || !matches!(level, LogicLevel::Low | LogicLevel::High) {
        return Err(Hp67CrcFlagError::InvalidPin { tick: bus.tick().get(), level, active_drivers, crc_drive });
    }
    Ok(level == LogicLevel::Low)
}

impl Hp67CrcFlagBridge {
    pub fn attach(bus: &mut Hp67ElectricalBackplane) -> Self {
        bus.drive(Hp67Net::F2, Hp67Driver::CardReaderController1820_1751, Drive::High);
        Self::default()
    }
    /// Read-only preflight; a fault cannot erase its own evidence by re-driving.
    pub fn check(&self, bus: &Hp67ElectricalBackplane) -> Result<(), Hp67CrcFlagError> {
        let actual = sample_crc_f2(bus)?;
        if actual != self.asserted {
            return Err(Hp67CrcFlagError::ChangedSource { expected: self.asserted, actual });
        }
        Ok(())
    }
    /// Release the previous boundary pulse before new-word transport. Physical
    /// release edges/duration are unresolved; no physical time is assigned here.
    pub fn begin_word(&mut self, bus: &mut Hp67ElectricalBackplane) -> Result<(), Hp67CrcFlagError> {
        self.check(bus)?;
        if !bus.at_transport_launch_boundary() || !matches!(bus.word_bit(), 0 | 2) {
            return Err(Hp67CrcFlagError::Phase { tick: bus.tick().get(), bit: bus.word_bit() });
        }
        self.publish_level(bus, false);
        Ok(())
    }
    /// Source is the autonomous receiver's decoded result, never a mirror flag.
    pub fn publish(&mut self, crc: &Hp67SerialCrcControl, bus: &mut Hp67ElectricalBackplane) -> Result<(), Hp67CrcFlagError> {
        self.check(bus)?;
        if !bus.at_transport_launch_boundary() || bus.word_bit() != 0 {
            return Err(Hp67CrcFlagError::Phase { tick: bus.tick().get(), bit: bus.word_bit() });
        }
        let asserted = crc.result().is_some_and(|result| result.condition == Some(true));
        self.publish_level(bus, asserted);
        Ok(())
    }
    fn publish_level(&mut self, bus: &mut Hp67ElectricalBackplane, asserted: bool) {
        if self.asserted != asserted {
            bus.drive(Hp67Net::F2, Hp67Driver::CardReaderController1820_1751, if asserted { Drive::Low } else { Drive::High });
            self.asserted = asserted;
        }
    }
}

#[cfg(test)]
#[path = "crc_flag_tests.rs"]
mod tests;
