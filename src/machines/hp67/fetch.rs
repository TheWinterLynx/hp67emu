//! Structural HP-67 ACT↔ROM serial fetch path.
//!
//! This module models the evidence-backed bit-cell transport on the shared IS/ISA
//! line without yet claiming the exact PHI launch/sample edge. The ACT can emit
//! the eight-bit ROM0 display byte at b0..b7 and the 12-bit ROM address LSB-first
//! at b16..b27 during the same 56-bit word. A ROM-side endpoint reconstructs the
//! address through the resolved electrical net, the selected ROM word is looked
//! up through a caller-supplied source, and the ROM returns the 10-bit word
//! LSB-first at b46..b55 for the ACT endpoint to reconstruct.

use crate::emulation::{Drive, DriverId, LogicLevel};

use super::{
    act::{
        display_register_index_for_scan_slot, ActArchitecturalState, ActDisplaySerialError,
        ActInstructionState,
    },
    act_serial_execution::{ActSerialExecution, ActSerialExecutionError},
    display::{Rom0DisplayEndpoint, Rom0DisplayError, Rom0StrEvent, HP67_DISPLAY_SCAN_SLOTS},
    isa::{act_address_drive, rom_word_drive, wired_high_drive, ROM_ADDRESS_MASK, ROM_WORD_MASK},
    machine::Hp67ElectricalBackplane,
    timing::{
        display_data_serial_bit, isa_window_for_bit, IsaWindow, BITS_PER_WORD, ROM_ADDRESS_BITS,
        ROM_WORD_BITS,
    },
    wiring::Hp67Net,
};

const COMPLETE_ADDRESS_MASK: u16 = (1u16 << ROM_ADDRESS_BITS) - 1;
const COMPLETE_WORD_MASK: u16 = (1u16 << ROM_WORD_BITS) - 1;
const ACT_IS_DRIVER: DriverId = DriverId::new("hp67-act-fetch-is");
const ROM_IS_DRIVER: DriverId = DriverId::new("hp67-rom-fetch-is");

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

/// Values reconstructed from one shared 56-bit HP-67 structural word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct StructuralWordResult {
    pub fetched_word: u16,
    pub display_byte: u8,
    pub str_event: Rom0StrEvent,
    pub rcd_falling: bool,
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
            received_word: 0,
            received_mask: 0,
        }
    }

    /// Bind one already-fetched word to the following 56-bit execution cycle.
    ///
    /// The architectural fallback may still compute unsupported internal effects
    /// at the instruction boundary, but the electrical endpoint now owns the
    /// complete b0..b55 lifetime of the same executing word. A new instruction
    /// cannot replace an execution that has not reached b55.
    pub fn begin_execution(
        &mut self,
        word: u16,
        instruction_state: ActInstructionState,
    ) -> Result<(), ActSerialExecutionError> {
        if let Some(execution) = self.execution {
            if let Some(next_word_bit) = execution.next_word_bit() {
                return Err(ActSerialExecutionError::ExecutionAlreadyActive {
                    word: execution.word(),
                    next_word_bit,
                });
            }
        }
        self.execution = Some(ActSerialExecution::new(word, instruction_state)?);
        Ok(())
    }

    pub const fn serial_execution(&self) -> Option<ActSerialExecution> {
        self.execution
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
    /// Only the source register index is latched here. A/B nibble contents are
    /// not snapshotted: each b0..b7 drive reads the current architectural state
    /// when that bit cell is visited. This prepares the transport for future
    /// intra-word ACT register/ALU evolution without claiming that evolution yet.
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

    /// ACT contribution to IS for the current bit cell.
    ///
    /// During b0..b7 the bit is read from the current A/B architectural state,
    /// rather than from a nibble snapshot captured at the start of the word.
    pub fn drive_for_bit(&self, word_bit: u8, state: Option<&ActArchitecturalState>) -> Drive {
        if let Some(serial_bit) = display_data_serial_bit(word_bit) {
            let (Some(register_index), Some(state)) = (self.display_register_index, state) else {
                return Drive::HighZ;
            };
            let bit = if serial_bit < 4 {
                ((state.a[register_index] >> serial_bit) & 1) != 0
            } else {
                ((state.b[register_index] >> (serial_bit - 4)) & 1) != 0
            };
            return wired_high_drive(bit);
        }
        act_address_drive(self.address, word_bit)
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
    /// This is a lifetime/coordinate update only. No internal register or carry
    /// mutation is attached to this boundary until 1820-2530 timing evidence
    /// identifies the real commit relation.
    fn advance_execution_for_bit(&mut self, word_bit: u8) -> Result<(), ActSerialExecutionError> {
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

    for expected_bit in 0..BITS_PER_WORD {
        debug_assert_eq!(backplane.word_bit(), expected_bit);

        backplane.drive(
            Hp67Net::Isa,
            ACT_IS_DRIVER,
            act.drive_for_bit(expected_bit, act_state),
        );
        backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom.drive_for_bit(expected_bit));

        if let Some(endpoint) = rom0.as_deref_mut() {
            if display_data_serial_bit(expected_bit).is_some() {
                endpoint.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa))?;
            }
        }

        if matches!(
            isa_window_for_bit(expected_bit),
            IsaWindow::RomAddress { .. }
        ) {
            rom.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa), source)?;
            backplane.drive(Hp67Net::Isa, ROM_IS_DRIVER, rom.drive_for_bit(expected_bit));
        }

        if matches!(isa_window_for_bit(expected_bit), IsaWindow::RomWord { .. }) {
            act.sample_for_bit(expected_bit, backplane.level(Hp67Net::Isa))?;
        }

        if backplane.level(Hp67Net::Isa) == LogicLevel::Contention {
            return Err(SerialFetchError::IsaContention {
                word_bit: expected_bit,
            }
            .into());
        }

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
        | Err(StructuralWordError::ActExecution(_)) => {
            unreachable!("fetch-only structural cycle cannot produce a non-fetch error")
        }
    }
}

/// Run one structural 56-bit word with ACT display traffic and instruction fetch
/// sharing the same resolved HP-67 IS net and backplane timing coordinate.
///
/// `ActSerialEndpoint` owns the fifteen-word display phase and therefore chooses
/// the A/B digit serialized at b0..b7. The contents of that source digit are read
/// from the ACT state at each bit cell instead of being snapshotted at word start.
/// If an executing word is bound to the endpoint, that same instruction advances
/// through b0..b55 in lockstep with this transport. ROM0 reconstructs the same
/// resolved bits and emits the returned `Rom0StrEvent`; ACT independently reports
/// the coarse RCD falling boundary after slot 15. No cathode state enters this API.
///
/// This establishes source-backed ownership at word/bit granularity only. It
/// does not yet make architectural A/B evolve inside the word, nor claim final
/// PHI-relative launch/sample edges, ROM0 sampling edges, or exact STR/RCD
/// overlap ordering.
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
    fn display_and_fetch_share_one_resolved_word_cycle() {
        let source = FixtureRom {
            words: [(0x07b, 0x04c), (0x001, 0x3e3)],
        };
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0x07b);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();

        let mut act_state = ActArchitecturalState::default();
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
    fn display_bits_are_read_live_instead_of_snapshotted_at_word_start() {
        let mut act = ActSerialEndpoint::new(0);
        let mut state = ActArchitecturalState::default();
        act.begin_display_fetch_cycle(0)
            .expect("slot 1 must select a valid ACT display digit");

        assert_eq!(act.drive_for_bit(0, Some(&state)), Drive::HighZ);
        state.a[0] = 0x01;
        assert_eq!(act.drive_for_bit(0, Some(&state)), Drive::High);

        assert_eq!(act.drive_for_bit(4, Some(&state)), Drive::HighZ);
        state.b[0] = 0x01;
        assert_eq!(act.drive_for_bit(4, Some(&state)), Drive::High);
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

        act.begin_execution(0x11a, ActInstructionState::Normal)
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
    }

    #[test]
    fn incomplete_execution_cannot_be_replaced_by_another_word() {
        let mut act = ActSerialEndpoint::new(0);
        act.begin_execution(0x11a, ActInstructionState::Normal)
            .expect("first execution must start");
        assert_eq!(
            act.begin_execution(0x000, ActInstructionState::Normal),
            Err(ActSerialExecutionError::ExecutionAlreadyActive {
                word: 0x11a,
                next_word_bit: 0,
            })
        );
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
            let level = match address.drive_for_bit(bit, None) {
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
