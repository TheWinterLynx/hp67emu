//! Resolved HDS/WPS reader contact inputs and retained CRC head/write gate.
//!
//! HP-97 service 2-37/2-39 and note 97-101 ground HDS at head arrival and
//! interrogate write permission there. Grounded WPS permits writing. Normalized
//! High/open levels and joint logical publication do not establish analog bias,
//! mechanical geometry, relative contact edges or PHI sampling timing.
use super::{Hp67Driver, Hp67ElectricalBackplane, Hp67Net};
use crate::emulation::{Drive, LogicLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67ReaderContactInputs {
    pub head_active: bool,
    pub write_permitted: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ReaderContactError {
    InvalidPin { tick: u64, net: Hp67Net, level: LogicLevel, active_drivers: u8, contact_drive: Drive },
    ChangedSource { expected: Hp67ReaderContactInputs, actual: Hp67ReaderContactInputs },
}

pub fn sample_reader_contacts(bus: &Hp67ElectricalBackplane) -> Result<Hp67ReaderContactInputs, Hp67ReaderContactError> {
    let mut closed = [false; 2];
    for (index, net) in [Hp67Net::Hds, Hp67Net::Wps].into_iter().enumerate() {
        let level = bus.level(net);
        let active_drivers = bus.active_driver_count(net);
        let contact_drive = bus.driver_drive(net, Hp67Driver::External);
        if active_drivers != 1 || contact_drive == Drive::HighZ || !matches!(level, LogicLevel::Low | LogicLevel::High) {
            return Err(Hp67ReaderContactError::InvalidPin { tick: bus.tick().get(), net, level, active_drivers, contact_drive });
        }
        closed[index] = level == LogicLevel::Low;
    }
    Ok(Hp67ReaderContactInputs { head_active: closed[0], write_permitted: closed[1] })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67ReaderContacts { held: Hp67ReaderContactInputs }
impl Hp67ReaderContacts {
    pub fn attach(bus: &mut Hp67ElectricalBackplane) -> Self {
        bus.commit_reader_contacts(Drive::High, Drive::High);
        Self::default()
    }
    pub fn check(&self, bus: &Hp67ElectricalBackplane) -> Result<Hp67ReaderContactInputs, Hp67ReaderContactError> {
        let actual = sample_reader_contacts(bus)?;
        if actual != self.held { return Err(Hp67ReaderContactError::ChangedSource { expected: self.held, actual }); }
        Ok(actual)
    }
    /// A joint software publication is not a claim that both physical contacts
    /// move simultaneously. No receiver/capture sees an intermediate pair.
    pub fn publish(&mut self, bus: &mut Hp67ElectricalBackplane, next: Hp67ReaderContactInputs) -> Result<(), Hp67ReaderContactError> {
        self.check(bus)?;
        if next != self.held {
            bus.commit_reader_contacts(if next.head_active { Drive::Low } else { Drive::High }, if next.write_permitted { Drive::Low } else { Drive::High });
            self.held = next;
        }
        Ok(())
    }
}

/// Independent CRC-side receiver: WPS is sampled on logical HDS closure, then
/// retained until the head opens. Host media metadata is not a receiver input.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67CrcReaderGate { head_active: bool, write_permitted: bool, wps_rearmed: bool }
impl Default for Hp67CrcReaderGate {
    fn default() -> Self {
        Self { head_active: false, write_permitted: false, wps_rearmed: true }
    }
}
impl Hp67CrcReaderGate {
    pub const fn head_active(&self) -> bool { self.head_active }
    pub const fn write_permitted(&self) -> bool { self.write_permitted }
    pub fn sample_contacts(&mut self, bus: &Hp67ElectricalBackplane) -> Result<(), Hp67ReaderContactError> {
        let inputs = sample_reader_contacts(bus)?;
        // Service note 97-101 requires opening/reclosing grounded WPS on
        // each pass. Reset starts armed as an explicit logical bridge policy.
        if !inputs.write_permitted { self.wps_rearmed = true; }
        if !inputs.head_active {
            self.write_permitted = false;
        } else if !self.head_active {
            self.write_permitted = inputs.write_permitted && self.wps_rearmed;
            if inputs.write_permitted { self.wps_rearmed = false; }
        }
        self.head_active = inputs.head_active;
        Ok(())
    }
}

#[cfg(test)]
#[path = "reader_contacts_tests.rs"]
mod tests;
