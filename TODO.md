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
- [x] Reconcile the 1820-1596/1820-2530 compatibility evidence sufficiently to target 1820-2530 while retaining 1820-1596 as a semantic compatibility reference; keep revision-dependent electrical differences open if later evidence appears.
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
- [x] Record the direct physical HP-67 startup instruction evidence at `0x000`, `0x001` and `0x0f8` and expose it through `rom_compare verify-startup`.
- [ ] Import or manually encode at least one trusted HP-67 startup waveform as a golden regression fixture, including bit/edge timing rather than only decoded words.

## ROM/microcode corpus

- [x] Record the 2022 Teenix ROM-reader release as the strongest physical-dump provenance lead; do not assume the current ZIP still contains those historical ROM files.
- [x] Record the current Teenix HP-67 module as updated 10 May 2026 and MultiCalc as updated 11 May 2026; treat it as a current high-priority corpus.
- [x] Identify Nonpareil's `67.asm`, `6797.asm`, `67b1.asm` and card-reader disassembly as symbolic cross-checks.
- [x] Identify x11-calc's built-in 8192-entry `i_rom[]` image as an independent HP-67 ROM corpus.
- [x] Add a local x11-calc `i_rom[]` extractor that requires exactly 8192 valid 10-bit words.
- [x] Define a normalized sparse `(bank, pc, word)` TSV corpus format.
- [x] Add explicit-radix address/opcode import for assembled/listed corpora.
- [x] Add conflict-safe merging of sparse bank/page corpora.
- [x] Add exact bank/page/PC corpus comparison with non-zero mismatch exit status.
- [x] Add physical-startup evidence checking for `0x000`, `0x001` and `0x0f8`.
- [x] Prove the current Teenix `.pfl` outer container: XOR every byte with `0x55`, decoded first line `NeWe`, decoded second line decimal payload length.
- [x] Add a tested `research::teenix` decoder that validates magic and payload length exactly.
- [x] Add `rom_compare decode-teenix` and `preview-teenix` for local inspection without committing firmware payloads.
- [x] Record the observed Teenix 2026 decoded headers: `cal67=92855`, `cal6713=92882`, `cal67b=87834` payload bytes; all begin with `L0000: no operation`.
- [ ] Establish the internal Teenix `.pfl` textual source/listing grammar: address directives, bank directives, labels, comments, mnemonics and any embedded metadata.
- [ ] Convert the proven Teenix source/listing grammar to explicit 10-bit `(bank, pc, word)` data with regression tests.
- [ ] Record SHA-256 for the current `HP67.zip`, each `.pfl`, decoded payloads and MultiCalc package used for analysis.
- [x] Extract the current `ROMreader.zip` and establish that a filename search for `67`, `1818` or `rom` exposes only `ROM Reader Help.pdf` and `ROMread..hex`; historical ROM payload is not yet located.
- [ ] Inventory every file in the current `ROMreader.zip` with size and SHA-256.
- [ ] Confirm `ROMread..hex` as reader-controller Intel HEX rather than HP-67 microcode.
- [ ] Read `ROM Reader Help.pdf` for reader output naming/encoding and database-file locations.
- [ ] Locate an archived 2022 `ROMreader.zip` or another physical-reader HP-67 dump if the current archive no longer ships the ROM data.
- [ ] Record chip-to-image mapping and SHA-256 for every physical HP-67 ROM image recovered.
- [ ] Add a tested normalizer for the actual physical-reader output format.
- [ ] Assemble/export Nonpareil's HP-67 sources to address/opcode listings and normalize both banks.
- [ ] Run x11-calc ↔ Nonpareil comparison and preserve the report/hashes.
- [ ] Require all normalized Teenix/x11-calc/Nonpareil candidates to pass the direct physical startup checkpoints.
- [ ] Run Teenix 2026 ↔ x11-calc ↔ Nonpareil full comparison and explain every mismatch.
- [ ] When recovered, run physical dump ↔ all software corpora and explain every mismatch.
- [ ] Cross-check the shared HP-67/97 ROM region against independent HP-97 data.
- [ ] Decide repository/distribution policy before committing any copyrighted ROM bytes.
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
