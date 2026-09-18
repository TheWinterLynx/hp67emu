use std::time::Duration;

use hp67emu::machines::hp67::{
    decode_rom0_display_byte, display_register_index_for_scan_slot,
    run_structural_display_fetch_cycle, ActOperation, ActSerialEndpoint, ActSerialRegister,
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
            let mut serial_pre_state = self.machine.act.state.clone();
            if !self.display_control_seen {
                // Direct power-on capture shows display traffic before firmware
                // executes its first explicit DISPLAY control instruction.
                serial_pre_state.display_enable = true;
            }
            self.act_serial
                .begin_execution(word, &serial_pre_state)
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
        let mut display_state = self.machine.act.state.clone();
        if !self.display_control_seen {
            display_state.display_enable = true;
        }
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            &display_state,
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
mod tests {
    use super::*;
    use hp67emu::{
        emulation::Drive,
        machines::hp67::{ActDisplayWordSerializer, ActRegister, HP67_DISPLAY_SCAN_SLOTS},
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
            live.machine
                .crc
                .external_flag(hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE),
            Some(true)
        );

        live.set_program_mode(false).unwrap();
        assert_eq!(
            live.machine
                .crc
                .external_flag(hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE),
            Some(false)
        );
    }

    const HP67_PROGRAM_RAM_START: u8 = 0x10;
    const HP67_PROGRAM_RAM_END: u8 = 0x2f;
    const HP67_PROGRAM_PC_RAM: u8 = 0x3d;
    const KEYS_TO_A_OPCODE: u16 = 0o0120;
    const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;

    fn live_program_ram_snapshot(live: &Hp67LiveMachine) -> Vec<Option<ActRegister>> {
        (HP67_PROGRAM_RAM_START..=HP67_PROGRAM_RAM_END)
            .map(|address| live.machine.ram.read(address))
            .collect()
    }

    fn wait_for_firmware_mode(live: &mut Hp67LiveMachine, program: bool, label: &str) {
        live.set_program_mode(program).unwrap();
        let wait_visits = live.main_wait_visits;
        for _ in 0..8_192 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_visits
                && !live.machine.act.state.status[15]
                && live.machine.act.state.status[11] == program
            {
                return;
            }
        }
        panic!(
            "{label} switch transition did not settle in firmware; pc={:04o} s11={} s15={}",
            live.machine.pc(),
            live.machine.act.state.status[11],
            live.machine.act.state.status[15]
        );
    }

    fn press_program_key_and_require_ram_change(
        live: &mut Hp67LiveMachine,
        key: Hp67Key,
        expected_step: u8,
    ) {
        let before_program = live_program_ram_snapshot(live);
        let before_pc = live.machine.ram.read(HP67_PROGRAM_PC_RAM);
        let mut memory_trace = Vec::new();
        let mut control_trace = Vec::new();
        live.set_key_contact(Some(key));

        let expected_code = key.scan_code();
        let mut keys_to_a_seen = false;
        let mut dispatch_target = None;
        for step in 0..512 {
            if step < 96 {
                control_trace.push(format!(
                    "#{step:03} pc={:04o} word={} s15={} s3={} key={:?}",
                    live.machine.pc(),
                    live.pipeline
                        .executing_word()
                        .map(|word| format!("{word:04o}"))
                        .unwrap_or_else(|| "----".to_owned()),
                    live.machine.act.state.status[15],
                    live.machine.act.state.status[3],
                    live.keyboard.code()
                ));
            }

            let executing_word = live.pipeline.executing_word();
            let normal = live.machine.act.state.instruction_state
                == hp67emu::machines::hp67::ActInstructionState::Normal;
            if let Some(word) = executing_word {
                if word == 0o1160 || word == 0o1360 || (word & 0o77) == 0o50 {
                    memory_trace.push(format!(
                        "pc={:04o} word={word:04o} addr=0x{:02x} c01={:x}{:x}",
                        live.machine.pc(),
                        live.machine.act.state.ram_address,
                        live.machine.act.state.c[1],
                        live.machine.act.state.c[0]
                    ));
                }
            }

            live.step_firmware_cycle().unwrap();

            if normal && executing_word == Some(KEYS_TO_A_OPCODE) {
                let observed_code =
                    (live.machine.act.state.a[2] << 4) | live.machine.act.state.a[1];
                assert_eq!(
                    observed_code, expected_code,
                    "PROGRAM {key:?} keys -> A produced {observed_code:04o}, expected physical code {expected_code:04o}"
                );
                keys_to_a_seen = true;
            }

            if normal && executing_word == Some(A_TO_ROM_ADDRESS_OPCODE) {
                assert!(
                    keys_to_a_seen,
                    "PROGRAM {key:?} executed A -> ROM address before keys -> A"
                );
                let target = live.machine.pc();
                assert!(
                    (0o1405..=0o1466).contains(&target),
                    "PROGRAM {key:?} dispatched outside the unshifted HP-67 key table: {target:04o}"
                );
                dispatch_target = Some(target);
                break;
            }
        }
        assert!(
            dispatch_target.is_some(),
            "PROGRAM {key:?} never completed keys -> A / A -> ROM dispatch; control trace: {}",
            control_trace.join(" | ")
        );

        let wait_visits = live.main_wait_visits;
        live.set_key_contact(None);
        let mut changed = Vec::new();
        let mut settled = false;
        for _ in 0..4_096 {
            if let Some(word) = live.pipeline.executing_word() {
                if word == 0o1160 || word == 0o1360 || (word & 0o77) == 0o50 {
                    memory_trace.push(format!(
                        "pc={:04o} word={word:04o} addr=0x{:02x} c01={:x}{:x}",
                        live.machine.pc(),
                        live.machine.act.state.ram_address,
                        live.machine.act.state.c[1],
                        live.machine.act.state.c[0]
                    ));
                }
            }
            live.step_firmware_cycle().unwrap();

            let after_program = live_program_ram_snapshot(live);
            changed = before_program
                .iter()
                .zip(&after_program)
                .enumerate()
                .filter_map(|(offset, (before, after))| {
                    (before != after).then_some(usize::from(HP67_PROGRAM_RAM_START) + offset)
                })
                .collect();

            if live.machine.ram.read(HP67_PROGRAM_PC_RAM) != before_pc
                && live.main_wait_visits > wait_visits
                && !live.machine.act.state.status[15]
            {
                settled = true;
                break;
            }
        }

        assert_ne!(
            live.machine.ram.read(HP67_PROGRAM_PC_RAM),
            before_pc,
            "PROGRAM {key:?} did not advance the user-program counter in RAM 0x3D; dispatch={dispatch_target:?}; pc={:04o}; program-RAM changes={changed:?}; memory trace: {}; control trace: {}",
            live.machine.pc(),
            if memory_trace.is_empty() {
                "<none>".to_owned()
            } else {
                memory_trace.join(" | ")
            },
            control_trace.join(" | ")
        );
        assert!(
            settled,
            "PROGRAM {key:?} advanced RAM 0x3D but did not return to the no-key firmware wait; program-RAM changes={changed:?}"
        );

        let pc_register = live
            .machine
            .ram
            .read(HP67_PROGRAM_PC_RAM)
            .expect("HP-67 program-counter register 0x3D must be installed");
        assert!(
            (1..=7).contains(&expected_step),
            "this M12 oracle covers the first seven program steps"
        );
        assert_eq!(
            &pc_register[0..3],
            &[0x0f, 0x02, 7 - expected_step],
            "PROGRAM {key:?} did not leave RAM 0x3D at expected user step {expected_step:03}"
        );
    }

    fn press_live_key_to_dispatch(
        live: &mut Hp67LiveMachine,
        key: Hp67Key,
        expected_target: u16,
    ) -> u16 {
        let expected_code = key.scan_code();
        live.set_key_contact(Some(key));
        let mut saw_keys_to_a = false;

        for _ in 0..512 {
            let executing_word = live.pipeline.executing_word();
            let normal = live.machine.act.state.instruction_state
                == hp67emu::machines::hp67::ActInstructionState::Normal;

            live.step_firmware_cycle().unwrap();

            if normal && executing_word == Some(KEYS_TO_A_OPCODE) {
                let observed = (live.machine.act.state.a[2] << 4) | live.machine.act.state.a[1];
                assert_eq!(
                    observed, expected_code,
                    "{key:?} keys -> A produced {observed:04o}, expected physical code {expected_code:04o}"
                );
                saw_keys_to_a = true;
            }

            if normal && executing_word == Some(A_TO_ROM_ADDRESS_OPCODE) {
                assert!(
                    saw_keys_to_a,
                    "{key:?} executed A -> ROM address before keys -> A"
                );
                let target = live.machine.pc();
                assert_eq!(
                    target, expected_target,
                    "{key:?} firmware dispatch target mismatch"
                );
                live.set_key_contact(None);
                return target;
            }
        }

        live.set_key_contact(None);
        panic!(
            "{key:?} firmware did not complete keys -> A / A -> ROM dispatch; pc={:04o} s11={} s15={}",
            live.machine.pc(),
            live.machine.act.state.status[11],
            live.machine.act.state.status[15]
        );
    }

    fn settle_live_dispatch(live: &mut Hp67LiveMachine, key: Hp67Key, target: u16) {
        let wait_visits = live.main_wait_visits;
        for _ in 0..4_096 {
            live.step_firmware_cycle().unwrap();
            if live.main_wait_visits > wait_visits && !live.machine.act.state.status[15] {
                return;
            }
        }
        panic!(
            "{key:?} dispatch {target:04o} did not return to the no-key firmware wait; pc={:04o}",
            live.machine.pc()
        );
    }

    fn press_live_key_and_settle(live: &mut Hp67LiveMachine, key: Hp67Key, expected_target: u16) {
        let target = press_live_key_to_dispatch(live, key, expected_target);
        settle_live_dispatch(live, key, target);
    }

    #[test]
    fn live_program_mode_stores_and_executes_simple_program() {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.phase = LiveBootPhase::Firmware;
        while !matches!(live.phase, LiveBootPhase::Idle) {
            live.step_firmware_cycle().unwrap();
        }

        wait_for_firmware_mode(&mut live, true, "RUN -> PRGM");
        assert!(
            live.machine.act.state.status[11],
            "firmware did not latch PROGRAM mode in S11"
        );

        let program_before = live_program_ram_snapshot(&live);
        press_program_key_and_require_ram_change(&mut live, Hp67Key::Digit1, 1);
        press_program_key_and_require_ram_change(&mut live, Hp67Key::Enter, 2);
        press_program_key_and_require_ram_change(&mut live, Hp67Key::Digit2, 3);
        press_program_key_and_require_ram_change(&mut live, Hp67Key::Add, 4);
        press_program_key_and_require_ram_change(&mut live, Hp67Key::RunStop, 5);
        let program_after = live_program_ram_snapshot(&live);
        assert_ne!(
            program_after, program_before,
            "PROGRAM entry advanced RAM 0x3D but left the entire 0x10..0x2F program store unchanged"
        );

        let first_program_register = live
            .machine
            .ram
            .read(0x2f)
            .expect("HP-67 RAM 0x2F must hold program steps 001..007");
        assert_eq!(
            &first_program_register[0..10],
            &[0x01, 0x01, 0x0b, 0x01, 0x02, 0x01, 0x07, 0x03, 0x00, 0x00],
            "PROGRAM steps 001..005 are not the expected 11 1B 12 37 00 byte sequence"
        );

        wait_for_firmware_mode(&mut live, false, "PRGM -> RUN");
        assert!(
            !live.machine.act.state.status[11],
            "firmware retained PROGRAM-mode latch S11 after returning to RUN"
        );

        // HP-67 RUN-mode RTN with no program running clears the return stack and
        // sets the user-program counter to step 000. Use the real h -> RTN key
        // path and verify the physical counter register rather than normalizing X.
        press_live_key_and_settle(&mut live, Hp67Key::FunctionH, 0o1405);
        press_live_key_and_settle(&mut live, Hp67Key::Gto, 0o0560);
        assert_eq!(
            live.machine.ram.read(HP67_PROGRAM_PC_RAM),
            Some([0; 14]),
            "RUN-mode RTN did not clear RAM 0x3D to user-program step 000"
        );

        let wait_visits = live.main_wait_visits;
        press_live_key_to_dispatch(&mut live, Hp67Key::RunStop, 0o1443);

        let mut saw_running = false;
        for _ in 0..20_000 {
            live.step_firmware_cycle().unwrap();
            saw_running |= live.machine.act.state.status[2];
            if saw_running
                && !live.machine.act.state.status[2]
                && live.main_wait_visits > wait_visits
                && !live.machine.act.state.status[15]
                && live.display_frame().segments()
                    == &[
                        0x00, 0x00, 0x00, 0x4f, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0,
                    ]
            {
                return;
            }
        }

        panic!(
            "stored 11 1B 12 37 00 program did not run and halt at physical 3.00; pc={:04o} s2={} s11={} ram3d={:?}",
            live.machine.pc(),
            live.machine.act.state.status[2],
            live.machine.act.state.status[11],
            live.machine.ram.read(HP67_PROGRAM_PC_RAM)
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
