//! Generic electrical/cycle simulation primitives.
//!
//! This layer knows nothing about HP-67 key labels, photographs or egui.  It
//! provides explicit time, electrical net resolution, device contracts and trace
//! capture that calculator-specific machines can build on.

pub mod clock;
pub mod device;
pub mod net;
pub mod trace;

pub use clock::{ClockLevels, Tick, TwoPhaseClock};
pub use device::{DriveSink, ElectricalDevice, NetReader};
pub use net::{Bias, Drive, DriverId, LogicLevel, Net};
pub use trace::{Trace, TraceSample};
