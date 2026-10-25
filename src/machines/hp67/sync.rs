//! ACT-owned resolved SYNC input integrity, without a physical edge claim.
use super::{Hp67Driver, Hp67ElectricalBackplane, Hp67Net};
use crate::emulation::{Drive, LogicLevel};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67SyncFault {
    pub tick: u64,
    pub word_bit: u8,
    pub level: LogicLevel,
    pub act_drive: Drive,
    pub active_mask: u16,
}

/// Sole ACT ownership is required even if a foreign source agrees electrically.
/// Normal validation uses committed occupancy metadata; diagnostics are failure-only.
#[inline]
pub fn validate_sync_source(bus: &Hp67ElectricalBackplane) -> Result<(), Hp67SyncFault> {
    if bus.sync_drives_valid()
        && matches!(bus.level(Hp67Net::Sync), LogicLevel::Low | LogicLevel::High)
    {
        return Ok(());
    }
    Err(sync_fault(bus))
}

/// A pristine, undriven tick-zero fixture may publish its first ACT image.
/// This is initialization policy, not an assertion about the reset waveform.
/// Once clocks have advanced, a missing source must not be repaired by launch.
#[inline]
pub fn validate_sync_transport_entry(bus: &Hp67ElectricalBackplane) -> Result<(), Hp67SyncFault> {
    if bus.tick().get() == 0 && bus.active_driver_count(Hp67Net::Sync) == 0 {
        return Ok(());
    }
    validate_sync_source(bus)
}

#[cold]
fn sync_fault(bus: &Hp67ElectricalBackplane) -> Hp67SyncFault {
    Hp67SyncFault {
        tick: bus.tick().get(),
        word_bit: bus.word_bit(),
        level: bus.level(Hp67Net::Sync),
        act_drive: bus.driver_drive(Hp67Net::Sync, Hp67Driver::Act1820_2530),
        active_mask: Hp67Driver::ALL.into_iter().fold(0, |mask, driver| {
            if bus.driver_drive(Hp67Net::Sync, driver) != Drive::HighZ {
                mask | (1 << driver.index())
            } else {
                mask
            }
        }),
    }
}

#[cfg(test)]
#[path = "sync_tests.rs"]
mod tests;
