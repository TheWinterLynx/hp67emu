//! HP-67 machine-specific electrical model.
//!
//! This module keeps wiring inventory, word timing, IS/ISA serial behavior, the
//! structural ACT<->ROM fetch path and the independent ACT architectural
//! bring-up core explicit before final pin/timing-accurate ACT, ROM/RAM,
//! display, keyboard and card-reader devices are completed.

pub mod act;
pub mod fetch;
pub mod isa;
pub mod machine;
pub mod timing;
pub mod wiring;

pub use act::{
    ActArchitecturalCore, ActArchitecturalState, ActError, ActExecution, ActInstructionState,
    ActOperation, ActRamImage, ActRegister, PowerOnActCore, PowerOnActError, PowerOnExecution,
    PowerOnOperation, ACT_RAM_WORDS, ACT_RETURN_STACK_DEPTH, ACT_STATUS_BITS, ACT_WORD_DIGITS,
};
pub use fetch::{
    run_structural_fetch_cycle, ActFetchEndpoint, FetchPipelineLatch, Hp67RomWordSource,
    RomFetchEndpoint, SerialFetchError,
};
pub use isa::{act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK};
pub use machine::Hp67ElectricalBackplane;
pub use timing::{
    isa_window_for_bit, sync_decision_window, Hp67WordTiming, IsaWindow, BITS_PER_DIGIT,
    BITS_PER_WORD, DIGITS_PER_WORD, ROM_ADDRESS_BITS, ROM_ADDRESS_FIRST_BIT,
    ROM_ADDRESS_LAST_BIT, ROM_WORD_BITS, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT,
};
pub use wiring::{Hp67Chip, Hp67Net, CHIPSET};
