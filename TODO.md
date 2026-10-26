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

## M14I validated integration

- [x] Derive successor fetch independently from pre-instruction ACT flow state.
- [x] Restore oracle PC/bank/delayed-ROM/return-stack mutations before transport.
- [x] Commit matching structural flow only after completed b0..b55 execution.
- [x] Owner reported M14I FULL GATE GREEN; integrated into main on 2026-10-03.
- [ ] Exact intra-word flow latch timing remains SOURCE-BLOCKED.

## M14J validated integration

- [x] Structural P-change history and previous-carry authority outside M14F.
- [x] Structural carry reset outside M14E/F and arithmetic condition-latch authority.
- [x] Explicit restore/compare/commit ownership masks and b55 lifetime regressions.
- [x] Owner reported M14J FULL GATE GREEN; integrated into main under standing authorization.
- [ ] Exact internal boundary-latch timing remains SOURCE-BLOCKED.

## M14K validated integration

- [x] Structural low-nibble RAM selector authority for family 050/070.
- [x] Logical installed-block clear authority, preserving sparse topology and neighbours.
- [x] Preserve M14C cross-word DATA and M14H/CRC ownership boundaries.
- [x] Owner reported M14K FULL GATE GREEN; integrated into main under standing authorization.
- [ ] Physical clear protocol, chip partitioning and DATA edges remain SOURCE-BLOCKED.

## M14L validated integration

- [x] Structural CRC control-latch and ACT S3 final authority.
- [x] Immutable binding, internal/external flag separation and exclusive commit boundary.
- [x] Owner local formatting/warnings/all-targets/release and diagnostic 12/12 validation.
- [ ] Exact F2 and internal CRC latch edges remain SOURCE-BLOCKED.

- [x] Owner reported M14L FULL GATE GREEN; validated head integrated into main.

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
- [x] Extend completed-word structural A/B/C/carry authority to clear/copy/exchange, shifts and zero/nonzero tests so all 32 arithmetic opcodes use the structural result image (M14E implementation).
- [x] Validate M14E locally with formatting/warnings/all-targets/release gate plus ignored 12-program release diagnostic (`DIAGNOSTIC SUITE OK: 12/12 passed`, owner-reported 2026-09-25).
- [x] Move final P/status condition state for the focused special-opcode family onto a completed-word structural result image while keeping exact internal control timing source-blocked (M14F implementation).
- [x] Validate M14F locally with formatting/warnings/all-targets/release gate plus ignored 12-program release diagnostic (`DIAGNOSTIC SUITE OK: 12/12 passed`, owner-reported 2026-09-26).
- [x] Move final special-register state for clear/stack/M1/M2/F/key/A-rotate plus load-constant C[P] onto a completed-word structural result image (M14G implementation).
- [x] Validate M14G locally with formatting/warnings/all-targets/release gate plus ignored 12-program release diagnostic (`M14G FULL GATE GREEN`, owner-reported 2026-09-27).
- [x] Move decimal/hex, DISPLAY enable, 14-digit display and RAM-address final state onto a completed-word structural mode/latch image while isolating the startup display-only override (M14H implementation).
- [x] Validate M14H locally with formatting/warnings/all-targets/release gate plus ignored 12-program release diagnostic (`M14H FULL GATE GREEN`, owner-reported 2026-09-28).
- [ ] Resolve source-backed intra-word ACT register/carry write timing before moving those effects onto specific bit/PHI edges.
- [ ] Implement serial arithmetic/ALU timing.
- [ ] Implement SYNC and RCD outputs.
- [ ] Implement DATA and ISA bus drive/release timing beyond the now-defined instruction-fetch windows.
- [ ] Add unknown-opcode hard failure with PC/word/tick context.
- [ ] At each instruction boundary compare against `reference::woodstock` rather than hand-authored expected states where possible.

## ROM/RAM devices

- [x] Implement M14C logical DATA authority for installed-RAM ACT read/write destinations while keeping CRC `0x99`/`0x9B` outside the slice.
- [x] Validate M14C locally with the formatting/warnings/all-targets/release gate before merge.
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

## M14M validated integration

- [x] Stage CRC logical 28-bit read/write payloads and FIFO/READY/F7 results independently before structural execution.
- [x] Restore oracle effects and require completed b55 plus exact comparison before commit.
- [x] Add address/opcode differential, FIFO wrap/capacity/failure, immutable binding and live read/write regressions.
- [x] Owner local formatting/warnings/all-targets/release and Custom Diagnostic Pac 12/12 gate.
- [ ] Source-blocked: CRC electrical DATA ownership/phase/PHI edges, magnetic serialization and sense timing.

- [x] Owner reported M14M FULL GATE GREEN; validated head integrated into main.

## M14N validated integration

- [x] Restore implied-GOTO instruction-state latch before structural transport.
- [x] Isolate absent-RAM zero-read compatibility in an explicitly approximate completed-word image.
- [x] Require every ACT semantic field to equal pre-instruction state before structural transport.
- [x] Whole ACT/RAM/CRC differential across all ten-bit words and six address contexts.
- [x] Owner local formatting/warnings/all-targets/release and Custom Diagnostic Pac 12/12 gate.
- [ ] Source-blocked: actual absent-address electrical behavior and physical ACT/DATA latch timing.

- [x] Owner reported M14N FULL GATE GREEN; validated head integrated into main.

## M14O validated and integrated

- [x] Add release-only measurement of actual live authority and synthetic RAM/CRC stress paths.
- [x] Lock full transition/execution counts and continuous odd/even port fixture state.
- [x] Correct historical dual benchmark labels without changing those workloads.
- [x] Owner M14O FULL GATE GREEN, diagnostic 12/12 and electrical/DATA/live benchmark results.
- [ ] Investigate measured material regressions; no optimization without host results.

## M14P validated and integrated

- [x] Load both Moving Average tracks from the existing library through real firmware transport.
- [x] Add nine bounded physical-key/raw-display acceptance checkpoints.
- [x] Owner M14P FULL GATE GREEN, including official acceptance 9/9; exact head integrated.
- [ ] Extend official acceptance to other Pac applications and saved-data recovery.
- [ ] Electrical DATA/PHI/STR/RCD and magnetic serialization remain source-blocked.

## M14Q validated and integrated

- [x] Extend Moving Average to firmware B/Crd writing a blank data card.
- [x] Transfer native physical media only to a freshly booted machine; reload program and data.
- [x] Require recovered mean, seventh-input rolling mean and repeated query.
- [x] Owner M14Q FULL GATE GREEN, diagnostic 12/12, M14P 9/9 and saved-data recovery; integrated.
- [ ] Extend official examples to two-pass data windows and further Standard/Games Pac applications.

## M14R validated and integrated

- [x] Nine/ten-point fixtures using the manual data-card pass boundary.
- [x] Real firmware Crd before End2; preserve primary media on secondary write.
- [x] Native media persistence, fresh-machine program/data reload and recovered rolling means.
- [x] Owner M14R FULL GATE GREEN, diagnostic 12/12, M14P/M14Q and boundary 2/2; integrated.
- [ ] Expand manual acceptance to other Standard/Games Pac applications; source-blocked electrical timing remains open.

## M14S validated and integrated

- [x] Add official Polynomial Evaluation Example 2 via physical card/key/firmware paths.
- [x] Verify negative/scientific coefficient entry and negative segment sign position.
- [x] Owner reported all green; exact validated head integrated into main.
- Pac expansion stopped by owner instruction; electrical source gaps remain open.

## M14T display dwell branch

- [x] Integrate approximate observed normal/decimal dwell from decoded live ROM0 words.
- [x] Publish complete fifteen-slot scans only; blank replacement and power-reset history.
- [x] Add phase/decoder/publication and real-firmware/reset regressions.
- [x] Owner FULL GATE GREEN; exact M14T head integrated.
- [ ] Physical LED current/optical routing and exact PHI STR/RCD edges remain SOURCE-BLOCKED.

## M14U completed-scan optical boundary

- [x] Route raw scan dwell to physical signs, mantissa and exponent emitters.
- [x] Actual panel consumes completed scans; immediate masks stay diagnostic.
- [x] Blank immediately on DISPLAY disable and power reset.
- [x] Projection, real-firmware publication and optical selection regressions.
- [x] Owner M14U FULL GATE GREEN; exact validated head integrated.
- [ ] Exact PHI-relative edges, LED current/energy and duplicate-slot optical behavior remain SOURCE-BLOCKED.

## M14V optical publication epochs

- [x] Invalidate previous optical snapshot on firmware DISPLAY control.
- [x] Require a subsequent complete scan; preserve ACT/cathode and raw dwell counters.
- [x] Exercise all fifteen interruption phases and real-firmware control path.
- [x] Owner M14V FULL GATE GREEN; validated head integrated (5dfdbffbd301eba8a923d5883e356258b5382ff0).
- [ ] Analog current, exact edges and physical duplicate-slot behavior remain unresolved.


## M14W optional structural IS/PHI trace

- [x] Capture committed PHI transitions and resolved IS from diagnostic display/fetch transport.
- [x] Export ordinal-tick TSV with explicit timing limitations; retain failed-word prefix.
- [x] Preserve production no-DATA/DATA loop code and live performance path.
- [x] Add complete-scan differential/bus reconstruction and missing-ROM prefix regressions.
- [x] Owner M14W FULL GATE GREEN; exact validated head integrated.
- [ ] Capture actual desktop live authority traffic and compare against physical recordings in a subsequent slice.
- [ ] DATA electrical ownership, exact PHI/sample edges and electrically timed SYNC/STR/RCD remain SOURCE-BLOCKED.


## M14X bounded live bus capture

- [x] Connect optional IS/PHI recorder to real live authority cycle and logical DATA twin.
- [x] Bounded native CLI export (1..4096 words), with workload/reset/uncalibrated labels.
- [x] Actual firmware boot parity, synthetic RAM cross-word authority parity and bounds regressions.
- [x] Owner M14X FULL GATE GREEN, including corrected waiting native-export gate; integrated a4c8ffa8d7e82b982bdabeda6757285e639fd47c.
- [ ] Physical recording ingestion/alignment remains next; exact electrical edges remain SOURCE-BLOCKED.


## M14Y ordinal trace comparison

- [x] Strict bounded native/pin-only TSV import and explicit-index comparison.
- [x] Preserve provenance, unknown levels, driver coverage and unmatched tails.
- [x] Native reports and distinct match/mismatch/error exit codes.
- [x] Regression coverage and native match/deliberate-mismatch gate prepared.
- [x] Owner M14Y FULL GATE GREEN; exact validated head integrated.
- [ ] Actual physical recordings and documented normalization remain required for hardware contrast.


## M14Z IS word decode

- [x] Strict complete-word decode from observed IS using source-locked bit windows.
- [x] Native report with numeric fields, provenance, origins and explicit subset coverage.
- [x] Reject unresolved/partial/shifted/incompatible observations.
- [x] Transport-causal regression tests and native export/decode gate prepared.
- [ ] Owner full local gate.
- [ ] Physical captures/normalization and exact electrical edge evidence remain unresolved.

## M15A SYNC evidence correction

- [x] Reinspect direct page-70 waveforms before scheduling electrical SYNC.
- [x] Withdraw unsupported common PHI2-rising anchor in edge table V2.
- [x] Regression-lock absent SYNC launch/sample/propagation.
- [ ] Owner local formatting/warnings/all-targets/release validation.
- [ ] Original acquisition metadata or independent hardware measurement.

## M15B IS contribution guard

- [x] Detect multiple active High contributors before IS receiver sampling.
- [x] Reject active Low IS contributions without inventing DATA polarity.
- [x] Preserve generic net resolution, atomic handoff and contention diagnostics.
- [x] Cover all four structural transport variants and retained failure prefixes.
- [ ] Owner formatting, focused/full tests, diagnostic and performance gates.
- [ ] Exact ownership handoff edges and releasing output-enable behavior.

## M15D SYNC-qualified fetch

- [x] Apply explicit owner HP-97 equivalence assumption to executable behavior.
- [x] Drive/sample resolved SYNC in all four structural transport loops.
- [x] Suppress IF successor pulse; keep implied-GOTO payload distinct from opcode.
- [x] Require ten stable qualifiers and reject floating/contention/mixed samples.
- [x] Connect received qualification to live next-word latch after oracle agreement.
- [ ] Owner full local gate, diagnostic and performance validation.
- [ ] Replace coarse bit-cell publication with measured PHI/propagation scheduling.

## M15D validated integration

- [x] Owner M15D FULL GATE GREEN; exact aa7241c integrated into main.
- [x] DATA phase 0.939 us/word; live idle 1.105 us/word (289.60x nominal).

## M15E resolved DATA level bridge

- [x] Live installed-RAM reconstruction consumes resolved DATA body and tail.
- [x] Bind ACT/aggregate RAM ownership and retain prior/current direction.
- [x] Atomic source handoff, HighZ idle and same-level/contending fault checks.
- [x] Keep default logical fixtures and label historical benchmark scope.
- [ ] Owner full local gate, diagnostic and actual-live performance.
- [ ] Physical output-stage/polarity/bias and PHI-relative DATA timing.
- [ ] Replace the early-tail scheduling bridge with physical sampling timing.

## M15E validated integration

- [x] Owner M15E FULL GATE GREEN; exact 2bedade integrated.
- [x] Diagnostic 12/12; live idle 1.100 us/word; RAM stress 1.554 us/word.

## M15F clocked DATA tail

- [x] Consume next-word b0/b1 through shared IS/display/DATA and eight PHI edges.
- [x] Commit completed C/RAM payload after prefix, before next instruction inputs.
- [x] Resume verified body at b2; preserve 56 cells, 224 edges and trace rows.
- [x] Freeze display source, preserve ROM0 prefix and reject invalid starts.
- [x] Whole-versus-split parity and actual-live write/read dependency regressions.
- [ ] Owner full local gate, diagnostic and live performance measurements.
- [ ] Exact PHI sample/write edges, DATA output stage and internal ACT mutation.

## M15G resolved STR/RCD bridge

The display transports now publish named ROM0 STR and ACT RCD logical outputs
and derive downstream events from resolved pins, rejecting extra contributors
and contention. STR Low in b7, RCD overlap in slot 15, and High restoration at
b8 form an explicit WORKING APPROXIMATION. Exact PHI edges, physical pulse widths,
output circuitry, propagation and cathode application inside the word remain
unproven. Word timing stays 56 cells/224 transitions, including DATA prefix
continuation. See `docs/research/M15G_RESOLVED_DISPLAY_CONTROLS_2026-10-16.md`.
Owner local M15G gate pending; M15F exact validated head is integrated.

## M15H PHI receiver scheduling

M15G is owner-validated and integrated. Production transport now publishes
sources before PHI1, commits PHI1 falling/rising, samples resolved IS/DATA, then
commits PHI2 falling/rising. All variants and the clocked DATA prefix share this
order. The PHI1-rising checkpoint is a WORKING APPROXIMATION within the observed
stable interval, not proof of silicon latch timing. DATA sampling no longer
republishes the source. Physical V2 edge-table unknowns and uncalibrated widths
remain explicit. See `docs/research/M15H_PHI_RECEIVER_SCHEDULING_2026-10-16.md`.
Owner local M15H gate pending.

## M15I atomic serial publication

M15H is owner-validated and integrated. All display/fetch transports and the
clocked DATA prefix now prepare immutable source proposals before a joint
IS/SYNC/optional-DATA commit. Wrong phase or invalid DATA proposal leaves the
previous pin image intact; word-start IS HighZ reset is removed. Full words
retain 224 transitions. This changes simulator commit semantics and preserves
M15H's provisional PHI policy; it does not establish unknown physical edges.
See `docs/research/M15I_ATOMIC_SERIAL_PUBLICATION_2026-10-16.md`.
Owner local M15I gate pending.

## M15J five-ROM live fetch

M15I is owner-validated and integrated. Live ROM fetch now uses five independent
serial address receivers and named IS drivers, releasing the aggregate IS source.
Four transports and the clocked DATA prefix preserve 224 transitions/word.
ROM1/2 part mapping uses the owner's HP-97 equivalence instruction; ROM4=0231
is an explicit inventory-based working inference. Autonomous bank control, RAM
DATA devices, ACT internal mutation and physical PHI calibration remain open.
See `docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md` for evidence and scope.
Owner local M15J gate GREEN; exact head integrated.

## M15K autonomous ROM bank control

M15J's exact formatted head `fa0c06364ffba7db7f8193b1301d970c6c9326a5`
is owner-validated and integrated. Live ROMs now select from persistent per-chip
bank latches driven by resolved IS/SYNC, not the firmware provider's bank view.
Normal word 0x230 sets bank 1; a complete address below 0x400 restores bank 0.
Bank-1 holes above 0x7ff fail instead of silently falling back. Provider-view
fixtures remain explicit references. ACT bank state is comparison-only on this
path. Timing remains 56 cells/224 PHI transitions; the b55 decoder checkpoint
retains M15H's provisional PHI policy, not a newly proven physical latch edge.
See `docs/research/M15K_AUTONOMOUS_ROM_BANKS_2026-10-16.md`.
Owner local M15K gate GREEN; exact v3 head integrated. Physical DATA/RAM devices, internal ACT mutation,
exact PHI calibration and propagation, display integration and magnetic sense
remain open; this milestone does not claim completion of electrical fidelity.

## M15L DATA receiver owner identity

M15K is owner-validated at exact head
`8937b8e99408f1df89e511554c8b183d0d21158e` and integrated. Diagnostic 12/12
retains its cycles; live idle measured 1.410 us/word (226.92x), synthetic pinned-PC
RAM 1.958 us (163.45x), CRC 1.397 us (229.11x). Synthetic workloads are stress
loads, not physical card throughput.

M15L rejects a sole DATA conductor with the wrong identity even when its level
is valid. It retains the previous frame's owner at b0/b1, checks the current one
at b2..b55 and consumes the actual resolved value without re-driving it. This is
validation of the existing logical-level bridge: ACT writes and aggregate
peripheral reads. Physical RAM chip/address mapping and output circuitry remain
unresolved. PHI scheduling/counts and all production authority paths remain.
See `docs/research/M15L_DATA_OWNER_IDENTITY_2026-10-17.md`.
Owner local M15L gate GREEN; exact head integrated.

## M15M ordered ROM reception

M15L's exact head `16e76444012abb27b8ea2f81de2d518202451568` is
owner-validated and integrated: diagnostic 12/12 with unchanged cycles; live
idle 1.469 us/word, synthetic RAM 1.872 us, CRC 1.441 us. M15M makes address
and autonomous return receivers require a contiguous sequence before selecting
ROM or decoding bank-set. Wrong coordinates, repetitions and replay are errors
before receive-state mutation; complete ROM-observed words are exposed for
comparison with ACT. This strengthens the simulator's serial contract without
claiming new physical PHI timing, RAM mapping or analog glitch behavior.
See `docs/research/M15M_ROM_RECEIVER_ORDER_2026-10-18.md`.
Owner local M15M gate GREEN; exact head integrated.

## M15N autonomous serial RAM device

M15M's exact validated head `1c5c8f987c9fde0bdd87f581db16a6ae99023ba8` is
integrated. M15N adds an autonomous aggregate logical RAM: own 64 registers,
commands from resolved IS/SYNC, address/write frames from DATA, stored-data read
responses, cross-word tails and logical block clear. It runs in a separate
224-transition electrical composition. Desktop live still uses its existing
bridge until C-to-address DATA and consistent command forwarding are integrated.
Physical RAM-chip mapping, output circuitry and exact latch timing remain open.
See `docs/research/M15N_AUTONOMOUS_SERIAL_RAM_2026-10-18.md`.
Owner local M15N FULL GATE GREEN; exact ad4e1053 head integrated.


## M15O live address DATA

Live C-to-address now transmits frozen C through resolved DATA body and clocked
next-word tail. It preserves M14H ACT selector authority, strict receive checks,
traced/untraced parity and 224 PHI transitions/word. Autonomous RAM remains a
separate fixture until command forwarding and device-owned live reads/storage
are connected. Physical DATA stage, per-IC mapping and exact PHI timing remain
SOURCE-BLOCKED. See research/M15O_LIVE_ADDRESS_DATA_2026-10-19.md. Owner M15O FULL GATE GREEN; exact f5548296 head integrated.


## M15P independent live RAM command receiver

Live now receives actual resolved IS/SYNC through the same receiver as autonomous
RAM and compares complete word/qualifier against ACT after every word. Synthetic
execution injection cannot feed this observer. Default live words use the
DATA-capable shared loop even with idle DATA. Autonomous RAM storage/read driving
is still separate; next is its joint read proposal and oracle comparison. Exact
physical edges/output stage/mapping remain blocked. See
`docs/research/M15P_LIVE_RAM_COMMANDS_2026-10-20.md`. Owner M15P FULL GATE GREEN; exact 627abbc3 head integrated.


## M15Q autonomous live aggregate RAM

Default live now uses device-owned 64-word storage, resolved command/address/write
reception and device-owned DATA read proposals. Oracle read bits are replaced by
a frame mask; reference RAM remains comparison only. Clocked tails and 224 PHI
transitions remain. Host-injected tests use an explicit test-only reference
composition, while real firmware/key/program/card/diagnostic paths keep autonomous
RAM. Physical per-IC mapping/output circuitry and exact PHI/propagation remain
blocked. See `docs/research/M15Q_LIVE_AUTONOMOUS_RAM_2026-10-20.md`. Owner M15Q FULL GATE GREEN; exact 73564395 head integrated.


## M15R independent RAM DATA driver

Autonomous RAM now drives a named RamAggregate contribution instead of the
reference ROM responder. Joint publication, foreign-source contention and strict
read body/tail ownership checks remain. This is logical aggregate identity;
physical chip mapping/output stage and exact PHI edges remain open. See
`docs/research/M15R_RAM_DATA_DRIVER_2026-10-20.md`. Owner M15R FULL GATE GREEN; exact 761a8583 head integrated.


## M15S ACT DATA receive-only path

Production RAM reads now bind only the ACT receiver; the dummy payload/source
and local read tail are removed. Required frame/owner/count metadata persists
through next b1 and then hands off to the next ACT source. Oracle expected data
remains comparison-only. 224 transitions/word and physical evidence boundaries
are unchanged. See `docs/research/M15S_ACT_DATA_RECEIVE_ONLY_2026-10-21.md`.
Owner M15S FULL GATE GREEN; exact 8ce3363b head integrated.


## M15T autonomous CRC control

Independent IS/SYNC reception now drives twelve retained CRC latches, the
one-word control pipeline and validated live result/S3 authority. Logical FIFO
and transport retain only an explicit READY/F7 handoff; production reset and
synthetic reference composition are preserved. Exhaustive command/qualifier/
contact tests and real firmware/card regressions accompany the complete lane.
CRC DATA serialization and exact F2/PHI/propagation remain open; no new physical
timing claim. See `docs/research/M15T_AUTONOMOUS_CRC_CONTROL_2026-10-22.md`.
Owner M15T FULL GATE GREEN; exact f635ea87 head integrated.


## M15U ACT received instruction pipeline

Production ACT now retains its own complete IS/SYNC frame and starts the next
word's execution from it. Ordered atomic reception rejects replay, qualification
faults and unconsumed overwrite. Live host word/qualifier mirrors are checked
before clocks or DATA tail consumption. Reset and explicit synthetic reference
composition are preserved. No new PHI edge or internal ACT write timing claim.
See `docs/research/M15U_ACT_RECEIVED_EXECUTION_2026-10-22.md`.
Owner M15U FULL GATE GREEN; exact 8fe8b797 head integrated.


## M15V owned ACT flow registers

PC, bank, delayed ROM and return stack now persist in the ACT endpoint. The
successor preview uses those latches and supplies the actual address serializer;
completed b55 execution commits own flow and low-page bank normalization.
Live validates all mirror fields before clocks/DATA tails and after commit.
Reset/reference composition and the 224-transition schedule remain unchanged.
Exact internal write edges remain unproven. See
`docs/research/M15V_ACT_OWNED_FLOW_2026-10-22.md`. Owner M15V FULL GATE GREEN; exact e1f8e221 head integrated.


## M15W owned ACT register file

Retained ACT registers now supply execution/display, with explicit keyboard,
resolved RAM C and independent CRC condition ports. Full mirror guards precede
clocks/DATA tails; checked structural completion supplies the mirror. Reset and
reference composition are preserved. Six regressions accompany this lane.
No exact PHI/internal-write timing claim. See
`docs/research/M15W_ACT_OWNED_REGISTERS_2026-10-23.md`. Owner M15W FULL GATE GREEN; exact 384c1119 head integrated.


## M15X retained CRC DATA and card transport

Production port staging and transport ingress/draining now use retained CRC
FIFO storage. Independent control owns mode/handshake commands; strict mirror
checks precede clocks and observer agreement precedes device publication.
Reset and injected reference fixtures preserve their composition. Seven new
regressions include all-speed record transport and full firmware card roundtrip.
Physical CRC DATA serialization and exact PHI edges remain open. See
`docs/research/M15X_CRC_OWNED_DATA_2026-10-23.md`. Owner M15X FULL GATE GREEN; exact 29b99098 head integrated (delivery 1/15).


## M15Y pin-derived display circuit — delivery 2/15

Independent ACT RCD and ROM0 STR proposals feed a strict ordered resolved
receiver; cathode selection, anode decode and nominal exposure live in the
library circuit. ACT slot annotations are comparison-only. Phase corruption
fails before clocks/tails; reset and optics retain existing policies. Nine new
regressions cover the complete lane. Physical pulse widths/PHI/propagation stay
open. See `docs/research/M15Y_DISPLAY_CIRCUIT_2026-10-23.md`. Owner M15Y FULL GATE GREEN; exact ae0865b0 head integrated (delivery 2/15).


## M15Z full-pin capture and physical-time audit — delivery 3/15

Opt-in capture covers all named pins and driver masks, atomic source bundles
and ordinal sub-events. Strict normalized physical-time import, PHI interval
audit, IS/SYNC/wrapped DATA decoding and explicit-word selected-pin comparison
are available through native CLI. Eleven regressions cover the complete lane.
This supplies validation tooling; no measured hardware fixture or new physical
timing claim is introduced. See `docs/PIN_CAPTURE_v1.0.md` and
`docs/research/M15Z_ELECTRICAL_CAPTURE_2026-10-23.md`. Owner M15Z FULL GATE GREEN; exact 38a64171 head integrated (delivery 3/15).


## M16A PHI windows and compiled-out diagnostic observer — delivery 4/15

Normal builds omit recorder storage and publication hooks; pin-capture builds
retain the complete recording lane and its tests. Offline PHI window audits
now report all-edge observed setup/hold separations, censoring, coincidences
and PHI1 stability. Both builds are required in the owner gate. No physical
edge or pulse-width approximation is promoted to exact timing. See
`docs/PIN_CAPTURE_v1.1.md` and `docs/research/M16A_PHI_WINDOWS_2026-10-24.md`. Owner M16A FULL GATE GREEN; exact 685407cf7739011478bd5eb5f643da27cd83bab9 integrated (delivery 4/15).


## M16B PHI ownership integrity — delivery 5/15

Production composition and live entry now reject invalid PHI images before
endpoint/input/DATA-tail mutation. Shared launch/sample guards additionally
require sole ACT clock ownership, catching agreeing extra drivers and replacement
sources that resolved-level tests alone missed. A checked edge API preserves
time and pins on failure and reports tick/net/expected/actual/driver mask.
Dense count checks avoid heap allocation and driver scans on the normal path;
all 224 edges and device work remain. Nine fault/recovery regressions accompany
the change. Physical widths/dead time, exact sampling edges and propagation
remain SOURCE-BLOCKED. See research/M16B_CLOCK_INTEGRITY_2026-10-24.md.
Owner M16B FULL GATE GREEN; exact 692ce6c5d1409e1acd167813630a1c3d5b9ce334 integrated (delivery 5/15). Both diagnostics 12/12; live median 3.690 us/word. Performance follow-up below.


### M16B performance follow-up (same delivery)

PHI ownership is summarized at committed High-Z occupancy changes, replacing
repeated count/ACT-slot inspection at receivers with a cached two-clock fault
mask. Level resolution, phase checks, contention, foreign-driver rejection and
224 transitions remain. Independent scan/staged/mixed-update regressions and a
rotating ownership kernel benchmark accompany the change. All eight electrical
benchmark rows now rotate together; the live benchmark already rotated.
No recovered performance is claimed before the owner gate. See
`docs/research/M16B_CLOCK_CACHE_2026-10-24.md`. This follow-up does not consume
delivery 6/15 or claim any new physical timing calibration.


## M16B cache validated integration

Owner M16B PERF FULL GATE GREEN; exact 976e8e4fadf269d0fba22f307ba11e3d4084a613
integrated. Both diagnostics 12/12 unchanged cycles. Live 3.348 us/word versus
3.690 in prior run; still above M16A 2.845. The isolated same-process ownership
kernel measures 0.549 cached versus 1.225 us/word count-based. These figures
are workload/host measurements, not a whole-emulator controlled A/B result.
Delivery 5/15 complete; no claim of full performance recovery.

## M16C resolved CRC FLAG/F2 — delivery 6/15

Autonomous CRC condition output now crosses the resolved active-low FLAG/F2
net using the CRC identity before retained ACT S3 reception. Foreign agreeing
drives, contention, floating/wrong-source inputs and changed source state are
rejected; preflight precedes inputs and pending RAM tails. Reset and explicit
reference fixtures preserve their composition. Nine new regressions include
all CRC control commands and true firmware fault recovery/capture. The existing
224-edge traversal remains; only completed-word source/receiver work is added.

Direct HP-67 schematic logic sheets 1/2 establish the barred FLAG connection
to ACT F2; Classic Notes page 55 establishes successful CRC tests asserting S3
and firmware-owned clearing. Pulse publication/release at logical word
boundaries and active High/Low output-stage modeling remain WORKING APPROXIMATION.
Exact F2 pulse width/PHI offset, voltage thresholds, physical output circuitry
and propagation remain SOURCE-BLOCKED. See
`docs/research/M16C_RESOLVED_CRC_FLAG_2026-10-25.md`. Owner M16C v2 FULL GATE GREEN; exact 8ea080b80242122062c986811e8818df63131633 integrated (delivery 6/15). Both diagnostics 12/12; live 2.910 us/word, 109.96x hardware. Host measurements do not establish a controlled performance comparison.


## M16D physical keyboard switch network — delivery 7/15

The 35 physical contacts now have explicit cathode/KC wiring and a cached,
bidirectional ideal-switch resolver supporting chords, backfeed, floating and
contention. Production press/release owns this switch bank; retained key code
and S15 remain the existing compatibility lane. No PHI hook or guessed
scan-to-code/latch timing is introduced. Exact KC bias, output circuitry and
ACT scan acquisition remain blocked. See
`docs/research/M16D_KEYBOARD_CONTACT_NETWORK_2026-10-25.md`. Owner M16D FULL GATE GREEN; exact 87e01acb9bd9ab8c3aa1441cd5c98d9e567810ba integrated (delivery 7/15). Both diagnostics 12/12 with unchanged cycles; live 2.745 us/word (116.56x).


## M16E SYNC source integrity — delivery 8/15

ACT, RAM and CRC now require sole ACT ownership of resolved SYNC before
consuming instruction qualifiers. Agreeing foreign sources and replacement
sources no longer masquerade as valid logic. Six structural composition entries
and live preflight reject before endpoint/input/tail/clock mutation. A pristine
undriven tick-zero fixture may initialize ACT output; later missing output is a
fault. No physical edge, width or delay is inferred. See
`docs/research/M16E_SYNC_SOURCE_INTEGRITY_2026-10-25.md`. Owner M16E FULL GATE GREEN; exact 61d2633d4ac58d48eb2661744048a6aabcede450 integrated (delivery 8/15). Both diagnostics 12/12 with unchanged cycles; live 2.942 us/word (108.75x), versus 2.745 previously. Performance follow-up below.


### M16E performance follow-up (same delivery)

Committed SYNC source occupancy is cached, replacing repeated count/ACT-slot
queries while retaining binary-level checks, complete diagnostics and every
receiver/source policy. Immediate, staged and joint source commits maintain the
same predicate. All 224 PHI transitions remain. Independent scan regressions and
a rotating same-process query benchmark accompany this change; the live gate
still determines production cost. See
`docs/research/M16E_SYNC_CACHE_2026-10-25.md`. Owner M16E PERF FULL GATE GREEN; exact b86849d5d0f7819fc0b6f7c1c749d7f5ff8f5491 integrated. Both diagnostics remain 12/12. Kernel cached 0.254 versus counted 0.252 us/word; live 3.023 versus 2.942 previously. No performance benefit demonstrated. M16F retires the cache and retains the source tests. This follow-up does not consume delivery 9/15.


## M16F IS window source identity — delivery 9/15

Shared normal/traced/DATA/prefix transport now validates the sole active High
IS contributor against its cell role before receiver consumption: ACT for
display/address, the independently selected ROM for return, none during idle.
Passive Low remains valid release. This closes wrong-chip/wrong-window source
substitution without comparing against expected payload bits. Exact launch/
latch/output-enable timing remains unresolved. The ineffective M16E SYNC cache
is retired while all source/staging/commit regressions remain. See
`docs/research/M16F_IS_WINDOW_OWNERS_2026-10-26.md`. Owner local gate pending.
