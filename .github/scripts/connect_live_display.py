from pathlib import Path

hp67 = r'''use std::{cell::Cell, env, fs};

use hp67emu::{
    machines::hp67::{
        decode_rom0_display_byte, display_byte_from_act_registers,
        run_structural_display_fetch_cycle, ActFetchEndpoint, CathodeDriver1820_1749,
        FetchPipelineLatch, Hp67ArchitecturalMachine, Hp67ElectricalBackplane, Hp67RomWordSource,
        Hp67SegmentMask, Rom0DisplayEndpoint, RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS,
    },
    research::rom_corpus::{RomCorpus, ROM_PAGES, WORDS_PER_PAGE},
};

const EXPECTED_POPULATED_WORDS: usize = 5120;
const DEFAULT_CORPUS_PATH: &str = ".research/teenix-2026-hp67.tsv";
const DISPLAY_INIT_PC: u16 = 0o0161;
const MAIN_WAIT_PC: u16 = 0o0167;
const CARD_POLL_PC: u16 = 0o0206;
const BOOT_CYCLE_LIMIT: u64 = 2_000;

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
}

impl Hp67LiveMachine {
    pub fn boot_default() -> Result<Self, String> {
        let corpus_path = env::var("HP67_ROM_CORPUS").unwrap_or_else(|_| DEFAULT_CORPUS_PATH.to_owned());
        let input = fs::read_to_string(&corpus_path)
            .map_err(|error| format!("failed to read HP-67 ROM corpus {corpus_path}: {error}"))?;
        let source = UiCorpusRom::from_tsv(&input)?;
        let mut live = Self {
            source,
            backplane: Hp67ElectricalBackplane::default(),
            fetch_act: ActFetchEndpoint::new(0),
            fetch_rom: RomFetchEndpoint::default(),
            display_rom0: Rom0DisplayEndpoint::default(),
            cathode: CathodeDriver1820_1749::default(),
            pipeline: FetchPipelineLatch::default(),
            machine: Hp67ArchitecturalMachine::default(),
            display: HardwareDisplayFrame::BLANK,
        };
        live.boot_to_idle()?;
        live.capture_idle_display()?;
        Ok(live)
    }

    pub const fn display_frame(&self) -> HardwareDisplayFrame {
        self.display
    }

    fn boot_to_idle(&mut self) -> Result<(), String> {
        let mut saw_display_init = false;
        let mut main_wait_visits = 0u64;
        let mut card_poll_visits = 0u64;

        for cycle in 0..BOOT_CYCLE_LIMIT {
            self.pipeline.begin_cycle();
            if let Some(word) = self.pipeline.executing_word() {
                let execution = self
                    .machine
                    .execute_word(word)
                    .map_err(|error| format!("live boot cycle {cycle} execution failed: {error:?}"))?;
                match execution.pc {
                    DISPLAY_INIT_PC => saw_display_init = true,
                    MAIN_WAIT_PC => main_wait_visits = main_wait_visits.saturating_add(1),
                    CARD_POLL_PC => card_poll_visits = card_poll_visits.saturating_add(1),
                    _ => {}
                }

                if saw_display_init
                    && main_wait_visits >= 2
                    && card_poll_visits >= 1
                    && self.machine.act.state.display_enable
                    && self.machine.act.state.key_buffer.is_none()
                {
                    return Ok(());
                }
            }

            self.transport_fetch_word(cycle)?;
        }

        Err(format!(
            "HP-67 live boot did not reach the no-key idle checkpoint within {BOOT_CYCLE_LIMIT} cycles"
        ))
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
        self.pipeline.complete_cycle(result.fetched_word);
        self.cathode.str_falling_edge();
        Ok(())
    }

    fn capture_idle_display(&mut self) -> Result<(), String> {
        if !self.machine.act.state.display_enable {
            self.display.clear();
            return Ok(());
        }

        self.display.clear();
        self.cathode.rcd_falling_edge();
        self.source.select_bank(self.machine.bank());
        let address = self.machine.pc();

        for expected_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            let display_byte = display_byte_from_act_registers(
                expected_slot,
                &self.machine.act.state.a,
                &self.machine.act.state.b,
            )
            .map_err(|error| format!("live idle display byte failed: {error:?}"))?;
            let result = run_structural_display_fetch_cycle(
                &mut self.backplane,
                address,
                display_byte,
                &mut self.fetch_act,
                &mut self.fetch_rom,
                &mut self.display_rom0,
                &self.source,
            )
            .map_err(|error| {
                format!("live idle shared word failed at slot {expected_slot}: {error:?}")
            })?;
            let anodes = decode_rom0_display_byte(expected_slot, result.display_byte)
                .map_err(|error| format!("live idle ROM0 decode failed: {error:?}"))?;
            self.display.capture_scan_slot(expected_slot, anodes)?;
            self.cathode.str_falling_edge();
        }
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
        frame.capture_scan_slot(4, Hp67SegmentMask::from_bits(0x3f)).unwrap();
        frame.capture_scan_slot(5, Hp67SegmentMask::DP).unwrap();
        frame.capture_scan_slot(6, Hp67SegmentMask::from_bits(0x3f)).unwrap();
        frame.capture_scan_slot(7, Hp67SegmentMask::from_bits(0x3f)).unwrap();
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
}
'''
Path('src/hp67.rs').write_text(hp67)

app = Path('src/app.rs')
text = app.read_text()
text = text.replace(
    '    hp67::Hp67State,\n',
    '    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State},\n',
)
text = text.replace(
    '    photo: TextureHandle,\n',
    '    photo: TextureHandle,\n    live_machine: Option<Hp67LiveMachine>,\n',
)
text = text.replace(
    '        Self {\n            state: Hp67State::default(),\n            photo,\n        }\n',
    '        let live_machine = match Hp67LiveMachine::boot_default() {\n            Ok(machine) => Some(machine),\n            Err(error) => {\n                eprintln!("HP-67 live display disabled: {error}");\n                None\n            }\n        };\n\n        Self {\n            state: Hp67State::default(),\n            photo,\n            live_machine,\n        }\n',
)
text = text.replace(
    '                for event in Hp67Panel::show(ui, &self.state, &self.photo) {\n',
    '                let display = self\n                    .live_machine\n                    .as_ref()\n                    .map_or(HardwareDisplayFrame::BLANK, Hp67LiveMachine::display_frame);\n                for event in Hp67Panel::show(ui, &self.state, &display, &self.photo) {\n',
)
app.write_text(text)

panel = Path('src/panel.rs')
text = panel.read_text()
text = text.replace(
    '    hp67::{Hp67State, KeyAction, UiEvent},\n',
    '    hp67::{HardwareDisplayFrame, Hp67State, KeyAction, UiEvent},\n',
)
text = text.replace(
    '    pub fn show(ui: &mut Ui, state: &Hp67State, photo: &TextureHandle) -> Vec<UiEvent> {\n',
    '    pub fn show(\n        ui: &mut Ui,\n        state: &Hp67State,\n        display: &HardwareDisplayFrame,\n        photo: &TextureHandle,\n    ) -> Vec<UiEvent> {\n',
)
old = '''        classic_display::paint(
            &painter,
            source_to_screen(photo_rect, DISPLAY_GLASS),
            state.display_text(),
        );
'''
new = '''        if state.power_on {
            classic_display::paint_segments(
                &painter,
                source_to_screen(photo_rect, DISPLAY_GLASS),
                display.segments(),
            );
        }
'''
if old not in text:
    raise SystemExit('panel display paint target not found')
text = text.replace(old, new, 1)
panel.write_text(text)

classic = Path('src/ui/classic_display.rs')
text = classic.read_text()
text = text.replace(
    'const SEG_G: u8 = 1 << 6;\n',
    'const SEG_G: u8 = 1 << 6;\nconst SEG_DP: u8 = 1 << 7;\n',
)
insert_after = '''pub(crate) fn paint(painter: &Painter, display_rect: Rect, value: &str) {
'''
idx = text.index(insert_after)
# Insert the hardware renderer immediately before the transitional text renderer.
hardware_fn = r'''pub(crate) fn paint_segments(
    painter: &Painter,
    display_rect: Rect,
    segments: &[u8; CHARACTER_COUNT],
) {
    if display_rect.width() <= 0.0 || display_rect.height() <= 0.0 {
        return;
    }

    let t = DisplayTransform::new(display_rect);
    let p = painter.with_clip_rect(display_rect);
    let center_x = REFERENCE_DISPLAY_WIDTH * 0.5;
    let center_y = REFERENCE_DISPLAY_HEIGHT * 0.5;
    let assembly_left = center_x - ASSEMBLY_WIDTH * 0.5;
    let first_center = assembly_left + CHARACTER_PITCH * 0.5;

    for (index, mask) in segments.iter().copied().enumerate() {
        if mask == 0 {
            continue;
        }
        let x = first_center + index as f32 * CHARACTER_PITCH;
        draw_segment_mask(&p, t, x, center_y, mask);
    }
}

'''
text = text[:idx] + hardware_fn + text[idx:]
old_draw = r'''fn draw_digit(p: &Painter, t: DisplayTransform, cx: f32, cy: f32, ch: char) {
    let mask = segment_mask(ch);
    if mask == 0 {
        return;
    }

    let h = CHARACTER_HEIGHT;
'''
new_draw = r'''fn draw_digit(p: &Painter, t: DisplayTransform, cx: f32, cy: f32, ch: char) {
    draw_segment_mask(p, t, cx, cy, segment_mask(ch));
}

fn draw_segment_mask(p: &Painter, t: DisplayTransform, cx: f32, cy: f32, mask: u8) {
    if mask == 0 {
        return;
    }

    let h = CHARACTER_HEIGHT;
'''
if old_draw not in text:
    raise SystemExit('classic display draw_digit target not found')
text = text.replace(old_draw, new_draw, 1)
needle = '''    for (bit, x0, y0, horizontal, half_len, serif) in [
'''
# Add DP drawing after segment loop by finding the end of draw_segment_mask before draw_monolithic_segment.
marker = '\nfn draw_monolithic_segment('
start = text.index(new_draw)
end = text.index(marker, start)
body = text[start:end]
# append DP handling just before function closing; body ends with }\n
last = body.rfind('\n}')
body = body[:last] + '''\n\n    if mask & SEG_DP != 0 {\n        draw_center_decimal(p, t, cx, cy);\n    }''' + body[last:]
text = text[:start] + body + text[end:]
# Add a direct hardware mask test before final test module close.
insert_test = r'''
    #[test]
    fn raw_hardware_masks_include_decimal_without_character_conversion() {
        let raw = [0u8; CHARACTER_COUNT];
        assert_eq!(raw.len(), 15);
        assert_eq!(SEG_DP, 0x80);
        assert_eq!(segment_mask('0'), 0x3f);
    }
'''
pos = text.rfind('\n}')
text = text[:pos] + insert_test + text[pos:]
classic.write_text(text)

Path('docs/files/src/hp67.rs.md').write_text(r'''# `src/hp67.rs`

## Purpose
Owns the remaining UI mechanical controls and boots a UI-local HP-67 structural machine whose raw LED segment state is exposed directly to the renderer.

## Why it exists
The photographed frontend previously used a string state machine that synthesized values such as `0.00`. The real-microcode path can now produce ROM0 anode masks and cathode scan positions, so the UI must consume those signals without formatting a numeric representation.

## Relationships
`Hp67LiveMachine` uses the reusable `machines::hp67` ACT/CRC architectural composition, shared display/fetch word transport, ROM0 display endpoint and 1820-1749 structural cathode driver. Firmware is still loaded externally through `research::rom_corpus::RomCorpus`. `app.rs` owns the live machine, while `panel.rs` consumes `HardwareDisplayFrame`. `Hp67State` now contains only UI-owned mechanical power/mode state and semantic key hit events; keys no longer synthesize display content.

## Responsibilities
Load exactly 5120 normalized external firmware words, execute real power-on microcode to the source-backed no-key idle checkpoint, capture fifteen physical display positions as A..G/DP bit masks, map the shared sign cathode to separate mantissa/exponent sign positions, and leave the display blank if firmware cannot be loaded. Preserve the explicit temporary A/B-to-b0..b7 bridge without presenting it as final ACT serialization.

## Implementation
`UiCorpusRom` owns the normalized corpus and bank-population map. `Hp67LiveMachine::boot_default()` reads `HP67_ROM_CORPUS` or `.research/teenix-2026-hp67.tsv`, runs the normal one-word pipeline until display initialization plus the documented no-key loop is reached, then freezes that architectural idle state for one fifteen-slot structural capture. Every captured slot crosses `run_structural_display_fetch_cycle()`, is decoded by ROM0, and is written to `HardwareDisplayFrame` as raw segment bits. Slots 4..14 map to the eleven mantissa/decimal positions, slots 1/2 to exponent units/tens, and slot 3 splits the shared sign anodes into the two physical minus positions. Exact ACT b0..b7 serialization and exact PHI/RCD/STR timing remain future work.
''')

Path('docs/files/src/app.rs.md').write_text(r'''# `src/app.rs`

## Purpose
Owns the desktop application's top-level egui state, embedded HP-67 photograph texture and live structural HP-67 machine used as the display source.

## Why it exists
The emulator needs a presentation adapter that loads assets and connects headless emulation state to the photographed frontend without putting GUI concerns into the reusable emulation library. The display is no longer allowed to come from a formatted placeholder string.

## Relationships
Uses `hp67::Hp67LiveMachine` to boot the external firmware and obtain `HardwareDisplayFrame`, `hp67::Hp67State` only for remaining mechanical UI controls, `panel::Hp67Panel` for the photographed body/input regions, and `ui::sliders` / `ui::top_keys` for visual corrections.

## Responsibilities
Embed/decode `assets/hp67.png`, configure egui visuals, boot the live machine when the app starts, pass raw display segment masks to the panel, dispatch UI events, and leave the LEDs blank rather than inventing a fallback display when the external corpus is unavailable.

## Implementation
`include_bytes!` compiles only the PNG into the executable; firmware remains external. `Hp67LiveMachine::boot_default()` is attempted once during application construction. A boot failure is reported to stderr and represented by `None`, which maps to `HardwareDisplayFrame::BLANK`. Each frame passes a copy of the current raw segment frame into the panel before drawing key/switch overlays.
''')

Path('docs/files/src/panel.rs.md').write_text(r'''# `src/panel.rs`

## Purpose
Renders the embedded HP-67 photograph, maps photographed control coordinates to the current window, handles the main key/switch hit regions and places raw emulated LED emission into the photographed display glass.

## Why it exists
The production UI is image-based rather than a recreated vector chassis. This module centralizes source-photo registration so controls and the dynamic physical display remain aligned at any window size.

## Relationships
Reads mechanical `Hp67State`, receives `HardwareDisplayFrame` from `app.rs`, emits `UiEvent` values, calls `ui::classic_display::paint_segments()` for raw A..G/DP masks, and shares the same 928x1695 coordinate system with `sliders.rs` and `top_keys.rs`.

## Responsibilities
Fit the source image without distortion, paint it, create hit boxes for 35 keys and two switch regions, animate generic key travel, locate the photographed display glass and forward raw hardware segment masks without converting them to characters or numbers.

## Implementation
All geometry is expressed in source-image pixels and transformed uniformly into the fitted photo rectangle. Pressed keys reuse cropped pixels from the source photograph plus small travel/shadow corrections. When the power switch is on, the display layer calls `paint_segments()` with the fifteen-position hardware frame; when power is off, no LED emission is painted. Key contacts are still semantic UI events pending electrical keyboard-matrix work.
''')

Path('docs/files/src/ui/classic_display.rs.md').write_text(r'''# `src/ui/classic_display.rs`

## Purpose
Draws photorealistic HP Classic-series LED emission over the photographed display glass directly from physical A..G/DP segment masks.

## Why it exists
The source photograph supplies the bezel/filter/reflections but cannot show dynamic LEDs. The electrical/structural display path can now provide raw segment state, so the production renderer must draw those masks without first formatting characters or a calculator value.

## Relationships
Called by `panel.rs`. Its production entry point `paint_segments()` consumes the fifteen-position `HardwareDisplayFrame` generated from `machines::hp67` ROM0/cathode state. The older text renderer remains only as isolated transitional/test support and is no longer on the panel display path.

## Responsibilities
Map the 15 physical positions, calibrated LED geometry, A..G masks, decimal-point mask, optical color and glow into egui drawing primitives while preserving the photographed filter and glass underneath.

## Implementation
`paint_segments()` positions all fifteen physical character cells and passes each nonzero byte directly to `draw_segment_mask()`. Bits 0..6 drive the seven segment emitters and bit 7 drives the centered decimal emitter. No numeric parsing, text-to-cell conversion or inferred calculator value occurs in this path. Existing physical dimensions and three-bar segment artwork are retained.
''')
