//! Research-only helpers for importing and comparing external ROM corpora.
//!
//! None of the code in this module is part of the emulated HP-67 hardware.
//! It exists so independently sourced firmware dumps and emulator containers
//! can be decoded, normalized and compared without embedding copyrighted ROM
//! bytes in the emulator.

pub mod nonpareil_obj;
pub mod rom_corpus;
pub mod teenix;
pub mod teenix_hp67;
pub mod woodstock_asm;
