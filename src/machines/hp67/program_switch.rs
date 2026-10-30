//! Physical RUN/W-PRGM selector -> resolved CRC /PGMSW input.
//!
//! HP-67 schematic keyboard/display sheet 1 connects the PGM switch contact
//! to ground; logic sheet 2 routes /PGMSW to CRC SEL. High/RUN normalization
//! does not establish idle bias, voltage, bounce or the CRC sampling edge.
use super::{Hp67Driver, Hp67ElectricalBackplane, Hp67Net};
use crate::emulation::{Drive, LogicLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ProgramSwitchError {
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

/// Receiver consumes the conductor, independently of the architectural mode observer.
pub fn sample_program_mode(bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67ProgramSwitchError> {
    let level = bus.level(Hp67Net::PgmSw);
    let active_drivers = bus.active_driver_count(Hp67Net::PgmSw);
    let contact_drive = bus.driver_drive(Hp67Net::PgmSw, Hp67Driver::External);
    if active_drivers != 1
        || contact_drive == Drive::HighZ
        || !matches!(level, LogicLevel::Low | LogicLevel::High)
    {
        return Err(Hp67ProgramSwitchError::InvalidPin {
            tick: bus.tick().get(),
            level,
            active_drivers,
            contact_drive,
        });
    }
    Ok(level == LogicLevel::Low)
}

/// Stores the last physical input publication so faults cannot repair themselves
/// during selector changes or repeated UI publication.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67ProgramSwitch {
    program: bool,
}
impl Hp67ProgramSwitch {
    pub fn attach(bus: &mut Hp67ElectricalBackplane, program: bool) -> Self {
        bus.drive(Hp67Net::PgmSw, Hp67Driver::External, if program { Drive::Low } else { Drive::High });
        Self { program }
    }
    /// Physical selector position retained across electronic reset.
    pub const fn program_selected(&self) -> bool { self.program }
    pub fn check(&self, bus: &Hp67ElectricalBackplane) -> Result<bool, Hp67ProgramSwitchError> {
        let actual = sample_program_mode(bus)?;
        if actual != self.program {
            return Err(Hp67ProgramSwitchError::ChangedSource {
                expected: self.program,
                actual,
            });
        }
        Ok(actual)
    }
    /// A selector event may occur between machine words. No calibrated
    /// contact/PHI edge or debounce policy is inferred from this publication.
    pub fn set_program(
        &mut self,
        bus: &mut Hp67ElectricalBackplane,
        program: bool,
    ) -> Result<(), Hp67ProgramSwitchError> {
        self.check(bus)?;
        if program != self.program {
            bus.drive(
                Hp67Net::PgmSw,
                Hp67Driver::External,
                if program { Drive::Low } else { Drive::High },
            );
            self.program = program;
        }
        Ok(())
    }
}


#[cfg(test)]
#[path = "program_switch_tests.rs"]
mod tests;
