from pathlib import Path

hp67 = Path("src/hp67.rs")
s = hp67.read_text()

s = s.replace("const RESET_DISPLAY_CODE: u8 = 0x00;\n", "")

start = s.find("fn power_on_reset_display_frame() -> Result<HardwareDisplayFrame, String> {")
if start < 0:
    raise SystemExit("power_on_reset_display_frame start not found")
end_marker = "\n\nimpl Hp67LiveMachine {"
end = s.find(end_marker, start)
if end < 0:
    raise SystemExit("power_on_reset_display_frame end not found")
s = s[:start] + s[end + 2:]

s = s.replace(
    "        let source = UiCorpusRom::from_tsv(&input)?;\n        let display = power_on_reset_display_frame()?;\n        Ok(Self {",
    "        let source = UiCorpusRom::from_tsv(&input)?;\n        Ok(Self {",
)
s = s.replace(
    "            machine: Hp67ArchitecturalMachine::default(),\n            display,\n            phase: LiveBootPhase::ResetHold,",
    "            machine: Hp67ArchitecturalMachine::default(),\n            display: HardwareDisplayFrame::BLANK,\n            phase: LiveBootPhase::ResetHold,",
)
s = s.replace(
    "        self.display = power_on_reset_display_frame()?;",
    "        self.display = HardwareDisplayFrame::BLANK;",
)

old_transport = '''        if self.display_control_seen {
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
'''
new_transport = '''        // Before firmware executes its first display-control instruction, do not
        // use the architectural `display_enable` default as an electrical inhibit.
        // A real HP-67 visibly emits during this interval, while the semantic
        // model's reset value is not hardware evidence.  The emitted CONTENT is
        // still entirely derived from ACT A/B -> IS -> ROM0 for this word.
        if !self.display_control_seen || self.machine.act.state.display_enable {
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
'''
if old_transport not in s:
    raise SystemExit("old transport display gate not found")
s = s.replace(old_transport, new_transport)

old_test = '''    #[test]
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
'''
new_test = '''    #[test]
    fn reset_act_registers_naturally_encode_zero_display_bytes() {
        let machine = Hp67ArchitecturalMachine::default();
        assert!(!machine.act.state.display_enable);
        for scan_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            let code = display_byte_from_act_registers(
                scan_slot,
                &machine.act.state.a,
                &machine.act.state.b,
            )
            .unwrap();
            assert_eq!(code, 0x00);
        }
    }
'''
if old_test not in s:
    raise SystemExit("synthetic reset-frame test not found")
s = s.replace(old_test, new_test)

hp67.write_text(s)

Path("docs/research/HP67_POWER_ON_DISPLAY.md").write_text('''# HP-67 power-on display transient

## Scope

This note records the evidence and implementation boundary for the visible HP-67 startup display. It is intentionally separate from the final PHI/RCD/STR electrical scheduler. No startup digit pattern is injected into the renderer or ROM0 decoder.

## Direct HP-67 timing evidence

Tony Nixon's *Notes on HP's Classic Calculators*, HP-67 section, reports from physical logic-analyser/scope captures:

- page 73: signals begin stabilizing roughly 330 us after switch-on; SYNC becomes active and ROM instruction fetching begins roughly 35 ms after switch-on;
- page 73: while reset is active, the IS bus follows PHI2 and SYNC is high; this is not enough evidence to choose a ROM0 sampling edge or fabricate a pre-SYNC display byte;
- page 76: one 56-bit HP-67 instruction cycle takes about 320 us; fifteen STR pulses form a complete display refresh of about 4.8 ms;
- pages 76-77: ROM0 display data occupies IS bits 0..7 and code `0x00` decodes as digit `0`; the shared sign slot uses bits 0/1 rather than the normal seven-segment digit decoder.

Source: https://literature.hpcalc.org/community/classic-notes.pdf

Sydney Smith's HP-67 startup trace independently shows the firmware constructing `B=03000000000022` (display `0.00`) immediately before the startup path executes `display off` and `display toggle` around octal addresses `00162` and `00163`.

Source: https://www.sydneysmith.com/wordpress/1190/hp67-flags/

The HP-97 service manual is useful corroboration for the same ACT generation: its power-on preset circuit exists specifically to reset the ACT into a defined logic state. It does not, however, document a literal startup display byte, so no such byte is assumed here.

## Emulator behavior

`ResetHold` now starts with a blank `HardwareDisplayFrame`. During the measured pre-SYNC interval the emulator does not claim a display pattern because the currently reviewed evidence does not fix the PHI-relative ROM0 sampling behavior while IS follows PHI2.

Once real firmware words begin, every visible slot is produced only by the normal structural path: current ACT A/B state -> `display_byte_from_act_registers()` -> resolved IS b0..b7 -> ROM0 decode -> current cathode slot. There is no `RESET_DISPLAY_CODE`, no prebuilt zero frame and no timer that changes the display to zeroes. The existing architectural reset state has A=B=0, so its first transported display bytes are naturally `0x00`; if the real startup zero row appears, it therefore comes from machine state and the normal decoder.

The architectural `display_enable` boolean is a semantic instruction-boundary convenience and its reset value is not accepted as evidence of the physical pre-initialization output gate. Until firmware executes its first explicit display-control instruction, structural display words are therefore allowed to reach ROM0. After the first `display off` or `display toggle`, the firmware-controlled `display_enable` state is honored normally. This models the observed fact that the real display emits before software has completed display initialization without hardcoding what it emits.

## Fidelity rule

If future pin-accurate ACT/ROM0 work causes the startup zero row to disappear, do not restore it with a special-case frame. Investigate the missing reset, serializer, SYNC, RCD/STR or display-gate behavior and update the electrical model only when supported by reviewed evidence.

## Limits

The first-valid-SYNC/ignored-SYNC detail seen in the power-on capture is not yet promoted into exact reset sequencing. Nor do the coarse durations define PHI1/PHI2 pulse widths, edge-relative IS timing, exact RCD/STR overlap, LED current decay, analogue persistence or the physical reset state of an internal display-enable latch. Those remain separate hardware-fidelity milestones.
''')

Path("docs/files/src/hp67.rs.md").write_text('''# `src/hp67.rs`

## Purpose
Owns the remaining UI mechanical controls and a UI-local HP-67 structural machine whose startup LED state is derived from machine state and real firmware rather than a fabricated power-on frame.

## Why it exists
The photographed frontend must expose hardware-derived startup behavior. Physical HP-67 evidence shows a reset interval before valid SYNC activity and the real calculator visibly emits a row of zeroes during startup, but the evidence does not justify injecting display byte `0x00` during reset. The implementation must let that pattern emerge from ACT/IS/ROM0 state or expose the missing hardware model.

## Relationships
`Hp67LiveMachine` uses the reusable `machines::hp67` ACT/CRC architectural composition, shared display/fetch word transport, ROM0 decoder, 1820-1749 structural cathode driver and coarse observed timing constants from `machines::hp67::timing`. Firmware remains external through `RomCorpus`. `app.rs` advances this machine with elapsed wall-clock time and `panel.rs` consumes `HardwareDisplayFrame`.

## Responsibilities
Load exactly 5120 normalized external firmware words; keep the display unclaimed during the pre-SYNC reset interval; after firmware fetch begins, update each physical display slot only from current ACT state transported across the shared structural word; detect the firmware's real display-off/toggle instructions; and never synthesize a startup number or segment frame. Preserve the temporary A/B-to-b0..b7 bridge explicitly.

## Implementation
`power_on_default()` starts in `ResetHold` with `HardwareDisplayFrame::BLANK`. After `HP67_OBSERVED_POWER_ON_SYNC_DELAY_US`, `advance()` executes structural cycles at `HP67_OBSERVED_WORD_TIME_US`. Before the first explicit display-control opcode, the semantic `display_enable` reset default is not used as a physical output inhibit; each word's A/B-derived byte is resolved on IS and decoded by ROM0 into its current cathode slot. The default ACT reset registers A=B=0 therefore naturally generate `0x00` display bytes without a reset-display constant. Once firmware executes `display off` or `display toggle`, its `display_enable` state governs emission. The existing no-key checkpoint marks `Idle`, and `reset_power_on()` repeats the same machine reset rather than installing a display frame.
''')

Path("docs/files/src/app.rs.md").write_text('''# `src/app.rs`

## Purpose
Owns the desktop egui state, embedded HP-67 photograph and wall-clock scheduling adapter for the live structural HP-67 display machine.

## Why it exists
The emulator must expose startup over time rather than constructing the machine at its final idle state. GUI frame timing belongs here, outside the reusable electrical and architectural modules, and must not fabricate display contents.

## Relationships
Uses `Hp67LiveMachine::power_on_default()` and `advance()` for the external firmware, `Hp67State` for mechanical controls, `Hp67Panel` for input/body rendering and the photo overlay modules for visual corrections. `HardwareDisplayFrame` remains the only display payload passed to the panel.

## Responsibilities
Decode the embedded photograph, load the external firmware source, measure elapsed host time between egui updates, advance HP-67 startup only while power is on, replay machine reset when the switch goes OFF->ON, and request repaints while boot is progressing. If the external corpus or runtime fails, leave LED emission blank rather than synthesize a calculator value.

## Implementation
The first UI frame advances by zero elapsed time and is blank because the pre-SYNC ROM0 sampling state is not yet established by reviewed evidence. Subsequent frames pass monotonic `Instant` deltas into `Hp67LiveMachine`. Once firmware transport starts, visible slots are filled by the structural machine itself. While `is_booting()` is true, egui requests another repaint after approximately one measured HP-67 display refresh (4.8 ms). Power-off stops advancement and suppresses LED painting in `panel.rs`; power-on resets the live machine and its UI clock so the natural transient repeats.
''')
