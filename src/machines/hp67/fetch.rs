//! Structural HP-67 ACT↔ROM serial fetch path.
//!
//! This module models the evidence-backed bit-cell transport on the shared IS/ISA
//! line without yet claiming the exact PHI launch/sample edge. The ACT can emit
//! the eight-bit ROM0 display byte at b0..b7 and the 12-bit ROM address LSB-first
//! at b16..b27 during the same 56-bit word. A ROM-side endpoint reconstructs the
//! address through the resolved electrical net, the selected ROM word is looked
//! up through a caller-supplied source, and the ROM returns the 10-bit word
//! LSB-first at b46..b55 for the ACT endpoint to reconstruct.

use crate::emulation::{Drive, LogicLevel};

use super::{
    act::{display_register_index_for_scan_slot, ActArchitecturalState, ActDisplaySerialError},
    act_serial_execution::{ActSerialExecution, ActSerialExecutionError, ActSerialRegister},
    act_serial_result::ActSerialArithmeticResultImage,
    act_serial_state::{ActSerialAluInputs, ActSerialDigitAluResult, ActSerialStateSnapshot},
    data::{Hp67DataSerialError, Hp67DataSerialWordPath},
    display::{
        Rom0DisplayEndpoint, Rom0DisplayError, Rom0StrEvent, HP67_DISPLAY_SCAN_SLOTS,
        HP67_ROM0_BLANK_CODE,
    },
    isa::{act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},
    machine::Hp67ElectricalBackplane,
    timing::{
        display_data_serial_bit, isa_window_for_bit, IsaWindow, BITS_PER_DIGIT, BITS_PER_WORD,
        ROM_ADDRESS_BITS, ROM_WORD_BITS,
    },
    wiring::{Hp67Driver, Hp67Net},
};

const COMPLETE_ADDRESS_MASK: u16 = (1u16 << ROM_ADDRESS_BITS) - 1;
const COMPLETE_WORD_MASK: u16 = (1u16 << ROM_WORD_BITS) - 1;
const ACT_IS_DRIVER: Hp67Driver = Hp67Driver::Act1820_2530;
const ROM_IS_DRIVER: Hp67Driver = Hp67Driver::StructuralRomResponder;

/// ROM lookup boundary used by the serial fetch responder.
///
/// Physical 1818-* devices will implement this boundary later. Keeping lookup
/// behind a trait lets the serial transport be tested without embedding HP ROM
/// payloads or depending on the semantic reference ROM implementation.
pub trait Hp67RomWordSource {
    fn read_word(&self, address: u16) -> Option<u16>;
}

/// Errors that make a serial fetch electrically or structurally invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialFetchError {
    FloatingIsa { word_bit: u8 },
    IsaContention { word_bit: u8 },
    MissingRomWord { address: u16 },
    IncompleteAddress { received_mask: u16 },
    IncompleteRomWord { received_mask: u16 },
}

/// Errors from a combined structural HP-67 word carrying display and fetch data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralWordError {
    Fetch(SerialFetchError),
    Display(Rom0DisplayError),
    ActDisplay(ActDisplaySerialError),
    ActExecution(ActSerialExecutionError),
    Data(Hp67DataSerialError),
}

impl From<SerialFetchError> for StructuralWordError {
    fn from(error: SerialFetchError) -> Self {
        Self::Fetch(error)
    }
}

impl From<Rom0DisplayError> for StructuralWordError {
    fn from(error: Rom0DisplayError) -> Self {
        Self::Display(error)
    }
}

impl From<ActDisplaySerialError> for StructuralWordError {
    fn from(error: ActDisplaySerialError) -> Self {
        Self::ActDisplay(error)
    }
}

impl From<ActSerialExecutionError> for StructuralWordError {
    fn from(error: ActSerialExecutionError) -> Self {
        Self::ActExecution(error)
    }
}

impl From<Hp67DataSerialError> for StructuralWordError {
    fn from(error: Hp67DataSerialError) -> Self {
        Self::Data(error)
    }
}

/// Values reconstructed from one shared 56-bit HP-67 structural word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralWordResult {
    pub fetched_word: u16,
    pub display_byte: u8,
    pub str_event: Rom0StrEvent,
    pub rcd_falling: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralDataWordResult {
    pub word: StructuralWordResult,
    pub completed_data: Option<super::act::ActRegister>,
}

fn sample_isa(level: LogicLevel, word_bit: u8) -> Result<bool, SerialFetchError> {
    match level {
        LogicLevel::Low => Ok(false),
        LogicLevel::High => Ok(true),
        LogicLevel::Floating => Err(SerialFetchError::FloatingIsa { word_bit }),
        LogicLevel::Contention => Err(SerialFetchError::IsaContention { word_bit }),
    }
}

/// ACT-side state for structural 56-bit machine words.
///
/// One endpoint owns every currently modeled ACT role on IS: display scan phase
/// and serialization at b0..b7, ROM address serialization at b16..b27,
/// ROM-word reception at b46..b55, and the lifetime of the instruction executing
/// concurrently with that shared word. No downstream cathode state enters this
/// endpoint or chooses which A/B digit is emitted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ActSerialEndpoint {
    address: u16,
    display_scan_slot: u8,
    display_register_index: Option<usize>,
    execution: Option<ActSerialExecution>,
    execution_state: Option<ActSerialStateSnapshot>,
    arithmetic_chain: Option<bool>,
    last_alu_digit_result: Option<ActSerialDigitAluResult>,
    arithmetic_result_image: Option<ActSerialArithmeticResultImage>,
    received_word: u16,
    received_mask: u16,
}

impl ActSerialEndpoint {
    pub const fn new(address: u16) -> Self {
        Self {
            address: address & ROM_ADDRESS_MASK,
            display_scan_slot: 1,
            display_register_index: None,
            execution: None,
            execution_state: None,
            arithmetic_chain: None,
            last_alu_digit_result: None,
            arithmetic_result_image: None,
            received_word: 0,
            received_mask: 0,
        }
    }

    /// Bind one already-fetched word and its pre-instruction ACT state to the
    /// following 56-bit execution cycle.
    ///
    /// The architectural oracle may still compute effects that have not migrated
    /// to the structural path, but the serial endpoint owns an immutable source
    /// snapshot from before those effects. For every arithmetic opcode, it also
    /// owns the result image accumulated by the real b0..b55 traversal. A new
    /// instruction cannot replace an execution that has not reached b55.
    pub fn begin_execution(
        &mut self,
        word: u16,
        state: &ActArchitecturalState,
    ) -> Result<(), ActSerialExecutionError> {
        if let Some(execution) = self.execution {
            if let Some(next_word_bit) = execution.next_word_bit() {
                return Err(ActSerialExecutionError::ExecutionAlreadyActive {
                    word: execution.word(),
                    next_word_bit,
                });
            }
        }

        let execution = ActSerialExecution::new(word, state.instruction_state)?;
        let snapshot = ActSerialStateSnapshot::capture(state);
        self.arithmetic_result_image = ActSerialArithmeticResultImage::begin(&snapshot, &execution);
        self.execution = Some(execution);
        self.execution_state = Some(snapshot);
        self.arithmetic_chain = None;
        self.last_alu_digit_result = None;
        Ok(())
    }

    pub const fn serial_execution(&self) -> Option<ActSerialExecution> {
        self.execution
    }

    pub const fn serial_execution_state(&self) -> Option<ActSerialStateSnapshot> {
        self.execution_state
    }

    pub fn serial_alu_inputs(&self) -> Option<ActSerialAluInputs> {
        let execution = self.execution.as_ref()?;
        let state = self.execution_state.as_ref()?;
        state.alu_inputs(execution)
    }

    pub const fn serial_arithmetic_chain(&self) -> Option<bool> {
        self.arithmetic_chain
    }

    pub const fn last_serial_alu_digit_result(&self) -> Option<ActSerialDigitAluResult> {
        self.last_alu_digit_result
    }

    pub const fn serial_arithmetic_result_image(&self) -> Option<ActSerialArithmeticResultImage> {
        self.arithmetic_result_image
    }

    /// Start a fetch-only word, explicitly releasing the display window.
    pub fn begin_fetch_cycle(&mut self, address: u16) {
        self.address = address & ROM_ADDRESS_MASK;
        self.display_register_index = None;
        self.received_word = 0;
        self.received_mask = 0;
    }

    /// Start a combined display/fetch word from the ACT-owned display phase.
    ///
    /// Only the source register index is latched here. For an executing word,
    /// b0..b7 are sourced from the immutable pre-instruction ACT snapshot bound
    /// by `begin_execution`; fetch-only cycles fall back to the supplied live
    /// architectural state. No additional A/B snapshot is taken at this boundary.
    pub fn begin_display_fetch_cycle(&mut self, address: u16) -> Result<(), ActDisplaySerialError> {
        self.address = address & ROM_ADDRESS_MASK;
        self.display_register_index = Some(
            display_register_index_for_scan_slot(self.display_scan_slot).ok_or(
                ActDisplaySerialError::InvalidScanSlot(self.display_scan_slot),
            )?,
        );
        self.received_word = 0;
        self.received_mask = 0;
        Ok(())
    }

    pub const fn address(&self) -> u16 {
        self.address
    }

    pub const fn display_scan_slot(&self) -> u8 {
        self.display_scan_slot
    }

    /// Advance the ACT-owned display phase after one complete shared word.
    ///
    /// Slot 15 is the source-backed duplicate exponent-units word. Completing it
    /// wraps the ACT phase to slot 1 and emits the coarse structural RCD event.
    /// This does not claim the still-unknown PHI-relative RCD edge.
    pub fn complete_display_word(&mut self) -> bool {
        let rcd_falling = self.display_scan_slot == HP67_DISPLAY_SCAN_SLOTS;
        self.display_scan_slot = if rcd_falling {
            1
        } else {
            self.display_scan_slot + 1
        };
        self.display_register_index = None;
        rcd_falling
    }

    fn encoded_display_byte(
        &self,
        state: Option<&ActArchitecturalState>,
    ) -> Result<Option<u8>, ActDisplaySerialError> {
        let Some(register_index) = self.display_register_index else {
            return Ok(None);
        };

        let (display_enable, a_nibble, b_nibble) =
            if let Some(snapshot) = self.execution_state.as_ref() {
                let digit = register_index as u8;
                (
                    snapshot.display_enable(),
                    snapshot
                        .register_digit(ActSerialRegister::A, digit)
                        .expect("latched display register index must be valid"),
                    snapshot
                        .register_digit(ActSerialRegister::B, digit)
                        .expect("latched display register index must be valid"),
                )
            } else {
                let Some(state) = state else {
                    return Ok(None);
                };
                (
                    state.display_enable,
                    state.a[register_index] & 0x0f,
                    state.b[register_index] & 0x0f,
                )
            };

        if !display_enable {
            return Ok(Some(HP67_ROM0_BLANK_CODE));
        }

        let code = match b_nibble {
            // HP-67 numeric formats accepted by the independent firmware-level
            // renderer all reach ROM0 as the measured $0x character class.
            0x00 | 0x04 | 0x09 => a_nibble,
            // B=2 is the sign/blank modifier class; B=3 adds the DP class.
            0x02 => 0x20 | a_nibble,
            0x03 => 0x30 | a_nibble,
            // B=1/F are blank formats; physical HP-67 capture identifies $4x
            // as the ROM0 blank class.
            0x01 | 0x0f => 0x40 | a_nibble,
            _ => {
                return Err(ActDisplaySerialError::UnsupportedModifier {
                    scan_slot: self.display_scan_slot,
                    b_nibble,
                });
            }
        };
        Ok(Some(code))
    }

    /// ACT contribution to IS for the current bit cell.
    ///
    /// During b0..b7 the selected pre-instruction A/B state is converted through
    /// the HP-67's observed ROM0 code classes. The B register is a display-format
    /// modifier and is not emitted as a raw high nibble.
    pub fn drive_for_bit(
        &self,
        word_bit: u8,
        state: Option<&ActArchitecturalState>,
    ) -> Result<Drive, ActDisplaySerialError> {
        if let Some(serial_bit) = display_data_serial_bit(word_bit) {
            let Some(code) = self.encoded_display_byte(state)? else {
                return Ok(Drive::HighZ);
            };
            return Ok(wired_high_drive(((code >> serial_bit) & 1) != 0));
        }
        Ok(act_address_drive(self.address, word_bit))
    }

    /// Sample a resolved IS/ISA level during the ROM return window.
    pub fn sample_for_bit(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomWord { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };

        if sample_isa(level, word_bit)? {
            self.received_word |= 1u16 << serial_bit;
        } else {
            self.received_word &= !(1u16 << serial_bit);
        }
        self.received_mask |= 1u16 << serial_bit;
        Ok(())
    }

    /// Advance the current instruction by one structural serial bit coordinate.
    ///
    /// The fourth coordinate of every arithmetic digit is the structural
    /// checkpoint used to accumulate the private result image. ADD/SUB also keep
    /// the source-backed carry/borrow chain between successive selected digits.
    /// No live ACT register is written here: the checkpoint is bookkeeping, not
    /// a claimed PHI-relative register-write edge.
    fn advance_execution_for_bit(&mut self, word_bit: u8) -> Result<(), ActSerialExecutionError> {
        let execution = self.execution;
        let state = self.execution_state;
        let digit_checkpoint = matches!(
            execution.and_then(|execution| execution.bit_in_digit()),
            Some(bit) if bit == BITS_PER_DIGIT - 1
        );

        let digit_result = match (execution.as_ref(), state.as_ref()) {
            (Some(execution), Some(state)) if digit_checkpoint => {
                state.alu_inputs(execution).and_then(|inputs| {
                    let chain_in = self.arithmetic_chain.unwrap_or(inputs.initial_carry);
                    state.alu_digit_result(execution, chain_in)
                })
            }
            _ => None,
        };

        if let Some(result) = digit_result {
            self.arithmetic_chain = Some(result.chain_out);
            self.last_alu_digit_result = Some(result);
        }

        if digit_checkpoint {
            if let (Some(execution), Some(state), Some(image)) = (
                execution.as_ref(),
                state.as_ref(),
                self.arithmetic_result_image.as_mut(),
            ) {
                image
                    .record_digit_checkpoint(state, execution, digit_result)
                    .expect("arithmetic digit checkpoint must be structurally valid");
            }
        }

        if let Some(execution) = &mut self.execution {
            execution.advance_word_bit(word_bit)?;
        }
        Ok(())
    }

    /// Return the complete reconstructed 10-bit word after b55 has been sampled.
    pub const fn fetched_word(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_WORD_MASK {
            return Err(SerialFetchError::IncompleteRomWord {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_word & ROM_WORD_MASK)
    }
}

/// ROM-side state for one 56-bit serial fetch cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct RomFetchEndpoint {
    received_address: u16,
    received_mask: u16,
    response_word: Option<u16>,
}

impl RomFetchEndpoint {
    pub fn begin_cycle(&mut self) {
        self.received_address = 0;
        self.received_mask = 0;
        self.response_word = None;
    }

    /// Sample ACT address bits from the resolved IS/ISA line. When b27 is
    /// received, the complete 12-bit address is used to latch the ROM response.
    pub fn sample_for_bit<S: Hp67RomWordSource>(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
        source: &S,
    ) -> Result<(), SerialFetchError> {
        let IsaWindow::RomAddress { serial_bit } = isa_window_for_bit(word_bit) else {
            return Ok(());
        };

        if sample_isa(level, word_bit)? {
            self.received_address |= 1u16 << serial_bit;
        } else {
            self.received_address &= !(1u16 << serial_bit);
        }
        self.received_mask |= 1u16 << serial_bit;

        if serial_bit + 1 == ROM_ADDRESS_BITS {
            if self.received_mask != COMPLETE_ADDRESS_MASK {
                return Err(SerialFetchError::IncompleteAddress {
                    received_mask: self.received_mask,
                });
            }
            let address = self.received_address & ROM_ADDRESS_MASK;
            self.response_word = Some(
                source
                    .read_word(address)
                    .ok_or(SerialFetchError::MissingRomWord { address })?
                    & ROM_WORD_MASK,
            );
        }

        Ok(())
    }

    pub const fn received_address(&self) -> Result<u16, SerialFetchError> {
        if self.received_mask != COMPLETE_ADDRESS_MASK {
            return Err(SerialFetchError::IncompleteAddress {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_address & ROM_ADDRESS_MASK)
    }

    /// ROM contribution to IS/ISA for the current bit cell. No response is
    /// driven until all twelve address bits have been captured and a word found.
    pub const fn drive_for_bit(&self, word_bit: u8) -> Drive {
        match self.response_word {
            Some(word) => rom_word_drive(word, word_bit),
            None => Drive::HighZ,
        }
    }
}

/// One-word pipeline latch separating a word executing in the current 56-bit
/// cycle from the word being prefetched for the following cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct FetchPipelineLatch {
    executing: Option<u16>,
    prefetched: Option<u16>,
}

impl FetchPipelineLatch {
    /// Enter a new machine cycle. The word fetched during the preceding cycle
    /// becomes the word eligible to execute now.
    pub fn begin_cycle(&mut self) {
        self.executing = self.prefetched.take();
    }

    /// Commit the word reconstructed during the current machine cycle so it can
    /// become executable when the following cycle begins.
    pub fn complete_cycle(&mut self, fetched_word: u16) {
        self.prefetched = Some(fetched_word & ROM_WORD_MASK);
    }

    pub const fn executing_word(&self) -> Option<u16> {
        self.executing
    }

    pub const fn prefetched_word(&self) -> Option<u16> {
        self.prefetched
    }
}

fn run_structural_word_transport<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    act: &mut ActSerialEndpoint,
    act_state: Option<&ActArchitecturalState>,
    rom: &mut RomFetchEndpoint,
    source: &S,
    mut rom0: Option<&mut Rom0DisplayEndpoint>,
) -> Result<u16, StructuralWordError> {
    rom.begin_cycle();
    if let Some(endpoint) = rom0.as_deref_mut() {
        endpoint.begin_word();
    }

    // The selected ACT display source is immutable for the whole b0..b7
    // window: executing words read the pre-instruction snapshot and fetch-only
    // words read the supplied live state. Encode it once per machine word
    // instead of rebuilding the same ROM0 byte for all eight serial bits.
    let display_byte = match act.encoded_display_byte(act_state) {
        Ok(byte) => byte,
        Err(ActDisplaySerialError::UnsupportedModifier { .. }) => Some(HP67_ROM0_BLANK_CODE),
        Err(error) => return Err(error.into()),
    };

    // Each endpoint owns its named IS driver. Reset that ownership once at the
    // word boundary, then publish only actual drive-state transitions below.
    // This preserves every resolved bit-cell level while avoiding redundant
    // High-Z/same-level writes on the shared electrical net.
    backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, Drive::HighZ);
    backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, Drive::HighZ);
    let mut previous_act_drive = Drive::HighZ;
    let mut previous_rom_drive = Drive::HighZ;

    for expected_bit in 0..BITS_PER_WORD {
        debug_assert_eq!(backplane.word_bit(), expected_bit);

        let display_bit = display_data_serial_bit(expected_bit);
        let act_drive = if let Some(serial_bit) = display_bit {
            match display_byte {
                Some(code) => wired_high_drive(((code >> serial_bit) & 1) != 0),
                None => Drive::HighZ,
            }
        } else {
            act_address_drive(act.address(), expected_bit)
        };
        if act_drive != previous_act_drive {
            backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, act_drive);
            previous_act_drive = act_drive;
        }

        let rom_drive = rom.drive_for_bit(expected_bit);
        if rom_drive != previous_rom_drive {
            backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom_drive);
            previous_rom_drive = rom_drive;
        }

        let is_level = backplane.level(Hp67Net::Isa);
        if is_level == LogicLevel::Contention {
            return Err(SerialFetchError::IsaContention {
                word_bit: expected_bit,
            }
            .into());
        }

        if let (Some(endpoint), Some(_)) = (rom0.as_deref_mut(), display_bit) {
            endpoint.sample_for_bit(expected_bit, is_level)?;
        }

        match isa_window_for_bit(expected_bit) {
            IsaWindow::RomAddress { .. } => {
                rom.sample_for_bit(expected_bit, is_level, source)?;
            }
            IsaWindow::RomWord { .. } => {
                act.sample_for_bit(expected_bit, is_level)?;
            }
            IsaWindow::Other => {}
        }

        for _ in 0..4 {
            backplane.advance_clock();
        }
        act.advance_execution_for_bit(expected_bit)?;
    }

    Ok(act.fetched_word()?)
}

/// M14B DATA-aware twin of the established structural transport.
///
/// This intentionally keeps the established no-DATA transport as a separate
/// monomorphic function so adding DATA fidelity cannot perturb its codegen or
/// historical performance baseline. The DATA variant visits the logical DATA
/// participant on the same b0..b55 coordinates. M14C may have already consumed
/// b0/b1 of the preceding DATA frame before the instruction-boundary bridge; in
/// that case the DATA participant skips only those duplicate logical samples
/// while the structural backplane still traverses the physical b0/b1 cells.
fn run_structural_word_transport_with_data<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    act: &mut ActSerialEndpoint,
    act_state: Option<&ActArchitecturalState>,
    rom: &mut RomFetchEndpoint,
    source: &S,
    mut rom0: Option<&mut Rom0DisplayEndpoint>,
    data: &mut Hp67DataSerialWordPath,
) -> Result<u16, StructuralWordError> {
    rom.begin_cycle();
    if let Some(endpoint) = rom0.as_deref_mut() {
        endpoint.begin_word();
    }

    // The selected ACT display source is immutable for the whole b0..b7
    // window: executing words read the pre-instruction snapshot and fetch-only
    // words read the supplied live state. Encode it once per machine word
    // instead of rebuilding the same ROM0 byte for all eight serial bits.
    let display_byte = match act.encoded_display_byte(act_state) {
        Ok(byte) => byte,
        Err(ActDisplaySerialError::UnsupportedModifier { .. }) => Some(HP67_ROM0_BLANK_CODE),
        Err(error) => return Err(error.into()),
    };

    // Each endpoint owns its named IS driver. Reset that ownership once at the
    // word boundary, then publish only actual drive-state transitions below.
    // This preserves every resolved bit-cell level while avoiding redundant
    // High-Z/same-level writes on the shared electrical net.
    backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, Drive::HighZ);
    backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, Drive::HighZ);
    let mut previous_act_drive = Drive::HighZ;
    let mut previous_rom_drive = Drive::HighZ;

    for expected_bit in 0..BITS_PER_WORD {
        debug_assert_eq!(backplane.word_bit(), expected_bit);

        let display_bit = display_data_serial_bit(expected_bit);
        let act_drive = if let Some(serial_bit) = display_bit {
            match display_byte {
                Some(code) => wired_high_drive(((code >> serial_bit) & 1) != 0),
                None => Drive::HighZ,
            }
        } else {
            act_address_drive(act.address(), expected_bit)
        };
        if act_drive != previous_act_drive {
            backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, act_drive);
            previous_act_drive = act_drive;
        }

        let rom_drive = rom.drive_for_bit(expected_bit);
        if rom_drive != previous_rom_drive {
            backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom_drive);
            previous_rom_drive = rom_drive;
        }

        let is_level = backplane.level(Hp67Net::Isa);
        if is_level == LogicLevel::Contention {
            return Err(SerialFetchError::IsaContention {
                word_bit: expected_bit,
            }
            .into());
        }

        if let (Some(endpoint), Some(_)) = (rom0.as_deref_mut(), display_bit) {
            endpoint.sample_for_bit(expected_bit, is_level)?;
        }

        match isa_window_for_bit(expected_bit) {
            IsaWindow::RomAddress { .. } => {
                rom.sample_for_bit(expected_bit, is_level, source)?;
            }
            IsaWindow::RomWord { .. } => {
                act.sample_for_bit(expected_bit, is_level)?;
            }
            IsaWindow::Other => {}
        }

        data.visit_transport_word_bit(expected_bit)?;

        for _ in 0..4 {
            backplane.advance_clock();
        }
        act.advance_execution_for_bit(expected_bit)?;
    }

    Ok(act.fetched_word()?)
}

/// Run one structural 56-bit ACT↔ROM fetch through the resolved HP-67 IS net.
///
/// This is a bit-cell harness, not the final edge-accurate device scheduler. It
/// deliberately advances the current four-subphase timing scaffold once per bit
/// cell while keeping every address/response bit on the resolved electrical net.
pub fn run_structural_fetch_cycle<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    source: &S,
) -> Result<u16, SerialFetchError> {
    act.begin_fetch_cycle(address);
    match run_structural_word_transport(backplane, act, None, rom, source, None) {
        Ok(word) => Ok(word),
        Err(StructuralWordError::Fetch(error)) => Err(error),
        Err(StructuralWordError::Display(_))
        | Err(StructuralWordError::ActDisplay(_))
        | Err(StructuralWordError::ActExecution(_))
        | Err(StructuralWordError::Data(_)) => {
            unreachable!("fetch-only structural cycle cannot produce a non-fetch error")
        }
    }
}

/// Run one structural 56-bit word with ACT display traffic and instruction fetch
/// sharing the same resolved HP-67 IS net and backplane timing coordinate.
///
/// `ActSerialEndpoint` owns the fifteen-word display phase and therefore chooses
/// the display source digit at b0..b7. When a word is executing, those early
/// display cells are sourced from the immutable pre-instruction ACT snapshot so
/// the instruction-boundary fallback cannot leak post-instruction A/B into the
/// same physical word. Fetch-only display cycles fall back to the supplied live
/// ACT state. If an executing word is bound to the endpoint, that same instruction advances
/// through b0..b55 in lockstep with this transport. ADD/SUB operations also carry
/// an immutable pre-instruction A/B/C/P/radix snapshot and a source-backed
/// carry/borrow chain across selected digit boundaries. ROM0 reconstructs the
/// same resolved bits and emits the returned `Rom0StrEvent`; ACT independently
/// reports the coarse RCD falling boundary after slot 15. No cathode state enters
/// this API.
///
/// This establishes source-backed ownership at word/bit and arithmetic-digit
/// granularity. It still does not claim the PHI edge that commits a result bit to
/// A/B/C, ROM0 sampling edges, or exact STR/RCD overlap ordering.
pub fn run_structural_display_fetch_cycle<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
) -> Result<StructuralWordResult, StructuralWordError> {
    let display_scan_slot = act.display_scan_slot();
    act.begin_display_fetch_cycle(address)?;
    let fetched_word = run_structural_word_transport(
        backplane,
        act,
        Some(act_state),
        rom,
        source,
        Some(&mut *rom0),
    )?;
    let display_byte = rom0.display_byte()?;
    let str_event = rom0.str_falling_event(display_scan_slot)?;
    let rcd_falling = act.complete_display_word();
    Ok(StructuralWordResult {
        fetched_word,
        display_byte,
        str_event,
        rcd_falling,
    })
}

/// M14B/M14C structural word that advances logical DATA alongside the same
/// b0..b55 IS/fetch/display transport.
///
/// DATA remains logical-only here. This function deliberately does not drive or
/// sample `Hp67Net::Data`, choose DATA polarity, or assign a PHI edge.
pub fn run_structural_display_fetch_data_phase_cycle<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
    data: &mut Hp67DataSerialWordPath,
    data_payload: Option<super::act::ActRegister>,
) -> Result<StructuralDataWordResult, StructuralWordError> {
    let display_scan_slot = act.display_scan_slot();
    act.begin_display_fetch_cycle(address)?;
    data.begin_word(data_payload);
    let fetched_word = run_structural_word_transport_with_data(
        backplane,
        act,
        Some(act_state),
        rom,
        source,
        Some(&mut *rom0),
        data,
    )?;
    let completed_data = data.complete_word();
    let display_byte = rom0.display_byte()?;
    let str_event = rom0.str_falling_event(display_scan_slot)?;
    let rcd_falling = act.complete_display_word();
    Ok(StructuralDataWordResult {
        word: StructuralWordResult {
            fetched_word,
            display_byte,
            str_event,
            rcd_falling,
        },
        completed_data,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{ActSerialWordClass, Hp67SegmentMask};

    struct FixtureRom {
        words: [(u16, u16); 2],
    }

    impl Hp67RomWordSource for FixtureRom {
        fn read_word(&self, address: u16) -> Option<u16> {
            self.words
                .iter()
                .find_map(|(candidate, word)| (*candidate == address).then_some(*word))
        }
    }

    #[test]
    fn real_hp67_example_crosses_the_resolved_is_net_bit_by_bit() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();

        let fetched =
            run_structural_fetch_cycle(&mut backplane, 0x07b, &mut act, &mut rom, &source)
                .expect("serial fetch must complete");

        assert_eq!(rom.received_address(), Ok(0x07b));
        assert_eq!(fetched, 0x04c);
        assert_eq!(backplane.word_index(), 1);
        assert_eq!(backplane.word_bit(), 0);
    }

    #[test]
    fn data_aware_transport_without_frame_matches_monomorphic_fast_path() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        state.a[0] = 0x07;
        state.b[0] = 0x03;

        let mut fast_backplane = Hp67ElectricalBackplane::default();
        let mut fast_act = ActSerialEndpoint::new(0x07b);
        let mut fast_rom = RomFetchEndpoint::default();
        let mut fast_rom0 = Rom0DisplayEndpoint::default();

        let mut data_backplane = Hp67ElectricalBackplane::default();
        let mut data_act = ActSerialEndpoint::new(0x07b);
        let mut data_rom = RomFetchEndpoint::default();
        let mut data_rom0 = Rom0DisplayEndpoint::default();
        let mut data = Hp67DataSerialWordPath::default();

        let fast = run_structural_display_fetch_cycle(
            &mut fast_backplane,
            0x07b,
            &state,
            &mut fast_act,
            &mut fast_rom,
            &mut fast_rom0,
            &source,
        )
        .expect("monomorphic fast path must complete");

        let data_result = run_structural_display_fetch_data_phase_cycle(
            &mut data_backplane,
            0x07b,
            &state,
            &mut data_act,
            &mut data_rom,
            &mut data_rom0,
            &source,
            &mut data,
            None,
        )
        .expect("DATA-aware transport without a frame must complete");

        assert_eq!(data_result.word, fast);
        assert_eq!(data_result.completed_data, None);
        assert!(!data.frame_in_progress());
        assert_eq!(data_backplane.tick(), fast_backplane.tick());
        assert_eq!(data_backplane.word_index(), fast_backplane.word_index());
        assert_eq!(data_backplane.word_bit(), fast_backplane.word_bit());
        assert_eq!(data_rom.received_address(), fast_rom.received_address());
        assert_eq!(data_act.display_scan_slot(), fast_act.display_scan_slot());
    }

    #[test]
    fn fused_data_phase_shares_structural_word_without_disturbing_fetch() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();
        let mut data = Hp67DataSerialWordPath::default();
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;

        let payload: super::super::act::ActRegister =
            std::array::from_fn(|digit| ((digit as u8 * 5) + 1) & 0x0f);

        let first = run_structural_display_fetch_data_phase_cycle(
            &mut backplane,
            0x07b,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
            &mut data,
            Some(payload),
        )
        .expect("first fused DATA word must complete");
        assert_eq!(first.word.fetched_word, 0x04c);
        assert_eq!(first.completed_data, None);
        assert!(data.frame_in_progress());

        let second = run_structural_display_fetch_data_phase_cycle(
            &mut backplane,
            0x07b,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
            &mut data,
            None,
        )
        .expect("following DATA tail word must complete");
        assert_eq!(second.word.fetched_word, 0x04c);
        assert_eq!(second.completed_data, Some(payload));
        assert!(!data.frame_in_progress());
        assert_eq!(backplane.word_index(), 2);
    }

    #[test]
    fn display_and_fetch_share_one_resolved_word_cycle() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        let mut act_state = ActArchitecturalState::default();
        act_state.display_enable = true;
        // ACT starts at slot 1 -> register digit 0. A=0, B=3 must serialize
        // 0,0,0,0,1,1,0,0 on b0..b7 without cathode selecting that digit.
        act_state.a[0] = 0x00;
        act_state.b[0] = 0x03;
        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            &act_state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("display and fetch transport must share one structural word");

        assert_eq!(result.fetched_word, 0x04c);
        assert_eq!(result.display_byte, 0x30);
        assert_eq!(result.str_event, Rom0StrEvent { scan_slot: 1 });
        assert!(!result.rcd_falling);
        assert_eq!(act.display_scan_slot(), 2);
        assert_eq!(rom.received_address(), Ok(0x07b));
        assert_eq!(rom0.decoded_anodes(1), Ok(Hp67SegmentMask::DP));
        assert_eq!(backplane.word_index(), 1);
        assert_eq!(backplane.word_bit(), 0);
    }

    #[test]
    fn fetch_only_display_bits_follow_live_a_and_recoded_b_state() {
        let mut act = ActSerialEndpoint::new(0);
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        act.begin_display_fetch_cycle(0)
            .expect("slot 1 must select a valid ACT display digit");

        assert_eq!(act.drive_for_bit(0, Some(&state)).unwrap(), Drive::HighZ);
        state.a[0] = 0x01;
        assert_eq!(act.drive_for_bit(0, Some(&state)).unwrap(), Drive::High);

        state.b[0] = 0x01;
        assert_eq!(act.drive_for_bit(4, Some(&state)).unwrap(), Drive::HighZ);
        assert_eq!(act.drive_for_bit(6, Some(&state)).unwrap(), Drive::High);
    }

    #[test]
    fn hp67_b_modifier_is_recoded_into_measured_rom0_classes() {
        let mut act = ActSerialEndpoint::new(0);
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        state.a[0] = 0x05;
        act.begin_display_fetch_cycle(0)
            .expect("slot 1 must select exponent-units source");

        for (b_nibble, expected) in [
            (0x00, 0x05),
            (0x04, 0x05),
            (0x09, 0x05),
            (0x02, 0x25),
            (0x03, 0x35),
            (0x01, 0x45),
            (0x0f, 0x45),
        ] {
            state.b[0] = b_nibble;
            assert_eq!(
                act.encoded_display_byte(Some(&state)).unwrap(),
                Some(expected),
                "B={b_nibble:x} must map to the source-backed ROM0 class"
            );
        }

        state.b[0] = 0x05;
        assert_eq!(
            act.encoded_display_byte(Some(&state)),
            Err(ActDisplaySerialError::UnsupportedModifier {
                scan_slot: 1,
                b_nibble: 0x05,
            })
        );
    }

    #[test]
    fn display_off_serializes_measured_rom0_blank_code() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut state = ActArchitecturalState::default();
        state.display_enable = false;
        state.a[0] = 0x00;
        state.b[0] = 0x05;

        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("disabled display must still transport a legal ROM0 blank code");

        assert_eq!(result.display_byte, HP67_ROM0_BLANK_CODE);
        assert_eq!(rom0.decoded_anodes(1), Ok(Hp67SegmentMask::BLANK));
    }

    #[test]
    fn executing_word_display_uses_pre_instruction_a_b_state() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        state.a[0] = 0x05;
        state.b[0] = 0x00;

        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        act.begin_execution(0o0132, &state)
            .expect("A/B exchange word must start serial execution");

        // Architectural fallback has already committed the exchange before the
        // structural b0..b7 window runs. Those post-state values must not leak
        // into the display traffic for this same word.
        state.a[0] = 0x00;
        state.b[0] = 0x05;

        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("display transport must use the pre-instruction serial snapshot");

        assert_eq!(result.display_byte, 0x05);
        assert_eq!(rom0.decoded_anodes(1).unwrap().bits(), 0x6d);
    }

    #[test]
    fn active_instruction_advances_with_the_same_structural_word() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let state = ActArchitecturalState::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        act.begin_execution(0x11a, &state)
            .expect("known arithmetic word must start execution");
        run_structural_display_fetch_cycle(
            &mut backplane,
            0x07b,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("active instruction must span the shared structural word");

        let execution = act
            .serial_execution()
            .expect("execution remains inspectable after b55");
        assert!(execution.is_complete());
        assert_eq!(
            execution.class(),
            ActSerialWordClass::Arithmetic {
                operation: 8,
                field: 6,
            }
        );
        assert!(act.serial_execution_state().is_some());
    }

    #[test]
    fn execution_owns_pre_instruction_state_and_digit_chain() {
        let mut state = ActArchitecturalState::default();
        state.a[0] = 9;
        state.b[0] = 1;
        state.p = 0;
        state.decimal = true;

        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0x122, &state)
            .expect("P-field A+B->A must start serial execution");

        state.a[0] = 0;
        state.b[0] = 0;
        state.p = 7;
        state.decimal = false;

        assert_eq!(state.a[0], 0);
        assert_eq!(state.b[0], 0);
        assert_eq!(state.p, 7);
        assert!(!state.decimal);
        let snapshot = act
            .serial_execution_state()
            .expect("pre-instruction serial snapshot must be retained");
        assert_eq!(snapshot.register_digit(ActSerialRegister::A, 0), Some(9));
        assert_eq!(snapshot.register_digit(ActSerialRegister::B, 0), Some(1));
        assert_eq!(snapshot.p(), 0);
        assert!(snapshot.decimal());

        assert_eq!(act.serial_arithmetic_chain(), None);
        for bit in 0..BITS_PER_DIGIT {
            act.advance_execution_for_bit(bit)
                .expect("first serial digit must advance");
        }

        let result = act
            .last_serial_alu_digit_result()
            .expect("selected ADD digit must produce a serial ALU result");
        assert_eq!(result.coordinate.digit, 0);
        assert_eq!(result.left_digit, 9);
        assert_eq!(result.right_digit, 1);
        assert_eq!(result.result_digit, 0);
        assert!(result.chain_out);
        assert_eq!(act.serial_arithmetic_chain(), Some(true));
    }

    #[test]
    fn arithmetic_chain_uses_previous_selected_digit_as_next_chain_in() {
        let mut state = ActArchitecturalState::default();
        state.a[0] = 9;
        state.b[0] = 1;
        state.a[1] = 0;
        state.b[1] = 0;
        state.p = 1;
        state.decimal = true;

        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0x126, &state)
            .expect("WP-field A+B->A must start serial execution");

        for bit in 0..(BITS_PER_DIGIT * 2) {
            act.advance_execution_for_bit(bit)
                .expect("two selected digits must advance");
        }

        let result = act
            .last_serial_alu_digit_result()
            .expect("second selected ADD digit must produce a result");
        assert_eq!(result.coordinate.digit, 1);
        assert_eq!(result.result_digit, 1);
        assert!(!result.chain_out);
        assert_eq!(act.serial_arithmetic_chain(), Some(false));
    }

    #[test]
    fn incomplete_execution_cannot_be_replaced_by_another_word() {
        let state = ActArchitecturalState::default();
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0x11a, &state)
            .expect("first execution must start");
        assert_eq!(
            act.begin_execution(0x000, &state),
            Err(ActSerialExecutionError::ExecutionAlreadyActive {
                word: 0x11a,
                next_word_bit: 0,
            })
        );
    }

    #[test]
    fn unsupported_transient_display_modifier_does_not_abort_shared_rom_fetch() {
        struct OneWordRom;
        impl Hp67RomWordSource for OneWordRom {
            fn read_word(&self, address: u16) -> Option<u16> {
                (address == 0x123).then_some(0x2ab)
            }
        }

        let source = OneWordRom;
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        state.a[0] = 0x05;
        state.b[0] = 0x05;

        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x123);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            0x123,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("unknown transient B modifier must not disable instruction fetch");

        assert_eq!(result.fetched_word, 0x2ab);
        assert_eq!(result.display_byte, HP67_ROM0_BLANK_CODE);
        assert_eq!(result.str_event.scan_slot, 1);
    }

    #[test]
    fn act_owns_full_fifteen_word_display_phase_and_rcd_boundary() {
        struct ZeroRom;
        impl Hp67RomWordSource for ZeroRom {
            fn read_word(&self, _address: u16) -> Option<u16> {
                Some(0)
            }
        }

        let source = ZeroRom;
        let state = ActArchitecturalState::default();
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        for expected_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            assert_eq!(act.display_scan_slot(), expected_slot);
            let result = run_structural_display_fetch_cycle(
                &mut backplane,
                0,
                &state,
                &mut act,
                &mut rom,
                &mut rom0,
                &source,
            )
            .expect("ACT-owned display phase must advance one slot per word");
            assert_eq!(result.str_event.scan_slot, expected_slot);
            assert_eq!(result.rcd_falling, expected_slot == HP67_DISPLAY_SCAN_SLOTS);
        }
        assert_eq!(act.display_scan_slot(), 1);
    }

    #[test]
    fn fetched_word_enters_execution_on_the_following_machine_cycle() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut pipeline = FetchPipelineLatch::default();

        pipeline.begin_cycle();
        assert_eq!(pipeline.executing_word(), None);
        let first = run_structural_fetch_cycle(&mut backplane, 0x07b, &mut act, &mut rom, &source)
            .expect("first serial fetch must complete");
        pipeline.complete_cycle(first);
        assert_eq!(pipeline.executing_word(), None);
        assert_eq!(pipeline.prefetched_word(), Some(0x04c));

        pipeline.begin_cycle();
        assert_eq!(pipeline.executing_word(), Some(0x04c));
        assert_eq!(pipeline.prefetched_word(), None);
        let second = run_structural_fetch_cycle(&mut backplane, 0x001, &mut act, &mut rom, &source)
            .expect("second serial fetch must complete");
        pipeline.complete_cycle(second);

        assert_eq!(pipeline.executing_word(), Some(0x04c));
        assert_eq!(pipeline.prefetched_word(), Some(0x3e3));
        assert_eq!(backplane.word_index(), 2);
    }

    #[test]
    fn missing_rom_location_fails_at_the_end_of_the_address_window() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut rom = RomFetchEndpoint::default();
        let address = ActSerialEndpoint::new(0x222);

        for bit in 16..=27 {
            let level = match address.drive_for_bit(bit, None).unwrap() {
                Drive::High => LogicLevel::High,
                Drive::HighZ => LogicLevel::Low,
                Drive::Low => unreachable!("IS zeros are represented by release"),
            };
            let result = rom.sample_for_bit(bit, level, &source);
            if bit == 27 {
                assert_eq!(
                    result,
                    Err(SerialFetchError::MissingRomWord { address: 0x222 })
                );
            } else {
                assert_eq!(result, Ok(()));
            }
        }
    }
}
