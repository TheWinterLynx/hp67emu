//! Reusable, UI-independent emulation library for hp67emu.
//!
//! The binary front end lives in `src/main.rs` and the egui modules.  Everything
//! in this library is intended to remain deterministic, headless-testable and
//! reusable by more than one calculator model.

pub mod emulation;
pub mod machines;
