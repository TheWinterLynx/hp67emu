use std::time::Duration;

use hp67emu::machines::hp67::{
    act_data_transfer_plan, decode_rom0_display_byte, display_register_index_for_scan_slot,
    run_structural_display_fetch_cycle, run_structural_display_fetch_data_phase_cycle,
    ActOperation, ActRegister, ActSerialEndpoint, ActSerialRegister, CardInsertionEnd,
    CathodeDriver1820_1749, CrcInstruction, FetchPipelineLatch, Hp67ArchitecturalExecution,
    Hp67ArchitecturalMachine, Hp67ArchitecturalOperation, Hp67CardTransport,
    Hp67DataSerialWordPath, Hp67DataTransferDirection, Hp67ElectricalBackplane, Hp67Firmware,
    Hp67Key, Hp67Keyboard, Hp67MagneticCard, Hp67SegmentMask, Rom0DisplayEndpoint,
    RomFetchEndpoint, CRC_FLAG_BUFFER_READY, CRC_FLAG_MOTOR_ON, CRC_FLAG_WRITE_MODE,
    HP67_OBSERVED_POWER_ON_SYNC_DELAY_US, HP67_OBSERVED_WORD_TIME_US,
};

const DISPLAY_INIT_PC: u16 = 0o0161;
const MAIN_WAIT_PC: u16 = 0o0167;
const CARD_POLL_PC: u16 = 0o0206;
const BOOT_CYCLE_LIMIT: u64 = 2_000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum LiveBootPhase {
    ResetHold,
    Firmware,
    Idle,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Run,
    Program,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Digit(u8),
    Decimal,
    ChangeSign,
    Exponent,
    ClearX,
    Enter,
    Add,
    Subtract,
    Multiply,
    Divide,
    FunctionF,
    FunctionG,
    FunctionH,
    SigmaPlus,
    Gto,
    Dsp,
    Indirect,
    Sst,
    Sto,
    Rcl,
    A,
    B,
    C,
    D,
    E,
    RunStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    TogglePower,
    ToggleMode,
}

/// Mechanical controls that are still owned by the UI layer.
///
/// Display contents are deliberately absent: visible LEDs now come from the
/// real-microcode structural machine instead of a formatted placeholder string.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67State {
    pub power_on: bool,
    pub mode: RunMode,
}

impl Default for Hp67State {
    fn default() -> Self {
        Self {
            power_on: false,
            mode: RunMode::Run,
        }
    }
}

impl KeyAction {
    pub const fn physical_key(self) -> Option<Hp67Key> {
        match self {
            Self::Digit(0) => Some(Hp67Key::Digit0),
            Self::Digit(1) => Some(Hp67Key::Digit1),
            Self::Digit(2) => Some(Hp67Key::Digit2),
            Self::Digit(3) => Some(Hp67Key::Digit3),
            Self::Digit(4) => Some(Hp67Key::Digit4),
            Self::Digit(5) => Some(Hp67Key::Digit5),
            Self::Digit(6) => Some(Hp67Key::Digit6),
            Self::Digit(7) => Some(Hp67Key::Digit7),
            Self::Digit(8) => Some(Hp67Key::Digit8),
            Self::Digit(9) => Some(Hp67Key::Digit9),
            Self::Digit(_) => None,
            Self::Decimal => Some(Hp67Key::Decimal),
            Self::ChangeSign => Some(Hp67Key::ChangeSign),
            Self::Exponent => Some(Hp67Key::Exponent),
            Self::ClearX => Some(Hp67Key::ClearX),
            Self::Enter => Some(Hp67Key::Enter),
            Self::Add => Some(Hp67Key::Add),
            Self::Subtract => Some(Hp67Key::Subtract),
            Self::Multiply => Some(Hp67Key::Multiply),
            Self::Divide => Some(Hp67Key::Divide),
            Self::FunctionF => Some(Hp67Key::FunctionF),
            Self::FunctionG => Some(Hp67Key::FunctionG),
            Self::FunctionH => Some(Hp67Key::FunctionH),
            Self::SigmaPlus => Some(Hp67Key::SigmaPlus),
            Self::Gto => Some(Hp67Key::Gto),
            Self::Dsp => Some(Hp67Key::Dsp),
            Self::Indirect => Some(Hp67Key::Indirect),
            Self::Sst => Some(Hp67Key::Sst),
            Self::Sto => Some(Hp67Key::Sto),
            Self::Rcl => Some(Hp67Key::Rcl),
            Self::A => Some(Hp67Key::A),
            Self::B => Some(Hp67Key::B),
            Self::C => Some(Hp67Key::C),
            Self::D => Some(Hp67Key::D),
            Self::E => Some(Hp67Key::E),
            Self::RunStop => Some(Hp67Key::RunStop),
        }
    }
}

impl Hp67State {
    pub fn handle(&mut self, event: UiEvent) {
        match event {
            UiEvent::TogglePower => self.power_on = !self.power_on,
            UiEvent::ToggleMode => {
                self.mode = match self.mode {
                    RunMode::Run => RunMode::Program,
                    RunMode::Program => RunMode::Run,
                };
            }
        }
    }
}

/// Fifteen physical HP-67 display positions, left-to-right.
///
/// Each byte is the directly rendered A..G/DP segment mask. No character or
/// numeric representation is stored in this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct HardwareDisplayFrame {
    segments: [u8; 15],
}

impl HardwareDisplayFrame {
    pub const BLANK: Self = Self { segments: [0; 15] };

    pub const fn segments(&self) -> &[u8; 15] {
        &self.segments
    }

    pub const fn shows_card_prompt(&self) -> bool {
        self.segments[1] == 0x39 && self.segments[2] == 0x50 && self.segments[3] == 0x5e
    }

    fn clear(&mut self) {
        self.segments = [0; 15];
    }

    fn capture_scan_slot(&mut self, scan_slot: u8, anodes: Hp67SegmentMask) -> Result<(), String> {
        match scan_slot {
            1 | 15 => self.segments[14] = anodes.bits(),
            2 => self.segments[13] = anodes.bits(),
            3 => {
                // The sign cathode is shared. ROM0 anode E selects the physical
                // mantissa sign and anode G selects the exponent sign. At the UI
                // glass both are horizontal minus emitters, not E/G digit shapes.
                self.segments[0] = if anodes.contains(Hp67SegmentMask::E) {
                    Hp67SegmentMask::G.bits()
                } else {
                    0
                };
                self.segments[12] = if anodes.contains(Hp67SegmentMask::G) {
                    Hp67SegmentMask::G.bits()
                } else {
                    0
                };
            }
            4..=14 => self.segments[usize::from(scan_slot - 3)] = anodes.bits(),
            _ => return Err(format!("invalid HP-67 display scan slot {scan_slot}")),
        }
        Ok(())
    }
}

impl Default for HardwareDisplayFrame {
    fn default() -> Self {
        Self::BLANK
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingRamDataTransfer {
    direction: Hp67DataTransferDirection,
    address: u8,
    expected: ActRegister,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PendingSerialArithmeticAuthority {
    word: u16,
    expected_a: ActRegister,
    expected_b: ActRegister,
    expected_c: ActRegister,
    expected_carry: bool,
}

/// UI-owned live HP-67 machine used only as the source of physical LED state.
///
/// `hp67firmware` is versioned with the emulator and embedded in the executable.
/// Startup executes through the same combined display/fetch word transport used
/// by the structural smoke tests. ACT owns the display phase and the b0..b55
/// lifetime of the executing instruction; M14D also makes that structural path
/// authoritative for ADD/SUB A/B/C/carry final state at the word boundary. ROM0
/// emits STR events and the cathode driver only consumes downstream STR/RCD
/// control events. Exact intra-word ACT write edges remain source-blocked.
pub struct Hp67LiveMachine {
    source: Hp67Firmware,
    backplane: Hp67ElectricalBackplane,
    act_serial: ActSerialEndpoint,
    fetch_rom: RomFetchEndpoint,
    display_rom0: Rom0DisplayEndpoint,
    cathode: CathodeDriver1820_1749,
    pipeline: FetchPipelineLatch,
    data_serial: Hp67DataSerialWordPath,
    pending_ram_data_transfer: Option<PendingRamDataTransfer>,
    machine: Hp67ArchitecturalMachine,
    keyboard: Hp67Keyboard,
    card_transport: Hp67CardTransport,
    card_startup_buffer_clears: u8,
    display: HardwareDisplayFrame,
    phase: LiveBootPhase,
    pending_us: u64,
    boot_cycle: u64,
    saw_display_init: bool,
    main_wait_visits: u64,
    card_poll_visits: u64,
    display_control_seen: bool,
}

impl Hp67LiveMachine {
    pub fn power_on_default() -> Result<Self, String> {
        Ok(Self {
            source: Hp67Firmware::default(),
            backplane: Hp67ElectricalBackplane::default(),
            act_serial: ActSerialEndpoint::new(0),
            fetch_rom: RomFetchEndpoint::default(),
            display_rom0: Rom0DisplayEndpoint::default(),
            cathode: CathodeDriver1820_1749::default(),
            pipeline: FetchPipelineLatch::default(),
            data_serial: Hp67DataSerialWordPath::default(),
            pending_ram_data_transfer: None,
            machine: Hp67ArchitecturalMachine::default(),
            keyboard: Hp67Keyboard::default(),
            card_transport: Hp67CardTransport::default(),
            card_startup_buffer_clears: 0,
            display: HardwareDisplayFrame::BLANK,
            phase: LiveBootPhase::ResetHold,
            pending_us: 0,
            boot_cycle: 0,
            saw_display_init: false,
            main_wait_visits: 0,
            card_poll_visits: 0,
            display_control_seen: false,
        })
    }

    pub const fn display_frame(&self) -> HardwareDisplayFrame {
        self.display
    }

    pub fn reset_power_on(&mut self) -> Result<(), String> {
        self.source.reset_bank();
        self.backplane = Hp67ElectricalBackplane::default();
        self.act_serial = ActSerialEndpoint::new(0);
        self.fetch_rom = RomFetchEndpoint::default();
        self.display_rom0 = Rom0DisplayEndpoint::default();
        self.cathode = CathodeDriver1820_1749::default();
        self.pipeline = FetchPipelineLatch::default();
        self.data_serial = Hp67DataSerialWordPath::default();
        self.pending_ram_data_transfer = None;
        self.machine = Hp67ArchitecturalMachine::default();
        self.keyboard = Hp67Keyboard::default();
        self.card_transport = Hp67CardTransport::default();
        self.card_startup_buffer_clears = 0;
        self.display = HardwareDisplayFrame::BLANK;
        self.phase = LiveBootPhase::ResetHold;
        self.pending_us = 0;
        self.boot_cycle = 0;
        self.saw_display_init = false;
        self.main_wait_visits = 0;
        self.card_poll_visits = 0;
        self.display_control_seen = false;
        Ok(())
    }

    pub fn set_key_contact(&mut self, key: Option<Hp67Key>) {
        match key {
            Some(key) if self.keyboard.pressed() != Some(key) => self.keyboard.press(key),
            Some(_) => {}
            None => self.keyboard.release(),
        }
    }

    pub fn set_program_mode(&mut self, program: bool) -> Result<(), String> {
        self.machine
            .set_program_mode(program)
            .map_err(|error| format!("HP-67 program-mode flag update failed: {error:?}"))
    }

    pub fn insert_magnetic_card(
        &mut self,
        card: Hp67MagneticCard,
        insertion_end: CardInsertionEnd,
    ) -> Result<(), String> {
        let previous_transport = self.card_transport.clone();
        self.card_transport
            .insert_card(card, insertion_end)
            .map_err(|error| format!("HP-67 card insertion failed: {error:?}"))?;
        self.card_startup_buffer_clears = 0;

        if let Err(error) = self.machine.set_card_present(true) {
            self.card_transport = previous_transport;
            return Err(format!("HP-67 card-present contact failed: {error:?}"));
        }

        Ok(())
    }

    pub fn magnetic_card_inserted(&self) -> bool {
        self.card_transport.card().is_some()
    }

    pub fn card_motor_on(&self) -> bool {
        self.machine.crc.flag(CRC_FLAG_MOTOR_ON) == Some(true)
    }

    pub fn card_write_mode(&self) -> bool {
        self.machine.crc.flag(CRC_FLAG_WRITE_MODE) == Some(true)
    }

    pub const fn card_record_stream_active(&self) -> bool {
        self.card_transport.record_stream_active()
    }

    pub const fn card_transport_complete(&self) -> bool {
        self.card_transport.is_complete()
    }

    pub const fn card_record_position(&self) -> usize {
        self.card_transport.next_record()
    }

    pub const fn card_prompt_visible(&self) -> bool {
        self.display.shows_card_prompt()
    }

    pub fn take_completed_magnetic_card(&mut self) -> Option<Hp67MagneticCard> {
        self.card_transport.take_completed_card()
    }

    pub fn take_magnetic_card_for_power_off(&mut self) -> Result<Option<Hp67MagneticCard>, String> {
        if self.card_transport.card().is_none() {
            return Ok(None);
        }

        self.machine
            .set_card_present(false)
            .map_err(|error| format!("HP-67 power-off card contact failed: {error:?}"))?;
        self.card_startup_buffer_clears = 0;
        Ok(self.card_transport.take_card_for_power_off())
    }

    pub fn withdraw_unstarted_magnetic_card(&mut self) -> Result<Option<Hp67MagneticCard>, String> {
        if self.card_motor_on()
            || self.card_transport.head_active()
            || self.card_transport.next_record() != 0
        {
            return Ok(None);
        }
        if self.card_transport.card().is_none() {
            return Ok(None);
        }

        self.machine
            .set_card_present(false)
            .map_err(|error| format!("HP-67 card withdrawal contact failed: {error:?}"))?;
        self.card_startup_buffer_clears = 0;
        Ok(self.card_transport.take_unstarted_card())
    }

    pub fn advance(&mut self, elapsed: Duration) -> Result<(), String> {
        let elapsed_us = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
        self.pending_us = self.pending_us.saturating_add(elapsed_us);

        if self.phase == LiveBootPhase::ResetHold {
            if self.pending_us < HP67_OBSERVED_POWER_ON_SYNC_DELAY_US {
                return Ok(());
            }
            self.pending_us -= HP67_OBSERVED_POWER_ON_SYNC_DELAY_US;
            self.phase = LiveBootPhase::Firmware;
        }

        while self.phase != LiveBootPhase::ResetHold
            && self.pending_us >= HP67_OBSERVED_WORD_TIME_US
        {
            self.pending_us -= HP67_OBSERVED_WORD_TIME_US;
            self.step_firmware_cycle()?;
        }
        Ok(())
    }

    fn step_firmware_cycle(&mut self) -> Result<(), String> {
        self.step_firmware_cycle_with_execution().map(|_| ())
    }

    fn step_firmware_cycle_with_execution(
        &mut self,
    ) -> Result<Option<Hp67ArchitecturalExecution>, String> {
        let cycle = self.boot_cycle;
        let card_motor_was_on = self.machine.crc.flag(CRC_FLAG_MOTOR_ON) == Some(true);
        if self.phase != LiveBootPhase::Idle && cycle >= BOOT_CYCLE_LIMIT {
            return Err(format!(
                "HP-67 live boot did not reach the no-key idle checkpoint within {BOOT_CYCLE_LIMIT} cycles"
            ));
        }

        let mut idle_after_word = false;
        let mut executed = None;
        let mut ram_data_transfer = None;
        let mut serial_arithmetic_authority = None;
        self.pipeline.begin_cycle();
        self.complete_pending_ram_data_before_word(cycle)?;
        if let Some(word) = self.pipeline.executing_word() {
            self.keyboard.sample_into_act(&mut self.machine.act.state);

            let startup_serial_state = if self.display_control_seen {
                None
            } else {
                // Direct power-on capture shows display traffic before firmware
                // executes its first explicit DISPLAY control instruction.
                let mut state = self.machine.act.state.clone();
                state.display_enable = true;
                Some(state)
            };
            let serial_state = startup_serial_state
                .as_ref()
                .unwrap_or(&self.machine.act.state);
            self.act_serial
                .begin_execution(word, serial_state)
                .map_err(|error| {
                    format!("live cycle {cycle} serial execution start failed: {error:?}")
                })?;

            let (execution, deferred_ram_data, deferred_serial_arithmetic) =
                self.execute_word_with_deferred_authority(cycle, word)?;
            ram_data_transfer = deferred_ram_data;
            serial_arithmetic_authority = deferred_serial_arithmetic;
            if self.card_transport.card().is_some()
                && !self.card_transport.head_active()
                && self.machine.crc.flag(CRC_FLAG_MOTOR_ON) == Some(true)
            {
                if let Hp67ArchitecturalOperation::CrcControl {
                    instruction: CrcInstruction::TestFlagAndClear { flag },
                    ..
                } = execution.operation
                {
                    if usize::from(flag) == CRC_FLAG_BUFFER_READY {
                        self.card_startup_buffer_clears =
                            self.card_startup_buffer_clears.saturating_add(1);
                        if self.card_startup_buffer_clears == 2 {
                            self.card_transport.set_head_active(true);
                        }
                    }
                }
            }

            executed = Some(execution);

            match execution.pc {
                DISPLAY_INIT_PC => self.saw_display_init = true,
                MAIN_WAIT_PC => self.main_wait_visits = self.main_wait_visits.saturating_add(1),
                CARD_POLL_PC => self.card_poll_visits = self.card_poll_visits.saturating_add(1),
                _ => {}
            }

            if let Hp67ArchitecturalOperation::Act(ActOperation::Special { opcode }) =
                execution.operation
            {
                if matches!(opcode, 0o0210 | 0o0310) {
                    self.display_control_seen = true;
                    if !self.machine.act.state.display_enable {
                        self.display.clear();
                    }
                }
            }

            idle_after_word = self.saw_display_init
                && self.main_wait_visits >= 2
                && self.card_poll_visits >= 1
                && self.machine.act.state.display_enable
                && self.machine.act.state.key_buffer.is_none();
        }

        self.transport_fetch_word(cycle, ram_data_transfer)?;

        if let Some(word) = self.pipeline.executing_word() {
            let serial_execution = self.act_serial.serial_execution().ok_or_else(|| {
                format!("live cycle {cycle} lost serial execution for word 0x{word:03x}")
            })?;
            if serial_execution.word() != word || !serial_execution.is_complete() {
                return Err(format!(
                    "live cycle {cycle} serial execution incomplete: expected word 0x{word:03x}, got word 0x{:03x}, next_bit={:?}",
                    serial_execution.word(),
                    serial_execution.next_word_bit()
                ));
            }
        }

        self.complete_serial_arithmetic_authority(cycle, serial_arithmetic_authority)?;

        self.card_transport
            .advance_us(
                HP67_OBSERVED_WORD_TIME_US,
                card_motor_was_on,
                &mut self.machine.crc,
            )
            .map_err(|error| format!("live cycle {cycle} card transport failed: {error:?}"))?;

        if self.card_transport.is_complete() {
            self.machine.set_card_present(false).map_err(|error| {
                format!("live cycle {cycle} card exit contact failed: {error:?}")
            })?;
            self.card_transport.set_head_active(false);
        }

        self.boot_cycle = cycle.saturating_add(1);
        if idle_after_word {
            self.phase = LiveBootPhase::Idle;
        }
        Ok(executed)
    }

    fn execute_word_with_deferred_authority(
        &mut self,
        cycle: u64,
        word: u16,
    ) -> Result<
        (
            Hp67ArchitecturalExecution,
            Option<PendingRamDataTransfer>,
            Option<PendingSerialArithmeticAuthority>,
        ),
        String,
    > {
        let serial_before = self.act_serial.serial_arithmetic_result_image().map(|_| {
            (
                self.machine.act.state.a,
                self.machine.act.state.b,
                self.machine.act.state.c,
                self.machine.act.state.carry,
            )
        });

        let (execution, ram_data_transfer) =
            self.execute_word_with_deferred_ram_data(cycle, word)?;

        let serial_arithmetic = serial_before.map(|(a, b, c, carry)| {
            let pending = PendingSerialArithmeticAuthority {
                word,
                expected_a: self.machine.act.state.a,
                expected_b: self.machine.act.state.b,
                expected_c: self.machine.act.state.c,
                expected_carry: self.machine.act.state.carry,
            };

            self.machine.act.state.a = a;
            self.machine.act.state.b = b;
            self.machine.act.state.c = c;
            self.machine.act.state.carry = carry;
            pending
        });

        Ok((execution, ram_data_transfer, serial_arithmetic))
    }

    fn execute_word_with_deferred_ram_data(
        &mut self,
        cycle: u64,
        word: u16,
    ) -> Result<(Hp67ArchitecturalExecution, Option<PendingRamDataTransfer>), String> {
        let c_before = self.machine.act.state.c;
        let transfer = act_data_transfer_plan(word, &self.machine.act.state).and_then(|plan| {
            self.machine.ram.read(plan.address).map(|ram_before| {
                let expected = match plan.direction {
                    Hp67DataTransferDirection::ActToPeripheral => self.machine.act.state.c,
                    Hp67DataTransferDirection::PeripheralToAct => ram_before,
                };
                (
                    PendingRamDataTransfer {
                        direction: plan.direction,
                        address: plan.address,
                        expected,
                    },
                    ram_before,
                )
            })
        });

        let execution = self
            .machine
            .execute_word(word)
            .map_err(|error| format!("live cycle {cycle} execution failed: {error:?}"))?;

        if let Some((pending, ram_before)) = transfer {
            match pending.direction {
                Hp67DataTransferDirection::ActToPeripheral => {
                    let architectural =
                        self.machine.ram.read(pending.address).ok_or_else(|| {
                            format!(
                                "live cycle {cycle} RAM DATA write target 0x{:02x} disappeared",
                                pending.address
                            )
                        })?;
                    if architectural != pending.expected {
                        return Err(format!(
                            "live cycle {cycle} architectural RAM DATA write oracle mismatch at 0x{:02x}: expected={:?} actual={architectural:?}",
                            pending.address, pending.expected
                        ));
                    }
                    if !self.machine.ram.write(pending.address, ram_before) {
                        return Err(format!(
                            "live cycle {cycle} RAM DATA write target 0x{:02x} could not be restored for deferred serial commit",
                            pending.address
                        ));
                    }
                }
                Hp67DataTransferDirection::PeripheralToAct => {
                    let architectural = self.machine.act.state.c;
                    if architectural != pending.expected {
                        return Err(format!(
                            "live cycle {cycle} architectural RAM DATA read oracle mismatch at 0x{:02x}: expected={:?} actual={architectural:?}",
                            pending.address, pending.expected
                        ));
                    }
                    self.machine.act.state.c = c_before;
                }
            }
        }

        Ok((execution, transfer.map(|(pending, _)| pending)))
    }

    fn complete_serial_arithmetic_authority(
        &mut self,
        cycle: u64,
        pending: Option<PendingSerialArithmeticAuthority>,
    ) -> Result<(), String> {
        let Some(pending) = pending else {
            return Ok(());
        };

        let actual = self
            .act_serial
            .serial_arithmetic_result_image()
            .ok_or_else(|| {
                format!(
                    "live cycle {cycle} lost serial arithmetic result image for word 0x{:03x}",
                    pending.word
                )
            })?;
        let actual_a = *actual.register(ActSerialRegister::A);
        let actual_b = *actual.register(ActSerialRegister::B);
        let actual_c = *actual.register(ActSerialRegister::C);
        let actual_carry = actual.final_chain();

        if actual_a != pending.expected_a
            || actual_b != pending.expected_b
            || actual_c != pending.expected_c
            || actual_carry != pending.expected_carry
        {
            return Err(format!(
                "live cycle {cycle} serial arithmetic authority mismatch for word 0x{:03x}: \
                 expected A={:?} B={:?} C={:?} carry={} actual A={actual_a:?} B={actual_b:?} \
                 C={actual_c:?} carry={actual_carry} processed_digits={}",
                pending.word,
                pending.expected_a,
                pending.expected_b,
                pending.expected_c,
                pending.expected_carry,
                actual.processed_digits(),
            ));
        }

        self.machine.act.state.a = actual_a;
        self.machine.act.state.b = actual_b;
        self.machine.act.state.c = actual_c;
        self.machine.act.state.carry = actual_carry;
        Ok(())
    }

    fn complete_pending_ram_data_before_word(&mut self, cycle: u64) -> Result<(), String> {
        let completed = if self.data_serial.frame_in_progress() {
            self.data_serial
                .preconsume_previous_tail()
                .map_err(|error| format!("live cycle {cycle} RAM DATA tail failed: {error:?}"))?
        } else {
            None
        };

        match (self.pending_ram_data_transfer.take(), completed) {
            (Some(pending), Some(actual)) if actual == pending.expected => {
                match pending.direction {
                    Hp67DataTransferDirection::ActToPeripheral => {
                        if !self.machine.ram.write(pending.address, actual) {
                            return Err(format!(
                                "live cycle {cycle} reconstructed RAM DATA write targeted uninstalled address 0x{:02x}",
                                pending.address
                            ));
                        }
                    }
                    Hp67DataTransferDirection::PeripheralToAct => {
                        self.machine.act.state.c = actual;
                    }
                }
                Ok(())
            }
            (Some(pending), Some(actual)) => Err(format!(
                "live cycle {cycle} RAM DATA authority mismatch at 0x{:02x}: expected={:?} actual={actual:?}",
                pending.address, pending.expected
            )),
            (Some(pending), None) => Err(format!(
                "live cycle {cycle} RAM DATA frame did not complete for 0x{:02x}: expected={:?}",
                pending.address, pending.expected
            )),
            (None, Some(actual)) => Err(format!(
                "live cycle {cycle} RAM DATA frame completed without a pending installed-RAM transfer: actual={actual:?}"
            )),
            (None, None) => Ok(()),
        }
    }

    fn transport_fetch_word(
        &mut self,
        cycle: u64,
        ram_data_transfer: Option<PendingRamDataTransfer>,
    ) -> Result<(), String> {
        let ram_data_payload = ram_data_transfer.map(|transfer| transfer.expected);
        let requested_bank = self.machine.prepare_hp67_fetch();
        self.source.select_bank(requested_bank);
        let address = self.machine.pc();
        let startup_display_state = if self.display_control_seen {
            None
        } else {
            let mut state = self.machine.act.state.clone();
            state.display_enable = true;
            Some(state)
        };
        let display_state = startup_display_state
            .as_ref()
            .unwrap_or(&self.machine.act.state);
        let result = if self.data_serial.prefix_preconsumed() || ram_data_payload.is_some() {
            let fused = run_structural_display_fetch_data_phase_cycle(
                &mut self.backplane,
                address,
                display_state,
                &mut self.act_serial,
                &mut self.fetch_rom,
                &mut self.display_rom0,
                &self.source,
                &mut self.data_serial,
                ram_data_payload,
            )
            .map_err(|error| format!("live cycle {cycle} shared DATA word failed: {error:?}"))?;

            if let Some(actual) = fused.completed_data {
                return Err(format!(
                    "live cycle {cycle} RAM DATA frame completed inside the structural body instead of the pre-word b0/b1 tail: actual={actual:?}"
                ));
            }

            debug_assert!(self.pending_ram_data_transfer.is_none());
            self.pending_ram_data_transfer = ram_data_transfer;
            fused.word
        } else {
            run_structural_display_fetch_cycle(
                &mut self.backplane,
                address,
                display_state,
                &mut self.act_serial,
                &mut self.fetch_rom,
                &mut self.display_rom0,
                &self.source,
            )
            .map_err(|error| format!("live cycle {cycle} shared word failed: {error:?}"))?
        };
        self.pipeline.complete_cycle(result.fetched_word);

        let scan_slot = result.str_event.scan_slot;

        // Before firmware executes its first display-control instruction, do not
        // use the architectural `display_enable` default as an electrical inhibit.
        // A real HP-67 visibly emits during this interval, while the semantic
        // model's reset value is not hardware evidence. The emitted CONTENT is
        // still entirely derived from ACT A/B -> IS -> ROM0 for this word.
        if !self.display_control_seen || self.machine.act.state.display_enable {
            let anodes =
                decode_rom0_display_byte(scan_slot, result.display_byte).map_err(|error| {
                    let register_index = display_register_index_for_scan_slot(scan_slot);
                    let post = register_index.map(|index| {
                        (
                            self.machine.act.state.a[index],
                            self.machine.act.state.b[index],
                        )
                    });
                    let pre = register_index.and_then(|index| {
                        self.act_serial.serial_execution_state().and_then(|state| {
                            Some((
                                state.register_digit(ActSerialRegister::A, index as u8)?,
                                state.register_digit(ActSerialRegister::B, index as u8)?,
                            ))
                        })
                    });
                    format!(
                        "live cycle {cycle} ROM0 decode failed at slot {scan_slot}: {error:?}; \
                         executing_word={:?} post_pc={:04o} display_enable={} register_index={register_index:?} \
                         pre_a_b={pre:?} post_a_b={post:?}",
                        self.pipeline.executing_word(),
                        self.machine.pc(),
                        self.machine.act.state.display_enable,
                    )
                })?;
            self.display.capture_scan_slot(scan_slot, anodes)?;
        } else {
            self.display.clear();
        }

        self.cathode
            .apply_control_edges(result.str_event, result.rcd_falling)
            .map_err(|error| format!("live cycle {cycle} cathode control failed: {error:?}"))?;
        Ok(())
    }
}

#[cfg(test)]
mod diagnostic_tests;

#[cfg(test)]
mod m12_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use hp67emu::{
        emulation::Drive,
        machines::hp67::{
            ActDisplayWordSerializer, ActOperation, Hp67CardTrack, Hp67Key, Hp67MagneticTrack,
            TeenixHppImport, CRC_FLAG_CARD_PRESENT, CRC_FLAG_F7_STATUS, CRC_FLAG_WRITE_MODE,
            HP67_CARD_RECORDS_PER_TRACK, HP67_DISPLAY_SCAN_SLOTS, HP67_NOMINAL_CARD_RECORD_US,
        },
    };

    fn track1_card(track: Hp67MagneticTrack) -> Hp67MagneticCard {
        Hp67MagneticCard::default().with_track(Hp67CardTrack::Track1, track)
    }

    fn insert_track1(live: &mut Hp67LiveMachine, track: Hp67MagneticTrack) -> Result<(), String> {
        live.insert_magnetic_card(track1_card(track), CardInsertionEnd::End1)
    }

    fn patterned_act_register(seed: u8) -> ActRegister {
        std::array::from_fn(|digit| (seed.wrapping_add(digit as u8 * 3)) & 0x0f)
    }

    fn carry_one_data_frame_to_next_word_tail(
        live: &mut Hp67LiveMachine,
        transfer: PendingRamDataTransfer,
    ) {
        live.pending_ram_data_transfer = Some(transfer);
        live.data_serial.begin_word(Some(transfer.expected));
        for word_bit in 0u8..56 {
            live.data_serial
                .visit_word_bit(word_bit)
                .expect("DATA body must accept the first word");
        }
        assert_eq!(live.data_serial.complete_word(), None);
    }

    #[test]
    fn serial_add_result_is_deferred_until_structural_word_completion() {
        let mut live = Hp67LiveMachine::power_on_default().expect("live machine must construct");
        live.machine.act.state.a[0] = 9;
        live.machine.act.state.b[0] = 1;
        live.machine.act.state.p = 0;
        live.machine.act.state.decimal = true;
        let before = live.machine.act.state.clone();
        let word = (0x09u16 << 5) | 0x02;

        live.act_serial
            .begin_execution(word, &before)
            .expect("serial ADD execution must start");
        let (_, ram_transfer, pending) = live
            .execute_word_with_deferred_authority(0, word)
            .expect("architectural ADD oracle must execute");
        let pending = pending.expect("ADD must use serial arithmetic authority");

        assert_eq!(ram_transfer, None);
        assert_eq!(live.machine.act.state.a, before.a);
        assert_eq!(live.machine.act.state.b, before.b);
        assert_eq!(live.machine.act.state.c, before.c);
        assert_eq!(live.machine.act.state.carry, before.carry);
        assert_eq!(pending.expected_a[0], 0);
        assert!(pending.expected_carry);

        live.transport_fetch_word(0, None)
            .expect("structural ADD word must complete");
        assert!(live
            .act_serial
            .serial_execution()
            .expect("serial execution must remain available")
            .is_complete());
        live.complete_serial_arithmetic_authority(0, Some(pending))
            .expect("serial ADD image must match and commit");

        assert_eq!(live.machine.act.state.a[0], 0);
        assert!(live.machine.act.state.carry);
    }

    #[test]
    fn serial_compare_commits_carry_but_preserves_registers_and_then_goto_state() {
        let mut live = Hp67LiveMachine::power_on_default().expect("live machine must construct");
        live.machine.act.state.a[0] = 2;
        live.machine.act.state.c[0] = 3;
        live.machine.act.state.p = 0;
        live.machine.act.state.decimal = true;
        let before = live.machine.act.state.clone();
        let word = (0x18u16 << 5) | 0x02;

        live.act_serial
            .begin_execution(word, &before)
            .expect("serial compare execution must start");
        let (_, ram_transfer, pending) = live
            .execute_word_with_deferred_authority(0, word)
            .expect("architectural compare oracle must execute");
        let pending = pending.expect("compare must use serial arithmetic authority");

        assert_eq!(ram_transfer, None);
        assert_eq!(live.machine.act.state.a, before.a);
        assert_eq!(live.machine.act.state.b, before.b);
        assert_eq!(live.machine.act.state.c, before.c);
        assert_eq!(live.machine.act.state.carry, before.carry);
        assert!(matches!(
            live.machine.act.state.instruction_state,
            hp67emu::machines::hp67::ActInstructionState::ThenGoto
        ));
        assert!(pending.expected_carry);

        live.transport_fetch_word(0, None)
            .expect("structural compare word must complete");
        live.complete_serial_arithmetic_authority(0, Some(pending))
            .expect("serial compare image must match and commit");

        assert_eq!(live.machine.act.state.a, before.a);
        assert_eq!(live.machine.act.state.b, before.b);
        assert_eq!(live.machine.act.state.c, before.c);
        assert!(live.machine.act.state.carry);
        assert!(matches!(
            live.machine.act.state.instruction_state,
            hp67emu::machines::hp67::ActInstructionState::ThenGoto
        ));
    }

    #[test]
    fn installed_ram_write_commits_only_from_reconstructed_data_frame() {
        let mut live = Hp67LiveMachine::power_on_default().expect("live machine must construct");
        live.machine.act.state.ram_address = 0x12;
        live.machine.act.state.c = patterned_act_register(3);
        let before = live
            .machine
            .ram
            .read(0x12)
            .expect("HP-67 RAM slot must exist");

        let (_, transfer) = live
            .execute_word_with_deferred_ram_data(0, 0o1360)
            .expect("RAM write-class word must execute");
        let transfer = transfer.expect("installed RAM write must become a DATA transfer");
        assert_eq!(
            transfer.direction,
            Hp67DataTransferDirection::ActToPeripheral
        );
        assert_eq!(transfer.address, 0x12);
        assert_eq!(live.machine.ram.read(0x12), Some(before));

        carry_one_data_frame_to_next_word_tail(&mut live, transfer);
        live.complete_pending_ram_data_before_word(1)
            .expect("reconstructed DATA tail must commit RAM");
        assert_eq!(live.machine.ram.read(0x12), Some(transfer.expected));
    }

    #[test]
    fn installed_ram_read_commits_c_only_from_reconstructed_data_frame() {
        let mut live = Hp67LiveMachine::power_on_default().expect("live machine must construct");
        let payload = patterned_act_register(9);
        let original_c = patterned_act_register(1);
        live.machine.act.state.ram_address = 0x21;
        live.machine.act.state.c = original_c;
        assert!(live.machine.ram.write(0x21, payload));

        let (_, transfer) = live
            .execute_word_with_deferred_ram_data(0, 0o0070)
            .expect("RAM read-class word must execute");
        let transfer = transfer.expect("installed RAM read must become a DATA transfer");
        assert_eq!(
            transfer.direction,
            Hp67DataTransferDirection::PeripheralToAct
        );
        assert_eq!(transfer.address, 0x21);
        assert_eq!(live.machine.act.state.c, original_c);

        carry_one_data_frame_to_next_word_tail(&mut live, transfer);
        live.complete_pending_ram_data_before_word(1)
            .expect("reconstructed DATA tail must commit ACT C");
        assert_eq!(live.machine.act.state.c, payload);
    }

    #[test]
    fn every_ui_key_used_by_the_panel_maps_to_a_physical_contact() {
        for action in [
            KeyAction::Digit(0),
            KeyAction::Digit(1),
            KeyAction::Digit(2),
            KeyAction::Digit(3),
            KeyAction::Digit(4),
            KeyAction::Digit(5),
            KeyAction::Digit(6),
            KeyAction::Digit(7),
            KeyAction::Digit(8),
            KeyAction::Digit(9),
            KeyAction::Decimal,
            KeyAction::ChangeSign,
            KeyAction::Exponent,
            KeyAction::ClearX,
            KeyAction::Enter,
            KeyAction::Add,
            KeyAction::Subtract,
            KeyAction::Multiply,
            KeyAction::Divide,
            KeyAction::FunctionF,
            KeyAction::FunctionG,
            KeyAction::FunctionH,
            KeyAction::SigmaPlus,
            KeyAction::Gto,
            KeyAction::Dsp,
            KeyAction::Indirect,
            KeyAction::Sst,
            KeyAction::Sto,
            KeyAction::Rcl,
            KeyAction::A,
            KeyAction::B,
            KeyAction::C,
            KeyAction::D,
            KeyAction::E,
            KeyAction::RunStop,
        ] {
            assert!(action.physical_key().is_some());
        }
        assert_eq!(KeyAction::Digit(10).physical_key(), None);
    }

    #[test]
    fn live_machine_consumes_a_held_digit_contact_after_boot_idle() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;

        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }
        assert_eq!(
            live.display_frame().segments(),
            &[0x00, 0x3f, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        live.set_key_contact(Some(Hp67Key::Digit1));
        let mut dispatched = false;
        for _ in 0..128 {
            live.step_firmware_cycle().unwrap();
            if live.machine.pc() == 0o1440 {
                dispatched = true;
                break;
            }
        }
        assert!(
            dispatched,
            "held Digit1 contact never reached firmware table 1440"
        );

        let wait_visits_before_release = live.main_wait_visits;
        live.set_key_contact(None);
        for _ in 0..512 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_visits_before_release
                && !live.machine.act.state.status[15]
                && live.display_frame().segments()
                    == &[0x00, 0x06, 0x80, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            {
                return;
            }
        }
        panic!("released Digit1 did not settle to physical 1. display");
    }

    #[test]
    fn live_run_mode_blank_card_finishes_on_physical_error_display() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .expect("blank card must insert in RUN mode");

        const ERROR_SEGMENTS: [u8; 5] = [0x79, 0x50, 0x50, 0x5c, 0x50];

        let mut motor_seen = false;
        let mut completed_seen = false;
        for _ in 0..65_536 {
            live.step_firmware_cycle().unwrap();
            motor_seen |= live.card_motor_on();
            completed_seen |= live.card_transport_complete();
            if live.display_frame().segments()[1..6] == ERROR_SEGMENTS[..] {
                assert!(motor_seen, "blank RUN card never started the reader motor");
                assert!(
                    completed_seen,
                    "blank RUN card reached Error before traversing the modeled track"
                );
                return;
            }
        }

        panic!(
            "blank RUN card did not reach physical Error; motor_seen={motor_seen}, completed_seen={completed_seen}, display={:?}",
            live.display_frame().segments()
        );
    }

    #[test]
    fn live_power_on_reset_rebuilds_electronic_state() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        let mut nonzero = [0u8; 14];
        nonzero[0] = 7;
        assert!(live.machine.ram.write(0x1f, nonzero));
        live.set_key_contact(Some(Hp67Key::Digit9));
        live.reset_power_on().unwrap();

        assert_eq!(live.phase, LiveBootPhase::ResetHold);
        assert_eq!(live.display_frame(), HardwareDisplayFrame::BLANK);
        assert_eq!(live.machine.ram.read(0x1f), Some([0; 14]));
        assert!(!live.magnetic_card_inserted());
        assert_eq!(
            live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(false)
        );
    }

    #[test]
    fn live_power_off_recovers_inserted_card_and_drops_card_present() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .expect("blank card must insert");
        wait_for_live_card_record_stream(&mut live);
        let card = live
            .take_magnetic_card_for_power_off()
            .expect("power-off card recovery must update the contact");

        assert!(card.is_some());
        assert!(!live.magnetic_card_inserted());
        assert_eq!(
            live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(false)
        );
    }

    #[test]
    fn live_unstarted_card_can_be_withdrawn_before_firmware_starts_motor() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .expect("blank card must insert");
        assert_eq!(
            live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(true)
        );
        let card = live
            .withdraw_unstarted_magnetic_card()
            .expect("withdrawal contact update must succeed");
        assert!(card.is_some());
        assert!(!live.magnetic_card_inserted());
        assert_eq!(
            live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(false)
        );
    }

    #[test]
    fn live_firmware_card_presence_reaches_real_motor_on_request() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;

        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        assert_eq!(
            live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(false)
        );
        assert_eq!(live.machine.crc.flag(CRC_FLAG_MOTOR_ON), Some(false));

        live.machine.set_card_present(true).unwrap();

        for _ in 0..1_024 {
            live.step_firmware_cycle().unwrap();
            if live.machine.crc.flag(CRC_FLAG_MOTOR_ON) == Some(true) {
                assert_eq!(
                    live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
                    Some(true)
                );
                return;
            }
        }

        panic!("card-present contact never reached firmware motor-on request");
    }

    #[test]
    fn live_motor_advances_card_transport_at_nominal_record_cadence() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;

        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        let mut words = [0x0300_0000; HP67_CARD_RECORDS_PER_TRACK];
        words[1] = 0x0311_1111;
        insert_track1(&mut live, Hp67MagneticTrack::from_words(words).unwrap()).unwrap();
        wait_for_live_card_record_stream(&mut live);

        assert_eq!(live.machine.crc.flag(CRC_FLAG_MOTOR_ON), Some(true));
        assert_eq!(live.card_transport.next_record(), 0);
        assert!(live.card_transport.record_stream_active());

        let cycles_per_record = HP67_NOMINAL_CARD_RECORD_US.div_ceil(HP67_OBSERVED_WORD_TIME_US);
        for cycle in 1..=cycles_per_record {
            live.step_firmware_cycle().unwrap();
            if cycle < cycles_per_record {
                assert_eq!(live.card_transport.next_record(), 0);
            }
        }

        assert_eq!(live.card_transport.next_record(), 1);
        assert_eq!(live.machine.crc.flag(CRC_FLAG_BUFFER_READY), Some(true));
        assert_eq!(live.machine.crc.buffered_read_word(), Some(0x0300_0000));
    }

    #[test]
    fn live_firmware_consumes_timed_crc_record_through_real_0x9b_path() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;

        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        let words = [0x0300_0000; HP67_CARD_RECORDS_PER_TRACK];
        insert_track1(&mut live, Hp67MagneticTrack::from_words(words).unwrap()).unwrap();
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..8_192 {
            let execution = live.step_firmware_cycle_with_execution().unwrap();
            if let Some(Hp67ArchitecturalExecution {
                operation: Hp67ArchitecturalOperation::CrcDataRead { address, card_word },
                ..
            }) = execution
            {
                assert_eq!(address, hp67emu::machines::hp67::CRC_RAM_READ_ADDRESS);
                assert_eq!(card_word, 0x0300_0000);
                assert_eq!(&live.machine.act.state.c[0..7], &[0, 0, 0, 0, 0, 0, 3]);
                assert_eq!(&live.machine.act.state.c[7..14], &[0, 0, 0, 0, 0, 0, 3]);
                return;
            }
        }

        panic!("timed card transport never reached the real CRC 0x9B read path");
    }

    fn press_live_key_to_dispatch_for_card_test(live: &mut Hp67LiveMachine, key: Hp67Key) -> u16 {
        const KEYS_TO_A_OPCODE: u16 = 0o0120;
        const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;

        let expected_code = key.scan_code();
        live.set_key_contact(Some(key));
        let mut saw_keys_to_a = false;

        for _ in 0..512 {
            let execution = live.step_firmware_cycle_with_execution().unwrap();
            let Some(execution) = execution else {
                continue;
            };

            if matches!(
                execution.operation,
                Hp67ArchitecturalOperation::Act(ActOperation::Special {
                    opcode: KEYS_TO_A_OPCODE
                })
            ) {
                let observed = (live.machine.act.state.a[2] << 4) | live.machine.act.state.a[1];
                assert_eq!(observed, expected_code);
                saw_keys_to_a = true;
            }

            if matches!(
                execution.operation,
                Hp67ArchitecturalOperation::Act(ActOperation::Special {
                    opcode: A_TO_ROM_ADDRESS_OPCODE
                })
            ) {
                assert!(saw_keys_to_a);
                live.set_key_contact(None);
                return execution.next_pc;
            }
        }

        live.set_key_contact(None);
        panic!("{key:?} did not reach firmware keyboard dispatch");
    }

    fn press_live_key_and_settle_for_card_test(live: &mut Hp67LiveMachine, key: Hp67Key) {
        let target = press_live_key_to_dispatch_for_card_test(live, key);
        let wait_visits = live.main_wait_visits;
        for _ in 0..4_096 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_visits && !live.machine.act.state.status[15] {
                return;
            }
        }

        panic!("{key:?} dispatch {target:04o} did not return to the no-key firmware wait");
    }

    fn settle_live_program_mode(live: &mut Hp67LiveMachine) {
        live.machine.set_program_mode(true).unwrap();
        let wait_before = live.main_wait_visits;
        for _ in 0..2_048 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_before && live.machine.act.state.status[11] {
                return;
            }
        }
        panic!("real firmware never settled into PROGRAM mode");
    }

    fn wait_for_live_card_record_stream(live: &mut Hp67LiveMachine) {
        for _ in 0..2_048 {
            live.step_firmware_cycle().unwrap();
            if live.card_record_stream_active() {
                return;
            }
        }

        panic!("live card lifecycle never reached the CRC record stream");
    }

    #[test]
    fn moon_rocket_card_load_and_a_key_execute_without_disabling_live_machine() {
        let imported = TeenixHppImport::from_bytes(include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/HP-67 Standard Pac/SD1-14A_1 Moon Rocket Lander.hpp"
        )))
        .expect("checked-in Moon Rocket Lander card must decode");
        assert_eq!(imported.header_id, 3);

        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        live.insert_magnetic_card(imported.into_card(), CardInsertionEnd::End1)
            .expect("Moon Rocket Lander card must insert");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..32_768 {
            live.step_firmware_cycle().unwrap();
            if live.card_transport_complete() {
                break;
            }
        }
        assert!(live.card_transport_complete());
        assert!(live.take_completed_magnetic_card().is_some());

        for _ in 0..4_096 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits >= 3
                && !live.machine.act.state.status[15]
                && !live.card_motor_on()
            {
                break;
            }
        }

        let dispatch = press_live_key_to_dispatch_for_card_test(&mut live, Hp67Key::A);
        eprintln!(
            "Moon Rocket A dispatch={dispatch:04o} pc={:04o} ram2f={:x?} ram3d={:x?}",
            live.machine.pc(),
            live.machine.ram.read(0x2f),
            live.machine.ram.read(0x3d)
        );

        let mut saw_nonblank = live
            .display_frame()
            .segments()
            .iter()
            .copied()
            .any(|segments| segments != 0);
        for cycle_after_a in 0..32_768 {
            match live.step_firmware_cycle_with_execution() {
                Ok(_) => {
                    saw_nonblank |= live
                        .display_frame()
                        .segments()
                        .iter()
                        .copied()
                        .any(|segments| segments != 0);
                }
                Err(error) => {
                    panic!(
                        "Moon Rocket A disabled the live machine after {cycle_after_a} cycles: {error}; pc={:04o} ram2f={:x?} ram3d={:x?} display={:02x?} A={:x?} B={:x?} C={:x?}",
                        live.machine.pc(),
                        live.machine.ram.read(0x2f),
                        live.machine.ram.read(0x3d),
                        live.display_frame().segments(),
                        live.machine.act.state.a,
                        live.machine.act.state.b,
                        live.machine.act.state.c,
                    );
                }
            }
        }

        assert!(
            saw_nonblank,
            "Moon Rocket A ran without a live-machine error but never produced a visible display"
        );
    }

    #[test]
    fn live_program_mode_reaches_real_crc_0x99_write_path() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }
        settle_live_program_mode(&mut live);

        insert_track1(&mut live, Hp67MagneticTrack::default()).expect("blank side must insert");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..8_192 {
            let execution = live.step_firmware_cycle_with_execution().unwrap();
            if let Some(Hp67ArchitecturalExecution {
                operation: Hp67ArchitecturalOperation::CrcDataWrite { address, card_word },
                ..
            }) = execution
            {
                assert_eq!(address, hp67emu::machines::hp67::CRC_RAM_WRITE_ADDRESS);
                assert_eq!(live.machine.crc.flag(CRC_FLAG_WRITE_MODE), Some(true));
                assert!(card_word <= hp67emu::machines::hp67::CRC_CARD_WORD_MASK);
                assert!(live.machine.crc.queued_write_words() > 0);
                return;
            }
        }

        panic!("PROGRAM-mode card write never reached the real CRC 0x99 path");
    }

    #[test]
    fn live_firmware_full_card_write_then_read_round_trip_preserves_34_records() {
        let mut writer = Hp67LiveMachine::power_on_default().unwrap();
        writer.phase = LiveBootPhase::Firmware;
        while !matches!(writer.phase, LiveBootPhase::Idle) {
            writer.step_firmware_cycle().unwrap();
        }
        settle_live_program_mode(&mut writer);

        insert_track1(&mut writer, Hp67MagneticTrack::default())
            .expect("blank side must insert for firmware write");
        wait_for_live_card_record_stream(&mut writer);

        let mut written = Vec::with_capacity(HP67_CARD_RECORDS_PER_TRACK);
        for _ in 0..32_768 {
            if let Some(Hp67ArchitecturalExecution {
                operation: Hp67ArchitecturalOperation::CrcDataWrite { card_word, .. },
                ..
            }) = writer.step_firmware_cycle_with_execution().unwrap()
            {
                written.push(card_word);
            }

            if writer.card_transport_complete()
                && written.len() == HP67_CARD_RECORDS_PER_TRACK
                && writer.machine.crc.queued_write_words() == 0
            {
                break;
            }
        }

        assert_eq!(written.len(), HP67_CARD_RECORDS_PER_TRACK);
        assert!(writer.card_transport_complete());
        assert_eq!(writer.machine.crc.queued_write_words(), 0);

        assert_eq!(
            writer.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(false)
        );
        assert!(!writer.card_transport.head_active());
        let completed = writer
            .take_completed_magnetic_card()
            .expect("fully written card must be ejectable");
        assert!(completed.track(Hp67CardTrack::Track1).dirty());

        let mut reader = Hp67LiveMachine::power_on_default().unwrap();
        reader.phase = LiveBootPhase::Firmware;
        while !matches!(reader.phase, LiveBootPhase::Idle) {
            reader.step_firmware_cycle().unwrap();
        }

        reader
            .insert_magnetic_card(completed, CardInsertionEnd::End1)
            .expect("written card must reinsert for firmware read");
        wait_for_live_card_record_stream(&mut reader);

        let mut read_back = Vec::with_capacity(HP67_CARD_RECORDS_PER_TRACK);
        for _ in 0..32_768 {
            if let Some(Hp67ArchitecturalExecution {
                operation: Hp67ArchitecturalOperation::CrcDataRead { card_word, .. },
                ..
            }) = reader.step_firmware_cycle_with_execution().unwrap()
            {
                read_back.push(card_word);
                if read_back.len() == HP67_CARD_RECORDS_PER_TRACK {
                    break;
                }
            }
        }

        assert_eq!(read_back.len(), HP67_CARD_RECORDS_PER_TRACK);
        assert_eq!(read_back, written);
        assert!(reader.card_transport_complete());
        assert_eq!(
            reader.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
            Some(false)
        );

        for _ in 0..512 {
            if !reader.card_motor_on() {
                break;
            }
            reader.step_firmware_cycle().unwrap();
        }
        assert!(!reader.card_motor_on());
        assert!(reader.take_completed_magnetic_card().is_some());
    }

    #[test]
    fn live_two_track_program_write_and_read_reinsert_same_card_through_crd() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }
        settle_live_program_mode(&mut live);

        let mut second_half_program = [0u8; 14];
        second_half_program[0] = 1;
        assert!(live.machine.ram.write(0x1f, second_half_program));

        live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .expect("blank physical card must insert for first program track");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..32_768 {
            live.step_firmware_cycle().unwrap();
            if live.card_transport_complete() && live.machine.crc.queued_write_words() == 0 {
                break;
            }
        }
        assert!(live.card_transport_complete());
        let first_pass = live
            .take_completed_magnetic_card()
            .expect("first written track must return the same physical card");
        assert_eq!(
            first_pass
                .track(Hp67CardTrack::Track1)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(3)
        );
        assert!(!first_pass.track(Hp67CardTrack::Track2).is_recorded());

        for _ in 0..8_192 {
            live.step_firmware_cycle().unwrap();
            if live.card_prompt_visible() {
                break;
            }
        }
        assert!(
            live.card_prompt_visible(),
            "firmware never displayed the physical Crd second-card prompt"
        );
        assert_eq!(&live.display_frame().segments()[1..4], &[0x39, 0x50, 0x5e]);

        live.insert_magnetic_card(first_pass, CardInsertionEnd::End2)
            .expect("same physical card must reinsert by the opposite end");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..32_768 {
            live.step_firmware_cycle().unwrap();
            if live.card_transport_complete() && live.machine.crc.queued_write_words() == 0 {
                break;
            }
        }
        assert!(live.card_transport_complete());

        let completed = live
            .take_completed_magnetic_card()
            .expect("second written track must return the same physical card");
        assert!(completed.track(Hp67CardTrack::Track1).dirty());
        assert!(completed.track(Hp67CardTrack::Track2).dirty());
        assert_eq!(
            completed
                .track(Hp67CardTrack::Track2)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(4)
        );

        let mut reader = Hp67LiveMachine::power_on_default().unwrap();
        reader.phase = LiveBootPhase::Firmware;
        while !matches!(reader.phase, LiveBootPhase::Idle) {
            reader.step_firmware_cycle().unwrap();
        }

        reader
            .insert_magnetic_card(completed, CardInsertionEnd::End1)
            .expect("two-track card must insert for first firmware read pass");
        wait_for_live_card_record_stream(&mut reader);

        for _ in 0..32_768 {
            reader.step_firmware_cycle().unwrap();
            if reader.card_transport_complete() {
                break;
            }
        }
        assert!(reader.card_transport_complete());

        let between_passes = reader
            .take_completed_magnetic_card()
            .expect("first read pass must return the same physical card");
        for _ in 0..8_192 {
            reader.step_firmware_cycle().unwrap();
            if reader.card_prompt_visible() {
                break;
            }
        }
        assert!(
            reader.card_prompt_visible(),
            "firmware never displayed Crd while waiting for program Track 2"
        );

        reader
            .insert_magnetic_card(between_passes, CardInsertionEnd::End2)
            .expect("same physical card must reinsert for second firmware read pass");
        wait_for_live_card_record_stream(&mut reader);

        for _ in 0..32_768 {
            reader.step_firmware_cycle().unwrap();
            if reader.card_transport_complete() {
                break;
            }
        }
        assert!(reader.card_transport_complete());
        assert_eq!(
            reader.machine.ram.read(0x1f),
            Some(second_half_program),
            "second logical track did not restore program steps 113-224"
        );
        assert!(reader.take_completed_magnetic_card().is_some());
    }

    #[test]
    fn live_wdata_two_track_write_and_read_use_headers_one_and_two_through_crd() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        press_live_key_and_settle_for_card_test(&mut live, Hp67Key::Digit1);
        press_live_key_and_settle_for_card_test(&mut live, Hp67Key::SigmaPlus);
        press_live_key_and_settle_for_card_test(&mut live, Hp67Key::FunctionF);
        press_live_key_to_dispatch_for_card_test(&mut live, Hp67Key::Enter);

        for _ in 0..8_192 {
            live.step_firmware_cycle().unwrap();
            if live.card_prompt_visible() {
                break;
            }
        }
        assert!(
            live.card_prompt_visible(),
            "W/DATA did not reach the firmware Crd insertion prompt"
        );
        assert!(live.card_write_mode());

        live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .expect("blank data card must insert for primary-register pass");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..32_768 {
            live.step_firmware_cycle().unwrap();
            if live.card_transport_complete() && live.machine.crc.queued_write_words() == 0 {
                break;
            }
        }
        assert!(live.card_transport_complete());
        let first_pass = live
            .take_completed_magnetic_card()
            .expect("primary data pass must return the same physical card");
        assert_eq!(
            first_pass
                .track(Hp67CardTrack::Track1)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(1)
        );

        for _ in 0..8_192 {
            live.step_firmware_cycle().unwrap();
            if live.card_prompt_visible() {
                break;
            }
        }
        assert!(
            live.card_prompt_visible(),
            "non-zero secondary registers did not request the W/DATA second pass"
        );

        live.insert_magnetic_card(first_pass, CardInsertionEnd::End2)
            .expect("same data card must reinsert for secondary-register pass");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..32_768 {
            live.step_firmware_cycle().unwrap();
            if live.card_transport_complete() && live.machine.crc.queued_write_words() == 0 {
                break;
            }
        }
        assert!(live.card_transport_complete());

        let completed = live
            .take_completed_magnetic_card()
            .expect("secondary data pass must return the same physical card");
        assert_eq!(
            completed
                .track(Hp67CardTrack::Track2)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(2)
        );

        let mut reader = Hp67LiveMachine::power_on_default().unwrap();
        reader.phase = LiveBootPhase::Firmware;
        while !matches!(reader.phase, LiveBootPhase::Idle) {
            reader.step_firmware_cycle().unwrap();
        }

        reader
            .insert_magnetic_card(completed, CardInsertionEnd::End1)
            .expect("two-track data card must insert for primary-register read");
        wait_for_live_card_record_stream(&mut reader);

        let mut primary_header = None;
        for _ in 0..32_768 {
            if let Some(Hp67ArchitecturalExecution {
                operation: Hp67ArchitecturalOperation::CrcDataRead { card_word, .. },
                ..
            }) = reader.step_firmware_cycle_with_execution().unwrap()
            {
                primary_header.get_or_insert((card_word >> 24) as u8);
            }

            if reader.card_transport_complete() {
                break;
            }
        }
        assert!(reader.card_transport_complete());
        assert_eq!(primary_header, Some(1));

        let between_passes = reader
            .take_completed_magnetic_card()
            .expect("primary data read must return the same physical card");
        for _ in 0..8_192 {
            reader.step_firmware_cycle().unwrap();
            if reader.card_prompt_visible() {
                break;
            }
        }
        assert!(
            reader.card_prompt_visible(),
            "data-card read did not request the secondary-register pass with Crd"
        );

        reader
            .insert_magnetic_card(between_passes, CardInsertionEnd::End2)
            .expect("same data card must reinsert for secondary-register read");
        wait_for_live_card_record_stream(&mut reader);

        let mut secondary_header = None;
        for _ in 0..32_768 {
            if let Some(Hp67ArchitecturalExecution {
                operation: Hp67ArchitecturalOperation::CrcDataRead { card_word, .. },
                ..
            }) = reader.step_firmware_cycle_with_execution().unwrap()
            {
                secondary_header.get_or_insert((card_word >> 24) as u8);
            }

            if reader.card_transport_complete() {
                break;
            }
        }
        assert!(reader.card_transport_complete());
        assert_eq!(secondary_header, Some(2));
        assert!(reader.take_completed_magnetic_card().is_some());
    }

    #[test]
    fn live_write_protected_side_reaches_crc_f7_without_modification() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }
        settle_live_program_mode(&mut live);

        insert_track1(
            &mut live,
            Hp67MagneticTrack::default().with_write_protected(true),
        )
        .expect("protected side must insert");
        wait_for_live_card_record_stream(&mut live);

        for _ in 0..8_192 {
            let execution = live.step_firmware_cycle_with_execution().unwrap();
            if let Some(Hp67ArchitecturalExecution {
                operation:
                    Hp67ArchitecturalOperation::CrcControl {
                        instruction: CrcInstruction::TestFlagAndClear { flag },
                        condition: Some(true),
                    },
                ..
            }) = execution
            {
                if usize::from(flag) == CRC_FLAG_F7_STATUS {
                    assert_eq!(live.card_transport.next_record(), 0);
                    assert!(!live
                        .card_transport
                        .active_track()
                        .expect("protected side remains inserted")
                        .dirty());
                    return;
                }
            }

            if matches!(
                execution,
                Some(Hp67ArchitecturalExecution {
                    operation: Hp67ArchitecturalOperation::CrcDataWrite { .. },
                    ..
                })
            ) {
                panic!("write-protected card reached CRC 0x99 data write");
            }
        }

        panic!("write-protected card never reached CRC F7 error status");
    }

    #[test]
    fn physical_display_frame_recognizes_firmware_crd_prompt_segments() {
        let mut frame = HardwareDisplayFrame::BLANK;
        frame.segments[1] = 0x39;
        frame.segments[2] = 0x50;
        frame.segments[3] = 0x5e;
        assert!(frame.shows_card_prompt());

        frame.segments[3] = 0x00;
        assert!(!frame.shows_card_prompt());
    }

    #[test]
    fn ui_state_tracks_mechanical_switch_positions() {
        let mut state = Hp67State::default();
        state.handle(UiEvent::ToggleMode);
        assert_eq!(state.mode, RunMode::Program);
        state.handle(UiEvent::TogglePower);
        assert!(state.power_on);
    }

    #[test]
    fn raw_scan_slots_map_to_physical_left_to_right_positions() {
        let mut frame = HardwareDisplayFrame::BLANK;
        frame
            .capture_scan_slot(4, Hp67SegmentMask::from_bits(0x3f))
            .unwrap();
        frame.capture_scan_slot(5, Hp67SegmentMask::DP).unwrap();
        frame
            .capture_scan_slot(6, Hp67SegmentMask::from_bits(0x3f))
            .unwrap();
        frame
            .capture_scan_slot(7, Hp67SegmentMask::from_bits(0x3f))
            .unwrap();
        assert_eq!(frame.segments()[1], 0x3f);
        assert_eq!(frame.segments()[2], 0x80);
        assert_eq!(frame.segments()[3], 0x3f);
        assert_eq!(frame.segments()[4], 0x3f);
    }

    #[test]
    fn shared_sign_anodes_are_split_into_two_physical_minus_positions() {
        let mut frame = HardwareDisplayFrame::BLANK;
        frame
            .capture_scan_slot(
                3,
                Hp67SegmentMask::from_bits(Hp67SegmentMask::E.bits() | Hp67SegmentMask::G.bits()),
            )
            .unwrap();
        assert_eq!(frame.segments()[0], Hp67SegmentMask::G.bits());
        assert_eq!(frame.segments()[12], Hp67SegmentMask::G.bits());
    }

    #[test]
    fn reset_act_registers_naturally_release_all_display_serial_bits() {
        let machine = Hp67ArchitecturalMachine::default();
        assert!(!machine.act.state.display_enable);
        for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            let serializer =
                ActDisplayWordSerializer::from_state(scan_slot, &machine.act.state).unwrap();
            for word_bit in 0..8 {
                assert_eq!(serializer.drive_for_bit(word_bit), Drive::HighZ);
            }
        }
    }
}
