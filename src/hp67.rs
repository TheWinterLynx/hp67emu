use std::time::Duration;

use hp67emu::machines::hp67::{
    decode_rom0_display_byte, display_register_index_for_scan_slot,
    run_structural_display_fetch_cycle, ActOperation, ActSerialEndpoint, ActSerialRegister,
    CathodeDriver1820_1749, FetchPipelineLatch, Hp67ArchitecturalExecution,
    Hp67ArchitecturalMachine, Hp67ArchitecturalOperation, Hp67CardTransport,
    Hp67ElectricalBackplane, Hp67Firmware, Hp67Key, Hp67Keyboard, Hp67SegmentMask,
    Rom0DisplayEndpoint, RomFetchEndpoint, CRC_FLAG_MOTOR_ON, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,
    HP67_OBSERVED_WORD_TIME_US,
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

/// UI-owned live HP-67 machine used only as the source of physical LED state.
///
/// `hp67firmware` is versioned with the emulator and embedded in the executable.
/// Startup executes through the same combined display/fetch word transport used
/// by the structural smoke tests. ACT owns the display phase and the b0..b55
/// lifetime of the executing instruction, ROM0 emits STR events, and the cathode
/// driver only consumes downstream STR/RCD control events.
pub struct Hp67LiveMachine {
    source: Hp67Firmware,
    backplane: Hp67ElectricalBackplane,
    act_serial: ActSerialEndpoint,
    fetch_rom: RomFetchEndpoint,
    display_rom0: Rom0DisplayEndpoint,
    cathode: CathodeDriver1820_1749,
    pipeline: FetchPipelineLatch,
    machine: Hp67ArchitecturalMachine,
    keyboard: Hp67Keyboard,
    card_transport: Hp67CardTransport,
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
            machine: Hp67ArchitecturalMachine::default(),
            keyboard: Hp67Keyboard::default(),
            card_transport: Hp67CardTransport::default(),
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
        self.machine = Hp67ArchitecturalMachine::default();
        self.keyboard = Hp67Keyboard::default();
        self.card_transport = Hp67CardTransport::default();
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
        self.pipeline.begin_cycle();
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

            let execution = self
                .machine
                .execute_word(word)
                .map_err(|error| format!("live cycle {cycle} execution failed: {error:?}"))?;
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

        self.transport_fetch_word(cycle)?;

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

        self.card_transport
            .advance_us(
                HP67_OBSERVED_WORD_TIME_US,
                card_motor_was_on,
                &mut self.machine.crc,
            )
            .map_err(|error| format!("live cycle {cycle} card transport failed: {error:?}"))?;

        self.boot_cycle = cycle.saturating_add(1);
        if idle_after_word {
            self.phase = LiveBootPhase::Idle;
        }
        Ok(executed)
    }

    fn transport_fetch_word(&mut self, cycle: u64) -> Result<(), String> {
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
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            display_state,
            &mut self.act_serial,
            &mut self.fetch_rom,
            &mut self.display_rom0,
            &self.source,
        )
        .map_err(|error| format!("live cycle {cycle} shared word failed: {error:?}"))?;
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
mod m12_tests;

#[cfg(test)]
mod tests {
    use super::*;
    use hp67emu::{
        emulation::Drive,
        machines::hp67::{
            ActDisplayWordSerializer, CrcInstruction, Hp67CardSide, CRC_FLAG_BUFFER_READY,
            CRC_FLAG_CARD_PRESENT, CRC_FLAG_F7_STATUS, CRC_FLAG_WRITE_MODE,
            HP67_CARD_RECORDS_PER_SIDE, HP67_DISPLAY_SCAN_SLOTS, HP67_NOMINAL_CARD_RECORD_US,
        },
    };

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

        let mut words = [0x0300_0000; HP67_CARD_RECORDS_PER_SIDE];
        words[1] = 0x0311_1111;
        live.card_transport
            .insert_side(Hp67CardSide::from_words(words).unwrap())
            .unwrap();
        live.card_transport.set_head_active(true);
        live.machine.set_card_present(true).unwrap();

        for _ in 0..1_024 {
            live.step_firmware_cycle().unwrap();
            if live.machine.crc.flag(CRC_FLAG_MOTOR_ON) == Some(true) {
                break;
            }
        }
        assert_eq!(live.machine.crc.flag(CRC_FLAG_MOTOR_ON), Some(true));
        assert_eq!(live.card_transport.next_record(), 0);

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

        let words = [0x0300_0000; HP67_CARD_RECORDS_PER_SIDE];
        live.card_transport
            .insert_side(Hp67CardSide::from_words(words).unwrap())
            .unwrap();
        live.card_transport.set_head_active(true);
        live.machine.set_card_present(true).unwrap();

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

    #[test]
    fn live_program_mode_reaches_real_crc_0x99_write_path() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }
        settle_live_program_mode(&mut live);

        live.card_transport
            .insert_side(Hp67CardSide::default())
            .expect("blank side must insert");
        live.card_transport.set_head_active(true);
        live.machine.set_card_present(true).unwrap();

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
    fn live_write_protected_side_reaches_crc_f7_without_modification() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }
        settle_live_program_mode(&mut live);

        live.card_transport
            .insert_side(Hp67CardSide::default().with_write_protected(true))
            .expect("protected side must insert");
        live.card_transport.set_head_active(true);
        live.machine.set_card_present(true).unwrap();

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
                        .side()
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
