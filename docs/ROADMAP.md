# Cycle-accurate HP-67 roadmap

## Integration milestone track — current

This is the active vertical integration track used by the current implementation work. The older electrical roadmap below remains the hardware-fidelity workstream and its historical M-numbers are retained for provenance; where the two tracks use the same milestone number, the integration track is authoritative for current branch/work naming.

- **M1 — complete:** real HP-67 ROM corpus.
- **M2 — complete:** ACT architectural execution.
- **M3 — complete:** real serial ROM fetch.
- **M4 — complete:** serial ADD/SUB execution image.
- **M5 — complete:** real firmware reset/power-on.
- **M6 — complete:** real firmware display initialization.
- **M7 — complete:** stable firmware idle loop.
- **M8 — complete:** physical keyboard to firmware.
- **M9 — complete:** real entered number on physical display.
- **M10 — complete:** real arithmetic through physical keyboard/firmware/display.
- **M11 — complete (2026-09-18):** all 35 direct keycodes independently cross-checked, all 105 shifted `f/g/h` dispatch paths exercised, and major RUN-mode numeric/function families exact-locked through real firmware without host calculator semantics.
- **M12 — complete (2026-09-18):** RUN/PRGM hardware flag, real PROGRAM entry, SST/BST navigation, h DEL editing, stored-program execution and RUN-mode single-step/back-step behavior are validated end-to-end through real firmware and the physical display path.
- **M13 — in progress (2026-09-19):** top-down card presentation uses one 71.1 × 11.4 mm physical card geometry; CRC external `card_present` and firmware-owned `motor_on` are regression-locked; nominal magnetic transport now advances at the documented 28 ms per 28-bit record cadence behind a separate head-active gate; CRC `buffer_ready` is transport-driven; and architectural read port `0x9B` consumes the 28-bit buffer. Head-switch geometry/first-record phase, real card-media lifecycle, CRC write port `0x99`, write-protect/write transport and lower-level sense/DATA timing remain to implement.
- **M14 — planned:** power/timing fidelity.
- **M15 — planned:** full hardware validation suite.

M11 closure evidence is recorded in `docs/research/M11_FIDELITY_AUDIT_2026-09-18.md`. M12 program-entry/execution evidence is recorded in `docs/research/M12_PROGRAM_EXECUTION_AUDIT_2026-09-18.md`; PROGRAM editing and SST/BST closure evidence is recorded in `docs/research/M12_PROGRAM_EDIT_STEP_AUDIT_2026-09-18.md`; the code-cleanup review is recorded in `docs/research/M12_CODE_AUDIT_2026-09-18.md`. M13 presentation-slice evidence is recorded in `docs/research/M13_CARD_UI_PRESENTATION_2026-09-19.md`; timed transport/read-boundary evidence is recorded in `docs/research/M13_CARD_TRANSPORT_2026-09-19.md`.

## Development rule

Each milestone is implemented in a focused branch created from the then-current `main`. After validation and merge, the next milestone branch starts from the updated `main`. We do not stack long-lived feature branches on top of one another.

## Electrical-fidelity workstream snapshot — 2026-09-16

The project has progressed vertically through several later milestones in order to validate real HP-67 firmware and the display path early. Therefore milestone numbers must not be read as a strictly completed prefix: M5/M6 have strong working slices while important M2/M3/M4 electrical details remain unfinished.

- **M0 — complete.** Generic architecture foundation, resolver, scheduler scaffold, tracing types, HP-67 wiring vocabulary and documentation contracts are in place.
- **M1 — partial.** The 56-bit convention, IS ownership, ROM/display windows, corpus provenance and several real-hardware timing observations are locked. The ACT serial-execution evidence boundary is documented explicitly. Exact/bounded PHI launch/sample relationships and formal machine-readable golden captures are still incomplete.
- **M2 — partial.** Resolve/snapshot/evaluate/commit infrastructure, clock/subphase scaffolding and contention diagnostics exist. The scheduler is not yet calibrated to exact HP-67 PHI timing and STR/RCD are not yet resolved electrical nets.
- **M3 — partial and currently the main fidelity bottleneck.** Instruction-boundary ACT semantics execute the real firmware to idle; ACT owns display serialization, ROM address serialization, ROM-word sampling and the fifteen-word display phase. The old A/B nibble snapshot is gone, and the currently executing prefetched instruction now has an explicit lifetime across the same complete b0..b55 structural word that fetches its successor. The transport refuses to replace an incomplete execution. The major missing piece is source-backed intra-word mutation of the 56-bit register/ALU/carry state rather than instruction-boundary fallback effects.
- **M4 — partial.** External ROM corpus loading, resolved serial ROM fetch and ROM0 display decode work. Physical per-part ROM/RAM devices and real RAM bus timing remain to be implemented.
- **M5 — strong structural partial.** Real firmware reaches the documented no-key idle loop, including the physical delayed-ROM landmark, without hard-coded display results. The final idle instruction is now required to complete its structural b0..b55 execution lifetime before the live machine freezes at the checkpoint. Startup still uses architectural fallback semantics for internal ACT mutation rather than a fully pin-timed chip boot.
- **M6 — strong structural partial, STR/RCD slice validated locally.** Raw ROM0 display decode, sign routing, 1820-1749 scan state, ACT-owned scan phase and physical raw-segment UI are implemented. ROM0 STR and ACT RCD are explicit downstream structural events and the cathode no longer feeds phase upstream. The local formatting/test/release/smoke gate reached idle at cycle 281 with the expected `20 20 01 00 30 00 00 0F 0F 0F 0F 0F 0F 0F 20` ROM0 sequence. Exact PHI-relative STR/RCD edges, LED on-time integration and optical persistence are still outstanding.
- **M7 — not started electrically.** UI keys are clickable but do not synthesize calculator results; the physical keyboard matrix/scan path is still absent.
- **M8 — early partial.** Architectural CRC control exists for firmware bring-up; magnetic-card electronics, motor/sense/data timing and physical CRC boundary behavior remain outstanding.
- **M9 — early partial.** Observed power-on delays and reset-visible behavior are modeled conservatively. The UI starts with the physical power switch OFF and no firmware time advances until an OFF→ON transition; full electrical power/reset qualification and state-loss behavior are still not modeled.
- **M10 — partial.** Differential/unit tests, structural bus tests and a long real-microcode boot smoke exist. The smoke now rejects any executing word that fails to span exactly one b0..b55 structural cycle. Full deterministic input replay, complete golden bit-level traces and performance validation remain outstanding.
- **M11 — intentionally not started.** No generic Woodstock extraction until HP-67 hardware behavior is proven end to end.

The current M3 boundary is precise: execution duration and external serial transport now share one 56-bit timeline; internal register/ALU/carry commit timing is the remaining architectural-to-electrical transition and must be migrated only from source-backed ACT timing.

## M0 — Architecture foundation

Branch: `foundation/cycle-accurate-architecture`

Status: implemented in this branch.

Deliverables:

- reusable `src/lib.rs` separate from the GUI binary;
- explicit simulation `Tick` and non-overlapping two-phase clock scaffold;
- pin-level net resolver with high-Z, bias and contention;
- device evaluation contract designed for snapshot/evaluate/commit scheduling;
- trace data type for future logic-analyzer regressions;
- HP-67 chip inventory and named-net vocabulary;
- initial HP-67 electrical backplane shell;
- regression that prevents GUI/image dependencies entering the core;
- regression that enforces per-source Markdown documentation;
- removal of stale vector-rendering comparison tooling.

M0 does **not** claim a functioning ACT or real HP timing yet.

## M1 — Evidence lock and golden trace format

Suggested branch: `hp67/hardware-evidence-and-traces`

Before implementing CPU semantics, lock down what we are trying to reproduce.

Deliverables:

- verified HP-67 schematic/pin map transcribed into machine documentation;
- source-backed polarity/ownership rules for ISA, DATA, SYNC, RCD and STR;
- exact or bounded PHI1/PHI2 timing from HP documentation or scope captures;
- documented 56-bit word/bit numbering convention used everywhere in code;
- ROM image provenance policy and known-good SHA-256 checksums;
- machine-readable trace format containing tick, PHI1, PHI2, SYNC, ISA, DATA, RCD, STR and optional decoded annotations;
- at least one power-on trace window and one steady display-refresh trace from real hardware or a trusted published capture.

Exit criterion: two engineers can interpret bit/word/edge numbers identically from the docs.

## M2 — Event scheduler and calibrated electrical backplane

Suggested branch: `emulation/timing-backplane`

Deliverables:

- resolve -> snapshot -> evaluate -> commit scheduler;
- calibrated two-phase clock timing;
- scheduled edge/event support where device propagation requires it;
- explicit bus ownership diagnostics and contention failures;
- trace recorder/exporter;
- golden tests for phase sequence, 56-bit word boundaries and SYNC placement independent of the HP-67 UI.

Exit criterion: a synthetic device fixture reproduces expected serial bus waveforms exactly tick-for-tick.

## M3 — ACT 1820-2530 microarchitecture

Suggested branch: `hp67/act-1820-2530`

Deliverables:

- 56-bit serial working registers used by the ACT family;
- 12-bit program counter and two-level return stack where confirmed for this ACT generation;
- status, pointer and format state required by HP-67 microcode;
- instruction decode implemented from source-backed opcode tables;
- bit-serial ALU/data movement with effects occurring on the correct bit times;
- SYNC/RCD/ISA/DATA pin behavior;
- reset/power-on state separated from GUI power state;
- per-microinstruction and per-bit trace diagnostics.

Do not implement an opcode until its semantics and timing are documented or experimentally verified. Unknown opcodes fail loudly rather than silently approximating behavior.

Exit criterion: isolated ACT instruction tests match trusted microcode documentation and bus timing.

## M4 — ROM/RAM and ROM0/display-anode devices

Suggested branch: `hp67/rom-ram-bus`

Deliverables:

- ROM image loader with explicit part/chip mapping;
- 1818-0231, 1818-0232, 1818-0550 and 1818-0551 ROM/RAM behavior;
- 1818-0268 ROM0 behavior and verified display-anode decode responsibilities;
- ISA/DATA bus ownership matching hardware timing;
- RAM read/write timing and address decode;
- branch/address fetch timing validated against traces.

Exit criterion: real HP-67 ROM contents can be fetched by the emulated ACT through the electrical bus without a direct ROM-call shortcut.

## M5 — First real microcode boot

Suggested branch: `hp67/microcode-power-on`

Deliverables:

- real ROM image attached to the machine through the chip models;
- startup sequence from electrical reset to first valid instruction fetch;
- trace comparison against published/real power-on captures;
- removal of any core-side hard-coded `0.00` behavior.

Exit criterion: the machine reaches the expected startup loop solely by executing microcode.

## M6 — Electrical display path

Suggested branch: `hp67/electrical-display`

Deliverables:

- 1820-1749 cathode driver;
- RCD reset and STR advance behavior;
- ROM0 anode drive behavior;
- sign-digit routing/transistor behavior required by the 15-position HP-67 display;
- per-LED on-time integration and optical persistence;
- UI renderer fed by physical segment intensity, not text;
- display refresh timing compared with real traces.

Exit criterion: power-on `0.00` appears because the microcode and display electronics generate it.

## M7 — Electrical keyboard and mode switches

Suggested branch: `hp67/electrical-keyboard`

Deliverables:

- physical contact matrix wiring;
- key down/up closures from the photographed UI;
- scan/keycode behavior through the actual electrical path;
- RUN/W/PRGM input through the appropriate hardware flag path;
- optional contact bounce only after base behavior is correct;
- removal of semantic `KeyAction` from the fidelity path.

Exit criterion: a digit appears only because the microcode recognized the electrically scanned key.

## M8 — CRC and magnetic card electronics

Suggested branch: `hp67/crc-card-reader`

Deliverables:

- 1820-1751 CRC behavior;
- 1826-0322 sense-amplifier-facing digital interface;
- motor/card-presence/control signals;
- bit timing and card data path;
- card file format adapter kept outside the electrical chip model;
- read/write tests against known cards/traces.

Exit criterion: card operations are microcode-driven and timing-correct at the CRC boundary.

## M9 — Power, reset and abnormal electrical cases

Suggested branch: `hp67/power-reset-fidelity`

Deliverables:

- documented startup delays/reset qualification where observable;
- power-off behavior and state loss/preservation according to hardware;
- floating/contended bus diagnostics;
- low-voltage or battery indication only where it affects machine-visible behavior.

Exit criterion: startup/shutdown traces are reproducible and no GUI-side reset shortcut is required.

## M10 — End-to-end validation and performance

Suggested branch: `hp67/cycle-accurate-validation`

Deliverables:

- ROM boot, arithmetic, modes, programming and card-reader regression corpus;
- trace checkpoints at instruction, word and selected bit boundaries;
- deterministic replay of input sequences;
- performance profiling without collapsing electrical timing;
- optional fast/debug execution mode explicitly separated from cycle-accurate mode.

## M11 — Reuse for other calculators

Only after HP-67 is working do we extract proven common devices/families.

Candidates include:

- shared Woodstock ACT behavior;
- common ROM/RAM protocol pieces;
- reusable display/keyboard scan components;
- machine profile/configuration separate from chip implementation.

The rule is “extract after two real users,” not “invent a generic abstraction before one machine works.”
