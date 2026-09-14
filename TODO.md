# TODO — cycle-accurate HP-67

This file is the operational checklist. Architectural rationale lives in `docs/ARCHITECTURE.md`; milestone sequencing lives in `docs/ROADMAP.md`.

## Foundation

- [x] Separate reusable library from desktop UI.
- [x] Add deterministic simulation tick type.
- [x] Add non-overlapping PHI1/PHI2 scaffold.
- [x] Add explicit High/Low/High-Z net drivers and contention detection.
- [x] Add electrical-device evaluation interfaces.
- [x] Add trace sample container.
- [x] Add HP-67 chip inventory and initial named nets.
- [x] Add HP-67 electrical backplane shell.
- [x] Add GUI/core architecture regression.
- [x] Add per-source documentation regression.
- [x] Define the local regression command with compiler warnings denied.
- [x] Remove stale vector-rendering comparison tooling.
- [ ] Normalize inherited photographic UI files with rustfmt, then add `cargo fmt --check` to the local regression command.

## Semantic/reference implementations

- [x] Analyse Nonpareil's Woodstock CPU, HP-67 calc definition, ROM disassembly, display scan and CRC model; see `docs/NONPAREIL_ANALYSIS.md`.
- [x] Analyse x11-calc's HP-67 implementation and record exactly what it can and cannot establish; see `docs/X11_CALC_ANALYSIS.md`.
- [x] Add a UI-independent Rust `reference::woodstock` namespace.
- [x] Encode the 14-digit architectural state and 10-bit four-way opcode classification in Rust.
- [x] Add regression tests covering the complete 1024-word opcode classification and field ranges.
- [x] Implement the 32 arithmetic/register operations in the semantic reference model.
- [x] Implement special instructions used by the HP-67 ROM: status/P, constants, ROM selection, delayed ROM selection, RAM/register access, display control, key dispatch, return and bank switch.
- [x] Implement semantic JSB/GOTO/THEN-GOTO state transitions and two-level return stack.
- [x] Add a semantic ROM/bank loader independent of host file I/O.
- [x] Add instruction-boundary snapshots and a readable state-diff formatter.
- [x] Build a reusable differential harness that waits for the timed target's next completed microinstruction and compares architectural snapshots.
- [ ] Connect the differential harness to the real electrical ACT once that device exposes instruction-boundary state.
- [x] Port the *behaviour* needed from Nonpareil's CRC model into an independently structured Rust reference peripheral.
- [x] Preserve Nonpareil's HP-67 P-wrap compatibility addresses as regression targets; do not copy the address-specific hack into the electrical ACT.
- [ ] Add an HP-67 P-wrap/label-search regression that must agree with both Nonpareil and x11-calc without PC-specific hacks in the electrical ACT.
- [ ] Reconcile the ACT part/revision identifiers in Nonpareil's HP-67 metadata with physical HP-67 sources before freezing chip identity.
- [ ] Do not copy GPL-covered Nonpareil or x11-calc source line-for-line unless the project deliberately makes a compatible licensing decision.

## Next: evidence and timing

- [ ] Transcribe the HP-67 schematic into a reviewed pin/net table.
- [ ] Confirm exact HP-67 PHI1/PHI2 pulse width, period and non-overlap from primary/service or scope evidence.
- [ ] Define the canonical 56-bit word numbering and bit numbering convention.
- [ ] Confirm ISA/IS bus idle state, active-drive polarity and ownership rules.
- [ ] Confirm DATA bus idle state, active-drive polarity and ownership rules.
- [ ] Confirm exact SYNC placement and width for the HP-67 ACT generation.
- [ ] Confirm RCD timing from ACT and STR timing from ROM0.
- [ ] Record uncertainty/source level for every pin semantic; no undocumented assumption enters chip code.
- [ ] Define a versioned text/binary trace format for logic-analyzer comparison.
- [ ] Import or manually encode at least one trusted HP-67 startup waveform as a golden regression fixture.

## ROM/microcode corpus

- [x] Identify the Teenix physical ROM-reader archive as the preferred first HP-67 dump source; see `docs/MICROCODE_PROVENANCE.md`.
- [x] Identify Nonpareil's `67.asm`, `6797.asm`, `67b1.asm` and card-reader disassembly as symbolic cross-checks.
- [x] Identify x11-calc's built-in 8192-entry `i_rom[]` image as a third HP-67 ROM corpus for comparison.
- [ ] Download and inspect the HP-67 ROM files from the Teenix ROM-reader archive locally.
- [ ] Record chip-to-image mapping and SHA-256 for every ROM image.
- [ ] Decide repository/distribution policy before committing any copyrighted ROM bytes.
- [ ] Cross-check Teenix raw words against the Nonpareil disassembly/object layout.
- [ ] Compare the complete logical HP-67 ROM image against x11-calc and record every bank/page/address mismatch.
- [ ] Build opcode conformance fixtures from the reference model plus trusted Woodstock/HP-67 microcode analysis.

## Scheduler

- [x] Implement resolve -> snapshot -> evaluate -> commit loop.
- [ ] Add scheduled transitions/propagation slots if traces prove they are required.
- [x] Make bus contention fail tests with stable driver names and tick number.
- [x] Add trace probes with deterministic ordering.
- [x] Verify with regression tests that device container iteration order cannot change propagation results.

## ACT 1820-2530 / HP-67 ACT revision

- [ ] Inventory every architecturally visible register and width, using the Rust reference state as the instruction-boundary contract.
- [ ] Implement 12-bit PC and verified return-stack behavior.
- [ ] Implement status bits, pointer and format state.
- [ ] Implement instruction fetch timing on ISA/IS.
- [ ] Implement instruction decode one verified opcode family at a time.
- [ ] Implement bit-serial register transfers.
- [ ] Implement serial arithmetic/ALU timing.
- [ ] Implement SYNC and RCD outputs.
- [ ] Implement DATA and ISA bus drive/release timing.
- [ ] Add unknown-opcode hard failure with PC/word/tick context.
- [ ] At each instruction boundary compare against `reference::woodstock` rather than hand-authored expected states where possible.

## ROM/RAM devices

- [ ] Model 1818-0231.
- [ ] Model 1818-0232.
- [ ] Model 1818-0550.
- [ ] Model 1818-0551.
- [ ] Model 1818-0268 ROM0/display-anode behavior separately from generic ROM/RAM behavior.
- [ ] Verify read/write RAM timing and bus ownership against traces.

## Display

- [ ] Implement 1820-1749 cathode driver.
- [ ] Implement RCD reset and STR stepping electrically.
- [ ] Verify 15-position/sign-routing topology for HP-67.
- [ ] Use Nonpareil and x11-calc HP-67 display semantics as cross-checks, never as timing implementation.
- [ ] Integrate segment on-time over real scan timing.
- [ ] Feed physical segment intensity into `classic_display.rs`.
- [ ] Delete text-to-segment formatting from the fidelity path.
- [ ] Make power-on `0.00` emerge from microcode/electronics.

## Keyboard and switches

- [ ] Transcribe the real HP-67 key contact matrix.
- [ ] Cross-check all 35 resulting hardware key codes against both Nonpareil `67.ncd.tmpl` and x11-calc `x11-calc-67.c`.
- [ ] Replace semantic key events with contact closures in cycle-accurate mode.
- [ ] Implement KC1-KC5 and related key-scan path.
- [ ] Route RUN/W/PRGM through the correct hardware-visible CRC flag/input; Nonpareil identifies CRC flag 1 as the semantic switch state.
- [ ] Route OFF/ON through machine power/reset rather than UI state.
- [ ] Add bounce only if measured behavior matters to firmware-visible timing.

## Card reader

- [ ] Document CRC 1820-1751 pins/protocol from HP-67/97 evidence.
- [ ] Implement the CRC electrical instruction/status/data behaviour.
- [x] Use the independent Rust CRC semantic reference for instruction-boundary comparisons.
- [ ] Model card-presence switches and transport timing.
- [ ] Model sense-amplifier digital output boundary.
- [ ] Separate host card-file representation from electrical CRC model.
- [ ] Use x11-calc's HP-67 diagnostic A/C card programs as external end-to-end acceptance tests after redistribution/licensing review.
- [ ] Add known-card read/write regressions.

## UI debt to remove

- [ ] Delete `src/hp67.rs` temporary formatted state.
- [ ] Remove `KeyAction` semantics from the cycle-accurate input path.
- [ ] Remove hard-coded `0.00` startup placeholder.
- [ ] Make slider state reflect actual electrical machine state.
- [ ] Keep photographic animation purely presentational; it must never alter machine timing.

## Generalization

- [ ] Do not move HP-67 code into shared chip families until a second calculator needs it.
- [ ] Keep the semantic reference layer separate from the production electrical scheduler so either can be reused independently.
- [ ] When a second Woodstock calculator is added, extract only behavior proven identical by sources/tests.
- [ ] Keep calculator wiring/configuration separate from reusable chip implementations.
