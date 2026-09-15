//! HP-67 machine-specific electrical model.
//!
//! This module keeps wiring inventory, word timing and IS/ISA serial behavior
//! explicit before full ACT, ROM/RAM, display, keyboard and card-reader devices
//! are added, so the generic electrical kernel remains reusable.

pub mod isa;
pub mod machine;
pub mod timing;
pub mod wiring;

pub use isa::{act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK};
pub use machine::Hp67ElectricalBackplane;
pub use timing::{
    isa_window_for_bit, sync_decision_window, Hp67WordTiming, IsaWindow, BITS_PER_DIGIT,
    BITS_PER_WORD, DIGITS_PER_WORD, ROM_ADDRESS_BITS, ROM_ADDRESS_FIRST_BIT,
    ROM_ADDRESS_LAST_BIT, ROM_WORD_BITS, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT,
};
pub use wiring::{Hp67Chip, Hp67Net, CHIPSET};
