//! Structural HP-67 ACT↔ROM serial fetch path.
//!
//! This module models the evidence-backed bit-cell transport on the shared IS/ISA
//! line with a provisional PHI1-rising receiver checkpoint. Exact physical
//! launch/latch edges remain unproven. The ACT can emit
//! the eight-bit ROM0 display byte at b0..b7 and the 12-bit ROM address LSB-first
//! at b16..b27 during the same 56-bit word. A ROM-side endpoint reconstructs the
//! address through the resolved electrical net, the selected ROM word is looked
//! up through a caller-supplied source, and the ROM returns the 10-bit word
//! LSB-first at b46..b55 for the ACT endpoint to reconstruct.

use crate::emulation::{Drive, LogicLevel};

use super::{
    act::{
        display_register_index_for_scan_slot, ActArchitecturalState, ActDisplaySerialError,
        ActRamImage,
    },
    act_serial_boundary::ActSerialBoundaryResultImage,
    act_serial_control::ActSerialControlResultImage,
    act_serial_crc_control::ActSerialCrcControlResultImage,
    act_serial_crc_data::ActSerialCrcDataResultImage,
    act_serial_execution::{
        ActSerialExecution, ActSerialExecutionError, ActSerialRegister, ActSerialWordClass,
    },
    act_serial_flow::{ActSerialFlowResultImage, ActSerialFlowState},
    act_serial_mode_latch::ActSerialModeLatchResultImage,
    act_serial_ram::{ActSerialAbsentRamReadImage, ActSerialRamResultImage},
    act_serial_result::ActSerialArithmeticResultImage,
    act_serial_special_register::ActSerialSpecialRegisterResultImage,
    act_serial_state::{ActSerialAluInputs, ActSerialDigitAluResult, ActSerialStateSnapshot},
    crc::CrcArchitecturalCore,
    data::{Hp67DataSerialError, Hp67DataSerialWordPath},
    display::{
        Rom0DisplayEndpoint, Rom0DisplayError, Rom0StrEvent, HP67_DISPLAY_SCAN_SLOTS,
        HP67_ROM0_BLANK_CODE,
    },
    isa::{act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},
    machine::Hp67ElectricalBackplane,
    rom_devices::{Hp67RomChipSet, RomIsDrives},
    serial_ram::{Hp67SerialRam, Hp67SerialRamError},
    timing::{
        display_data_serial_bit, isa_window_for_bit, sync_decision_window, IsaWindow,
        BITS_PER_DIGIT, BITS_PER_WORD, ROM_ADDRESS_BITS, ROM_WORD_BITS,
    },
    wiring::{Hp67Driver, Hp67Net},
};

const COMPLETE_ADDRESS_MASK: u16 = (1u16 << ROM_ADDRESS_BITS) - 1;
const COMPLETE_WORD_MASK: u16 = (1u16 << ROM_WORD_BITS) - 1;
const ACT_IS_DRIVER: Hp67Driver = Hp67Driver::Act1820_2530;

/// One resolved display checkpoint, with independent ACT/ROM0 output proposals.
/// Data/control latches commit together only after both resolved pins validate.
fn sample_display_transport(
    act: &ActSerialEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    backplane: &mut Hp67ElectricalBackplane,
) -> Result<(), Rom0DisplayError> {
    let mut next = *rom0;
    next.sample_data_at_edge(backplane)?;
    let bit = backplane.word_bit();
    let str_drive = next.str_drive(bit)?;
    let rcd_drive = act.rcd_drive(bit);
    if matches!(bit, 0 | 7 | 8) {
        backplane.commit_display_controls(str_drive, rcd_drive);
    }
    next.sample_controls_at_edge(backplane)?;
    *rom0 = next;
    Ok(())
}

/// ROM lookup boundary used by the serial fetch responder.
///
/// The physical-ROM composition and aggregate reference fixtures share this
/// immutable lookup boundary. Live ROMs use explicit bank/address reads after
/// their own IS/SYNC decoding; provider-selected views are reference-only.
pub trait Hp67RomWordSource {
    fn read_word(&self, address: u16) -> Option<u16>;

    /// Explicit immutable bank lookup; autonomous ROMs never consult the
    /// provider's selected-bank state. Bank-0-only fixtures need no override.
    fn read_word_in_bank(&self, bank: u8, address: u16) -> Option<u16> {
        if bank == 0 {
            self.read_word(address)
        } else {
            None
        }
    }

    /// Reference image/bank selection bridge, not an electrical bank-select pin.
    fn selected_rom_bank(&self, _address: u16) -> u8 {
        0
    }
}

/// Errors that make a serial fetch electrically or structurally invalid.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SerialFetchError {
    InvalidSyncSource(super::sync::Hp67SyncFault),
    InvalidClock(super::machine::Hp67ClockFault),
    FloatingIsa {
        word_bit: u8,
    },
    IsaContention {
        word_bit: u8,
    },
    UnexpectedIsaSource {
        word_bit: u8,
        expected: Option<Hp67Driver>,
        driver_mask: u16,
    },
    InvalidIsaDrives {
        word_bit: u8,
        driver_mask: u16,
    },
    MissingRomWord {
        address: u16,
    },
    UnexpectedRomAddressBit {
        word_bit: u8,
        received_bits: u8,
    },
    UnexpectedRomReturnBit {
        word_bit: u8,
        received_bits: u8,
    },
    IncompleteAddress {
        received_mask: u16,
    },
    IncompleteRomWord {
        received_mask: u16,
    },
    UnresolvedSync {
        word_bit: u8,
        level: LogicLevel,
    },
    ChangedSyncQualifier {
        word_bit: u8,
    },
    MissingSyncQualifier,
    UnexpectedActReturnBit {
        word_bit: u8,
        received_bits: u8,
    },
    UnconsumedActInstruction,

    InvalidTransportStart {
        tick: u64,
    },
    InvalidSamplePhase {
        tick: u64,
    },
    InvalidLaunchPhase {
        tick: u64,
    },
    InvalidRomSelection {
        address: u16,
        bank: u8,
        selected: u8,
    },
}

/// Errors from a combined structural HP-67 word carrying display and fetch data.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StructuralWordError {
    Fetch(SerialFetchError),
    Display(Rom0DisplayError),
    ActDisplay(ActDisplaySerialError),
    ActExecution(ActSerialExecutionError),
    Data(Hp67DataSerialError),
    Ram(Hp67SerialRamError),
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

impl From<Hp67SerialRamError> for StructuralWordError {
    fn from(error: Hp67SerialRamError) -> Self {
        Self::Ram(error)
    }
}

// RAM proposes its own read bits; the DATA path supplies ACT output and frame
// metadata only. No architectural read value reaches the RAM contribution.
fn propose_data_with_ram(
    data: &Hp67DataSerialWordPath,
    ram: Option<&Hp67SerialRam>,
    bit: u8,
    bus: &Hp67ElectricalBackplane,
) -> Result<Option<(Drive, Drive)>, StructuralWordError> {
    let drives = data.propose_transport_word_bit(bit, bus)?;
    if let Some(ram) = ram {
        let (act, _) = drives.ok_or(Hp67DataSerialError::ElectricalBridgeRequired)?;
        Ok(Some((act, ram.propose_data_drive(bus)?)))
    } else {
        Ok(drives)
    }
}

fn sample_data_with_ram(
    data: &mut Hp67DataSerialWordPath,
    ram: Option<&mut Hp67SerialRam>,
    bit: u8,
    bus: &Hp67ElectricalBackplane,
) -> Result<(), StructuralWordError> {
    if let Some(ram) = ram {
        let sample = ram.prepare_sample(bus)?;
        data.sample_transport_word_bit_at_edge(bit, bus)?;
        let _ = ram.commit_sample(sample);
    } else {
        data.sample_transport_word_bit_at_edge(bit, bus)?;
    }
    Ok(())
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

/// Sample after both endpoint contributions have been committed. A valid logic
/// level alone does not prove sole IS ownership or active-high/release polarity.
fn sample_transport_isa(
    backplane: &Hp67ElectricalBackplane,
    word_bit: u8,
) -> Result<LogicLevel, SerialFetchError> {
    let level = backplane.level(Hp67Net::Isa);
    if level == LogicLevel::Contention {
        return Err(SerialFetchError::IsaContention { word_bit });
    }
    if !backplane.isa_drives_valid() {
        return Err(SerialFetchError::InvalidIsaDrives {
            word_bit,
            driver_mask: backplane.active_isa_driver_mask(),
        });
    }
    Ok(level)
}

/// The working boundary is after PHI2; it is not an exact physical output delay.
/// Validation and DATA proposal evaluation precede every pin mutation.
fn publish_transport_sources(
    backplane: &mut Hp67ElectricalBackplane,
    word_bit: u8,
    act_is: Drive,
    rom_is: RomIsDrives,
    sync: Drive,
    data: Option<(Drive, Drive)>,
) -> Result<(), SerialFetchError> {
    publish_transport_sources_with_ram_owner(backplane, word_bit, act_is, rom_is, sync, data, false)
}

fn publish_transport_sources_with_ram_owner(
    backplane: &mut Hp67ElectricalBackplane,
    word_bit: u8,
    act_is: Drive,
    rom_is: RomIsDrives,
    sync: Drive,
    data: Option<(Drive, Drive)>,
    autonomous_ram: bool,
) -> Result<(), SerialFetchError> {
    if !backplane.at_transport_launch_boundary() || backplane.word_bit() != word_bit {
        return Err(SerialFetchError::InvalidLaunchPhase {
            tick: backplane.tick().get(),
        });
    }
    if autonomous_ram {
        backplane.commit_autonomous_ram_launch(act_is, rom_is, sync, data);
    } else {
        backplane.commit_serial_launch(act_is, rom_is, sync, data);
    }
    Ok(())
}

fn advance_transport_to_sample(backplane: &mut Hp67ElectricalBackplane) {
    backplane.advance_clock_edge();
    let (_, edge) = backplane.advance_clock_edge();
    debug_assert_eq!(edge, super::timing::HP67_WORKING_TRANSPORT_SAMPLE_EDGE);
}

/// Provisional receiver checkpoint. Physical latch edges remain in the
/// separate evidence table; this guard prevents pre-clock receiver consumption.
fn sample_transport_isa_at_edge(
    backplane: &Hp67ElectricalBackplane,
    word_bit: u8,
) -> Result<LogicLevel, SerialFetchError> {
    if !backplane.at_transport_sample_edge() || backplane.word_bit() != word_bit {
        return Err(SerialFetchError::InvalidSamplePhase {
            tick: backplane.tick().get(),
        });
    }
    sample_transport_isa(backplane, word_bit)
}

/// Validate the source role after existing polarity/count/phase checks. A Low
/// remains passive release: there is no claim about invisible output enables.
/// The selected chip comes from the ROM's received address/bank latches.
fn sample_transport_owned_isa_at_edge(
    backplane: &Hp67ElectricalBackplane,
    word_bit: u8,
    return_driver: Option<Hp67Driver>,
) -> Result<LogicLevel, SerialFetchError> {
    let level = sample_transport_isa_at_edge(backplane, word_bit)?;
    if level == LogicLevel::High {
        let expected = match isa_window_for_bit(word_bit) {
            IsaWindow::RomAddress { .. } => Some(ACT_IS_DRIVER),
            IsaWindow::RomWord { .. } => return_driver,
            IsaWindow::Other if display_data_serial_bit(word_bit).is_some() => Some(ACT_IS_DRIVER),
            IsaWindow::Other => None,
        };
        if !expected.is_some_and(|driver| backplane.driver_drive(Hp67Net::Isa, driver) == Drive::High) {
            return Err(SerialFetchError::UnexpectedIsaSource {
                word_bit,
                expected,
                driver_mask: backplane.active_isa_driver_mask(),
            });
        }
    }
    Ok(level)
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
    execution_display_enable_override: Option<bool>,
    arithmetic_chain: Option<bool>,
    last_alu_digit_result: Option<ActSerialDigitAluResult>,
    arithmetic_result_image: Option<ActSerialArithmeticResultImage>,
    control_result_image: Option<ActSerialControlResultImage>,
    special_register_result_image: Option<ActSerialSpecialRegisterResultImage>,
    mode_latch_result_image: Option<ActSerialModeLatchResultImage>,
    flow_result_image: Option<ActSerialFlowResultImage>,
    boundary_result_image: Option<ActSerialBoundaryResultImage>,
    ram_result_image: Option<ActSerialRamResultImage>,
    crc_control_result_image: Option<ActSerialCrcControlResultImage>,
    crc_control_bound: bool,
    crc_data_result_image: Option<ActSerialCrcDataResultImage>,
    absent_ram_read_image: Option<ActSerialAbsentRamReadImage>,
    ram_topology_bound: bool,
    received_word: u16,
    received_mask: u16,
    received_sync: Option<bool>,
    received_sync_mask: u16,
    clocked_prefix_display: Option<u8>,
    received_execution_enabled: bool,
    owned_flow: Option<ActSerialFlowState>,
    owned_registers: Option<owned_registers::ActOwnedRegisters>,
    pending_instruction: Option<super::ram_command::Hp67RamCommandFrame>,
}

impl ActSerialEndpoint {
    pub const fn new(address: u16) -> Self {
        Self {
            address: address & ROM_ADDRESS_MASK,
            display_scan_slot: 1,
            display_register_index: None,
            execution: None,
            execution_state: None,
            execution_display_enable_override: None,
            arithmetic_chain: None,
            last_alu_digit_result: None,
            arithmetic_result_image: None,
            control_result_image: None,
            special_register_result_image: None,
            mode_latch_result_image: None,
            flow_result_image: None,
            boundary_result_image: None,
            ram_result_image: None,
            crc_control_result_image: None,
            crc_control_bound: false,
            crc_data_result_image: None,
            absent_ram_read_image: None,
            ram_topology_bound: false,
            received_word: 0,
            received_mask: 0,
            received_sync: None,
            received_sync_mask: 0,
            clocked_prefix_display: None,
            received_execution_enabled: false,
            owned_flow: None,
            owned_registers: None,
            pending_instruction: None,
        }
    }

    /// Production executes only qualified instructions retained by its own receiver.
    /// Explicit word binding remains available for isolated reference fixtures.
    pub fn with_received_execution(mut self) -> Self {
        self.received_execution_enabled = true;
        self
    }

    /// Power-on flow latches, separate from the architectural comparison mirror.
    pub fn with_owned_flow(mut self) -> Self {
        assert!(
            self.received_execution_enabled,
            "owned ACT flow requires received execution"
        );
        self.owned_flow = Some(ActSerialFlowState {
            pc: self.address,
            bank: 0,
            delayed_rom: None,
            return_stack: [0; super::act::ACT_RETURN_STACK_DEPTH],
            stack_pointer: 0,
        });
        self
    }

    pub const fn owned_flow_state(&self) -> Option<ActSerialFlowState> {
        self.owned_flow
    }

    pub fn check_owned_flow_mirror(&self, state: &ActArchitecturalState) -> Result<(), String> {
        if let Some(owned) = self.owned_flow {
            let mirror = ActSerialFlowState::capture(state);
            if owned != mirror {
                return Err(format!(
                    "ACT owned flow/mirror mismatch: device={owned:?} mirror={mirror:?}"
                ));
            }
        }
        Ok(())
    }

    /// Own successor preview feeds the address serializer before b16. This is
    /// the established scheduling bridge, not a physical PC write-edge claim.
    pub fn owned_fetch_request(&self) -> Option<(u16, u8)> {
        self.owned_flow.map(|flow| match self.flow_result_image {
            Some(image) => (image.result().pc, image.fetch_bank()),
            None => (flow.pc, if flow.pc < 0x400 { 0 } else { flow.bank & 1 }),
        })
    }

    pub const fn received_execution_enabled(&self) -> bool {
        self.received_execution_enabled
    }

    pub const fn pending_instruction(&self) -> Option<super::ram_command::Hp67RamCommandFrame> {
        self.pending_instruction
    }

    /// Test/semantic composition boundary; production never selects this mode.
    pub fn disable_received_execution(&mut self) {
        self.received_execution_enabled = false;
        self.owned_flow = None;
        self.owned_registers = None;
        self.pending_instruction = None;
    }

    pub fn begin_received_execution(
        &mut self,
        state: &ActArchitecturalState,
    ) -> Result<Option<u16>, String> {
        if !self.received_execution_enabled {
            return Err("ACT received execution is disabled".into());
        }
        self.check_owned_flow_mirror(state)?;
        self.check_owned_register_mirror(state)?;
        let Some(frame) = self.pending_instruction else {
            return Ok(None);
        };
        let qualified_state = if frame.instruction {
            super::act::ActInstructionState::Normal
        } else {
            super::act::ActInstructionState::ThenGoto
        };
        if state.instruction_state != qualified_state {
            return Err(format!(
                "ACT received SYNC/mirror mismatch: pins={qualified_state:?} mirror={:?}",
                state.instruction_state
            ));
        }
        self.bind_execution(frame.word, state)
            .map_err(|error| format!("ACT received execution binding: {error:?}"))?;
        self.pending_instruction = None;
        Ok(Some(frame.word))
    }

    /// Bind one already-fetched word and its pre-instruction ACT state to the
    /// following 56-bit execution cycle.
    ///
    /// The architectural oracle may still compute effects that have not migrated
    /// to the structural path, but the serial endpoint owns an immutable source
    /// snapshot from before those effects. For every arithmetic opcode it owns
    /// the structural arithmetic result image; M14F adds P/status control state,
    /// M14G adds selected special-register state and M14H adds selected mode/latch
    /// state. All completed-word images become eligible only at b55. A new
    /// instruction cannot replace an execution that has not reached b55.
    pub fn begin_execution(
        &mut self,
        word: u16,
        state: &ActArchitecturalState,
    ) -> Result<(), ActSerialExecutionError> {
        if self.received_execution_enabled {
            return Err(ActSerialExecutionError::ReceivedInstructionRequired);
        }
        self.bind_execution(word, state)
    }

    fn bind_execution(
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

        let owned_source = self.owned_register_state();
        let state = owned_source.as_ref().unwrap_or(state);
        let execution = ActSerialExecution::new(word, state.instruction_state)?;
        let snapshot = ActSerialStateSnapshot::capture(state);
        self.arithmetic_result_image = ActSerialArithmeticResultImage::begin(&snapshot, &execution);
        self.control_result_image = ActSerialControlResultImage::begin(&snapshot, &execution);
        self.special_register_result_image =
            ActSerialSpecialRegisterResultImage::begin(&snapshot, &execution);
        self.mode_latch_result_image = ActSerialModeLatchResultImage::begin(&snapshot, &execution);
        self.flow_result_image = Some(ActSerialFlowResultImage::begin_with_flow(
            state,
            word,
            self.owned_flow
                .unwrap_or_else(|| ActSerialFlowState::capture(state)),
        ));
        self.boundary_result_image = ActSerialBoundaryResultImage::begin(&snapshot, &execution);
        self.ram_result_image = ActSerialRamResultImage::begin(&snapshot, &execution);
        self.crc_control_result_image = None;
        self.crc_control_bound = false;
        self.crc_data_result_image = None;
        self.absent_ram_read_image = None;
        self.ram_topology_bound = false;
        self.execution = Some(execution);
        self.execution_state = Some(snapshot);
        self.execution_display_enable_override = None;
        self.arithmetic_chain = None;
        self.last_alu_digit_result = None;
        Ok(())
    }

    pub fn bind_ram_topology(&mut self, ram: &ActRamImage) -> Result<(), String> {
        if self.ram_topology_bound {
            return Err("RAM topology is already bound".into());
        }
        let execution = self
            .execution
            .as_ref()
            .ok_or("RAM topology binding requires an executing word")?;
        if execution.next_word_bit() != Some(0) {
            return Err("RAM topology must bind before b0".into());
        }
        let snapshot = self
            .execution_state
            .as_ref()
            .ok_or("RAM topology binding requires ACT pre-state")?;
        self.absent_ram_read_image = ActSerialAbsentRamReadImage::begin(snapshot, execution, ram);
        self.ram_topology_bound = true;
        Ok(())
    }

    pub const fn serial_absent_ram_read_image(&self) -> Option<ActSerialAbsentRamReadImage> {
        self.absent_ram_read_image
    }

    pub fn bind_crc_control(&mut self, crc: &CrcArchitecturalCore) -> Result<(), String> {
        self.bind_crc_sources(crc, None)
    }

    pub fn bind_crc_control_with_owned_data(
        &mut self,
        crc: &CrcArchitecturalCore,
        data: &super::crc_owned_data::Hp67CrcOwnedData,
        write_mode: bool,
    ) -> Result<(), String> {
        self.bind_crc_sources(crc, Some((data, write_mode)))
    }

    fn bind_crc_sources(
        &mut self,
        crc: &CrcArchitecturalCore,
        owned: Option<(&super::crc_owned_data::Hp67CrcOwnedData, bool)>,
    ) -> Result<(), String> {
        if self.crc_control_bound {
            return Err("CRC pre-instruction inputs are already bound".into());
        }
        let execution = self
            .execution
            .as_ref()
            .ok_or("CRC binding requires an executing word")?;
        if execution.next_word_bit() != Some(0) {
            return Err("CRC inputs must bind before b0".into());
        }
        let snapshot = self
            .execution_state
            .as_ref()
            .ok_or("CRC binding requires pre-instruction ACT state")?;
        let control = ActSerialCrcControlResultImage::begin(snapshot, execution, crc)?;
        let data = match owned {
            Some((data, write_mode)) => {
                ActSerialCrcDataResultImage::begin_owned(snapshot, execution, data, write_mode)?
            }
            None => ActSerialCrcDataResultImage::begin(snapshot, execution, crc)?,
        };
        self.crc_control_result_image = control;
        self.crc_data_result_image = data;
        self.crc_control_bound = true;
        Ok(())
    }

    pub const fn serial_crc_data_result_image(&self) -> Option<ActSerialCrcDataResultImage> {
        self.crc_data_result_image
    }

    pub const fn serial_crc_control_result_image(&self) -> Option<ActSerialCrcControlResultImage> {
        self.crc_control_result_image
    }

    pub const fn serial_ram_result_image(&self) -> Option<ActSerialRamResultImage> {
        self.ram_result_image
    }

    pub const fn serial_boundary_result_image(&self) -> Option<ActSerialBoundaryResultImage> {
        self.boundary_result_image
    }

    pub const fn serial_flow_result_image(&self) -> Option<ActSerialFlowResultImage> {
        self.flow_result_image
    }

    pub const fn serial_execution(&self) -> Option<ActSerialExecution> {
        self.execution
    }

    pub const fn serial_execution_state(&self) -> Option<ActSerialStateSnapshot> {
        self.execution_state
    }

    /// Override only the DISPLAY-enable source used by b0..b7 for the current
    /// execution. This preserves the direct HP-67 power-on display observation
    /// without contaminating the immutable architectural snapshot used by M14H.
    pub fn set_execution_display_enable_override(&mut self, display_enable: bool) {
        self.execution_display_enable_override = Some(display_enable);
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

    pub const fn serial_control_result_image(&self) -> Option<ActSerialControlResultImage> {
        self.control_result_image
    }

    pub const fn serial_special_register_result_image(
        &self,
    ) -> Option<ActSerialSpecialRegisterResultImage> {
        self.special_register_result_image
    }

    pub const fn serial_mode_latch_result_image(&self) -> Option<ActSerialModeLatchResultImage> {
        self.mode_latch_result_image
    }

    /// Start a fetch-only word, explicitly releasing the display window.
    pub fn begin_fetch_cycle(&mut self, address: u16) {
        self.address = address & ROM_ADDRESS_MASK;
        self.display_register_index = None;
        self.received_word = 0;
        self.received_mask = 0;
        self.received_sync = None;
        self.received_sync_mask = 0;
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
        self.received_sync = None;
        self.received_sync_mask = 0;
        Ok(())
    }

    pub const fn address(&self) -> u16 {
        self.address
    }

    pub const fn display_scan_slot(&self) -> u8 {
        self.display_scan_slot
    }

    /// Coarse RCD output belongs to ACT, independently of ROM0's STR proposal.
    /// The b7 interval remains the existing uncalibrated pulse approximation.
    fn rcd_drive(&self, bit: u8) -> Drive {
        if bit == 7 && self.display_scan_slot == HP67_DISPLAY_SCAN_SLOTS {
            Drive::Low
        } else {
            Drive::High
        }
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

    fn take_clocked_prefix(
        &mut self,
        backplane: &Hp67ElectricalBackplane,
        has_rom0: bool,
    ) -> Result<(u8, Option<u8>), SerialFetchError> {
        let first_bit = if self.clocked_prefix_display.is_some() {
            2
        } else {
            0
        };
        if backplane.word_bit() != first_bit
            || backplane.tick().get() & 3 != 0
            || (first_bit == 2 && !has_rom0)
        {
            return Err(SerialFetchError::InvalidTransportStart {
                tick: backplane.tick().get(),
            });
        }
        Ok((first_bit, self.clocked_prefix_display.take()))
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
                    self.execution_display_enable_override
                        .unwrap_or(snapshot.display_enable()),
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

    /// HP-97 service manual 2-15, applied under the owner's equivalence
    /// assumption: executing an IF suppresses the concurrent successor qualifier.
    /// Decode from the bound structural word, never post-oracle ACT state.
    fn successor_is_instruction(&self) -> bool {
        match self.execution.map(|execution| execution.class()) {
            Some(ActSerialWordClass::Arithmetic { operation, .. }) => {
                !(0x16..=0x1b).contains(&operation)
            }
            Some(ActSerialWordClass::SpecialOrPeripheral { opcode }) => {
                !matches!(opcode & 0o77, 0o24 | 0o34 | 0o44 | 0o54)
            }
            // The currently executing implied-GOTO is data, even if its bits
            // would decode as another IF. Fetch after that word is normal.
            _ => true,
        }
    }

    #[cfg(test)]
    fn sample_sync_for_bit(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
    ) -> Result<(), SerialFetchError> {
        if !sync_decision_window(word_bit) {
            return Ok(());
        }
        let qualifier = match level {
            LogicLevel::High => true,
            LogicLevel::Low => false,
            _ => return Err(SerialFetchError::UnresolvedSync { word_bit, level }),
        };
        if self
            .received_sync
            .is_some_and(|previous| previous != qualifier)
        {
            return Err(SerialFetchError::ChangedSyncQualifier { word_bit });
        }
        self.received_sync = Some(qualifier);
        self.received_sync_mask |= 1u16 << (word_bit - super::timing::ROM_WORD_FIRST_BIT);
        Ok(())
    }

    /// Instruction/data classification actually received on the resolved SYNC
    /// net for all ten returned bits. Standalone IS-only sampling has no qualifier.
    pub fn fetched_instruction_state(
        &self,
    ) -> Result<super::act::ActInstructionState, SerialFetchError> {
        self.fetched_word()?;
        if self.received_sync_mask != COMPLETE_WORD_MASK {
            return Err(SerialFetchError::MissingSyncQualifier);
        }
        match self.received_sync {
            Some(true) => Ok(super::act::ActInstructionState::Normal),
            Some(false) => Ok(super::act::ActInstructionState::ThenGoto),
            None => Err(SerialFetchError::MissingSyncQualifier),
        }
    }

    /// Consume IS and SYNC together, once per ordered return coordinate.
    /// All electrical/order/overwrite checks precede either latch mutation.
    pub fn sample_instruction_at_edge(
        &mut self,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<(), SerialFetchError> {
        if !bus.at_transport_sample_edge() {
            return Err(SerialFetchError::InvalidSamplePhase {
                tick: bus.tick().get(),
            });
        }
        let bit = bus.word_bit();
        let IsaWindow::RomWord { serial_bit } = isa_window_for_bit(bit) else {
            return Ok(());
        };
        self.check_return_order(bit)?;
        if self.received_sync_mask != self.received_mask {
            return Err(SerialFetchError::MissingSyncQualifier);
        }
        let value = sample_isa(bus.level(Hp67Net::Isa), bit)?;
        let qualifier = match bus.level(Hp67Net::Sync) {
            LogicLevel::High => true,
            LogicLevel::Low => false,
            level => {
                return Err(SerialFetchError::UnresolvedSync {
                    word_bit: bit,
                    level,
                })
            }
        };
        super::sync::validate_sync_source(bus).map_err(SerialFetchError::InvalidSyncSource)?;
        if self
            .received_sync
            .is_some_and(|previous| previous != qualifier)
        {
            return Err(SerialFetchError::ChangedSyncQualifier { word_bit: bit });
        }
        if serial_bit == ROM_WORD_BITS - 1
            && self.received_execution_enabled
            && self.pending_instruction.is_some()
        {
            return Err(SerialFetchError::UnconsumedActInstruction);
        }
        if value {
            self.received_word |= 1 << serial_bit;
        }
        self.received_mask |= 1 << serial_bit;
        self.received_sync = Some(qualifier);
        self.received_sync_mask |= 1 << serial_bit;
        if serial_bit == ROM_WORD_BITS - 1 && self.received_execution_enabled {
            self.pending_instruction = Some(super::ram_command::Hp67RamCommandFrame {
                word: self.received_word & ROM_WORD_MASK,
                instruction: qualifier,
            });
        }
        Ok(())
    }

    fn check_return_order(&self, word_bit: u8) -> Result<(), SerialFetchError> {
        let received_bits = self.received_mask.count_ones() as u8;
        if word_bit != super::timing::ROM_WORD_FIRST_BIT + received_bits {
            return Err(SerialFetchError::UnexpectedActReturnBit {
                word_bit,
                received_bits,
            });
        }
        Ok(())
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

        self.check_return_order(word_bit)?;
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
            if execution.is_complete() {
                if let Some(image) = &mut self.absent_ram_read_image {
                    image.complete_word();
                }
                if let Some(image) = &mut self.crc_data_result_image {
                    image.complete_word();
                }
                if let Some(image) = &mut self.crc_control_result_image {
                    image.complete_word();
                }
                if let Some(image) = &mut self.ram_result_image {
                    image.complete_word();
                }
                if let Some(image) = &mut self.boundary_result_image {
                    image.complete_word();
                }
                if let Some(image) = &mut self.flow_result_image {
                    image.complete_word();
                    if self.owned_flow.is_some() {
                        let mut result = image.result();
                        result.bank = image.fetch_bank();
                        self.owned_flow = Some(result);
                    }
                }
                if let Some(image) = &mut self.control_result_image {
                    image.complete_word();
                }
                if let (Some(image), Some(snapshot)) = (
                    self.special_register_result_image.as_mut(),
                    self.execution_state.as_ref(),
                ) {
                    image.complete_word(snapshot);
                }
                if let (Some(image), Some(snapshot)) = (
                    self.mode_latch_result_image.as_mut(),
                    self.execution_state.as_ref(),
                ) {
                    image.complete_word(snapshot);
                }
            }
        }
        if word_bit == BITS_PER_WORD - 1 && self.execution.is_some_and(|word| word.is_complete()) {
            self.complete_owned_register_word();
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
    physical_chips: Option<Hp67RomChipSet>,
}

impl RomFetchEndpoint {
    /// Five-chip reference fixture retaining provider-selected bank views.
    pub fn physical_roms() -> Self {
        Self {
            physical_chips: Some(Hp67RomChipSet::default()),
            ..Self::default()
        }
    }

    /// Live five-chip composition with persistent, bus-controlled bank latches.
    pub fn autonomous_roms() -> Self {
        Self {
            physical_chips: Some(Hp67RomChipSet::autonomous()),
            ..Self::default()
        }
    }

    fn sample_instruction_for_bit(
        &mut self,
        bit: u8,
        level: LogicLevel,
        sync: LogicLevel,
    ) -> Result<(), SerialFetchError> {
        if let Some(chips) = self.physical_chips.as_mut() {
            chips.sample_instruction_for_bit(bit, level, sync)?;
        }
        Ok(())
    }

    pub const fn physical_chip_set(&self) -> Option<&Hp67RomChipSet> {
        self.physical_chips.as_ref()
    }

    /// Identity selected by received address and bank state, not payload bits.
    fn return_driver(&self) -> Option<Hp67Driver> {
        match self.physical_chips.as_ref() {
            Some(chips) => chips.chips().iter().find(|chip| chip.selected()).map(|chip| chip.profile().driver),
            None => self.response_word.map(|_| Hp67Driver::StructuralRomResponder),
        }
    }

    pub(crate) fn is_drives_for_bit(&self, word_bit: u8) -> RomIsDrives {
        match self.physical_chips.as_ref() {
            Some(chips) => chips.is_drives_for_bit(word_bit),
            None => RomIsDrives::Aggregate(self.drive_for_bit(word_bit)),
        }
    }

    pub fn begin_cycle(&mut self) {
        self.received_address = 0;
        self.received_mask = 0;
        self.response_word = None;
        if let Some(chips) = self.physical_chips.as_mut() {
            chips.begin_cycle();
        }
    }

    /// Sample ACT address bits from the resolved IS/ISA line. When b27 is
    /// received, the complete 12-bit address is used to latch the ROM response.
    pub fn sample_for_bit<S: Hp67RomWordSource>(
        &mut self,
        word_bit: u8,
        level: LogicLevel,
        source: &S,
    ) -> Result<(), SerialFetchError> {
        if let Some(chips) = self.physical_chips.as_mut() {
            return chips.sample_for_bit(word_bit, level, source);
        }
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
        if let Some(chips) = self.physical_chips.as_ref() {
            return chips.chips()[0].received_address();
        }
        if self.received_mask != COMPLETE_ADDRESS_MASK {
            return Err(SerialFetchError::IncompleteAddress {
                received_mask: self.received_mask,
            });
        }
        Ok(self.received_address & ROM_ADDRESS_MASK)
    }

    /// ROM contribution to IS/ISA for the current bit cell. No response is
    /// driven until all twelve address bits have been captured and a word found.
    pub fn drive_for_bit(&self, word_bit: u8) -> Drive {
        if let Some(chips) = self.physical_chips.as_ref() {
            return chips.trace_drive_for_bit(word_bit);
        }
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

/// Process following-word b0/b1 through shared IS/display/DATA and eight PHI
/// transitions before the next instruction can consume the reconstructed tail.
/// The remaining transport resumes at b2; no electrical cell is replayed.
/// Exact within-cell sample/write edges remain uncalibrated.
pub fn run_structural_data_tail_prefix(
    backplane: &mut Hp67ElectricalBackplane,
    state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    data: &mut Hp67DataSerialWordPath,
    trace: Option<&mut StructuralBusTrace>,
) -> Result<Option<super::act::ActRegister>, StructuralWordError> {
    run_structural_data_tail_prefix_with_ram(backplane, state, act, rom, rom0, data, trace, None)
}

/// Shared transport with an optional autonomous aggregate RAM participant.
pub fn run_structural_data_tail_prefix_with_ram(
    backplane: &mut Hp67ElectricalBackplane,
    state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    data: &mut Hp67DataSerialWordPath,
    mut trace: Option<&mut StructuralBusTrace>,
    mut ram: Option<&mut Hp67SerialRam>,
) -> Result<Option<super::act::ActRegister>, StructuralWordError> {
    backplane
        .validate_clock()
        .map_err(SerialFetchError::InvalidClock)?;
    if backplane.word_bit() != 0
        || backplane.tick().get() & 3 != 0
        || act.clocked_prefix_display.is_some()
    {
        return Err(SerialFetchError::InvalidTransportStart {
            tick: backplane.tick().get(),
        }
        .into());
    }
    super::sync::validate_sync_transport_entry(backplane)
        .map_err(SerialFetchError::InvalidSyncSource)?;
    if ram.is_some() {
        data.use_autonomous_ram_owner();
    }
    if !data.electrical_level_bridge_enabled() {
        return Err(Hp67DataSerialError::ElectricalBridgeRequired.into());
    }
    // DISPLAY depends on current A/B, not the completing C/RAM payload. Ignore
    // the previous completed instruction's snapshot, which belongs to another word.
    let mut source = *act;
    source.execution_state = None;
    source.execution_display_enable_override = None;
    source.begin_display_fetch_cycle(act.address())?;
    let byte = match source.encoded_display_byte(Some(state)) {
        Ok(Some(byte)) => byte,
        Ok(None) => HP67_ROM0_BLANK_CODE,
        Err(ActDisplaySerialError::UnsupportedModifier { .. }) => HP67_ROM0_BLANK_CODE,
        Err(error) => return Err(error.into()),
    };
    if let Some(ram) = ram.as_deref_mut() {
        ram.begin_cycle(backplane)?;
    }
    rom0.begin_word();
    for word_bit in 0..2 {
        let act_drive = wired_high_drive(((byte >> word_bit) & 1) != 0);
        let data_drives = propose_data_with_ram(data, ram.as_deref(), word_bit, backplane)?;
        publish_transport_sources_with_ram_owner(
            backplane,
            word_bit,
            act_drive,
            rom.is_drives_for_bit(word_bit),
            Drive::Low,
            data_drives,
            ram.is_some(),
        )?;
        for _ in 0..2 {
            backplane.advance_clock();
            if let Some(trace) = trace.as_deref_mut() {
                trace.samples.push(StructuralBusSample {
                    tick: backplane.tick().get(),
                    transport_bit: word_bit,
                    phi1: backplane.level(Hp67Net::Phi1),
                    phi2: backplane.level(Hp67Net::Phi2),
                    isa: backplane.level(Hp67Net::Isa),
                    act_drive,
                    rom_drive: Drive::HighZ,
                });
            }
        }
        sample_transport_owned_isa_at_edge(backplane, word_bit, None)?;
        sample_display_transport(act, rom0, backplane)?;
        sample_data_with_ram(data, ram.as_deref_mut(), word_bit, backplane)?;
        for _ in 0..2 {
            backplane.advance_clock();
            if let Some(trace) = trace.as_deref_mut() {
                trace.samples.push(StructuralBusSample {
                    tick: backplane.tick().get(),
                    transport_bit: word_bit,
                    phi1: backplane.level(Hp67Net::Phi1),
                    phi2: backplane.level(Hp67Net::Phi2),
                    isa: backplane.level(Hp67Net::Isa),
                    act_drive,
                    rom_drive: Drive::HighZ,
                });
            }
        }
    }
    act.clocked_prefix_display = Some(byte);
    Ok(data.take_clocked_prefix_result())
}

fn run_structural_word_transport<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    act: &mut ActSerialEndpoint,
    act_state: Option<&ActArchitecturalState>,
    rom: &mut RomFetchEndpoint,
    source: &S,
    mut rom0: Option<&mut Rom0DisplayEndpoint>,
) -> Result<u16, StructuralWordError> {
    let (first_bit, prefix_byte) = act.take_clocked_prefix(backplane, rom0.is_some())?;
    rom.begin_cycle();
    if first_bit == 0 {
        if let Some(endpoint) = rom0.as_deref_mut() {
            endpoint.begin_word();
        }
    }
    // These two execution coordinates are currently structural bookkeeping;
    // their physical cells have already completed in the clocked prefix.
    for bit in 0..first_bit {
        act.advance_execution_for_bit(bit)?;
    }

    // The selected ACT display source is immutable for the whole b0..b7
    // window: executing words read the pre-instruction snapshot and fetch-only
    // words read the supplied live state. Encode it once per machine word
    // instead of rebuilding the same ROM0 byte for all eight serial bits.
    let display_byte = if let Some(byte) = prefix_byte {
        Some(byte)
    } else {
        match act.encoded_display_byte(act_state) {
            Ok(byte) => byte,
            Err(ActDisplaySerialError::UnsupportedModifier { .. }) => Some(HP67_ROM0_BLANK_CODE),
            Err(error) => return Err(error.into()),
        }
    };

    // Hold committed sources until the next complete launch image is ready.
    let instruction_qualifier = act.successor_is_instruction();

    let mut return_driver = None;
    for expected_bit in first_bit..BITS_PER_WORD {
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
        let data_drives = None;
        let sync_drive = if sync_decision_window(expected_bit) && instruction_qualifier {
            Drive::High
        } else {
            Drive::Low
        };
        publish_transport_sources(
            backplane,
            expected_bit,
            act_drive,
            rom.is_drives_for_bit(expected_bit),
            sync_drive,
            data_drives,
        )?;

        advance_transport_to_sample(backplane);
        let is_level = sample_transport_owned_isa_at_edge(backplane, expected_bit, return_driver)?;

        if let Some(endpoint) = rom0.as_deref_mut() {
            sample_display_transport(act, endpoint, backplane)?;
        }

        match isa_window_for_bit(expected_bit) {
            IsaWindow::RomAddress { .. } => {
                rom.sample_for_bit(expected_bit, is_level, source)?;
                if expected_bit == super::timing::ROM_ADDRESS_LAST_BIT {
                    // One role lookup after the complete received address,
                    // rather than scanning chip selection on each return bit.
                    return_driver = rom.return_driver();
                }
            }
            IsaWindow::RomWord { .. } => {
                act.sample_instruction_at_edge(backplane)?;
                rom.sample_instruction_for_bit(
                    expected_bit,
                    is_level,
                    backplane.level(Hp67Net::Sync),
                )?;
            }
            IsaWindow::Other => {}
        }

        for _ in 0..2 {
            backplane.advance_clock();
        }
        act.advance_execution_for_bit(expected_bit)?;
    }

    // Coarse word boundary, not an asserted PHI launch edge or propagation time.
    backplane.drive(Hp67Net::Sync, ACT_IS_DRIVER, Drive::Low);
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
    mut ram: Option<&mut Hp67SerialRam>,
) -> Result<u16, StructuralWordError> {
    if ram.is_some() {
        data.use_autonomous_ram_owner();
    }
    let (first_bit, prefix_byte) = act.take_clocked_prefix(backplane, rom0.is_some())?;
    if first_bit == 0 {
        if let Some(ram) = ram.as_deref_mut() {
            ram.begin_cycle(backplane)?;
        }
    }
    rom.begin_cycle();
    if first_bit == 0 {
        if let Some(endpoint) = rom0.as_deref_mut() {
            endpoint.begin_word();
        }
    }
    // These two execution coordinates are currently structural bookkeeping;
    // their physical cells have already completed in the clocked prefix.
    for bit in 0..first_bit {
        act.advance_execution_for_bit(bit)?;
    }

    // The selected ACT display source is immutable for the whole b0..b7
    // window: executing words read the pre-instruction snapshot and fetch-only
    // words read the supplied live state. Encode it once per machine word
    // instead of rebuilding the same ROM0 byte for all eight serial bits.
    let display_byte = if let Some(byte) = prefix_byte {
        Some(byte)
    } else {
        match act.encoded_display_byte(act_state) {
            Ok(byte) => byte,
            Err(ActDisplaySerialError::UnsupportedModifier { .. }) => Some(HP67_ROM0_BLANK_CODE),
            Err(error) => return Err(error.into()),
        }
    };

    // Hold committed sources until the next complete launch image is ready.
    let instruction_qualifier = act.successor_is_instruction();

    let mut return_driver = None;
    for expected_bit in first_bit..BITS_PER_WORD {
        debug_assert_eq!(backplane.word_bit(), expected_bit);
        if expected_bit == 2 {
            if let Some(ram) = ram.as_deref_mut() {
                ram.bind_body(backplane)?;
            }
        }
        let display_bit = display_data_serial_bit(expected_bit);
        let act_drive = if let Some(serial_bit) = display_bit {
            match display_byte {
                Some(code) => wired_high_drive(((code >> serial_bit) & 1) != 0),
                None => Drive::HighZ,
            }
        } else {
            act_address_drive(act.address(), expected_bit)
        };
        let data_drives = propose_data_with_ram(data, ram.as_deref(), expected_bit, backplane)?;
        let sync_drive = if sync_decision_window(expected_bit) && instruction_qualifier {
            Drive::High
        } else {
            Drive::Low
        };
        publish_transport_sources_with_ram_owner(
            backplane,
            expected_bit,
            act_drive,
            rom.is_drives_for_bit(expected_bit),
            sync_drive,
            data_drives,
            ram.is_some(),
        )?;

        advance_transport_to_sample(backplane);
        let is_level = sample_transport_owned_isa_at_edge(backplane, expected_bit, return_driver)?;

        if let Some(endpoint) = rom0.as_deref_mut() {
            sample_display_transport(act, endpoint, backplane)?;
        }

        match isa_window_for_bit(expected_bit) {
            IsaWindow::RomAddress { .. } => {
                rom.sample_for_bit(expected_bit, is_level, source)?;
                if expected_bit == super::timing::ROM_ADDRESS_LAST_BIT {
                    // One role lookup after the complete received address,
                    // rather than scanning chip selection on each return bit.
                    return_driver = rom.return_driver();
                }
            }
            IsaWindow::RomWord { .. } => {
                act.sample_instruction_at_edge(backplane)?;
                rom.sample_instruction_for_bit(
                    expected_bit,
                    is_level,
                    backplane.level(Hp67Net::Sync),
                )?;
            }
            IsaWindow::Other => {}
        }

        sample_data_with_ram(data, ram.as_deref_mut(), expected_bit, backplane)?;

        for _ in 0..2 {
            backplane.advance_clock();
        }
        act.advance_execution_for_bit(expected_bit)?;
    }

    // Coarse word boundary, not an asserted PHI launch edge or propagation time.
    backplane.drive(Hp67Net::Sync, ACT_IS_DRIVER, Drive::Low);
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
    backplane
        .validate_clock()
        .map_err(SerialFetchError::InvalidClock)?;
    super::sync::validate_sync_transport_entry(backplane)
        .map_err(SerialFetchError::InvalidSyncSource)?;
    act.begin_fetch_cycle(address);
    match run_structural_word_transport(backplane, act, None, rom, source, None) {
        Ok(word) => Ok(word),
        Err(StructuralWordError::Fetch(error)) => Err(error),
        Err(StructuralWordError::Display(_))
        | Err(StructuralWordError::ActDisplay(_))
        | Err(StructuralWordError::ActExecution(_))
        | Err(StructuralWordError::Data(_))
        | Err(StructuralWordError::Ram(_)) => {
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
/// through b0..b55 in lockstep with this transport. Arithmetic operations also carry
/// an immutable pre-instruction A/B/C/P/radix snapshot and a private structural
/// result image; ADD/SUB additionally maintain the source-backed carry/borrow
/// chain across selected digit boundaries. ROM0 reconstructs the
/// same resolved bits. The coarse STR/RCD level bridge supplies the returned
/// control events from resolved pins, with RCD overlapping STR in slot 15. No cathode state enters
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
    backplane
        .validate_clock()
        .map_err(SerialFetchError::InvalidClock)?;
    super::sync::validate_sync_transport_entry(backplane)
        .map_err(SerialFetchError::InvalidSyncSource)?;
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
    let strobe = rom0.resolved_strobe()?;
    // Compatibility annotation only. Production cathode selection ignores it.
    let str_event = super::display::Rom0StrEvent {
        scan_slot: display_scan_slot,
    };
    let rcd_falling = strobe.rcd_falling();
    act.complete_display_word();
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
/// DATA uses either the reference logical fixture or the opted-in live resolved
/// level bridge. Neither path assigns physical output circuitry or a PHI edge.
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
    run_structural_display_fetch_data_phase_cycle_with_ram(
        backplane,
        address,
        act_state,
        act,
        rom,
        rom0,
        source,
        data,
        data_payload,
        None,
    )
}

/// Shared transport with an optional autonomous aggregate RAM participant.
pub fn run_structural_display_fetch_data_phase_cycle_with_ram<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
    data: &mut Hp67DataSerialWordPath,
    data_payload: Option<super::act::ActRegister>,
    ram: Option<&mut Hp67SerialRam>,
) -> Result<StructuralDataWordResult, StructuralWordError> {
    backplane
        .validate_clock()
        .map_err(SerialFetchError::InvalidClock)?;
    super::sync::validate_sync_transport_entry(backplane)
        .map_err(SerialFetchError::InvalidSyncSource)?;
    let display_scan_slot = act.display_scan_slot();
    act.begin_display_fetch_cycle(address)?;
    if ram.is_some() {
        data.begin_autonomous_word(data_payload);
    } else {
        data.begin_word(data_payload);
    }
    let fetched_word = run_structural_word_transport_with_data(
        backplane,
        act,
        Some(act_state),
        rom,
        source,
        Some(&mut *rom0),
        data,
        ram,
    )?;
    let completed_data = data.complete_word();
    let display_byte = rom0.display_byte()?;
    let strobe = rom0.resolved_strobe()?;
    // Compatibility annotation only. Production cathode selection ignores it.
    let str_event = super::display::Rom0StrEvent {
        scan_slot: display_scan_slot,
    };
    let rcd_falling = strobe.rcd_falling();
    act.complete_display_word();
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

/// One observation after a committed scaffold PHI transition.
///
/// `transport_bit` names the cell just processed, including b55 on the
/// word-wrapping fourth transition. Ticks are ordinal, not physical durations.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralBusSample {
    pub tick: u64,
    pub transport_bit: u8,
    pub phi1: LogicLevel,
    pub phi2: LogicLevel,
    pub isa: LogicLevel,
    pub act_drive: Drive,
    pub rom_drive: Drive,
}

/// Opt-in IS/PHI capture. No storage or hooks are added to production transport.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct StructuralBusTrace {
    pub samples: Vec<StructuralBusSample>,
}

impl StructuralBusTrace {
    /// Export actual observations, with explicit uncalibrated timing scope.
    ///
    /// Driver columns describe ACT and aggregate ROM ownership only. DATA,
    /// SYNC, STR and RCD are intentionally absent: this API does not capture
    /// their electrical timing. On a failed word the trace retains its prefix.
    pub fn to_tsv(&self) -> String {
        use std::fmt::Write;
        let mut output = String::from(
            "# HP67 structural IS/PHI; ordinal ticks; PHI widths and sampling edges uncalibrated\n\
             tick\ttransport_bit\tphi1\tphi2\tisa\tact_drive\trom_drive\n",
        );
        for sample in &self.samples {
            writeln!(
                output,
                "{}\t{}\t{:?}\t{:?}\t{:?}\t{:?}\t{:?}",
                sample.tick,
                sample.transport_bit,
                sample.phi1,
                sample.phi2,
                sample.isa,
                sample.act_drive,
                sample.rom_drive,
            )
            .expect("writing to String cannot fail");
        }
        output
    }
}

/// Diagnostic twin: same IS/display/fetch traversal, recording each committed
/// PHI transition. Kept separate to preserve production fast-path codegen.
fn run_traced_structural_word_transport<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    act: &mut ActSerialEndpoint,
    act_state: Option<&ActArchitecturalState>,
    rom: &mut RomFetchEndpoint,
    source: &S,
    mut rom0: Option<&mut Rom0DisplayEndpoint>,
    trace: &mut StructuralBusTrace,
) -> Result<u16, StructuralWordError> {
    let (first_bit, prefix_byte) = act.take_clocked_prefix(backplane, rom0.is_some())?;
    rom.begin_cycle();
    if first_bit == 0 {
        if let Some(endpoint) = rom0.as_deref_mut() {
            endpoint.begin_word();
        }
    }
    // These two execution coordinates are currently structural bookkeeping;
    // their physical cells have already completed in the clocked prefix.
    for bit in 0..first_bit {
        act.advance_execution_for_bit(bit)?;
    }

    // The selected ACT display source is immutable for the whole b0..b7
    // window: executing words read the pre-instruction snapshot and fetch-only
    // words read the supplied live state. Encode it once per machine word
    // instead of rebuilding the same ROM0 byte for all eight serial bits.
    let display_byte = if let Some(byte) = prefix_byte {
        Some(byte)
    } else {
        match act.encoded_display_byte(act_state) {
            Ok(byte) => byte,
            Err(ActDisplaySerialError::UnsupportedModifier { .. }) => Some(HP67_ROM0_BLANK_CODE),
            Err(error) => return Err(error.into()),
        }
    };

    // Hold committed sources until the next complete launch image is ready.
    let instruction_qualifier = act.successor_is_instruction();

    let mut return_driver = None;
    for expected_bit in first_bit..BITS_PER_WORD {
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
        let rom_drive = rom.drive_for_bit(expected_bit);
        let data_drives = None;
        let sync_drive = if sync_decision_window(expected_bit) && instruction_qualifier {
            Drive::High
        } else {
            Drive::Low
        };
        publish_transport_sources(
            backplane,
            expected_bit,
            act_drive,
            rom.is_drives_for_bit(expected_bit),
            sync_drive,
            data_drives,
        )?;

        for _ in 0..2 {
            backplane.advance_clock();
            trace.samples.push(StructuralBusSample {
                tick: backplane.tick().get(),
                transport_bit: expected_bit,
                phi1: backplane.level(Hp67Net::Phi1),
                phi2: backplane.level(Hp67Net::Phi2),
                isa: backplane.level(Hp67Net::Isa),
                act_drive,
                rom_drive,
            });
        }
        let is_level = sample_transport_owned_isa_at_edge(backplane, expected_bit, return_driver)?;

        if let Some(endpoint) = rom0.as_deref_mut() {
            sample_display_transport(act, endpoint, backplane)?;
        }

        match isa_window_for_bit(expected_bit) {
            IsaWindow::RomAddress { .. } => {
                rom.sample_for_bit(expected_bit, is_level, source)?;
                if expected_bit == super::timing::ROM_ADDRESS_LAST_BIT {
                    // One role lookup after the complete received address,
                    // rather than scanning chip selection on each return bit.
                    return_driver = rom.return_driver();
                }
            }
            IsaWindow::RomWord { .. } => {
                act.sample_instruction_at_edge(backplane)?;
                rom.sample_instruction_for_bit(
                    expected_bit,
                    is_level,
                    backplane.level(Hp67Net::Sync),
                )?;
            }
            IsaWindow::Other => {}
        }

        for _ in 0..2 {
            backplane.advance_clock();
            trace.samples.push(StructuralBusSample {
                tick: backplane.tick().get(),
                transport_bit: expected_bit,
                phi1: backplane.level(Hp67Net::Phi1),
                phi2: backplane.level(Hp67Net::Phi2),
                isa: backplane.level(Hp67Net::Isa),
                act_drive,
                rom_drive,
            });
        }
        act.advance_execution_for_bit(expected_bit)?;
    }

    // Coarse word boundary, not an asserted PHI launch edge or propagation time.
    backplane.drive(Hp67Net::Sync, ACT_IS_DRIVER, Drive::Low);
    Ok(act.fetched_word()?)
}

/// Run the structural display/fetch path with an opt-in IS/PHI trace.
///
/// This diagnostic API executes real resolved-net transport, not a waveform
/// synthesized from address/word values. It appends 224 samples on success;
/// failed transport retains only the transitions actually committed.
/// It is not the desktop live authority bridge or a physical timing capture.
pub fn run_structural_display_fetch_cycle_traced<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
    trace: &mut StructuralBusTrace,
) -> Result<StructuralWordResult, StructuralWordError> {
    backplane
        .validate_clock()
        .map_err(SerialFetchError::InvalidClock)?;
    super::sync::validate_sync_transport_entry(backplane)
        .map_err(SerialFetchError::InvalidSyncSource)?;
    let display_scan_slot = act.display_scan_slot();
    act.begin_display_fetch_cycle(address)?;
    let fetched_word = run_traced_structural_word_transport(
        backplane,
        act,
        Some(act_state),
        rom,
        source,
        Some(&mut *rom0),
        trace,
    )?;
    let display_byte = rom0.display_byte()?;
    let strobe = rom0.resolved_strobe()?;
    // Compatibility annotation only. Production cathode selection ignores it.
    let str_event = super::display::Rom0StrEvent {
        scan_slot: display_scan_slot,
    };
    let rcd_falling = strobe.rcd_falling();
    act.complete_display_word();
    Ok(StructuralWordResult {
        fetched_word,
        display_byte,
        str_event,
        rcd_falling,
    })
}

// Dedicated diagnostic DATA twin; production loops remain unchanged.
fn run_traced_structural_word_transport_with_data<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    act: &mut ActSerialEndpoint,
    act_state: Option<&ActArchitecturalState>,
    rom: &mut RomFetchEndpoint,
    source: &S,
    mut rom0: Option<&mut Rom0DisplayEndpoint>,
    data: &mut Hp67DataSerialWordPath,
    trace: &mut StructuralBusTrace,
    mut ram: Option<&mut Hp67SerialRam>,
) -> Result<u16, StructuralWordError> {
    if ram.is_some() {
        data.use_autonomous_ram_owner();
    }
    let (first_bit, prefix_byte) = act.take_clocked_prefix(backplane, rom0.is_some())?;
    if first_bit == 0 {
        if let Some(ram) = ram.as_deref_mut() {
            ram.begin_cycle(backplane)?;
        }
    }
    rom.begin_cycle();
    if first_bit == 0 {
        if let Some(endpoint) = rom0.as_deref_mut() {
            endpoint.begin_word();
        }
    }
    // These two execution coordinates are currently structural bookkeeping;
    // their physical cells have already completed in the clocked prefix.
    for bit in 0..first_bit {
        act.advance_execution_for_bit(bit)?;
    }

    // The selected ACT display source is immutable for the whole b0..b7
    // window: executing words read the pre-instruction snapshot and fetch-only
    // words read the supplied live state. Encode it once per machine word
    // instead of rebuilding the same ROM0 byte for all eight serial bits.
    let display_byte = if let Some(byte) = prefix_byte {
        Some(byte)
    } else {
        match act.encoded_display_byte(act_state) {
            Ok(byte) => byte,
            Err(ActDisplaySerialError::UnsupportedModifier { .. }) => Some(HP67_ROM0_BLANK_CODE),
            Err(error) => return Err(error.into()),
        }
    };

    // Hold committed sources until the next complete launch image is ready.
    let instruction_qualifier = act.successor_is_instruction();

    let mut return_driver = None;
    for expected_bit in first_bit..BITS_PER_WORD {
        debug_assert_eq!(backplane.word_bit(), expected_bit);
        if expected_bit == 2 {
            if let Some(ram) = ram.as_deref_mut() {
                ram.bind_body(backplane)?;
            }
        }
        let display_bit = display_data_serial_bit(expected_bit);
        let act_drive = if let Some(serial_bit) = display_bit {
            match display_byte {
                Some(code) => wired_high_drive(((code >> serial_bit) & 1) != 0),
                None => Drive::HighZ,
            }
        } else {
            act_address_drive(act.address(), expected_bit)
        };
        let rom_drive = rom.drive_for_bit(expected_bit);
        let data_drives = propose_data_with_ram(data, ram.as_deref(), expected_bit, backplane)?;
        let sync_drive = if sync_decision_window(expected_bit) && instruction_qualifier {
            Drive::High
        } else {
            Drive::Low
        };
        publish_transport_sources_with_ram_owner(
            backplane,
            expected_bit,
            act_drive,
            rom.is_drives_for_bit(expected_bit),
            sync_drive,
            data_drives,
            ram.is_some(),
        )?;

        for _ in 0..2 {
            backplane.advance_clock();
            trace.samples.push(StructuralBusSample {
                tick: backplane.tick().get(),
                transport_bit: expected_bit,
                phi1: backplane.level(Hp67Net::Phi1),
                phi2: backplane.level(Hp67Net::Phi2),
                isa: backplane.level(Hp67Net::Isa),
                act_drive,
                rom_drive,
            });
        }
        let is_level = sample_transport_owned_isa_at_edge(backplane, expected_bit, return_driver)?;

        if let Some(endpoint) = rom0.as_deref_mut() {
            sample_display_transport(act, endpoint, backplane)?;
        }

        match isa_window_for_bit(expected_bit) {
            IsaWindow::RomAddress { .. } => {
                rom.sample_for_bit(expected_bit, is_level, source)?;
                if expected_bit == super::timing::ROM_ADDRESS_LAST_BIT {
                    // One role lookup after the complete received address,
                    // rather than scanning chip selection on each return bit.
                    return_driver = rom.return_driver();
                }
            }
            IsaWindow::RomWord { .. } => {
                act.sample_instruction_at_edge(backplane)?;
                rom.sample_instruction_for_bit(
                    expected_bit,
                    is_level,
                    backplane.level(Hp67Net::Sync),
                )?;
            }
            IsaWindow::Other => {}
        }

        sample_data_with_ram(data, ram.as_deref_mut(), expected_bit, backplane)?;

        for _ in 0..2 {
            backplane.advance_clock();
            trace.samples.push(StructuralBusSample {
                tick: backplane.tick().get(),
                transport_bit: expected_bit,
                phi1: backplane.level(Hp67Net::Phi1),
                phi2: backplane.level(Hp67Net::Phi2),
                isa: backplane.level(Hp67Net::Isa),
                act_drive,
                rom_drive,
            });
        }
        act.advance_execution_for_bit(expected_bit)?;
    }

    // Coarse word boundary, not an asserted PHI launch edge or propagation time.
    backplane.drive(Hp67Net::Sync, ACT_IS_DRIVER, Drive::Low);
    Ok(act.fetched_word()?)
}

/// Traced logical-DATA word. Only IS/PHI are observed, never electrical DATA.
pub fn run_structural_display_fetch_data_phase_cycle_traced<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
    data: &mut Hp67DataSerialWordPath,
    data_payload: Option<super::act::ActRegister>,
    trace: &mut StructuralBusTrace,
) -> Result<StructuralDataWordResult, StructuralWordError> {
    run_structural_display_fetch_data_phase_cycle_traced_with_ram(
        backplane,
        address,
        act_state,
        act,
        rom,
        rom0,
        source,
        data,
        data_payload,
        trace,
        None,
    )
}

/// Shared transport with an optional autonomous aggregate RAM participant.
pub fn run_structural_display_fetch_data_phase_cycle_traced_with_ram<S: Hp67RomWordSource>(
    backplane: &mut Hp67ElectricalBackplane,
    address: u16,
    act_state: &ActArchitecturalState,
    act: &mut ActSerialEndpoint,
    rom: &mut RomFetchEndpoint,
    rom0: &mut Rom0DisplayEndpoint,
    source: &S,
    data: &mut Hp67DataSerialWordPath,
    data_payload: Option<super::act::ActRegister>,
    trace: &mut StructuralBusTrace,
    ram: Option<&mut Hp67SerialRam>,
) -> Result<StructuralDataWordResult, StructuralWordError> {
    backplane
        .validate_clock()
        .map_err(SerialFetchError::InvalidClock)?;
    super::sync::validate_sync_transport_entry(backplane)
        .map_err(SerialFetchError::InvalidSyncSource)?;
    let display_scan_slot = act.display_scan_slot();
    act.begin_display_fetch_cycle(address)?;
    if ram.is_some() {
        data.begin_autonomous_word(data_payload);
    } else {
        data.begin_word(data_payload);
    }
    let fetched_word = run_traced_structural_word_transport_with_data(
        backplane,
        act,
        Some(act_state),
        rom,
        source,
        Some(&mut *rom0),
        data,
        trace,
        ram,
    )?;
    let completed_data = data.complete_word();
    let display_byte = rom0.display_byte()?;
    let strobe = rom0.resolved_strobe()?;
    // Compatibility annotation only. Production cathode selection ignores it.
    let str_event = super::display::Rom0StrEvent {
        scan_slot: display_scan_slot,
    };
    let rcd_falling = strobe.rcd_falling();
    act.complete_display_word();
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

    #[test]
    fn m15q_shared_ram_data_receive_rejects_faults_before_either_commit() {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut data = Hp67DataSerialWordPath::default();
        data.enable_electrical_level_bridge(None);
        data.begin_word(None);
        let mut ram = Hp67SerialRam::default();
        bus.advance_clock();
        bus.advance_clock();
        let before_data = data;
        let before_ram = ram.clone();
        assert!(sample_data_with_ram(&mut data, Some(&mut ram), 0, &bus).is_err());
        assert_eq!(data, before_data);
        assert_eq!(ram, before_ram);
        // Return to the b0 launch checkpoint with a separate deterministic bus.
        let mut bus = Hp67ElectricalBackplane::default();
        ram.begin_cycle(&bus).unwrap();
        bus.advance_clock();
        bus.advance_clock();
        let before_ram = ram.clone();
        bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::High);
        assert!(sample_data_with_ram(&mut data, Some(&mut ram), 0, &bus).is_err());
        assert_eq!(data, before_data);
        assert_eq!(ram, before_ram);
        bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::HighZ);
        sample_data_with_ram(&mut data, Some(&mut ram), 0, &bus).unwrap();
        assert_eq!(bus.tick().get(), 2);
    }

    #[test]
    fn m15k_all_transports_follow_captured_bank_roundtrip_without_provider_selection() {
        use super::super::hp67firmware::Hp67Firmware;
        for path in 0..4 {
            let mut backplane = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0);
            let mut rom = RomFetchEndpoint::autonomous_roms();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            let state = ActArchitecturalState::default();
            let source = Hp67Firmware::default();
            let mut trace = StructuralBusTrace::default();
            // Teenix printed p71/72 captured real firmware route. Deliberately
            // poison the provider's reference bank view before each transfer.
            for (pc, bank, expected) in [
                (0x7f4, 0, 0x230),
                (0x7f5, 1, 0x11a),
                (0x495, 1, 0x210),
                (0x0d5, 0, 0x0b4),
            ] {
                source.select_bank(bank ^ 1);
                let result = match path {
                    0 => run_structural_display_fetch_cycle(
                        &mut backplane,
                        pc,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                    )
                    .map(|word| word.fetched_word),
                    1 => run_structural_display_fetch_data_phase_cycle(
                        &mut backplane,
                        pc,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut data,
                        None,
                    )
                    .map(|word| word.word.fetched_word),
                    2 => run_structural_display_fetch_cycle_traced(
                        &mut backplane,
                        pc,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut trace,
                    )
                    .map(|word| word.fetched_word),
                    _ => run_structural_display_fetch_data_phase_cycle_traced(
                        &mut backplane,
                        pc,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut data,
                        None,
                        &mut trace,
                    )
                    .map(|word| word.word.fetched_word),
                };
                assert_eq!(result, Ok(expected), "path={path} pc={pc:03x}");
                let chips = rom.physical_chip_set().unwrap();
                assert!(chips.has_autonomous_banks());
                assert_eq!(
                    chips.chips().iter().filter(|chip| chip.selected()).count(),
                    1
                );
                assert_eq!(
                    chips
                        .chips()
                        .iter()
                        .find(|chip| chip.selected())
                        .unwrap()
                        .profile()
                        .bank,
                    bank
                );
            }
            assert_eq!(backplane.tick().get(), 4 * 224);
            if path >= 2 {
                assert_eq!(trace.samples.len(), 4 * 224);
            }
        }
    }

    #[test]
    fn m15j_all_transport_variants_publish_the_selected_physical_rom_driver() {
        use super::super::rom_devices::HP67_ROM_PROFILES;
        struct Source(u8);
        impl Hp67RomWordSource for Source {
            fn selected_rom_bank(&self, _address: u16) -> u8 {
                self.0
            }
            fn read_word(&self, _address: u16) -> Option<u16> {
                Some(0x200)
            }
        }
        for path in 0..4 {
            let mut backplane = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0);
            let mut rom = RomFetchEndpoint::physical_roms();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            let state = ActArchitecturalState::default();
            let mut trace = StructuralBusTrace::default();
            for profile in HP67_ROM_PROFILES {
                let source = Source(profile.bank);
                let address = profile.first_address | 0x15;
                let result = match path {
                    0 => run_structural_display_fetch_cycle(
                        &mut backplane,
                        address,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                    )
                    .map(|word| word.fetched_word),
                    1 => run_structural_display_fetch_data_phase_cycle(
                        &mut backplane,
                        address,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut data,
                        None,
                    )
                    .map(|word| word.word.fetched_word),
                    2 => run_structural_display_fetch_cycle_traced(
                        &mut backplane,
                        address,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut trace,
                    )
                    .map(|word| word.fetched_word),
                    _ => run_structural_display_fetch_data_phase_cycle_traced(
                        &mut backplane,
                        address,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut data,
                        None,
                        &mut trace,
                    )
                    .map(|word| word.word.fetched_word),
                };
                assert_eq!(result, Ok(0x200));
                assert_eq!(
                    backplane.driver_drive(Hp67Net::Isa, Hp67Driver::StructuralRomResponder),
                    Drive::HighZ
                );
                for candidate in HP67_ROM_PROFILES {
                    assert_eq!(
                        backplane.driver_drive(Hp67Net::Isa, candidate.driver),
                        if candidate == profile {
                            Drive::High
                        } else {
                            Drive::HighZ
                        }
                    );
                }
                assert_eq!(backplane.active_driver_count(Hp67Net::Isa), 1);
                for chip in rom.physical_chip_set().unwrap().chips() {
                    assert_eq!(chip.received_address(), Ok(address));
                    assert_eq!(chip.selected(), chip.profile() == profile);
                }
            }
            assert_eq!(backplane.tick().get(), 5 * 224);
            if path >= 2 {
                assert_eq!(trace.samples.len(), 5 * 224);
            }
        }
    }

    #[test]
    fn m15i_invalid_launch_phase_preserves_the_entire_committed_image() {
        let mut backplane = Hp67ElectricalBackplane::default();
        publish_transport_sources(
            &mut backplane,
            0,
            Drive::High,
            RomIsDrives::Aggregate(Drive::HighZ),
            Drive::High,
            Some((Drive::Low, Drive::HighZ)),
        )
        .unwrap();
        for (bit, expected_tick) in [(1, 0), (0, 1)] {
            let before = Hp67Net::ALL.map(|net| backplane.level(net));
            let result = publish_transport_sources(
                &mut backplane,
                bit,
                Drive::HighZ,
                RomIsDrives::Aggregate(Drive::High),
                Drive::Low,
                Some((Drive::HighZ, Drive::High)),
            );
            assert_eq!(
                result,
                Err(SerialFetchError::InvalidLaunchPhase {
                    tick: expected_tick
                })
            );
            assert_eq!(Hp67Net::ALL.map(|net| backplane.level(net)), before);
            assert_eq!(backplane.tick().get(), expected_tick);
            if expected_tick == 0 {
                backplane.advance_clock();
            }
        }
    }

    #[test]
    fn m15i_data_proposal_failure_does_not_partially_publish_next_is_bit() {
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
        state.a[0] = 4; // b1 Low, next b2 High: a partial IS publication is visible.
        data.enable_electrical_level_bridge(None); // Required DATA owner missing.
        let result = run_structural_display_fetch_data_phase_cycle(
            &mut backplane,
            0x07b,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
            &mut data,
            Some([1; 14]),
        );
        assert_eq!(
            result,
            Err(StructuralWordError::Data(
                Hp67DataSerialError::MissingElectricalOwner { word_bit: 2 }
            ))
        );
        assert_eq!(backplane.tick().get(), 8);
        assert_eq!(backplane.word_bit(), 2);
        assert_eq!(backplane.level(Hp67Net::Isa), LogicLevel::Low);
        assert_eq!(backplane.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(backplane.level(Hp67Net::Sync), LogicLevel::Low);
        assert_eq!(
            rom0.display_byte(),
            Err(Rom0DisplayError::IncompleteDisplayByte { received_mask: 3 })
        );
        assert_eq!(act.display_scan_slot(), 1);
    }

    #[test]
    fn m15h_is_receiver_rejects_early_and_late_phases_and_observes_interedge_faults() {
        let mut backplane = Hp67ElectricalBackplane::default();
        backplane.drive(Hp67Net::Isa, ACT_IS_DRIVER, Drive::High);
        for tick in 0..=4 {
            let sampled = sample_transport_isa_at_edge(&backplane, backplane.word_bit());
            if tick == 2 {
                assert_eq!(sampled, Ok(LogicLevel::High));
                backplane.drive(Hp67Net::Phi1, Hp67Driver::External, Drive::Low);
                assert_eq!(
                    sample_transport_isa_at_edge(&backplane, 0),
                    Err(SerialFetchError::InvalidSamplePhase { tick: 2 })
                );
                backplane.drive(Hp67Net::Phi1, Hp67Driver::External, Drive::HighZ);
                backplane.drive(Hp67Net::Isa, Hp67Driver::External, Drive::High);
                assert!(matches!(
                    sample_transport_isa_at_edge(&backplane, 0),
                    Err(SerialFetchError::InvalidIsaDrives { .. })
                ));
                backplane.drive(Hp67Net::Isa, Hp67Driver::External, Drive::HighZ);
            } else {
                assert_eq!(sampled, Err(SerialFetchError::InvalidSamplePhase { tick }));
            }
            if tick < 4 {
                backplane.advance_clock();
            }
        }
    }

    #[test]
    fn m15g_display_transport_rejects_control_fault_without_advancing_scan() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        for net in [Hp67Net::Str, Hp67Net::Rcd] {
            let mut backplane = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0x07b);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            backplane.drive(net, Hp67Driver::External, Drive::High);
            let result = run_structural_display_fetch_cycle(
                &mut backplane,
                0x07b,
                &ActArchitecturalState::default(),
                &mut act,
                &mut rom,
                &mut rom0,
                &source,
            );
            assert!(matches!(result, Err(StructuralWordError::Display(
                Rom0DisplayError::InvalidControlSample { net: failed, .. })) if failed == net));
            assert_eq!(act.display_scan_slot(), 1);
            assert_eq!(backplane.tick().get(), 2);
        }
    }

    #[test]
    fn m15f_clocked_tail_split_matches_whole_word_transport_and_trace() {
        clocked_tail_split_matches_whole_word_transport_and_trace(false);
    }

    #[test]
    fn m15j_physical_roms_preserve_clocked_data_tail_and_trace_parity() {
        clocked_tail_split_matches_whole_word_transport_and_trace(true);
    }

    fn clocked_tail_split_matches_whole_word_transport_and_trace(physical: bool) {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        state.a = [7; 14];
        let mut split = Hp67ElectricalBackplane::default();
        let mut whole = Hp67ElectricalBackplane::default();
        let mut split_act = ActSerialEndpoint::new(0x07b);
        let mut whole_act = ActSerialEndpoint::new(0x07b);
        let mut split_rom = if physical {
            RomFetchEndpoint::physical_roms()
        } else {
            RomFetchEndpoint::default()
        };
        let mut whole_rom = if physical {
            RomFetchEndpoint::physical_roms()
        } else {
            RomFetchEndpoint::default()
        };
        let mut split_rom0 = Rom0DisplayEndpoint::default();
        let mut whole_rom0 = Rom0DisplayEndpoint::default();
        let mut split_data = Hp67DataSerialWordPath::default();
        let mut whole_data = Hp67DataSerialWordPath::default();
        let mut split_trace = StructuralBusTrace::default();
        let mut whole_trace = StructuralBusTrace::default();
        let first = [3; 14];
        let second = [9; 14];
        for (direction, payload, tail) in [
            (
                Some(super::super::data::Hp67DataTransferDirection::ActToPeripheral),
                Some(first),
                None,
            ),
            (
                Some(super::super::data::Hp67DataTransferDirection::PeripheralToAct),
                Some(second),
                Some(first),
            ),
            (None, None, Some(second)),
        ] {
            if tail.is_some() {
                let tick = split.tick().get();
                let received = run_structural_data_tail_prefix(
                    &mut split,
                    &state,
                    &mut split_act,
                    &split_rom,
                    &mut split_rom0,
                    &mut split_data,
                    Some(&mut split_trace),
                )
                .unwrap();
                assert_eq!(received, tail);
                assert_eq!(split.tick().get(), tick + 8);
                assert_eq!(split.word_bit(), 2);
                assert!(split_rom0.display_byte().is_err());
            }
            split_data.enable_electrical_level_bridge(direction);
            whole_data.enable_electrical_level_bridge(direction);
            split_act.begin_execution(0, &state).unwrap();
            whole_act.begin_execution(0, &state).unwrap();
            let partial = run_structural_display_fetch_data_phase_cycle_traced(
                &mut split,
                0x07b,
                &state,
                &mut split_act,
                &mut split_rom,
                &mut split_rom0,
                &source,
                &mut split_data,
                payload,
                &mut split_trace,
            )
            .unwrap();
            let full = run_structural_display_fetch_data_phase_cycle_traced(
                &mut whole,
                0x07b,
                &state,
                &mut whole_act,
                &mut whole_rom,
                &mut whole_rom0,
                &source,
                &mut whole_data,
                payload,
                &mut whole_trace,
            )
            .unwrap();
            assert_eq!(partial.completed_data, None);
            assert_eq!(full.completed_data, tail);
            assert_eq!(partial.word, full.word);
            assert_eq!(split_act, whole_act);
            assert_eq!(split_data, whole_data);
            assert_eq!(split.tick(), whole.tick());
            for net in Hp67Net::ALL {
                assert_eq!(split.level(net), whole.level(net));
            }
        }
        assert_eq!(split_trace, whole_trace);
        assert_eq!(split_trace.samples.len(), 3 * 224);
        assert_eq!(split.tick().get(), 3 * 224);
    }

    #[test]
    fn m15f_prefix_rejects_unbound_or_mid_cell_transport() {
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0);
        let mut rom0 = Rom0DisplayEndpoint::default();
        let mut data = Hp67DataSerialWordPath::default();
        let state = ActArchitecturalState::default();
        assert_eq!(
            run_structural_data_tail_prefix(
                &mut backplane,
                &state,
                &mut act,
                &RomFetchEndpoint::default(),
                &mut rom0,
                &mut data,
                None,
            ),
            Err(StructuralWordError::Data(
                Hp67DataSerialError::ElectricalBridgeRequired
            ))
        );
        backplane.advance_clock();
        assert_eq!(
            run_structural_data_tail_prefix(
                &mut backplane,
                &state,
                &mut act,
                &RomFetchEndpoint::default(),
                &mut rom0,
                &mut data,
                None,
            ),
            Err(StructuralWordError::Fetch(
                SerialFetchError::InvalidTransportStart { tick: 1 }
            ))
        );
        assert_eq!(backplane.tick().get(), 1);
    }

    #[test]
    fn m15d_sync_successor_decode_matches_all_supported_oracle_words() {
        use crate::machines::hp67::{ActArchitecturalCore, ActInstructionState};
        let mut checked = 0;
        for mode in [ActInstructionState::Normal, ActInstructionState::ThenGoto] {
            for carry in [false, true] {
                for word in 0..=ROM_WORD_MASK {
                    let mut core = ActArchitecturalCore::default();
                    core.state.instruction_state = mode;
                    core.state.carry = carry;
                    let before = core.state.clone();
                    if core.execute_word(&mut ActRamImage::hp67(), word).is_err() {
                        continue;
                    }
                    let mut act = ActSerialEndpoint::new(0x07b);
                    act.begin_execution(word, &before).unwrap();
                    assert_eq!(
                        act.successor_is_instruction(),
                        core.state.instruction_state == ActInstructionState::Normal,
                        "word={word:03x} mode={mode:?} carry={carry}"
                    );
                    checked += 1;
                }
            }
        }
        assert!(checked > 2048);
    }

    #[test]
    fn m15d_resolved_sync_qualifies_normal_if_and_implied_goto_in_all_paths() {
        use crate::machines::hp67::ActInstructionState;
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        for path in 0..4 {
            let mut backplane = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0x07b);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            let mut trace = StructuralBusTrace::default();
            for (mode, executing, expected) in [
                (ActInstructionState::Normal, 0, ActInstructionState::Normal),
                (
                    ActInstructionState::Normal,
                    0o0034,
                    ActInstructionState::ThenGoto,
                ),
                (
                    ActInstructionState::ThenGoto,
                    0o0034,
                    ActInstructionState::Normal,
                ),
            ] {
                let mut state = ActArchitecturalState::default();
                state.instruction_state = mode;
                act.begin_execution(executing, &state).unwrap();
                let result = match path {
                    0 => run_structural_display_fetch_cycle(
                        &mut backplane,
                        0x07b,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                    )
                    .map(|result| result.fetched_word),
                    1 => run_structural_display_fetch_data_phase_cycle(
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
                    .map(|result| result.word.fetched_word),
                    2 => run_structural_display_fetch_cycle_traced(
                        &mut backplane,
                        0x07b,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut trace,
                    )
                    .map(|result| result.fetched_word),
                    _ => run_structural_display_fetch_data_phase_cycle_traced(
                        &mut backplane,
                        0x07b,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &source,
                        &mut data,
                        None,
                        &mut trace,
                    )
                    .map(|result| result.word.fetched_word),
                };
                assert_eq!(result.unwrap(), 0x04c);
                assert_eq!(act.fetched_instruction_state().unwrap(), expected);
                assert_eq!(backplane.level(Hp67Net::Sync), LogicLevel::Low);
            }
            assert_eq!(backplane.tick().get(), 3 * 224);
            if path >= 2 {
                assert_eq!(trace.samples.len(), 3 * 224);
            }
        }
    }

    #[test]
    fn m15d_sync_contention_rejects_return_before_is_receiver_consumes_bit() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        backplane.drive(Hp67Net::Sync, Hp67Driver::External, Drive::Low);
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let result = run_structural_fetch_cycle(&mut backplane, 0x07b, &mut act, &mut rom, &source);
        assert!(
            matches!(result, Err(SerialFetchError::InvalidSyncSource(fault))
            if fault.tick == 0 && fault.word_bit == 0
                && fault.active_mask == 1 << Hp67Driver::External.index())
        );
        assert_eq!(backplane.tick().get(), 0);
        assert_eq!(act.received_mask, 0);
        assert!(act.fetched_instruction_state().is_err());
    }

    #[test]
    fn m15d_sync_receiver_rejects_floating_and_changed_qualifier() {
        let mut act = ActSerialEndpoint::new(0);
        assert_eq!(
            act.sample_sync_for_bit(46, LogicLevel::Floating),
            Err(SerialFetchError::UnresolvedSync {
                word_bit: 46,
                level: LogicLevel::Floating
            })
        );
        act.sample_sync_for_bit(46, LogicLevel::High).unwrap();
        assert_eq!(
            act.sample_sync_for_bit(47, LogicLevel::Low),
            Err(SerialFetchError::ChangedSyncQualifier { word_bit: 47 })
        );
        assert!(act.fetched_instruction_state().is_err());
    }

    #[test]
    fn m15b_all_transport_paths_reject_hidden_high_overlap_before_sampling() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        // DISPLAY-off emits 0x20; the foreign sole High at b0 must fail before overlap at b5.
        let state = ActArchitecturalState::default();
        assert!(!state.display_enable);
        for path in 0..4 {
            let mut backplane = Hp67ElectricalBackplane::default();
            backplane.drive(Hp67Net::Isa, Hp67Driver::External, Drive::High);
            let mut act = ActSerialEndpoint::new(0x07b);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            let mut trace = StructuralBusTrace::default();
            let result = match path {
                0 => run_structural_display_fetch_cycle(
                    &mut backplane,
                    0x07b,
                    &state,
                    &mut act,
                    &mut rom,
                    &mut rom0,
                    &source,
                )
                .map(|_| ()),
                1 => run_structural_display_fetch_data_phase_cycle(
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
                .map(|_| ()),
                2 => run_structural_display_fetch_cycle_traced(
                    &mut backplane,
                    0x07b,
                    &state,
                    &mut act,
                    &mut rom,
                    &mut rom0,
                    &source,
                    &mut trace,
                )
                .map(|_| ()),
                _ => run_structural_display_fetch_data_phase_cycle_traced(
                    &mut backplane,
                    0x07b,
                    &state,
                    &mut act,
                    &mut rom,
                    &mut rom0,
                    &source,
                    &mut data,
                    None,
                    &mut trace,
                )
                .map(|_| ()),
            };
            assert_eq!(
                result,
                Err(StructuralWordError::Fetch(
                    SerialFetchError::UnexpectedIsaSource {
                        word_bit: 0,
                        expected: Some(Hp67Driver::Act1820_2530),
                        driver_mask: 1u16 << Hp67Driver::External.index(),
                    }
                )),
                "path {path}"
            );
            assert_eq!(backplane.level(Hp67Net::Isa), LogicLevel::High);
            assert_eq!(backplane.tick().get(), 2);
            assert_eq!(backplane.word_bit(), 0);
            assert!(rom0.display_byte().is_err());
            if path >= 2 {
                assert_eq!(trace.samples.len(), 2);
            }
        }
    }

    #[test]
    fn m15b_active_low_is_rejected_even_without_opposite_drive() {
        let mut backplane = Hp67ElectricalBackplane::default();
        backplane.drive(Hp67Net::Isa, Hp67Driver::External, Drive::Low);
        assert_eq!(backplane.level(Hp67Net::Isa), LogicLevel::Low);
        assert_eq!(
            sample_transport_isa(&backplane, 0),
            Err(SerialFetchError::InvalidIsaDrives {
                word_bit: 0,
                driver_mask: 1u16 << Hp67Driver::External.index(),
            })
        );
        backplane.drive(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::High);
        assert_eq!(
            sample_transport_isa(&backplane, 0),
            Err(SerialFetchError::IsaContention { word_bit: 0 })
        );
    }

    #[test]
    fn m14w_traced_transport_matches_fast_path_across_complete_scan() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut state = ActArchitecturalState::default();
        state.display_enable = true;
        state.a[0] = 7;
        state.b[0] = 3;
        let mut fast = Hp67ElectricalBackplane::default();
        let mut traced = Hp67ElectricalBackplane::default();
        let mut fast_act = ActSerialEndpoint::new(0x07b);
        let mut traced_act = ActSerialEndpoint::new(0x07b);
        let mut fast_rom = RomFetchEndpoint::default();
        let mut traced_rom = RomFetchEndpoint::default();
        let mut fast_rom0 = Rom0DisplayEndpoint::default();
        let mut traced_rom0 = Rom0DisplayEndpoint::default();
        let mut trace = StructuralBusTrace::default();
        for slot in 0..15 {
            let address = if slot % 2 == 0 { 0x07b } else { 0x001 };
            fast_act.begin_execution(0o0210, &state).unwrap();
            traced_act.begin_execution(0o0210, &state).unwrap();
            let expected = run_structural_display_fetch_cycle(
                &mut fast,
                address,
                &state,
                &mut fast_act,
                &mut fast_rom,
                &mut fast_rom0,
                &source,
            )
            .unwrap();
            let actual = run_structural_display_fetch_cycle_traced(
                &mut traced,
                address,
                &state,
                &mut traced_act,
                &mut traced_rom,
                &mut traced_rom0,
                &source,
                &mut trace,
            )
            .unwrap();
            assert_eq!(actual, expected);
            assert_eq!(traced_act, fast_act);
            let fast_image = fast_act.serial_mode_latch_result_image().unwrap();
            let traced_image = traced_act.serial_mode_latch_result_image().unwrap();
            assert!(fast_image.is_complete() && traced_image.is_complete());
            assert_eq!(traced_image.display_enable(), fast_image.display_enable());
            assert_eq!(traced_rom.received_address(), fast_rom.received_address());
            assert_eq!(traced_act.display_scan_slot(), fast_act.display_scan_slot());
            assert_eq!(traced.tick(), fast.tick());
            for net in Hp67Net::ALL {
                assert_eq!(traced.level(net), fast.level(net));
            }
            let rows = &trace.samples[slot * 224..(slot + 1) * 224];
            let mut address_from_bus = 0u16;
            let mut word_from_bus = 0u16;
            let mut display_from_bus = 0u8;
            for bit in 0..56usize {
                let cell = &rows[bit * 4..bit * 4 + 4];
                for (phase, sample) in cell.iter().enumerate() {
                    assert_eq!(sample.tick, (slot * 224 + bit * 4 + phase + 1) as u64);
                    assert_eq!(sample.transport_bit, bit as u8);
                    assert_eq!(sample.phi1 == LogicLevel::Low, phase == 0);
                    assert_eq!(sample.phi2 == LogicLevel::Low, phase == 2);
                    assert_eq!(sample.isa, cell[0].isa);
                    assert_ne!(sample.isa, LogicLevel::Contention);
                    assert_ne!(sample.isa, LogicLevel::Floating);
                }
                let high = u16::from(cell[0].isa == LogicLevel::High);
                match bit {
                    0..=7 => display_from_bus |= (high as u8) << bit,
                    16..=27 => address_from_bus |= high << (bit - 16),
                    46..=55 => word_from_bus |= high << (bit - 46),
                    _ => {}
                }
                if bit < 46 {
                    assert_eq!(cell[0].rom_drive, Drive::HighZ);
                }
                if (8..16).contains(&bit) || bit > 27 {
                    assert_eq!(cell[0].act_drive, Drive::HighZ);
                }
            }
            assert_eq!(address_from_bus, address);
            assert_eq!(word_from_bus, actual.fetched_word);
            assert_eq!(display_from_bus, actual.display_byte);
        }
        let tsv = trace.to_tsv();
        assert_eq!(tsv.lines().count(), 2 + 15 * 224);
        assert!(tsv.contains("uncalibrated"));
        assert!(tsv.contains("\n3360\t55\tHigh\tHigh\t"));
        assert_eq!(trace.samples.len(), 15 * 224);
        assert_eq!(traced.word_index(), 15);
        assert_eq!(traced.word_bit(), 0);
    }

    #[test]
    fn m14w_trace_keeps_only_committed_prefix_on_missing_rom() {
        trace_keeps_only_committed_prefix_on_missing_rom(false);
    }

    #[test]
    fn m15j_physical_missing_rom_retains_the_exact_committed_prefix() {
        trace_keeps_only_committed_prefix_on_missing_rom(true);
    }

    fn trace_keeps_only_committed_prefix_on_missing_rom(physical: bool) {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x002);
        let mut rom = if physical {
            RomFetchEndpoint::physical_roms()
        } else {
            RomFetchEndpoint::default()
        };
        let mut rom0 = Rom0DisplayEndpoint::default();
        let mut trace = StructuralBusTrace::default();
        let result = run_structural_display_fetch_cycle_traced(
            &mut backplane,
            0x002,
            &ActArchitecturalState::default(),
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
            &mut trace,
        );
        assert!(result.is_err());
        // Address lookup now fails at the b27 PHI1-rising sample checkpoint.
        assert_eq!(trace.samples.len(), 27 * 4 + 2);
        assert_eq!(trace.samples.last().unwrap().transport_bit, 27);
        assert_eq!(trace.samples.last().unwrap().phi1, LogicLevel::High);
        assert_eq!(trace.samples.last().unwrap().phi2, LogicLevel::High);
        assert_eq!(
            backplane.clock_phase(),
            super::super::timing::Hp67ClockPhase::InterphaseAfterPhi1
        );
        assert_eq!(backplane.tick().get(), 110);
        assert_eq!(backplane.word_bit(), 27);
    }

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
    fn control_result_image_completes_only_after_b55() {
        let mut state = ActArchitecturalState::default();
        state.p = 13;
        state.p_change = [0, -1, 1];
        state.carry = true;

        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0720, &state)
            .expect("P increment must start serial execution");

        let initial = act
            .serial_control_result_image()
            .expect("P increment must create a control result image");
        assert!(!initial.is_complete());
        assert_eq!(initial.p(), 13);

        for bit in 0..(BITS_PER_WORD - 1) {
            act.advance_execution_for_bit(bit)
                .expect("control execution must advance before b55");
        }
        let before_last = act
            .serial_control_result_image()
            .expect("control result image must remain present");
        assert!(!before_last.is_complete());
        assert_eq!(before_last.p(), 13);

        act.advance_execution_for_bit(BITS_PER_WORD - 1)
            .expect("b55 must complete control execution");
        let complete = act
            .serial_control_result_image()
            .expect("completed control result image must remain inspectable");
        assert!(complete.is_complete());
        assert_eq!(complete.p(), 0);
        assert_eq!(complete.p_change(), [1, 0, -1]);
        assert!(!complete.carry());
        assert!(complete.previous_carry());
    }

    #[test]
    fn special_register_result_image_completes_only_after_b55() {
        let mut state = ActArchitecturalState::default();
        state.c = std::array::from_fn(|digit| digit as u8);
        state.m1 = std::array::from_fn(|digit| (13 - digit) as u8);

        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0410, &state)
            .expect("C/M1 exchange must start serial execution");

        let initial = act
            .serial_special_register_result_image()
            .expect("C/M1 exchange must create a special-register result image");
        assert!(!initial.is_complete());
        assert_eq!(initial.c(), &state.c);
        assert_eq!(initial.m1(), &state.m1);

        for bit in 0..(BITS_PER_WORD - 1) {
            act.advance_execution_for_bit(bit)
                .expect("special-register execution must advance before b55");
        }
        assert!(!act
            .serial_special_register_result_image()
            .expect("special-register image must remain present")
            .is_complete());

        act.advance_execution_for_bit(BITS_PER_WORD - 1)
            .expect("b55 must complete special-register execution");
        let complete = act
            .serial_special_register_result_image()
            .expect("completed special-register image must remain inspectable");
        assert!(complete.is_complete());
        assert_eq!(complete.c(), &state.m1);
        assert_eq!(complete.m1(), &state.c);
    }

    #[test]
    fn display_override_does_not_change_mode_latch_pre_state() {
        let mut state = ActArchitecturalState::default();
        state.display_enable = false;

        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0210, &state)
            .expect("display toggle must start serial execution");
        act.set_execution_display_enable_override(true);

        let image = act
            .serial_mode_latch_result_image()
            .expect("display toggle must create a mode/latch image");
        assert!(!image.is_complete());
        assert!(!image.display_enable());
        assert_eq!(
            act.serial_execution_state()
                .expect("execution snapshot must remain present")
                .display_enable(),
            false
        );
    }

    #[test]
    fn m14n_absent_ram_image_follows_structural_lifetime() {
        let state = ActArchitecturalState {
            ram_address: 0xff,
            c: [9; 14],
            ..Default::default()
        };
        let ram = ActRamImage::hp67();
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0070, &state).unwrap();
        act.bind_ram_topology(&ram).unwrap();
        assert!(act.bind_ram_topology(&ram).is_err());
        for bit in 0..BITS_PER_WORD - 1 {
            act.advance_execution_for_bit(bit).unwrap();
            assert!(!act.serial_absent_ram_read_image().unwrap().is_complete());
        }
        act.advance_execution_for_bit(BITS_PER_WORD - 1).unwrap();
        assert!(act.serial_absent_ram_read_image().unwrap().is_complete());
        act.begin_execution(0, &state).unwrap();
        assert!(act.serial_absent_ram_read_image().is_none());
        act.advance_execution_for_bit(0).unwrap();
        assert!(act.bind_ram_topology(&ram).is_err());
    }

    #[test]
    fn m14m_crc_data_binding_is_frozen_and_requires_b55() {
        let state = ActArchitecturalState {
            ram_address: super::super::CRC_RAM_READ_ADDRESS,
            ..Default::default()
        };
        let mut crc = CrcArchitecturalCore::default();
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0070, &state).unwrap();
        // Failed staging must permit a corrected bind before execution starts.
        assert!(act.bind_crc_control(&crc).is_err());
        crc.present_read_word(0x1234567).unwrap();
        act.bind_crc_control(&crc).unwrap();
        assert!(act.bind_crc_control(&crc).is_err());
        for bit in 0..BITS_PER_WORD - 1 {
            act.advance_execution_for_bit(bit).unwrap();
            assert!(!act.serial_crc_data_result_image().unwrap().is_complete());
            assert_eq!(crc.buffered_read_word(), Some(0x1234567));
        }
        act.advance_execution_for_bit(BITS_PER_WORD - 1).unwrap();
        assert!(act.serial_crc_data_result_image().unwrap().is_complete());
        act.begin_execution(0, &state).unwrap();
        assert!(act.serial_crc_data_result_image().is_none());
        act.bind_crc_control(&crc).unwrap();
        assert!(act.serial_crc_data_result_image().is_none());
        let mut late = ActSerialEndpoint::new(0);
        late.begin_execution(0o0070, &state).unwrap();
        late.advance_execution_for_bit(0).unwrap();
        assert!(late.bind_crc_control(&crc).is_err());
    }

    #[test]
    fn m14l_crc_control_freezes_pre_state_and_completes_only_after_b55() {
        let state = ActArchitecturalState::default();
        let mut crc = CrcArchitecturalCore::default();
        crc.set_external_flag(0, true).unwrap();
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o100, &state).unwrap();
        act.bind_crc_control(&crc).unwrap();
        crc.set_external_flag(0, false).unwrap();
        assert!(act.bind_crc_control(&crc).is_err());
        for bit in 0..BITS_PER_WORD - 1 {
            act.advance_execution_for_bit(bit).unwrap();
            let image = act.serial_crc_control_result_image().unwrap();
            assert!(!image.is_complete());
            assert_eq!(image.condition(), Some(true));
        }
        act.advance_execution_for_bit(BITS_PER_WORD - 1).unwrap();
        assert!(act.serial_crc_control_result_image().unwrap().is_complete());
        act.begin_execution(0o0000, &state).unwrap();
        act.bind_crc_control(&crc).unwrap();
        assert!(act.serial_crc_control_result_image().is_none());
        let mut late = ActSerialEndpoint::new(0);
        late.begin_execution(0o100, &state).unwrap();
        late.advance_execution_for_bit(0).unwrap();
        assert!(late.bind_crc_control(&crc).is_err());
    }

    #[test]
    fn m14k_ram_image_completes_only_after_b55() {
        let state = ActArchitecturalState {
            ram_address: 0x23,
            ..Default::default()
        };
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution((5 << 6) | 0o70, &state).unwrap();
        for bit in 0..BITS_PER_WORD - 1 {
            act.advance_execution_for_bit(bit).unwrap();
            let image = act.serial_ram_result_image().unwrap();
            assert!(!image.is_complete());
            assert_eq!(image.ram_address(), 0x25);
        }
        act.advance_execution_for_bit(BITS_PER_WORD - 1).unwrap();
        assert!(act.serial_ram_result_image().unwrap().is_complete());
        act.begin_execution(0o0070, &state).unwrap();
        assert!(act.serial_ram_result_image().is_none());
    }

    #[test]
    fn m14j_boundary_image_completes_only_after_b55() {
        let state = ActArchitecturalState {
            carry: true,
            previous_carry: false,
            p_change: [1, -1, 0],
            ..Default::default()
        };
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0000, &state).unwrap();
        for bit in 0..BITS_PER_WORD - 1 {
            act.advance_execution_for_bit(bit).unwrap();
            assert!(!act.serial_boundary_result_image().unwrap().is_complete());
        }
        act.advance_execution_for_bit(BITS_PER_WORD - 1).unwrap();
        let image = act.serial_boundary_result_image().unwrap();
        assert!(image.is_complete());
        let mut actual = state;
        image.commit(&mut actual);
        assert_eq!(actual.p_change, [0, 1, -1]);
        assert!(actual.previous_carry);
        assert!(!actual.carry);
        act.begin_execution(0o0720, &actual).unwrap();
        assert!(act.serial_boundary_result_image().is_none());
    }

    #[test]
    fn m14i_flow_image_completes_only_after_b55() {
        let state = ActArchitecturalState {
            pc: 0x0068,
            delayed_rom: Some(15),
            ..Default::default()
        };
        let mut act = ActSerialEndpoint::new(state.pc);
        act.begin_execution((0xc6 << 2) | 1, &state).unwrap();
        for bit in 0..BITS_PER_WORD - 1 {
            act.advance_execution_for_bit(bit).unwrap();
            let image = act.serial_flow_result_image().unwrap();
            assert!(!image.is_complete());
            assert_eq!(image.result().pc, 0x0fc6);
        }
        act.advance_execution_for_bit(BITS_PER_WORD - 1).unwrap();
        assert!(act.serial_flow_result_image().unwrap().is_complete());
        // The next word captures the committed state, not a stale preview.
        let mut committed = state;
        act.serial_flow_result_image()
            .unwrap()
            .commit(&mut committed);
        act.begin_execution(0o0000, &committed).unwrap();
        assert_eq!(act.serial_flow_result_image().unwrap().result().pc, 0x0fc7);
        assert!(!act.serial_flow_result_image().unwrap().is_complete());
    }

    #[test]
    fn mode_latch_result_image_completes_only_after_b55() {
        let mut state = ActArchitecturalState::default();
        state.decimal = true;

        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0o0420, &state)
            .expect("HEX mode must start serial execution");

        let initial = act
            .serial_mode_latch_result_image()
            .expect("HEX mode must create a mode/latch image");
        assert!(!initial.is_complete());
        assert!(initial.decimal());

        for bit in 0..(BITS_PER_WORD - 1) {
            act.advance_execution_for_bit(bit)
                .expect("mode/latch execution must advance before b55");
        }
        assert!(!act
            .serial_mode_latch_result_image()
            .expect("mode/latch image must remain present")
            .is_complete());

        act.advance_execution_for_bit(BITS_PER_WORD - 1)
            .expect("b55 must complete mode/latch execution");
        let complete = act
            .serial_mode_latch_result_image()
            .expect("completed mode/latch image must remain inspectable");
        assert!(complete.is_complete());
        assert!(!complete.decimal());
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

#[cfg(test)]
#[path = "act_received_tests.rs"]
mod act_received_tests;

#[cfg(test)]
#[path = "act_owned_flow_tests.rs"]
mod act_owned_flow_tests;

#[path = "act_owned_registers.rs"]
mod owned_registers;

#[cfg(test)]
mod display_circuit_transport_tests {
    use super::*;
    use crate::machines::hp67::display_circuit::Hp67DisplayCircuit;
    struct Source;
    impl Hp67RomWordSource for Source {
        fn read_word(&self, _: u16) -> Option<u16> {
            Some(0x3e3)
        }
    }
    #[test]
    fn m15y_all_four_transports_feed_raw_display_without_slot_annotations() {
        for path in 0..4 {
            let mut bus = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            data.enable_electrical_level_bridge(None);
            let mut trace = StructuralBusTrace::default();
            let mut circuit = Hp67DisplayCircuit::default();
            let state = ActArchitecturalState {
                display_enable: true,
                a: [8; 14],
                b: [0; 14],
                ..Default::default()
            };
            for slot in 1..=15 {
                let mut result = match path {
                    0 => run_structural_display_fetch_cycle(
                        &mut bus, 0, &state, &mut act, &mut rom, &mut rom0, &Source,
                    )
                    .unwrap(),
                    1 => {
                        run_structural_display_fetch_data_phase_cycle(
                            &mut bus, 0, &state, &mut act, &mut rom, &mut rom0, &Source, &mut data,
                            None,
                        )
                        .unwrap()
                        .word
                    }
                    2 => run_structural_display_fetch_cycle_traced(
                        &mut bus, 0, &state, &mut act, &mut rom, &mut rom0, &Source, &mut trace,
                    )
                    .unwrap(),
                    _ => {
                        run_structural_display_fetch_data_phase_cycle_traced(
                            &mut bus, 0, &state, &mut act, &mut rom, &mut rom0, &Source, &mut data,
                            None, &mut trace,
                        )
                        .unwrap()
                        .word
                    }
                };
                result.str_event.scan_slot = if slot == 1 { 15 } else { 1 };
                let emission = circuit.capture_rom0(&rom0, true).unwrap();
                assert_eq!(emission.scan_slot, slot);
                assert_ne!(emission.scan_slot, result.str_event.scan_slot);
                assert_eq!(emission.code, result.display_byte);
                assert_eq!(result.rcd_falling, slot == 15);
                assert_eq!(emission.strobe_tick, u64::from(slot - 1) * 224 + 30);
            }
            assert_eq!(circuit.scan_slot(), 1);
            assert_eq!(circuit.exposure().completed_scans(), 1);
            assert_eq!(bus.tick().get(), 15 * 224);
            if path >= 2 {
                assert_eq!(trace.samples.len(), 15 * 224);
            }
        }
    }
}

#[cfg(test)]
#[path = "is_owner_tests.rs"]
mod is_owner_tests;
