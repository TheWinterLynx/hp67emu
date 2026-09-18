# M11 Fidelity Audit — 2026-09-18

## Scope

This audit reviews the HP-67 path from the last clean M10 firmware checkpoint through the UI keyboard integration and the current M11 keyboard/function matrix. The goal is to distinguish production behavior from test-only changes, identify circular tests, and state precisely which fidelity claims are proven versus still open.

## Production-change boundary

The M10 firmware baseline is commit `0768de27bf8a4bf10803ee320bfe0bea5d11f5ba`.

The UI keyboard integration was manually validated at commit `405dcf7a6446649a633d9090b9db4d8d7ffe65ee`: after building the release executable, the visible calculator accepted physical UI presses `1 ENTER 2 +` and displayed `3.00`.

Between those two points the only production Rust files changed were:

- `src/hp67.rs`: adds `Hp67Keyboard` to the live machine, maps UI key identities to `Hp67Key`, samples the held contact at instruction boundaries, and continues firmware execution after the boot-idle checkpoint.
- `src/panel.rs`: reports a held physical key contact instead of a semantic click action; also corrects EEX from the former accidental ENTER mapping to its own physical key.
- `src/app.rs`: applies the current held contact before advancing the live machine and keeps the powered machine advancing after boot.

From validated UI commit `405dcf7...` through the M11 branch, no production machine/UI behavior was changed. M11 changes are validator/documentation changes only, except that the existing M10 smoke harness was later hardened so its display observation is non-invasive.

## Keyboard path audit

The production path contains no host calculator operation:

`pointer held -> KeyAction identity -> Hp67Key -> Hp67Keyboard -> S15/key_buffer -> real firmware -> ACT/CRC -> ROM0/cathode display`.

`Hp67Keyboard::press()` closes one contact and latches its scan code. `release()` only opens the contact. `sample_into_act()` reasserts S15 while held and leaves firmware responsible for clearing S15 after release.

The ACT instructions used by firmware are modeled directly:

- `0020`: key code to low ROM address;
- `0120`: key code to A[2:1];
- `0220`: A[2:1] to low ROM address.

The shifted destinations are therefore firmware-selected. The M11 harness does not synthesize `07xx/06xx/05xx` targets.

## Independent 35-key code cross-check

The production `Hp67Key::scan_code()` table was independently cross-checked against the complete HP-67 keyboard definitions in `mike632t/x11-calc`, pinned at commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`, file `src/x11-calc-67.c`.

This reference is used only as an independent semantic oracle; it is not a runtime/build dependency and its name remains in Markdown in accordance with repository policy.

All 35 codes agree:

| Key | Octal code | Key | Octal code | Key | Octal code |
|---|---:|---|---:|---|---:|
| A | 0244 | B | 0243 | C | 0242 |
| D | 0241 | E | 0240 | Σ+ | 0224 |
| GTO | 0223 | DSP | 0222 | (i) | 0221 |
| SST | 0220 | f | 0024 | g | 0023 |
| STO | 0022 | RCL | 0021 | h | 0020 |
| ENTER | 0063 | CHS | 0062 | EEX | 0061 |
| CLX | 0060 | − | 0103 | 7 | 0102 |
| 8 | 0101 | 9 | 0100 | + | 0123 |
| 4 | 0122 | 5 | 0121 | 6 | 0120 |
| × | 0143 | 1 | 0142 | 2 | 0141 |
| 3 | 0140 | ÷ | 0163 | 0 | 0162 |
| . | 0161 | R/S | 0160 |  |  |

The M11 executable now carries a literal independent expected-code oracle and compares production `scan_code()` against it before injecting a contact. This removes the previous circular check in which the same function supplied both the injected and expected code.

## Display-observation audit

The original M10/M11 diagnostic `capture_display()` used a local ACT serial endpoint and cathode but reused the harness backplane/fetch/ROM0 objects. It did not modify ACT architectural registers or the pipeline, but it advanced diagnostic transport state while observing the display.

The audit hardens both M10 and M11: `capture_display(&self)` now creates fresh local firmware, backplane, ACT-serial, fetch, ROM0 and cathode objects. Display observation therefore cannot mutate the harness machine, keyboard, pipeline, display phase or transport state being tested.

## Exact regressions currently established

The following are exact raw-segment regressions through real `hp67firmware` and the structural ROM0/cathode path:

- power-on firmware idle `0.00`;
- digits 0 through 9;
- decimal entry `1.2`;
- CLX;
- `1 ENTER 2 + = 3.00`;
- `5 ENTER 2 - = 3.00`;
- `2 ENTER 3 × = 6.00`;
- `6 ENTER 2 ÷ = 3.00`;
- `1 CHS`, including the shared mantissa-sign output;
- `1 EEX 2`, including exponent display.

Hewlett-Packard's HP-67 Owner's Handbook independently states that CHS changes the sign of the mantissa or exponent and that EEX causes subsequent digits to be entered as the exponent of ten. The HP-67 Quick Reference Card also independently documents RUN/PROGRAM behavior.

## Shifted-function coverage: what it proves and what it does not

The matrix exercises all 105 combinations `f/g/h + one of 35 physical keys` from a fresh boot with X=1.

For each case it proves:

1. the prefix key itself traverses the physical keyboard/firmware path;
2. the second physical key is captured correctly;
3. firmware chooses a shifted dispatch destination;
4. the path executes a bounded 768-word observation window without an architectural/serial model error.

This is **coverage**, not semantic-result certification. A path ending at an arbitrary PC after 768 words does not prove that SIN, COS, LN, GSB, flags, storage, program control, etc. produced the correct final result or side effects.

The executable therefore reports `SHIFT EXERCISED` and `SHIFT COVERAGE COMPLETE`, not `SHIFT PASS`.

## UI contact semantics

The front panel uses egui `Response::is_pointer_button_down_on()`, not `clicked()`. egui documents this state as true from the press frame onward, without the click-versus-drag decision delay, and as remaining true for the widget while the pointer is held/dragged even outside its rectangle. This matches the emulator abstraction of closing one key contact until release and prevents a drag across the panel from synthesizing keyboard rollover.

Reference: https://docs.rs/egui/latest/egui/response/struct.Response.html#method.is_pointer_button_down_on

## Known fidelity gaps

### RUN/PRGM switch

The UI currently changes `Hp67State.mode` visually, but `src/app.rs` does not yet call `Hp67ArchitecturalMachine::set_program_mode()`. The architectural machine already exposes that CRC external-flag input. Hewlett-Packard documentation explicitly distinguishes PROGRAM mode from RUN mode, including key storage and single-step behavior.

Therefore RUN/PRGM behavior is not yet certified and belongs to M12 program entry/execution.

### Shifted function semantics

All shifted dispatch paths are reachable, but their individual mathematical/programming semantics are not yet independently exact-locked. The independent HP-67 key definition supplies the physical legends, and Hewlett-Packard documentation supplies user-visible behavior. Those sources should drive the next semantic regression set; emulator output must not be promoted to an expected value merely because the emulator produced it.

### Electrical timing

The current ACT core is an instruction-boundary architectural model with a structural bit/word transport. It intentionally does not yet claim final PHI launch/sample edges, bit-serial ALU commit timing, DATA timing, or exact STR/RCD overlap. These remain M14/M15 work and are already documented in `docs/HARDWARE_SOURCES.md`.

### Keyboard electrical matrix

The 35 scan codes now have independent semantic agreement, but direct Tier-A physical keyboard-matrix wiring evidence remains explicitly open in `docs/HARDWARE_SOURCES.md`. The project must not claim transistor/net-level keyboard exactness until that evidence is captured.

## Patch-history assessment

The iterative fixes during M11 were validator defects, not attempts to force production behavior:

- rustfmt-only reflows;
- a mistaken MULTIPLY expected value of 3.00 corrected to the mathematically and firmware-produced 6.00;
- an invalid assumption that shifted keys must dispatch into the unshifted 1405..1466 page removed after firmware correctly selected 07xx/06xx/05xx pages;
- CHS/EEX observations promoted to exact segment regressions after their structural meaning was checked;
- shifted output wording downgraded from PASS to COVERAGE;
- display snapshots made non-invasive;
- direct keycode verification made independent rather than circular.

None of these changes alters ACT, CRC, firmware, keyboard, fetch, display, or UI production behavior after the user-validated `405dcf7...` checkpoint.

## Audit conclusion

There is no evidence that production behavior has been patched opportunistically to satisfy M11. The currently proven RUN-mode numeric path is strong: real versioned firmware, physical key contacts, real firmware dispatch, architectural ACT/CRC execution and structural ROM0/cathode display generation all participate without host calculator semantics.

M11 should **not** yet be marked fully complete. Its direct keyboard/basic numeric portion is exact-locked; its shifted-function portion is presently comprehensive dispatch/execution coverage. M11 becomes complete only when a representative and then systematic set of shifted mathematical, storage and control functions is independently specified and exact-locked, or when those program-control families are explicitly moved to M12 with documented scope.
