from pathlib import Path

# ---- timing.rs: add source-backed coarse HP-67 wall-clock observations ----
p = Path('src/machines/hp67/timing.rs')
s = p.read_text()
old = '''/// Number of serial bit times in one HP-67 machine word.\npub const BITS_PER_WORD: u8 = BITS_PER_DIGIT * DIGITS_PER_WORD;\n'''
new = '''/// Number of serial bit times in one HP-67 machine word.\npub const BITS_PER_WORD: u8 = BITS_PER_DIGIT * DIGITS_PER_WORD;\n\n/// Approximate whole-word duration measured on a physical HP-67 logic-analyser trace.\n///\n/// Tony Nixon's HP-67 capture reports 320 us for the complete 56-bit instruction\n/// cycle. This is a coarse observed duration, not a claim about exact PHI pulse\n/// widths or launch/sample edges.\npub const HP67_OBSERVED_WORD_TIME_US: u64 = 320;\n/// Approximate full 15-STR display refresh measured on the same physical HP-67.\npub const HP67_OBSERVED_DISPLAY_REFRESH_US: u64 = 4_800;\n/// Approximate delay from switch-on until SYNC becomes active on the measured HP-67.\npub const HP67_OBSERVED_POWER_ON_SYNC_DELAY_US: u64 = 35_000;\n/// Approximate delay after switch-on at which the measured power-on signals start stabilizing.\npub const HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US: u64 = 330;\n'''
if old not in s:
    raise SystemExit('timing word geometry anchor not found')
s = s.replace(old, new, 1)
old_test = '''    fn word_geometry_is_fourteen_four_bit_digits() {\n        assert_eq!(BITS_PER_DIGIT, 4);\n        assert_eq!(DIGITS_PER_WORD, 14);\n        assert_eq!(BITS_PER_WORD, 56);\n    }\n'''
new_test = old_test + '''\n    #[test]\n    fn coarse_hp67_power_on_and_refresh_times_match_measured_trace() {\n        assert_eq!(HP67_OBSERVED_WORD_TIME_US, 320);\n        assert_eq!(HP67_OBSERVED_DISPLAY_REFRESH_US, 4_800);\n        assert_eq!(HP67_OBSERVED_POWER_ON_SYNC_DELAY_US, 35_000);\n        assert_eq!(HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US, 330);\n        assert_eq!(\n            HP67_OBSERVED_DISPLAY_REFRESH_US,\n            HP67_OBSERVED_WORD_TIME_US * 15\n        );\n    }\n'''
if old_test not in s:
    raise SystemExit('timing test anchor not found')
s = s.replace(old_test, new_test, 1)
p.write_text(s)

# ---- mod.rs: export coarse observations ----
p = Path('src/machines/hp67/mod.rs')
s = p.read_text()
old = '''    DISPLAY_DATA_LAST_BIT, DISPLAY_STR_BIT, ROM_ADDRESS_BITS, ROM_ADDRESS_FIRST_BIT,\n    ROM_ADDRESS_LAST_BIT, ROM_WORD_BITS, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT,\n};\n'''
new = '''    DISPLAY_DATA_LAST_BIT, DISPLAY_STR_BIT, HP67_OBSERVED_DISPLAY_REFRESH_US,\n    HP67_OBSERVED_POWER_ON_SIGNAL_STABILIZE_US, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,\n    HP67_OBSERVED_WORD_TIME_US, ROM_ADDRESS_BITS, ROM_ADDRESS_FIRST_BIT, ROM_ADDRESS_LAST_BIT,\n    ROM_WORD_BITS, ROM_WORD_FIRST_BIT, ROM_WORD_LAST_BIT,\n};\n'''
if old not in s:
    raise SystemExit('mod timing export anchor not found')
s = s.replace(old, new, 1)
p.write_text(s)

# ---- hp67.rs: replace instant boot with timed power-on state machine ----
p = Path('src/hp67.rs')
s = p.read_text()
s = s.replace('use std::{cell::Cell, env, fs};', 'use std::{cell::Cell, env, fs, time::Duration};', 1)
old_import = '''        decode_rom0_display_byte, display_byte_from_act_registers,\n        run_structural_display_fetch_cycle, ActFetchEndpoint, CathodeDriver1820_1749,\n        FetchPipelineLatch, Hp67ArchitecturalMachine, Hp67ElectricalBackplane, Hp67RomWordSource,\n        Hp67SegmentMask, Rom0DisplayEndpoint, RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS,\n'''
new_import = '''        decode_rom0_display_byte, display_byte_from_act_registers,\n        run_structural_display_fetch_cycle, ActFetchEndpoint, ActOperation, CathodeDriver1820_1749,\n        FetchPipelineLatch, Hp67ArchitecturalMachine, Hp67ArchitecturalOperation,\n        Hp67ElectricalBackplane, Hp67RomWordSource, Hp67SegmentMask, Rom0DisplayEndpoint,\n        RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS, HP67_OBSERVED_POWER_ON_SYNC_DELAY_US,\n        HP67_OBSERVED_WORD_TIME_US,\n'''
if old_import not in s:
    raise SystemExit('hp67 import anchor not found')
s = s.replace(old_import, new_import, 1)
old_consts = '''const CARD_POLL_PC: u16 = 0o0206;\nconst BOOT_CYCLE_LIMIT: u64 = 2_000;\n'''
new_consts = '''const CARD_POLL_PC: u16 = 0o0206;\nconst BOOT_CYCLE_LIMIT: u64 = 2_000;\nconst RESET_DISPLAY_CODE: u8 = 0x00;\n\n#[derive(Debug, Clone, Copy, PartialEq, Eq)]\nenum LiveBootPhase {\n    ResetHold,\n    Firmware,\n    Idle,\n}\n'''
if old_consts not in s:
    raise SystemExit('hp67 constants anchor not found')
s = s.replace(old_consts, new_consts, 1)
old_fields = '''    pipeline: FetchPipelineLatch,\n    machine: Hp67ArchitecturalMachine,\n    display: HardwareDisplayFrame,\n}\n\nimpl Hp67LiveMachine {\n'''
new_fields = '''    pipeline: FetchPipelineLatch,\n    machine: Hp67ArchitecturalMachine,\n    display: HardwareDisplayFrame,\n    phase: LiveBootPhase,\n    pending_us: u64,\n    boot_cycle: u64,\n    saw_display_init: bool,\n    main_wait_visits: u64,\n    card_poll_visits: u64,\n    display_control_seen: bool,\n}\n\nfn power_on_reset_display_frame() -> Result<HardwareDisplayFrame, String> {\n    let mut frame = HardwareDisplayFrame::BLANK;\n    for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {\n        let anodes = decode_rom0_display_byte(scan_slot, RESET_DISPLAY_CODE).map_err(|error| {\n            format!(\n                "power-on reset ROM0 decode failed at slot {scan_slot}: {error:?}"\n            )\n        })?;\n        frame.capture_scan_slot(scan_slot, anodes)?;\n    }\n    Ok(frame)\n}\n\nimpl Hp67LiveMachine {\n'''
if old_fields not in s:
    raise SystemExit('hp67 struct anchor not found')
s = s.replace(old_fields, new_fields, 1)
start = s.index('impl Hp67LiveMachine {')
end = s.index('\n#[cfg(test)]', start)
new_impl = r'''impl Hp67LiveMachine {
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

        while self.phase == LiveBootPhase::Firmware
            && self.pending_us >= HP67_OBSERVED_WORD_TIME_US
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
                let anodes = decode_rom0_display_byte(scan_slot, result.display_byte).map_err(|error| {
                    format!("live cycle {cycle} ROM0 decode failed at slot {scan_slot}: {error:?}")
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
'''
s = s[:start] + new_impl + s[end:]
old_test_end = '''    fn shared_sign_anodes_are_split_into_two_physical_minus_positions() {\n        let mut frame = HardwareDisplayFrame::BLANK;\n        frame\n            .capture_scan_slot(\n                3,\n                Hp67SegmentMask::from_bits(Hp67SegmentMask::E.bits() | Hp67SegmentMask::G.bits()),\n            )\n            .unwrap();\n        assert_eq!(frame.segments()[0], Hp67SegmentMask::G.bits());\n        assert_eq!(frame.segments()[12], Hp67SegmentMask::G.bits());\n    }\n'''
new_test_end = old_test_end + '''\n    #[test]\n    fn reset_bus_zero_code_produces_observed_all_zero_power_on_pattern() {\n        let frame = power_on_reset_display_frame().unwrap();\n        assert_eq!(frame.segments()[0], Hp67SegmentMask::G.bits());\n        for index in 1..=11 {\n            assert_eq!(frame.segments()[index], 0x3f);\n        }\n        assert_eq!(frame.segments()[12], 0);\n        assert_eq!(frame.segments()[13], 0x3f);\n        assert_eq!(frame.segments()[14], 0x3f);\n    }\n'''
if old_test_end not in s:
    raise SystemExit('hp67 test tail anchor not found')
s = s.replace(old_test_end, new_test_end, 1)
p.write_text(s)

# ---- app.rs: advance the machine in wall-clock time and replay startup on power-on ----
p = Path('src/app.rs')
s = p.read_text()
s = 'use std::time::{Duration, Instant};\n\n' + s
s = s.replace(
    '    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State},',
    '    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State, UiEvent},',
    1,
)
s = s.replace(
    '''    photo: TextureHandle,\n    live_machine: Option<Hp67LiveMachine>,\n}''',
    '''    photo: TextureHandle,\n    live_machine: Option<Hp67LiveMachine>,\n    last_live_tick: Option<Instant>,\n}''',
    1,
)
s = s.replace('Hp67LiveMachine::boot_default()', 'Hp67LiveMachine::power_on_default()', 1)
s = s.replace(
    '''            photo,\n            live_machine,\n        }''',
    '''            photo,\n            live_machine,\n            last_live_tick: None,\n        }''',
    1,
)
old_update_start = '''    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {\n        egui::CentralPanel::default()'''
new_update_start = '''    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {\n        let now = Instant::now();\n        let elapsed = self\n            .last_live_tick\n            .replace(now)\n            .map_or(Duration::ZERO, |previous| now.saturating_duration_since(previous));\n\n        let mut live_error = None;\n        if self.state.power_on {\n            if let Some(machine) = self.live_machine.as_mut() {\n                if let Err(error) = machine.advance(elapsed) {\n                    live_error = Some(error);\n                }\n            }\n        }\n        if let Some(error) = live_error {\n            eprintln!("HP-67 live display disabled: {error}");\n            self.live_machine = None;\n        }\n\n        egui::CentralPanel::default()'''
if old_update_start not in s:
    raise SystemExit('app update start anchor not found')
s = s.replace(old_update_start, new_update_start, 1)
old_events = '''                for event in Hp67Panel::show(ui, &self.state, &display, &self.photo) {\n                    self.state.handle(event);\n                }'''
new_events = '''                for event in Hp67Panel::show(ui, &self.state, &display, &self.photo) {\n                    let was_power_on = self.state.power_on;\n                    self.state.handle(event);\n                    if matches!(event, UiEvent::TogglePower) {\n                        self.last_live_tick = None;\n                        if !was_power_on && self.state.power_on {\n                            let reset_error = self\n                                .live_machine\n                                .as_mut()\n                                .and_then(|machine| machine.reset_power_on().err());\n                            if let Some(error) = reset_error {\n                                eprintln!("HP-67 live power-on reset failed: {error}");\n                                self.live_machine = None;\n                            }\n                        }\n                    }\n                }'''
if old_events not in s:
    raise SystemExit('app event loop anchor not found')
s = s.replace(old_events, new_events, 1)
old_repaint = '''        if ctx.input(|i| i.pointer.any_down()) {\n            ctx.request_repaint();\n        }\n'''
new_repaint = '''        if self.state.power_on\n            && self\n                .live_machine\n                .as_ref()\n                .is_some_and(Hp67LiveMachine::is_booting)\n        {\n            // One HP-67 display refresh is about 4.8 ms. Requesting another frame\n            // on that cadence makes the source-backed power-on transient visible\n            // without inventing extra display states.\n            ctx.request_repaint_after(Duration::from_micros(4_800));\n        }\n        if ctx.input(|i| i.pointer.any_down()) {\n            ctx.request_repaint();\n        }\n'''
if old_repaint not in s:
    raise SystemExit('app repaint anchor not found')
s = s.replace(old_repaint, new_repaint, 1)
p.write_text(s)

# ---- docs companions ----
Path('docs/files/src/hp67.rs.md').write_text('''# `src/hp67.rs`\n\n## Purpose\nOwns the remaining UI mechanical controls and a UI-local HP-67 structural machine whose raw LED state now progresses through the measured physical power-on sequence instead of jumping directly to idle.\n\n## Why it exists\nThe photographed frontend must expose hardware-derived startup behavior. A physical HP-67 capture shows a reset interval before valid SYNC activity, while ROM0 code `0x00` decodes as zero. The previous constructor executed all firmware synchronously to idle, hiding every startup display state.\n\n## Relationships\n`Hp67LiveMachine` uses the reusable `machines::hp67` ACT/CRC architectural composition, shared display/fetch word transport, ROM0 decoder, 1820-1749 structural cathode driver and the coarse observed timing constants from `machines::hp67::timing`. Firmware remains external through `RomCorpus`. `app.rs` advances this machine with elapsed wall-clock time and `panel.rs` consumes `HardwareDisplayFrame`.\n\n## Responsibilities\nLoad exactly 5120 normalized external firmware words; expose the ROM0-decoded reset display pattern before valid firmware fetch; wait the measured ~35 ms power-on SYNC delay; then execute one structural machine word per measured ~320 us and update only the scan slot actually transported in that word. Detect the firmware's real display-off/toggle instructions rather than replacing the reset pattern on an arbitrary UI timer. Preserve the temporary A/B-to-b0..b7 bridge explicitly.\n\n## Implementation\n`power_on_default()` starts in `ResetHold` with a frame produced by decoding display byte `0x00` through ROM0 for all fifteen scan slots. That yields zeroes in numeric positions and the source-defined shared-sign result, without formatting text. `advance()` accumulates elapsed time, crosses into `Firmware` after `HP67_OBSERVED_POWER_ON_SYNC_DELAY_US`, and executes structural cycles at `HP67_OBSERVED_WORD_TIME_US`. Until the firmware executes an explicit display-control special (`display off` or `display toggle`), the reset frame remains visible. After control is established, each combined display/fetch word updates its current physical slot from ROM0 anodes. The existing no-key checkpoint marks `Idle`. `reset_power_on()` replays the same state when the mechanical power switch is turned back on.\n''')

Path('docs/files/src/app.rs.md').write_text('''# `src/app.rs`\n\n## Purpose\nOwns the desktop egui state, embedded HP-67 photograph and wall-clock scheduling adapter for the live structural HP-67 display machine.\n\n## Why it exists\nThe emulator must show the physical power-on transient rather than constructing the machine at its final idle state. GUI frame timing belongs here, outside the reusable electrical and architectural modules.\n\n## Relationships\nUses `Hp67LiveMachine::power_on_default()` and `advance()` for the external firmware, `Hp67State` for mechanical controls, `Hp67Panel` for input/body rendering and the photo overlay modules for visual corrections. `HardwareDisplayFrame` remains the only display payload passed to the panel.\n\n## Responsibilities\nDecode the embedded photograph, load the external firmware source, measure elapsed host time between egui updates, advance the HP-67 startup only while power is on, replay the power-on state when the switch goes OFF->ON, and request repaints while boot is still progressing. If the external corpus or runtime fails, leave LED emission blank rather than synthesize a calculator value.\n\n## Implementation\nThe first UI frame advances by zero elapsed time so the reset display can actually be seen. Subsequent frames pass monotonic `Instant` deltas into `Hp67LiveMachine`. While that machine reports `is_booting()`, egui requests another repaint after approximately one measured HP-67 display refresh (4.8 ms). Power-off stops advancement and suppresses LED painting in `panel.rs`; power-on resets the live machine and its UI clock so the transient repeats.\n''')

Path('docs/files/src/machines/hp67/timing.rs.md').write_text('''# `src/machines/hp67/timing.rs`\n\n## Purpose\nDefines the HP-67 serial machine-word coordinate system, evidenced ROM0/IS windows, and coarse whole-machine timing observations that are directly visible in HP-67 logic-analyser captures.\n\n## Why it exists\nThe Woodstock datapath uses a 56-bit word made from fourteen 4-bit digit times. Tony Nixon's HP-67 captures fix display data at `b0..b7`, ROM address at `b16..b27`, ROM result at `b46..b55`, and also report coarse physical durations: about 320 us per 56-bit HP-67 word, about 4.8 ms for fifteen STR display slots, signals beginning to stabilize around 330 us after switch-on, and valid SYNC activity around 35 ms after switch-on.\n\n## Relationships\nUsed by the electrical backplane/fetch/display modules for serial coordinates and by `src/hp67.rs` only for coarse wall-clock pacing of the visible startup sequence. `docs/research/HP67_ISA_TIMING.md` and `docs/HARDWARE_SOURCES.md` record the evidence boundaries.\n\n## Responsibilities\nDefine 4 bits per digit, 14 digits per word and 56 bits per word; encode ROM0 display, address, ROM-return and SYNC windows; expose the coarse STR coordinate; and publish the observed HP-67 word/refresh/power-on durations without converting them into unsupported PHI edge claims.\n\n## Implementation\n`Hp67WordTiming` still advances through the temporary four-subphase PHI scaffold. The `HP67_OBSERVED_*_US` constants are separate measured whole-interval observations. They do not determine exact PHI1/PHI2 pulse widths, dead time, launch edges, sample edges, RCD placement or propagation delay; those remain intentionally open.\n''')

# ---- research note and source policy ----
Path('docs/research/HP67_POWER_ON_DISPLAY.md').write_text('''# HP-67 power-on display transient\n\n## Scope\n\nThis note records the evidence and implementation boundary for the visible HP-67 startup display. It is intentionally separate from the final PHI/RCD/STR electrical scheduler.\n\n## Direct HP-67 timing evidence\n\nTony Nixon's *Notes on HP's Classic Calculators*, HP-67 section, reports from physical logic-analyser/scope captures:\n\n- page 73: signals begin stabilizing roughly 330 us after switch-on; SYNC becomes active and ROM instruction fetching begins roughly 35 ms after switch-on;\n- page 76: one 56-bit HP-67 instruction cycle takes about 320 us; fifteen STR pulses form a complete display refresh of about 4.8 ms;\n- pages 76-77: ROM0 display data occupies IS bits 0..7 and code `0x00` decodes as digit `0`; the shared sign slot uses bits 0/1 rather than the normal seven-segment digit decoder.\n\nSource: https://literature.hpcalc.org/community/classic-notes.pdf\n\nSydney Smith's HP-67 startup trace independently shows the firmware constructing `B=03000000000022` (display `0.00`) immediately before the startup path executes `display off` and `display toggle` around octal addresses `00162` and `00163`.\n\nSource: https://www.sydneysmith.com/wordpress/1190/hp67-flags/\n\n## Emulator behavior\n\nBefore valid firmware fetch, the production UI now presents a reset-phase frame obtained by passing display code `0x00` through the existing ROM0 decoder for every structural scan slot. This is not a formatted number string. It naturally gives zeroes in digit-capable positions and the decoder-defined state of the shared sign slot.\n\nThe reset frame is retained during the measured ~35 ms pre-SYNC interval and while firmware runs until an explicit display-control instruction establishes software ownership. Firmware is paced at the measured coarse 320 us/word interval. After display control is established, each shared structural word updates only its actual current scan slot.\n\n## Limits\n\nThe first-valid-SYNC/ignored-SYNC detail seen in the power-on capture is not yet promoted into exact reset sequencing. Nor do these coarse durations define PHI1/PHI2 pulse widths, edge-relative IS timing, exact RCD/STR overlap, LED current decay, or analogue persistence. Those remain separate hardware-fidelity milestones.\n''')

p = Path('docs/HARDWARE_SOURCES.md')
s = p.read_text()
old = '''- Pages 64-66 establish exact instruction-fetch coordinates: the 12-bit ROM address is sent LSB-first during bit times `16..27`; the selected ROM returns its 10-bit word LSB-first during bit times `46..55`; a normal instruction has SYNC asserted over those final ten times, while after an `IF` the same 10-bit word arrives with SYNC low and is consumed as the implied-GOTO destination.\n- Page 74 records shared-bus behavior: IS is weakly/passively biased low and active participants pull it high rather than actively driving zero; only one device is intended to control the bus at a time.\n- Page 73 separately shows HP-67 power-on stabilization and the first valid SYNC activity. Use the bit windows and bus polarity as direct HP-67 evidence, while keeping exact launch/sample PHI edges and propagation delay open until the page-70 expanded waveforms are transcribed into a reviewed edge convention.\n'''
new = '''- Pages 64-66 establish exact instruction-fetch coordinates: the 12-bit ROM address is sent LSB-first during bit times `16..27`; the selected ROM returns its 10-bit word LSB-first during bit times `46..55`; a normal instruction has SYNC asserted over those final ten times, while after an `IF` the same 10-bit word arrives with SYNC low and is consumed as the implied-GOTO destination.\n- Page 73 shows HP-67 power-on behavior: signals start stabilizing at about 330 us and valid SYNC/fetch activity starts at about 35 ms after switch-on.\n- Page 74 records shared-bus behavior: IS is weakly/passively biased low and active participants pull it high rather than actively driving zero; only one device is intended to control the bus at a time.\n- Page 76 reports an observed HP-67 56-bit word time of about 320 us and a fifteen-STR display refresh of about 4.8 ms; pages 76-77 also anchor ROM0 display byte `0x00` as digit `0` and document the shared-sign decode. These whole-interval measurements may pace visible startup behavior, but they do not settle exact PHI launch/sample edges, RCD/STR overlap ordering or propagation delay.\n'''
if old not in s:
    raise SystemExit('hardware sources HP67 waveform anchor not found')
s = s.replace(old, new, 1)
p.write_text(s)
