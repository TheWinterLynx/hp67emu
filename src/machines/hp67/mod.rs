//! HP-67 machine-specific electrical model.
//!
//! This module intentionally starts with wiring inventory, canonical serial-word
//! timing coordinates and a backplane shell. ACT, ROM/RAM, display, keyboard and
//! card-reader devices will be added as separate modules so the generic
//! electrical kernel remains reusable.

pub mod machine;
pub mod timing;
pub mod wiring;

pub use machine::Hp67ElectricalBackplane;
pub use timing::{Hp67WordTiming, BITS_PER_DIGIT, BITS_PER_WORD, DIGITS_PER_WORD};
pub use wiring::{Hp67Chip, Hp67Net, CHIPSET};
