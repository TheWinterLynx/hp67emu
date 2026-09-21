//! HP-67 machine-specific electrical model.
//!
//! This module keeps wiring inventory, word timing, IS/ISA serial behavior, the
//! structural ACT<->ROM fetch path, the ROM0/cathode display path and the
//! independent ACT+CRC architectural bring-up composition explicit before final
//! pin/timing-accurate devices are completed.

pub mod act;
pub mod act_serial_execution;
pub mod act_serial_result;
pub mod act_serial_state;
pub mod architectural;
pub mod card_flux;
pub mod card_transport;
pub mod crc;
pub mod display;
pub mod display_snapshot;
pub mod fetch;
pub mod hp67firmware;
pub mod isa;
pub mod keyboard;
pub mod machine;
pub mod magnetic_card;
pub mod timing;
pub mod wiring;

pub use act::{
    display_register_index_for_scan_slot, ActArchitecturalCore, ActArchitecturalState,
    ActDisplaySerialError, ActDisplayWordSerializer, ActError, ActExecution, ActInstructionState,
    ActOperation, ActRamImage, ActRegister, PowerOnActCore, PowerOnActError, PowerOnExecution,
    PowerOnOperation, ACT_RAM_WORDS, ACT_RETURN_STACK_DEPTH, ACT_STATUS_BITS, ACT_WORD_DIGITS,
};
pub use act_serial_execution::{
    decode_serial_arithmetic_action, ActSerialArithmeticAction, ActSerialArithmeticCoordinate,
    ActSerialExecution, ActSerialExecutionError, ActSerialOperand, ActSerialRegister,
    ActSerialWordClass,
};
pub use act_serial_result::ActSerialArithmeticResultImage;
pub use act_serial_state::{ActSerialAluInputs, ActSerialDigitAluResult, ActSerialStateSnapshot};
pub use architectural::{
    Hp67ArchitecturalError, Hp67ArchitecturalExecution, Hp67ArchitecturalMachine,
    Hp67ArchitecturalOperation,
};
pub use card_flux::{
    Hp67FluxCell, Hp67PhysicalFluxTrack, Hp67SelfClockingFluxPair,
    HP67_PHYSICAL_FLUX_TRACKS_PER_LOGICAL_TRACK,
};
pub use card_transport::{
    Hp67CardSpeed, Hp67CardSpeedError, Hp67CardTransport, Hp67CardTransportError,
    HP67_MAX_CARD_SPEED_PERCENT, HP67_MIN_CARD_SPEED_PERCENT, HP67_NOMINAL_CARD_RECORD_US,
};
pub use crc::{
    decode_crc_opcode, CrcArchitecturalCore, CrcArchitecturalError, CrcInstruction,
    CRC_CARD_WORD_BITS, CRC_CARD_WORD_MASK, CRC_FLAG_BUFFER_READY, CRC_FLAG_CARD_PRESENT,
    CRC_FLAG_COUNT, CRC_FLAG_F7_STATUS, CRC_FLAG_MOTOR_ON, CRC_FLAG_PROGRAM_MODE,
    CRC_FLAG_WRITE_MODE, CRC_RAM_READ_ADDRESS, CRC_RAM_WRITE_ADDRESS, CRC_READ_BUFFER_COUNT,
    CRC_WRITE_BUFFER_COUNT,
};
pub use display::{
    decode_rom0_display_byte, display_role_for_scan_slot, CathodeDriver1820_1749, CathodeScanError,
    Hp67DisplayRole, Hp67SegmentMask, Rom0DisplayEndpoint, Rom0DisplayError, Rom0StrEvent,
    HP67_DISPLAY_SCAN_SLOTS, HP67_SHARED_SIGN_SLOT,
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
pub use hp67firmware::Hp67Firmware;
pub use isa::{
    act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK,
};
pub use keyboard::{Hp67Key, Hp67Keyboard, HP67_KEY_PRESSED_STATUS_BIT};
pub use machine::Hp67ElectricalBackplane;
pub use magnetic_card::{
    CardInsertionEnd, Hp67CardTrack, Hp67MagneticCard, Hp67MagneticCardError, Hp67MagneticTrack,
    TeenixHppError, TeenixHppImport, TrackMedia, HP67_CARD_BITS_PER_TRACK,
    HP67_CARD_CONTAINER_BYTES, HP67_CARD_LOGICAL_BYTES, HP67_CARD_LOGICAL_BYTES_PER_TRACK,
    HP67_CARD_RECORDS_PER_TRACK,
};
pub use timing::{
    data_serial_bit_for_word_bit, display_data_serial_bit, isa_window_for_bit,
    sync_decision_window, Hp67WordTiming, IsaWindow, BITS_PER_DIGIT, BITS_PER_WORD,
    DATA_STREAM_BITS, DATA_STREAM_FIRST_WORD_BIT, DIGITS_PER_WORD, DISPLAY_DATA_BITS,
    DISPLAY_DATA_FIRST_BIT, DISPLAY_DATA_LAST_BIT, DISPLAY_STR_BIT, HP67_OBSERVED_DISPLAY_DP_ON_US,
    HP67_OBSERVED_DISPLAY_DP_TO_STR_GAP_US, HP67_OBSERVED_DISPLAY_REFRESH_US,
    HP67_OBSERVED_DISPLAY_SEGMENT_ON_US, HP67_OBSERVED_DISPLAY_STR_PULSE_US,
    HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,
    HP67_OBSERVED_WORD_TIME_US, ROM_ADDRESS_BITS, ROM_ADDRESS_FIRST_BIT, ROM_ADDRESS_LAST_BIT,
    ROM_WORD_BITS, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT,
};
pub use wiring::{Hp67Chip, Hp67Net, CHIPSET};
