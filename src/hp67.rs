use std::{cell::Cell, env, fs, time::Duration};

use hp67emu::{
    machines::hp67::{
        decode_rom0_display_byte, display_byte_from_act_registers,
        run_structural_display_fetch_cycle, ActFetchEndpoint, ActOperation, CathodeDriver1820_1749,
        FetchPipelineLatch, Hp67ArchitecturalMachine, Hp67ArchitecturalOperation,
        Hp67ElectricalBackplane, Hp67RomWordSource, Hp67SegmentMask, Rom0DisplayEndpoint,
        RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,
        HP67_OBSERVED_WORD_TIME_US,
    },
    research::rom_corpus::{RomCorpus, ROM_PAGES, WORDS_PER_PAGE},
};

const EXPECTED_POPULATED_WORDS: usize = 5120;
const DEFAULT_CORPUS_PATH: &str = ".research/teenix-2026-hp67.tsv";
const DISPLAY_INIT_PC: u16 = 0o0161;
const MAIN_WAIT_PC: u16 = 0o0167;
const CARD_POLL_PC: u16 = 0o0206;
const BOOT_CYCLE_LIMIT: u64 = 2_000;
const RESET_DISPLAY_CODE: u8 = 0x00;

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
    Key(KeyAction),
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
            power_on: true,
            mode: RunMode::Run,
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
            UiEvent::Key(_) => {
                // Key hit regions remain interactive, but no semantic key action
                // is allowed to synthesize display contents. Electrical keyboard
                // scanning will connect these contacts to the machine later.
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

struct UiCorpusRom {
    corpus: RomCorpus,
    requested_bank: Cell<u8>,
    page_bank_mask: [u8; ROM_PAGES],
}

impl UiCorpusRom {
    fn from_tsv(input: &str) -> Result<Self, String> {
        let corpus = RomCorpus::from_normalized_tsv(input)
            .map_err(|error| format!("failed to parse normalized ROM corpus: {error}"))?;
        if corpus.populated_words() != EXPECTED_POPULATED_WORDS {
            return Err(format!(
                "corpus has {} populated words; expected {EXPECTED_POPULATED_WORDS}",
                corpus.populated_words()
            ));
        }

        let mut page_bank_mask = [0u8; ROM_PAGES];
        for (page, mask) in page_bank_mask.iter_mut().enumerate() {
            let page_base = page * WORDS_PER_PAGE;
            for bank in 0..2usize {
                if (page_base..page_base + WORDS_PER_PAGE)
                    .any(|pc| corpus.get(bank, pc).ok().flatten().is_some())
                {
                    *mask |= 1u8 << bank;
                }
            }
        }

        Ok(Self {
            corpus,
            requested_bank: Cell::new(0),
            page_bank_mask,
        })
    }

    fn select_bank(&self, bank: u8) {
        self.requested_bank.set(bank & 1);
    }
}

impl Hp67RomWordSource for UiCorpusRom {
    fn read_word(&self, address: u16) -> Option<u16> {
        let requested = usize::from(self.requested_bank.get());
        let page = usize::from(address) / WORDS_PER_PAGE;
        let effective = if self.page_bank_mask[page] & (1u8 << requested) != 0 {
            requested
        } else {
            0
        };
        self.corpus
            .get(effective, usize::from(address))
            .ok()
            .flatten()
    }
}

/// UI-owned live HP-67 machine used only as the source of physical LED state.
///
/// The firmware remains external. Startup executes through the same combined
/// display/fetch word transport used by the structural smoke test. The remaining
/// temporary bridge is A/B -> b0..b7 inside `display_byte_from_act_registers()`.
pub struct Hp67LiveMachine {
    source: UiCorpusRom,
    backplane: Hp67ElectricalBackplane,
    fetch_act: ActFetchEndpoint,
    fetch_rom: RomFetchEndpoint,
    display_rom0: Rom0DisplayEndpoint,
    cathode: CathodeDriver1820_1749,
    pipeline: FetchPipelineLatch,
    machine: Hp67ArchitecturalMachine,
    display: HardwareDisplayFrame,
    phase: LiveBootPhase,
    pending_us: u64,
    boot_cycle: u64,
    saw_display_init: bool,
    main_wait_visits: u64,
    card_poll_visits: u64,
    display_control_seen: bool,
}

fn power_on_reset_display_frame() -> Result<HardwareDisplayFrame, String> {
    let mut frame = HardwareDisplayFrame::BLANK;
    for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
        let anodes = decode_rom0_display_byte(scan_slot, RESET_DISPLAY_CODE).map_err(|error| {
            format!("power-on reset ROM0 decode failed at slot {scan_slot}: {error:?}")
        })?;
        frame.capture_scan_slot(scan_slot, anodes)?;
    }
    Ok(frame)
}

impl Hp67LiveMachine {
    pub fn power_on_default() -> Result<Self, String> {
        let corpus_path =
            env::var("HP67_ROM_CORPUS").unwrap_or_else(|_| DEFAULT_CORPUS_PATH.to_owned());
        let input = fs::read_to_string(&corpus_path)
            .map_err(|error| format!("failed to read HP-67 ROM corpus {corpus_path}: {error}"))?;
        let source = UiCorpusRom::from_tsv(&input)?;
        let display = power_on_reset_display_frame()?;
        Ok(Self {
            source,
            backplane: Hp67ElectricalBackplane::default(),
            fetch_act: ActFetchEndpoint::new(0),
            fetch_rom: RomFetchEndpoint::default(),
            display_rom0: Rom0DisplayEndpoint::default(),
            cathode: CathodeDriver1820_1749::default(),
            pipeline: FetchPipelineLatch::default(),
            machine: Hp67ArchitecturalMachine::default(),
            display,
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

    pub const fn is_booting(&self) -> bool {
        !matches!(self.phase, LiveBootPhase::Idle)
    }

    pub fn reset_power_on(&mut self) -> Result<(), String> {
        self.source.select_bank(0);
        self.backplane = Hp67ElectricalBackplane::default();
        self.fetch_act = ActFetchEndpoint::new(0);
        self.fetch_rom = RomFetchEndpoint::default();
        self.display_rom0 = Rom0DisplayEndpoint::default();
        self.cathode = CathodeDriver1820_1749::default();
        self.pipeline = FetchPipelineLatch::default();
        self.machine = Hp67ArchitecturalMachine::default();
        self.display = power_on_reset_display_frame()?;
        self.phase = LiveBootPhase::ResetHold;
        self.pending_us = 0;
        self.boot_cycle = 0;
        self.saw_display_init = false;
        self.main_wait_visits = 0;
        self.card_poll_visits = 0;
        self.display_control_seen = false;
        Ok(())
    }

    pub fn advance(&mut self, elapsed: Duration) -> Result<(), String> {
        if self.phase == LiveBootPhase::Idle {
            return Ok(());
        }

        let elapsed_us = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
        self.pending_us = self.pending_us.saturating_add(elapsed_us);

        if self.phase == LiveBootPhase::ResetHold {
            if self.pending_us < HP67_OBSERVED_POWER_ON_SYNC_DELAY_US {
                return Ok(());
            }
            self.pending_us -= HP67_OBSERVED_POWER_ON_SYNC_DELAY_US;
            self.phase = LiveBootPhase::Firmware;
        }

        while self.phase == LiveBootPhase::Firmware && self.pending_us >= HP67_OBSERVED_WORD_TIME_US
        {
            self.pending_us -= HP67_OBSERVED_WORD_TIME_US;
            self.step_firmware_cycle()?;
        }
        Ok(())
    }

    fn step_firmware_cycle(&mut self) -> Result<(), String> {
        let cycle = self.boot_cycle;
        if cycle >= BOOT_CYCLE_LIMIT {
            return Err(format!(
                "HP-67 live boot did not reach the no-key idle checkpoint within {BOOT_CYCLE_LIMIT} cycles"
            ));
        }

        self.pipeline.begin_cycle();
        if let Some(word) = self.pipeline.executing_word() {
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

            if self.saw_display_init
                && self.main_wait_visits >= 2
                && self.card_poll_visits >= 1
                && self.machine.act.state.display_enable
                && self.machine.act.state.key_buffer.is_none()
            {
                self.phase = LiveBootPhase::Idle;
                self.boot_cycle = cycle.saturating_add(1);
                return Ok(());
            }
        }

        self.transport_fetch_word(cycle)?;
        self.boot_cycle = cycle.saturating_add(1);
        Ok(())
    }

    fn transport_fetch_word(&mut self, cycle: u64) -> Result<(), String> {
        let requested_bank = self.machine.prepare_hp67_fetch();
        self.source.select_bank(requested_bank);
        let address = self.machine.pc();
        let scan_slot = self.cathode.scan_slot();
        let display_byte = display_byte_from_act_registers(
            scan_slot,
            &self.machine.act.state.a,
            &self.machine.act.state.b,
        )
        .map_err(|error| {
            format!("live cycle {cycle} display byte composition failed: {error:?}")
        })?;
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            display_byte,
            &mut self.fetch_act,
            &mut self.fetch_rom,
            &mut self.display_rom0,
            &self.source,
        )
        .map_err(|error| format!("live cycle {cycle} shared word failed: {error:?}"))?;
        if result.display_byte != display_byte {
            return Err(format!(
                "live cycle {cycle} ROM0 reconstructed display byte 0x{:02x}, expected 0x{display_byte:02x}",
                result.display_byte
            ));
        }
        self.pipeline.complete_cycle(result.fetched_word);

        if self.display_control_seen {
            if self.machine.act.state.display_enable {
                let anodes =
                    decode_rom0_display_byte(scan_slot, result.display_byte).map_err(|error| {
                        format!(
                            "live cycle {cycle} ROM0 decode failed at slot {scan_slot}: {error:?}"
                        )
                    })?;
                self.display.capture_scan_slot(scan_slot, anodes)?;
            } else {
                self.display.clear();
            }
        }

        self.cathode.str_falling_edge();
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn ui_state_no_longer_contains_a_fake_display_value() {
        let mut state = Hp67State::default();
        state.handle(UiEvent::Key(KeyAction::Digit(7)));
        assert!(state.power_on);
        assert_eq!(state.mode, RunMode::Run);
    }

    #[test]
    fn mode_and_power_controls_remain_ui_owned() {
        let mut state = Hp67State::default();
        state.handle(UiEvent::ToggleMode);
        assert_eq!(state.mode, RunMode::Program);
        state.handle(UiEvent::TogglePower);
        assert!(!state.power_on);
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
    fn reset_bus_zero_code_produces_observed_all_zero_power_on_pattern() {
        let frame = power_on_reset_display_frame().unwrap();
        assert_eq!(frame.segments()[0], Hp67SegmentMask::G.bits());
        for index in 1..=11 {
            assert_eq!(frame.segments()[index], 0x3f);
        }
        assert_eq!(frame.segments()[12], 0);
        assert_eq!(frame.segments()[13], 0x3f);
        assert_eq!(frame.segments()[14], 0x3f);
    }
}
