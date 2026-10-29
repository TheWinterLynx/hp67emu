//! MTRS card-insertion contact -> resolved CRC input.
//!
//! HP-97 service 2-36 grounds MTRS on insertion; HP-67 schematic names the
//! same CRC input. Normalized High/Open and Low/Closed are a working bridge,
//! not a claim about physical pull-up voltage, bounce or sampling edge.
use super::{Hp67Driver, Hp67ElectricalBackplane, Hp67Net};
use crate::emulation::{Drive, LogicLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67CardContactError {
    InvalidPin {
        tick: u64,
        level: LogicLevel,
        active_drivers: u8,
        contact_drive: Drive,
    },
    ChangedSource {
        expected: bool,
        actual: bool,
    },
}

/// Receiver consumes the conductor, independently of the host card object.
pub fn sample_card_present(bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67CardContactError> {
    let level = bus.level(Hp67Net::Mtrs);
    let active_drivers = bus.active_driver_count(Hp67Net::Mtrs);
    let contact_drive = bus.driver_drive(Hp67Net::Mtrs, Hp67Driver::External);
    if active_drivers != 1
        || contact_drive == Drive::HighZ
        || !matches!(level, LogicLevel::Low | LogicLevel::High)
    {
        return Err(Hp67CardContactError::InvalidPin {
            tick: bus.tick().get(),
            level,
            active_drivers,
            contact_drive,
        });
    }
    Ok(level == LogicLevel::Low)
}

/// Stores the last physical input publication so faults cannot repair themselves
/// during insertion, withdrawal, or completed-card exit.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67CardPresentContact {
    closed: bool,
}
impl Hp67CardPresentContact {
    pub fn attach(bus: &mut Hp67ElectricalBackplane) -> Self {
        bus.drive(Hp67Net::Mtrs, Hp67Driver::External, Drive::High);
        Self::default()
    }
    pub fn check(&self, bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67CardContactError> {
        let actual = sample_card_present(bus)?;
        if actual != self.closed {
            return Err(Hp67CardContactError::ChangedSource {
                expected: self.closed,
                actual,
            });
        }
        Ok(actual)
    }
    /// A host physical event may occur between machine words. No calibrated
    /// contact/PHI edge or debounce policy is inferred from this publication.
    pub fn set_closed(
        &mut self,
        bus: &mut Hp67ElectricalBackplane,
        closed: bool,
    ) -> Result<(), Hp67CardContactError> {
        self.check(bus)?;
        if closed != self.closed {
            bus.drive(
                Hp67Net::Mtrs,
                Hp67Driver::External,
                if closed { Drive::Low } else { Drive::High },
            );
            self.closed = closed;
        }
        Ok(())
    }
}

#[cfg(test)]
#[path = "card_contact_tests.rs"]
mod tests;
