# Cycle-accurate HP-67 roadmap

## Development rule

Each milestone is implemented in a focused branch created from the then-current `main`. After validation and merge, the next milestone branch starts from the updated `main`. We do not stack long-lived feature branches on top of one another.

## Current status snapshot — 2026-09-16

The project has progressed vertically through several later milestones in order to validate real HP-67 firmware and the display path early. Therefore milestone numbers must not be read as a strictly completed prefix: M5/M6 have strong working slices while important M2/M3/M4 electrical details remain unfinished.

- **M0 — complete.** Generic architecture foundation, resolver, scheduler scaffold, tracing types, HP-67 wiring vocabulary and documentation contracts are in place.
- **M1 — partial.** The 56-bit convention, IS ownership, ROM/display windows, corpus provenance and several real-hardware timing observations are locked. Exact/bounded PHI launch/sample relationships and formal machine-readable golden captures are still incomplete.
- **M2 — partial.** Resolve/snapshot/evaluate/commit infrastructure, clock/subphase scaffolding and contention diagnostics exist. The scheduler is not yet calibrated to exact HP-67 PHI timing and STR/RCD are not yet resolved electrical nets.
- **M3 — partial and currently the main fidelity bottleneck.** Instruction-boundary ACT semantics are strong enough to execute the real firmware to idle; ACT owns display serialization, ROM address serialization, ROM-word sampling and the fifteen-word display phase. The old word-start A/B nibble snapshot has now been removed from production transport: b0..b7 read the current ACT state at each bit cell. The major missing piece is still true intra-word 56-bit serial register/ALU evolution with source-backed pin-edge timing.
- **M4 — partial.** External ROM corpus loading, resolved serial ROM fetch and ROM0 display decode work. Physical per-part ROM/RAM devices and real RAM bus timing remain to be implemented.
- **M5 — strong structural partial.** Real firmware reaches the documented no-key idle loop, including the physical delayed-ROM landmark, without hard-coded display results. Startup is still a hybrid of architectural ACT execution and structural electrical transport rather than a fully pin-timed chip boot.
- **M6 — strong structural partial, STR/RCD slice validated locally.** Raw ROM0 display decode, sign routing, 1820-1749 scan state, ACT-owned scan phase and physical raw-segment UI are implemented. ROM0 STR and ACT RCD are explicit downstream structural events and the cathode no longer feeds phase upstream. The local formatting/test/release/smoke gate reached idle at cycle 281 with the expected `20 20 01 00 30 00 00 0F 0F 0F 0F 0F 0F 0F 20` ROM0 sequence. Exact PHI-relative STR/RCD edges, LED on-time integration and optical persistence are still outstanding.
- **M7 — not started electrically.** UI keys are clickable but do not synthesize calculator results; the physical keyboard matrix/scan path is still absent.
- **M8 — early partial.** Architectural CRC control exists for firmware bring-up; magnetic-card electronics, motor/sense/data timing and physical CRC boundary behavior remain outstanding.
- **M9 — early partial.** Observed power-on delays and reset-visible behavior are modeled conservatively. The UI now starts with the physical power switch OFF and no firmware time advances until an OFF→ON transition; full electrical power/reset qualification and state-loss behavior are still not modeled.
- **M10 — partial.** Differential/unit tests, structural bus tests and a long real-microcode boot smoke exist. Full deterministic input replay, complete golden bit-level traces and performance validation remain outstanding.
- **M11 — intentionally not started.** No generic Woodstock extraction until HP-67 hardware behavior is proven end to end.

The next high-value fidelity work is M3/M2: make the ACT registers and ALU themselves evolve inside the 56-bit word. The transport is now ready to observe changing A/B values bit by bit; the next step must define those changes from source-backed serial ACT timing rather than inventing PHI edges.

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
