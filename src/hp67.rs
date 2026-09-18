use std::time::Duration;

use hp67emu::machines::hp67::{
    decode_rom0_display_byte, run_structural_display_fetch_cycle, ActOperation, ActSerialEndpoint,
    CathodeDriver1820_1749, FetchPipelineLatch, Hp67ArchitecturalMachine,
    Hp67ArchitecturalOperation, Hp67ElectricalBackplane, Hp67Firmware, Hp67Key, Hp67Keyboard,
    Hp67SegmentMask, Rom0DisplayEndpoint, RomFetchEndpoint, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,
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
        let cycle = self.boot_cycle;
        if self.phase != LiveBootPhase::Idle && cycle >= BOOT_CYCLE_LIMIT {
            return Err(format!(
                "HP-67 live boot did not reach the no-key idle checkpoint within {BOOT_CYCLE_LIMIT} cycles"
            ));
        }

        let mut idle_after_word = false;
        self.pipeline.begin_cycle();
        if let Some(word) = self.pipeline.executing_word() {
            self.keyboard.sample_into_act(&mut self.machine.act.state);
            self.act_serial
                .begin_execution(word, &self.machine.act.state)
                .map_err(|error| {
                    format!("live boot cycle {cycle} serial execution start failed: {error:?}")
                })?;

            let execution = self
                .machine
                .execute_word(word)
                .map_err(|error| format!("live boot cycle {cycle} execution failed: {error:?}"))?;

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
                format!("live boot cycle {cycle} lost serial execution for word 0x{word:03x}")
            })?;
            if serial_execution.word() != word || !serial_execution.is_complete() {
                return Err(format!(
                    "live boot cycle {cycle} serial execution incomplete: expected word 0x{word:03x}, got word 0x{:03x}, next_bit={:?}",
                    serial_execution.word(),
                    serial_execution.next_word_bit()
                ));
            }
        }

        self.boot_cycle = cycle.saturating_add(1);
        if idle_after_word {
            self.phase = LiveBootPhase::Idle;
        }
        Ok(())
    }

    fn transport_fetch_word(&mut self, cycle: u64) -> Result<(), String> {
        let requested_bank = self.machine.prepare_hp67_fetch();
        self.source.select_bank(requested_bank);
        let address = self.machine.pc();
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            &self.machine.act.state,
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
                    format!("live cycle {cycle} ROM0 decode failed at slot {scan_slot}: {error:?}")
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
mod tests {
    use super::*;
    use hp67emu::{
        emulation::Drive,
        machines::hp67::{ActDisplayWordSerializer, HP67_DISPLAY_SCAN_SLOTS},
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
    fn ui_state_tracks_mechanical_switch_positions() {
        let mut state = Hp67State::default();
        state.handle(UiEvent::ToggleMode);
        assert_eq!(state.mode, RunMode::Program);
        state.handle(UiEvent::TogglePower);
        assert!(state.power_on);
    }

    #[test]
    fn live_program_switch_drives_crc_external_flag() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();

        live.set_program_mode(true).unwrap();
        assert_eq!(
            live.machine.crc.external_flag(
                hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE
            ),
            Some(true)
        );

        live.set_program_mode(false).unwrap();
        assert_eq!(
            live.machine.crc.external_flag(
                hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE
            ),
            Some(false)
        );
    }

    fn press_live_key_to_target(
        live: &mut Hp67LiveMachine,
        key: Hp67Key,
        target: u16,
    ) {
        live.set_key_contact(Some(key));
        let mut dispatched = false;
        for _ in 0..512 {
            live.step_firmware_cycle().unwrap();
            if live.machine.pc() == target {
                dispatched = true;
                break;
            }
        }
        assert!(
            dispatched,
            "held {key:?} contact never reached firmware target {target:04o}"
        );

        let wait_visits = live.main_wait_visits;
        live.set_key_contact(None);
        for _ in 0..4_096 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_visits
                && !live.machine.act.state.status[15]
            {
                return;
            }
        }
        panic!(
            "released {key:?} did not return to no-key firmware wait after target {target:04o}"
        );
    }

    #[test]
    fn live_program_mode_stores_and_executes_simple_program() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        live.set_program_mode(true).unwrap();
        press_live_key_to_target(&mut live, Hp67Key::Digit1, 0o1440);
        press_live_key_to_target(&mut live, Hp67Key::Enter, 0o1422);
        press_live_key_to_target(&mut live, Hp67Key::Digit2, 0o1437);
        press_live_key_to_target(&mut live, Hp67Key::Add, 0o1434);
        press_live_key_to_target(&mut live, Hp67Key::RunStop, 0o1443);

        live.set_program_mode(false).unwrap();
        press_live_key_to_target(&mut live, Hp67Key::ClearX, 0o1417);
        assert_eq!(
            live.display_frame().segments(),
            &[0x00, 0x3f, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        );

        press_live_key_to_target(&mut live, Hp67Key::FunctionH, 0o1405);
        press_live_key_to_target(&mut live, Hp67Key::Gto, 0o0560);

        let wait_visits = live.main_wait_visits;
        live.set_key_contact(Some(Hp67Key::RunStop));
        let mut started = false;
        for _ in 0..512 {
            live.step_firmware_cycle().unwrap();
            if live.machine.pc() == 0o1443 {
                started = true;
                live.set_key_contact(None);
                break;
            }
        }
        assert!(started, "RUN R/S never reached firmware table 1443");

        for _ in 0..20_000 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_visits
                && !live.machine.act.state.status[15]
                && live.display_frame().segments()
                    == &[0x00, 0x4f, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
            {
                return;
            }
        }

        panic!(
            "stored 1 ENTER 2 + R/S program did not execute to physical 3.00 and stop"
        );
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
