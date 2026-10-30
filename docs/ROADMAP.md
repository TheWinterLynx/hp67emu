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
- **M13 — complete (2026-09-20):** one physical `Hp67MagneticCard` owns independent logical Track 1/Track 2 media and is selected end-for-end; `.hpp`, `.hp67raw` and `.hp67card` host adapters remain outside the reader electronics; real firmware owns card-present→motor→head/startup handshake, two 28-bit CRC read buffers, two write buffers, `0x9B`/`0x99`, write-protect/F7 and the complete 34-record lifecycle; transport speed supports the documented 95–105% range without accumulated rounding drift; firmware write→eject→reinsert→read round trips are regression-locked for program headers 3/4 and W/DATA headers 1/2; the UI reader follows live record position and real firmware `Crd` drives same-card opposite-end reinsertion, with End2 rendered as a true 180° in-plane rotation; blank cards can be inserted host-side and written entirely through firmware; `Ctrl+S` persists returned media as lossless `.hp67card` or, when both tracks exist, `.hp67raw`. The physical self-clocking Zero/One flux-pair model is implemented separately. Three lower-level electrical facts remain deliberately source-blocked rather than guessed: HP-67/97 intra-record 28-bit serialization order, exact insertion-to-head geometry/acceleration, and sense-amplifier/PHI-relative bit timing. The built-in SD-14A artwork also remains unbound to magnetic media until a redistributable verified Moon Rocket Lander card image is inspected.
- **M14 — in progress:** M14A locks the source-backed timing/evidence contract; M14B places the continuous 56-bit DATA phase in the shared structural word; M14C moves installed-RAM ACT read/write destinations onto reconstructed DATA frames; M14D (validated locally 2026-09-24, diagnostic 12/12) makes ADD/SUB structural-result authoritative for final A/B/C/carry; M14E (validated locally 2026-09-25, diagnostic 12/12) extends that authority to all 32 arithmetic opcodes; M14F (validated locally 2026-09-26, diagnostic 12/12) moves the focused P/status condition family onto completed-word structural authority; M14G (validated locally 2026-09-27) moves the focused non-arithmetic register-transfer family (working-stack clear, M1/M2 transfers, stack transfers, F/A0, key-to-A, A rotate and load-constant C[P]) onto its own completed-word structural result image; M14H (validated locally 2026-09-28) moves decimal/hex, DISPLAY enable, 14-digit display and RAM-address selection onto a completed-word mode/latch image while separating the startup display-only override from the real ACT snapshot. The architectural executor remains only the semantic oracle and temporary owner of still-unmigrated effects. Exact DATA polarity/passive bias, PHI-relative launch/sample/commit edges, per-device propagation, physical 1818-* RAM partitioning and true intra-word ACT register/control mutation remain source-blocked or pending.
- **M14I — validated locally and integrated (2026-10-03):** independent successor-fetch preview and completed-word authority for PC, bank, delayed ROM and return stack. Exact internal flow timing remains source-blocked. See `docs/research/M14I_CONTROL_FLOW_AUTHORITY_2026-10-03.md`.
- **M14J — validated locally and integrated (2026-10-03):** remaining P-change/previous-carry/reset authority and arithmetic condition-latch authority, with exclusive M14E/F/I ownership. See `docs/research/M14J_BOUNDARY_STATE_AUTHORITY_2026-10-03.md`.
- **M14K — validated locally and integrated (2026-10-04):** remaining low-nibble RAM selectors and logical installed-block clear authority, preserving M14C cross-word DATA and CRC boundaries. See `docs/research/M14K_RAM_CONTROL_AUTHORITY_2026-10-04.md`.
- **M14L — validated locally and integrated (2026-10-04):** CRC set/test-and-clear and ACT S3 completed-word authority from immutable inputs. Exact F2/internal CRC edges remain source-blocked. See `docs/research/M14L_CRC_CONTROL_AUTHORITY_2026-10-04.md`.
- **M14M — validated locally and integrated (2026-10-04):** logical CRC data-port payload/FIFO completed-word authority. Electrical CRC DATA and magnetic sense timing remain source-blocked. See `docs/research/M14M_CRC_DATA_AUTHORITY_2026-10-04.md`.
- **M14N — validated locally and integrated (2026-10-04):** full ACT pre-transport restoration guard, implied-GOTO latch restoration, isolated absent-RAM compatibility and whole-state differential coverage. See `docs/research/M14N_AUTHORITY_COVERAGE_2026-10-04.md`.
- **M14O — validated locally and integrated (2026-10-04):** actual live authority throughput benchmark, continuous RAM/CRC stress fixtures and corrected historical benchmark scope. See `docs/research/M14O_LIVE_AUTHORITY_BENCHMARK_2026-10-04.md`.
- **M14P — validated locally and integrated (2026-10-04):** two-end Moving Average corpus load and nine official/shared functional checkpoints through physical keys, real firmware and raw display. HP-97 behavioral corroboration does not prove HP-67 electrical equivalence. See `docs/research/M14P_OFFICIAL_PAC_ACCEPTANCE_2026-10-04.md`.
- **M14Q — validated locally and integrated (2026-10-04):** official Moving Average data write, native media persistence, fresh firmware boot, program/data reload and recovered rolling mean. See `docs/research/M14Q_OFFICIAL_PAC_DATA_RECOVERY_2026-10-04.md`.
- **M14R — validated locally and integrated (2026-10-04):** nine/ten-point Moving Average data-pass boundary, End2 primary preservation and fresh-machine recovery; derived fixtures from manual instructions, not printed HP examples. See `docs/research/M14R_DATA_CARD_PASS_BOUNDARY_2026-10-04.md`.
- **M14S — implemented, local validation pending (2026-10-04):** official Polynomial Evaluation temperature example, negative and scientific inputs, and eight raw-display checkpoints. See `docs/research/M14S_POLYNOMIAL_ACCEPTANCE_2026-10-04.md`.
- **M15 — planned:** full hardware validation suite.

M11 closure evidence is recorded in `docs/research/M11_FIDELITY_AUDIT_2026-09-18.md`. M12 program-entry/execution evidence is recorded in `docs/research/M12_PROGRAM_EXECUTION_AUDIT_2026-09-18.md`; PROGRAM editing and SST/BST closure evidence is recorded in `docs/research/M12_PROGRAM_EDIT_STEP_AUDIT_2026-09-18.md`; the code-cleanup review is recorded in `docs/research/M12_CODE_AUDIT_2026-09-18.md`. M13 presentation-slice evidence is recorded in `docs/research/M13_CARD_UI_PRESENTATION_2026-09-19.md`; timed transport/read-boundary evidence is recorded in `docs/research/M13_CARD_TRANSPORT_2026-09-19.md`. Final closure, including the real-corpus Teenix compatibility fixes and the successful local formatting/warnings/all-targets/release gate on 2026-09-20, is recorded in `docs/research/M13_CLOSURE_AUDIT_2026-09-19.md`. M14D's ADD/SUB final-state authority and remaining source-blocked ACT timing boundary are recorded in `docs/research/M14D_SERIAL_ARITHMETIC_AUTHORITY_2026-09-24.md`; M14E's extension to the complete arithmetic opcode family is recorded in `docs/research/M14E_SERIAL_REGISTER_AUTHORITY_2026-09-25.md`; M14F's P/status authority boundary is recorded in `docs/research/M14F_P_STATUS_AUTHORITY_2026-09-25.md`; M14G's special-register boundary is recorded in `docs/research/M14G_SPECIAL_REGISTER_AUTHORITY_2026-09-27.md`; M14H's mode/latch boundary is recorded in `docs/research/M14H_MODE_LATCH_AUTHORITY_2026-09-27.md`.

## Development rule

Each milestone is implemented in a focused branch created from the then-current `main`. After validation and merge, the next milestone branch starts from the updated `main`. We do not stack long-lived feature branches on top of one another.

## Electrical-fidelity workstream snapshot — 2026-09-16

The project has progressed vertically through several later milestones in order to validate real HP-67 firmware and the display path early. Therefore milestone numbers must not be read as a strictly completed prefix: M5/M6 have strong working slices while important M2/M3/M4 electrical details remain unfinished.

- **M0 — complete.** Generic architecture foundation, resolver, scheduler scaffold, tracing types, HP-67 wiring vocabulary and documentation contracts are in place.
- **M1 — partial.** The 56-bit convention, IS ownership, ROM/display windows, corpus provenance and several real-hardware timing observations are locked. The ACT serial-execution evidence boundary is documented explicitly. Exact/bounded PHI launch/sample relationships and formal machine-readable golden captures are still incomplete.
- **M2 — partial.** Resolve/snapshot/evaluate/commit infrastructure, clock/subphase scaffolding and contention diagnostics exist. The scheduler is not yet calibrated to exact HP-67 PHI timing and STR/RCD are not yet resolved electrical nets.
- **M3 — partial and currently the main fidelity bottleneck.** Instruction-boundary ACT semantics execute the real firmware to idle; ACT owns display serialization, ROM address serialization, ROM-word sampling and the fifteen-word display phase. The old A/B nibble snapshot is gone, and the currently executing prefetched instruction now has an explicit lifetime across the same complete b0..b55 structural word that fetches its successor. M14E makes the structural result image authoritative for final A/B/C/carry across all 32 arithmetic opcodes; M14F, M14G and M14H progressively move P/status, non-arithmetic register transfers and selected mode/latch state onto completed-word structural authority. The transport refuses to replace an incomplete execution. The major missing piece remains source-backed intra-word mutation/observability on real bit/PHI timing plus migration of the remaining ACT control/state families.
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

## M14T display dwell branch

No further Pac expansion is planned under the owner's instruction. M14S was owner-validated and integrated at 5f7c999662d1e7dc682913cf5aad5f4d4ed6f6e9. M14T integrates nominal 40/30 us segment dwell over complete live scans; owner execution/performance gates pending. See research/M14T_DISPLAY_DWELL_2026-10-10.md for evidence boundaries and subsequent source-dependent slices.

## M14U completed-scan optical boundary

M14T owner FULL GATE GREEN, integrated at a18c38998e3b9751db0ff336edefaa7b0e46f260. M14U projects nominal dwell to physical emitters and connects the completed snapshot to the actual panel renderer. Retains optical artwork gain and duplicate-slot compatibility without claiming physical energy or exact control edges. Owner full gate pending. No additional Pac acceptance suites.

## M14V optical publication epochs

M14U owner FULL GATE GREEN and integrated at fd56b529beeb9009768e37a52e012e7bf10ecad7. M14V prevents stale completed optical output from reappearing across firmware DISPLAY control. Discards the interrupted scan and waits for the following full scan, preserving raw transport and dwell. Owner local gate pending; no Pac expansion.


## M14W structural bus trace

M14V owner FULL GATE GREEN and integrated at 5dfdbffbd301eba8a923d5883e356258b5382ff0. M14W adds opt-in IS/PHI transition capture and ordinal-tick TSV export on the diagnostic structural display/fetch path, keeping both production loops untouched. Complete-scan differential and missing-ROM prefix regressions guard the diagnostic twin. Local owner gate pending. Subsequent work can connect live authority traffic and measured captures to this observation contract; electrical timing gaps remain explicit. No further Pac suites.


## M14X live bus capture

M14W owner FULL GATE GREEN and integrated at c0aeca92b4d7179e7320871c43f08e5c5358e250. M14X exports bounded actual live power-on firmware IS/PHI traffic via native CLI, including logical DATA transport parity, while retaining the ordinary loops. Physical recording alignment is the next comparison task. No calibrated timing or new Pac acceptance is claimed. Owner local gate pending.


## M14Y ordinal trace comparison

M14X owner FULL GATE GREEN, integrated at a4c8ffa8d7e82b982bdabeda6757285e639fd47c. M14Y imports native and normalized pin-only TSV, compares explicitly aligned suffixes and reports mismatches/coverage/provenance. Physical recordings remain unavailable; no hardware accuracy claim is inferred from self-comparison. Native validation covers match and deliberate mismatch. Owner local gate pending; no Pac expansion.


## M14Z IS word decode

M14Y owner FULL GATE GREEN, integrated at 4d673754661a5c20f3a7788d5af1a19de075ac8c. M14Z decodes complete native-observation IS words into display/address/fetched-instruction values, preserving provenance and coverage and rejecting incompatible observations. Exact physical edge claims remain blocked. Owner local gate pending; no new Pac tests.

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
`docs/research/M16F_IS_WINDOW_OWNERS_2026-10-26.md`. Owner M16F FULL GATE GREEN; exact 2186b6bc1e524fe784e51ef2051224109b80b4a2 integrated (delivery 9/15). Both diagnostics 12/12 unchanged cycles; live 2.885 us/word (110.92x), full structural 1.314, PHI 0.347, autonomous RAM 1.098. Separate-run measurements do not isolate change cost.


## M16G resolved CRC MOTOR — delivery 10/15

Independent CRC control now publishes a named active-low MOTOR output. Strict
resolved sampling, rather than the latch parameter, supplies the production
transport motor gate. Preflight preserves faults before inputs/clocks/tails;
changed outputs publish at the existing completed-word boundary. Prior-word
interval sampling, head/startup policy and record cadence remain unchanged.
Capture v1.2 exports MOTOR; absent legacy columns stay unknown. Output stage,
exact motor/PHI delay and analog mechanics remain unresolved. See
`docs/research/M16G_RESOLVED_MOTOR_2026-10-26.md`. Owner gate pending.


## M16G accepted / M16H prepared — 2026-10-29

Owner M16G v2 FULL GATE GREEN at b2fc0de22c464b3a3eba56f1a3d41f8556b4e5e6 is integrated unchanged: delivery 10/15, both diagnostic builds 12/12 with unchanged cycles. Live median 2.971 us/word (107.72x), approximately 3.0% higher time than M16F; no isolated attribution.

M16H (delivery 11/15, validation pending) routes card-insertion MTRS through a checked External source, resolved active-low pin, atomic CRC input freeze and real-firmware F2/MOTOR path. Insertion/withdrawal/power-off/exit update the source, and the reference mirror is comparison-only for this production input. Captures append MTRS at index 15 and preserve missing observations as unknown. Existing head/startup/record policies and all 56 cells/224 transitions remain. High/open normalization and word-frozen reception are working approximations; physical voltage, bounce, exact contact/PHI edges, HDS geometry and magnetic timing remain source-blocked. See research/M16H_RESOLVED_MTRS_2026-10-29.md and PIN_CAPTURE_v1.3.md.


## M16H accepted / M16I prepared — 2026-10-29

M16H v2 owner FULL GATE GREEN b68f97facc5b3c900eef5a5e03269112065d5d91 is integrated unchanged: delivery 11/15, both 12/12 diagnostics and cycle counts preserved. Live median 2.777 us/word (115.25x), best 2.748/worst 2.799.

M16I, delivery 12/15 prepared/pending validation, adds resolved HDS/WPS inputs and an independent retained CRC head/write receiver. Head closure samples the selected physical corner's resolved write contact; owned transport consumes the receiver instead of metadata for production write inhibit. Grounded WPS permits the documented service bypass, and held-ground reuse requires a real open/reclose. Word/card preflight, atomic source publication, power-off/exit rearming, both protected-end firmware tests and the full 34-record roundtrip are covered. Eighteen pins retain earlier indices; absent capture observations stay unknown. The 56-cell/224-transition schedule and existing logical head/startup/record policies remain. High/open normalization, reset arming and joint publication are explicit approximations; exact head geometry, contact/PHI edges, voltage/bounce, magnetic sense and DATA timing remain source-blocked. See docs/research/M16I_READER_HEAD_WPS_2026-10-29.md and docs/PIN_CAPTURE_v1.4.md.


## M16I accepted / M16J prepared — 2026-10-30

M16I owner FULL GATE GREEN 774f0eb408e98da9469204ec7f350f99fa4bd6e6 is integrated unchanged: delivery 12/15, both 12/12 diagnostics and cycle counts preserved. Live 2.703 us/word (118.37x), best 2.674/worst 2.728.

M16J, delivery 13/15 prepared/pending validation, adds the physical grounded PROGRAM selector /PGMSW, resolved CRC input reception, atomic two-contact freeze and mechanical selector persistence across electronic reset. Firmware retains mode/S11/program-memory semantics. Existing M12 and full card-write/read acceptance use the same physical selector API. Nineteen capture columns preserve earlier indices and missing observations remain unknown. Word scheduling is unchanged at 56 cells/224 transitions. High/RUN normalization, exact CRC/PHI edges, voltage/bounce, RAM mapping, CRC/magnetic serialization and other existing timing gaps remain explicit. See docs/research/M16J_RESOLVED_PROGRAM_SWITCH_2026-10-30.md and docs/PIN_CAPTURE_v1.5.md.
