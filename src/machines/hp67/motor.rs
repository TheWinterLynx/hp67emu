//! CRC MOTOR -> sense-amplifier/transport logical-level output.
//!
//! Active Low follows HP-67 barred MOT wiring and HP-97 service section 2-36.
//! Push-pull normalization and word-boundary publication remain approximations.
use super::{
    Hp67Driver, Hp67ElectricalBackplane, Hp67Net, Hp67SerialCrcControl, CRC_FLAG_MOTOR_ON,
};
use crate::emulation::{Drive, LogicLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67MotorError {
    Phase {
        tick: u64,
        bit: u8,
    },
    InvalidPin {
        tick: u64,
        level: LogicLevel,
        active_drivers: u8,
        crc_drive: Drive,
    },
    ChangedSource {
        expected: bool,
        actual: bool,
    },
}

/// Read the actual conductor. Same-level extra drivers are still invalid.
pub fn sample_motor(bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67MotorError> {
    let level = bus.level(Hp67Net::Motor);
    let active_drivers = bus.active_driver_count(Hp67Net::Motor);
    let crc_drive = bus.driver_drive(Hp67Net::Motor, Hp67Driver::CardReaderController1820_1751);
    if active_drivers != 1
        || crc_drive == Drive::HighZ
        || !matches!(level, LogicLevel::Low | LogicLevel::High)
    {
        return Err(Hp67MotorError::InvalidPin {
            tick: bus.tick().get(),
            level,
            active_drivers,
            crc_drive,
        });
    }
    Ok(level == LogicLevel::Low)
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67MotorOutput {
    on: bool,
}
impl Hp67MotorOutput {
    pub fn attach(bus: &mut Hp67ElectricalBackplane) -> Self {
        bus.drive(
            Hp67Net::Motor,
            Hp67Driver::CardReaderController1820_1751,
            Drive::High,
        );
        Self::default()
    }
    /// Preflight cannot hide a fault by overwriting the previously held source.
    pub fn check(&self, bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67MotorError> {
        let actual = sample_motor(bus)?;
        if actual != self.on {
            return Err(Hp67MotorError::ChangedSource {
                expected: self.on,
                actual,
            });
        }
        Ok(actual)
    }
    /// This is a retained output, not a one-shot pulse. Only independent CRC
    /// latches supply it; the architectural mirror is never an output parameter.
    pub fn publish(
        &mut self,
        crc: &Hp67SerialCrcControl,
        bus: &mut Hp67ElectricalBackplane,
    ) -> Result<(), Hp67MotorError> {
        self.check(bus)?;
        if !bus.at_transport_launch_boundary() || bus.word_bit() != 0 {
            return Err(Hp67MotorError::Phase {
                tick: bus.tick().get(),
                bit: bus.word_bit(),
            });
        }
        let on = crc.flags()[CRC_FLAG_MOTOR_ON];
        if self.on != on {
            bus.drive(
                Hp67Net::Motor,
                Hp67Driver::CardReaderController1820_1751,
                if on { Drive::Low } else { Drive::High },
            );
            self.on = on;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "motor_tests.rs"]
mod tests;
