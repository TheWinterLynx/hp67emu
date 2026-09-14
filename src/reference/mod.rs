//! Behavioural reference models used to validate the electrical emulator.
//!
//! A reference model is deliberately not the fidelity path.  It may execute a
//! complete microinstruction atomically so that the much lower-level electrical
//! implementation can be checked at well-defined architectural boundaries.

pub mod woodstock;
