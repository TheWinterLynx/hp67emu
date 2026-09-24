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
- [x] Lock the documented Woodstock stack-special mapping with regressions: `0o1110 = down rotate`, `0o1310 = c -> stack`.
- [ ] Do not copy GPL-covered Nonpareil or x11-calc source line-for-line unless the project deliberately makes a compatible licensing decision.

## Next: evidence and timing

- [ ] Transcribe the HP-67 schematic into a reviewed pin/net table.
- [x] Lock direct HP-67 PHI1/PHI2 pin polarity and ordering from page-70 captures: normally high, alternating low-going non-overlapping pulses.
- [ ] Confirm exact HP-67 PHI1/PHI2 pulse width, period and dead-time durations from primary/service or quantified scope evidence.
- [x] Define the canonical 56-bit word numbering and bit numbering convention (`b0..b55`, 14 digits × 4 bits).
- [x] Add an HP-67-specific word-timing scaffold that keeps the abstract PHI sequence aligned with the 56-bit coordinate without inventing physical durations.
- [x] Confirm HP-67 IS/ISA passive/active behavior and fetch ownership: weak/passive low, active high/release, ACT address window followed by selected-ROM response.
- [x] Lock the direct HP-67 DATA stream phase: 56 bits LSB-first, serial bit 0 at machine-word `b2`, bits 54/55 wrapping into next-word `b0`/`b1`.
- [ ] Confirm DATA bus idle state, active-drive polarity and exact PHI-relative ownership rules.
- [x] Record HP-67-specific evidence that SYNC is present for normal instruction fetches and suppressed for the implied-GOTO target word following an `IF`.
- [x] Fix the HP-67 SYNC decision window to `b46..b55`; leave exact PHI sampling edge open.
- [x] Fix the HP-67 IS fetch windows from direct logic-analyser evidence: 12-bit ROM address LSB-first at `b16..b27`, 10-bit ROM result LSB-first at `b46..b55`.
- [x] Add tested IS serializers that emit address/ROM bits LSB-first and represent zero as bus release against the passive low bias.
- [x] Add structural ACT↔ROM serial fetch endpoints that reconstruct the 12-bit address and 10-bit result only from resolved IS levels, plus a one-cycle fetch/execution pipeline latch.
- [x] Lock the page-70 SYNC edge anchor: both rising and falling SYNC transitions align with PHI2 rising.
- [x] Separate current HP-67 PHI pin phase from the word-timing subphase counter and expose the physical edge crossed by each scheduler step.
- [x] Add versioned M14A edge-contract table with driver/receiver/source/evidence metadata and regression protection for unknown edges.
- [ ] Resolve the remaining page-70 HP-67 IS/DATA launch/sample edges and propagation constraints in that table.
- [x] Lock source-backed display timing facts: STR occurs on display bit 7; low-going STR starts the segment interval; normal segments are about 40 us, DP about 30 us, DP-to-STR gap about 5 us and STR pulse about 5 us; low-going RCD resets the cathode scan and overlaps final-slot STR.
- [ ] Convert STR/RCD observations into exact PHI-relative electrical edges and propagation constraints.
- [ ] Record uncertainty/source level for every pin semantic; no undocumented assumption enters chip code.
- [ ] Define a versioned text/binary trace format for logic-analyzer comparison.
- [x] Record the direct physical HP-67 startup instruction evidence at `0x000`, `0x001` and `0x0f8` and expose it through `rom_compare verify-startup`.
- [x] Add the physical `0x0067/0x0068 -> 0x0fc6` delayed-ROM/JSB flow as a semantic regression.
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
- [x] Add directional ROM subset verification so the 5120 physically populated Teenix words can be checked against broader 8192-entry software corpora without treating reference-only locations as failures.
- [x] Add physical-startup evidence checking for `0x000`, `0x001` and `0x0f8`.
- [x] Prove the current Teenix `.pfl` outer container: XOR every byte with `0x55`, decoded first line `NeWe`, decoded second line decimal payload length.
- [x] Add a tested `research::teenix` decoder that validates magic and payload length exactly.
- [x] Add `rom_compare decode-teenix` and `preview-teenix` for local inspection without committing firmware payloads.
- [x] Record the observed Teenix 2026 decoded headers: `cal67=92855`, `cal6713=92882`, `cal67b=87834` payload bytes; `cal67.pfl` contains 5127 decoded text lines and begins with the physical startup sequence.
- [x] Establish the Teenix listing grammar needed by `cal67.pfl`: blank/comment records do not consume ROM, `org $1400` switches to bank 1 PC `0x400`, and both `Lxxxx:` and `Hxxxx:` are hexadecimal address anchors.
- [x] Add a strict documented Woodstock mnemonic assembler covering control flow, all 32 arithmetic operations/eight fields, status/P families, ROM/data-register families, CRC commands and fixed specials.
- [x] Add `research::teenix_hp67`, `analyze-teenix-hp67` and all-or-nothing `extract-teenix-hp67`; unknown text, label mismatches and wrong word counts are hard failures.
- [x] Run `analyze-teenix-hp67` against the full current `cal67.pfl`: 5120 candidate instructions, 0 unknown instructions, 0 label mismatches and 0 overflow.
- [x] Extract the current Teenix 2026 listing to exactly 5120 `(bank, pc, word)` entries and verify all three direct physical startup checkpoints.
- [ ] Record SHA-256 for the current `HP67.zip`, each `.pfl`, decoded payloads and MultiCalc package used for analysis.
- [x] Extract the current `ROMreader.zip` and establish that a filename search for `67`, `1818` or `rom` exposes only `ROM Reader Help.pdf` and `ROMread..hex`; historical ROM payload is not yet located.
- [ ] Inventory every file in the current `ROMreader.zip` with size and SHA-256.
- [ ] Confirm `ROMread..hex` as reader-controller Intel HEX rather than HP-67 microcode.
- [ ] Read `ROM Reader Help.pdf` for reader output naming/encoding and database-file locations.
- [ ] Locate an archived 2022 `ROMreader.zip` or another physical-reader HP-67 dump if the current archive no longer ships the ROM data.
- [ ] Record chip-to-image mapping and SHA-256 for every physical HP-67 ROM image recovered.
- [ ] Add a tested normalizer for the actual physical-reader output format.
- [x] Extract x11-calc at reviewed commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983` and preserve the first 5120-word Teenix subset report: 5091 matches, 29 `0o1110/0o1310` mismatches, 0 missing locations; diagnose all 29 as a local assembler mnemonic-table swap and preserve the exact report in `docs/research/TEENIX_X11_COMPARISON_2026-09-15.md`.
- [x] Rerun Teenix 2026 ↔ x11-calc after correcting the stack-special mapping: all 5120 populated words match exactly, with 0 value mismatches and 0 missing reference locations.
- [x] Pin current Nonpareil research baseline `c347bc1ab20170c253512042f7aac0d952f304ea`; document official `uasm` Woodstock object records as optional bank mask plus octal address/opcode.
- [x] Add strict `research::nonpareil_obj`, `nonpareil_rom`, and `docs/research/compare_nonpareil.ps1` so official upstream `uasm` output can be normalized without reimplementing Nonpareil's symbolic assembler.
- [x] Assemble/export Nonpareil's `67.asm`, `6797.asm` and `67b1.asm` with official `uasm`, record source/object hashes, normalize the resulting objects and require the physical startup check to pass.
- [x] Run Teenix ↔ Nonpareil and x11-calc ↔ Nonpareil comparisons and preserve the exact report/hashes.
- [x] Require all normalized Teenix/x11-calc/Nonpareil candidates to pass the direct physical startup checkpoints.
- [x] Run Teenix 2026 ↔ x11-calc ↔ Nonpareil over equivalent populated regions: all 5120 words match exactly in all three paths with zero missing/mismatched words.
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
- [ ] Bind ACT serial execution state to the canonical HP-67 `b0..b55` word coordinate.
- [x] Establish and regression-test the structural ACT address-shift/ROM-result-shift path over the real IS net at `b16..b27` and `b46..b55`; exact PHI edges remain pending.
- [x] Establish a one-word pipeline latch where a word fetched in cycle N becomes executable when cycle N+1 begins.
- [ ] Integrate those fetch endpoints into the production ACT and physical 1818-* device scheduler once launch/sample edges are fixed.
- [ ] Implement instruction decode one verified opcode family at a time.
- [ ] Implement bit-serial register transfers.
- [x] Make the structural ADD/SUB result image authoritative for final A/B/C/carry at the completed word boundary while retaining the architectural executor only as an oracle for those outputs (M14D).
- [ ] Resolve source-backed intra-word ACT register/carry write timing before moving those effects onto specific bit/PHI edges.
- [ ] Implement serial arithmetic/ALU timing.
- [ ] Implement SYNC and RCD outputs.
- [ ] Implement DATA and ISA bus drive/release timing beyond the now-defined instruction-fetch windows.
- [ ] Add unknown-opcode hard failure with PC/word/tick context.
- [ ] At each instruction boundary compare against `reference::woodstock` rather than hand-authored expected states where possible.

## ROM/RAM devices

- [x] Implement M14C logical DATA authority for installed-RAM ACT read/write destinations while keeping CRC `0x99`/`0x9B` outside the slice.
- [x] Validate M14C locally with the formatting/warnings/all-targets/release gate and ignored 12-program release diagnostic before merge.
- [x] Validate M14D locally with the full formatting/warnings/all-targets/release gate and ignored 12-program release diagnostic (`DIAGNOSTIC SUITE OK: 12/12 passed`, owner-reported 2026-09-24).
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
- [ ] Route RUN/W-PRGM through the correct hardware-visible CRC flag/input; Nonpareil identifies CRC flag 1 as the semantic switch state.
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
