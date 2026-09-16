//! HP-67 machine-specific electrical model.
//!
//! This module keeps wiring inventory, word timing, IS/ISA serial behavior, the
//! structural ACT<->ROM fetch path, the ROM0/cathode display path and the
//! independent ACT+CRC architectural bring-up composition explicit before final
//! pin/timing-accurate devices are completed.

pub mod act;
pub mod architectural;
pub mod crc;
pub mod display;
pub mod display_snapshot;
pub mod fetch;
pub mod isa;
pub mod machine;
pub mod timing;
pub mod wiring;

pub use act::{
    display_register_index_for_scan_slot, ActArchitecturalCore, ActArchitecturalState,
    ActDisplaySerialError, ActDisplayWordSerializer, ActError, ActExecution, ActInstructionState,
    ActOperation, ActRamImage, ActRegister, PowerOnActCore, PowerOnActError, PowerOnExecution,
    PowerOnOperation, ACT_RAM_WORDS, ACT_RETURN_STACK_DEPTH, ACT_STATUS_BITS, ACT_WORD_DIGITS,
};
pub use architectural::{
    Hp67ArchitecturalError, Hp67ArchitecturalExecution, Hp67ArchitecturalMachine,
    Hp67ArchitecturalOperation,
};
pub use crc::{
    decode_crc_opcode, CrcArchitecturalCore, CrcArchitecturalError, CrcInstruction, CRC_FLAG_COUNT,
    CRC_FLAG_PROGRAM_MODE, CRC_RAM_READ_ADDRESS, CRC_RAM_WRITE_ADDRESS,
};
pub use display::{
    decode_rom0_display_byte, display_role_for_scan_slot, CathodeDriver1820_1749, Hp67DisplayRole,
    Hp67SegmentMask, Rom0DisplayEndpoint, Rom0DisplayError, HP67_DISPLAY_SCAN_SLOTS,
    HP67_SHARED_SIGN_SLOT,
};
pub use display_snapshot::{
    display_byte_from_act_registers, structural_display_scan_from_act_registers,
    StructuralDisplaySlot,
};
pub use fetch::{
    run_structural_display_fetch_cycle, run_structural_fetch_cycle, ActSerialEndpoint,
    FetchPipelineLatch, Hp67RomWordSource, RomFetchEndpoint, SerialFetchError, StructuralWordError,
    StructuralWordResult,
};
pub use isa::{
    act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK,
};
pub use machine::Hp67ElectricalBackplane;
pub use timing::{
    display_data_serial_bit, isa_window_for_bit, sync_decision_window, Hp67WordTiming, IsaWindow,
    BITS_PER_DIGIT, BITS_PER_WORD, DIGITS_PER_WORD, DISPLAY_DATA_BITS, DISPLAY_DATA_FIRST_BIT,
    DISPLAY_DATA_LAST_BIT, DISPLAY_STR_BIT, HP67_OBSERVED_DISPLAY_REFRESH_US,
    HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,
    HP67_OBSERVED_WORD_TIME_US, ROM_ADDRESS_BITS, ROM_ADDRESS_FIRST_BIT, ROM_ADDRESS_LAST_BIT,
    ROM_WORD_BITS, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT,
};
pub use wiring::{Hp67Chip, Hp67Net, CHIPSET};
