//! Order-independent contracts for electrical devices.
//!
//! Devices sample a resolved snapshot through `NetReader` and publish their next
//! output drives through `DriveSink`.  A future scheduler will resolve all device
//! outputs only after every device has evaluated the same tick, preventing Rust
//! iteration order from becoming accidental hardware timing.

use super::{Drive, DriverId, LogicLevel, Tick};

/// Read-only view of resolved nets for one simulation instant.
pub trait NetReader<N: Copy> {
    fn read(&self, net: N) -> LogicLevel;
}

/// Collector for output pin drives produced during one device evaluation.
pub trait DriveSink<N: Copy> {
    fn drive(&mut self, net: N, driver: DriverId, drive: Drive);
}

/// Contract implemented by emulated ICs or other electrical subassemblies.
pub trait ElectricalDevice<N: Copy> {
    fn device_name(&self) -> &'static str;

    /// Evaluate one scheduler tick from a stable input snapshot.
    fn evaluate(
        &mut self,
        tick: Tick,
        inputs: &dyn NetReader<N>,
        outputs: &mut dyn DriveSink<N>,
    );
}
