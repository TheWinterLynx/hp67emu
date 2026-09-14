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
- [x] Add CI regression with compiler warnings denied.
- [x] Remove stale vector-rendering comparison tooling.
- [ ] Normalize inherited photographic UI files with rustfmt, then enable `cargo fmt --check` in CI.

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

- [ ] Obtain known-good HP-67 ROM dumps from a source we are allowed to use for development.
- [ ] Record chip-to-image mapping and SHA-256 for every ROM image.
- [ ] Decide repository/distribution policy before committing any copyrighted ROM bytes.
- [ ] Build a ROM loader that can use external development images without coupling the core to file I/O.
- [ ] Cross-reference available disassembly/listings with raw words.
- [ ] Build opcode conformance fixtures from trusted Woodstock/HP-67 microcode analysis.

## Scheduler

- [ ] Implement resolve -> snapshot -> evaluate -> commit loop.
- [ ] Add scheduled transitions/propagation slots if traces prove they are required.
- [ ] Make bus contention fail tests with driver names and tick number.
- [ ] Add trace probes with deterministic ordering.
- [ ] Verify that device container iteration order cannot change results.

## ACT 1820-2530

- [ ] Inventory every architecturally visible register and width.
- [ ] Implement 12-bit PC and verified return-stack behavior.
- [ ] Implement status bits, pointer and format state.
- [ ] Implement instruction fetch timing on ISA/IS.
- [ ] Implement instruction decode one verified opcode family at a time.
- [ ] Implement bit-serial register transfers.
- [ ] Implement serial arithmetic/ALU timing.
- [ ] Implement SYNC and RCD outputs.
- [ ] Implement DATA and ISA bus drive/release timing.
- [ ] Add unknown-opcode hard failure with PC/word/tick context.

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
- [ ] Integrate segment on-time over real scan timing.
- [ ] Feed physical segment intensity into `classic_display.rs`.
- [ ] Delete text-to-segment formatting from the fidelity path.
- [ ] Make power-on `0.00` emerge from microcode/electronics.

## Keyboard and switches

- [ ] Transcribe the real HP-67 key contact matrix.
- [ ] Replace semantic key events with contact closures in cycle-accurate mode.
- [ ] Implement KC1-KC5 and related key-scan path.
- [ ] Route RUN/W/PRGM through the correct hardware-visible flag.
- [ ] Route OFF/ON through machine power/reset rather than UI state.
- [ ] Add bounce only if measured behavior matters to firmware-visible timing.

## Card reader

- [ ] Document CRC 1820-1751 pins/protocol from HP-67/97 evidence.
- [ ] Implement CRC instruction/status/data behavior.
- [ ] Model card-presence switches and transport timing.
- [ ] Model sense-amplifier digital output boundary.
- [ ] Separate host card-file representation from electrical CRC model.
- [ ] Add known-card read/write regressions.

## UI debt to remove

- [ ] Delete `src/hp67.rs` temporary formatted state.
- [ ] Remove `KeyAction` semantics from the cycle-accurate input path.
- [ ] Remove hard-coded `0.00` startup placeholder.
- [ ] Make slider state reflect actual electrical machine state.
- [ ] Keep photographic animation purely presentational; it must never alter machine timing.

## Generalization

- [ ] Do not move HP-67 code into shared chip families until a second calculator needs it.
- [ ] When a second Woodstock calculator is added, extract only behavior proven identical by sources/tests.
- [ ] Keep calculator wiring/configuration separate from reusable chip implementations.
