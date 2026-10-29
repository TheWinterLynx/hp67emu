# Architecture

## Goal

The end state is not an instruction-level calculator reimplementation. The target is a **cycle-accurate, pin-level digital-electrical simulation** of the HP-67 that executes the original machine microcode and makes the front panel a consequence of the emulated electronics.

“Electrical” here means that behaviorally relevant IC pins and shared wires are explicit nets with drivers, high-impedance states, passive bias and detectable contention. It does **not** initially mean transistor-level SPICE simulation of every analog component. Analog details are added only where they affect externally observable digital timing, card sensing, display brightness or power/reset behavior.

## Layering

```text
┌──────────────────────────────────────────────┐
│ Desktop UI / photographed HP-67             │
│ src/main.rs, app.rs, panel.rs, ui/*          │
└──────────────────────┬───────────────────────┘
                       │ physical controls / emitted light
┌──────────────────────▼───────────────────────┐
│ Machine adapter                              │
│ maps key contacts, switches and LED energy   │
└──────────────────────┬───────────────────────┘
                       │ pins / nets
┌──────────────────────▼───────────────────────┐
│ HP-67 machine composition                    │
│ src/machines/hp67/*                          │
│ ACT + ROM/RAM + display + CRC + card reader  │
└──────────────────────┬───────────────────────┘
                       │ generic electrical primitives
┌──────────────────────▼───────────────────────┐
│ Reusable emulation kernel                    │
│ src/emulation/*                              │
│ time, nets, scheduling, traces, device API   │
└──────────────────────────────────────────────┘
```

The reusable library is intentionally GUI-free. `tests/architecture_contract.rs` fails if the core starts depending on `eframe`, `egui`, image decoding or current front-panel modules.

## Timing model

The HP calculator family uses serial data paths and explicit clocking. The generic kernel therefore treats time as a monotonic integer `Tick`, not wall-clock time. The current `TwoPhaseClock` is a four-slot scaffold:

1. PHI1 asserted;
2. non-overlap dead time;
3. PHI2 asserted;
4. non-overlap dead time.

This scaffold establishes deterministic ordering but deliberately does **not** claim final HP-67 pulse widths. Measured/documented timing will calibrate the mapping from scheduler ticks to real bit times before ACT behavior relies on it.

The HP-67-related documentation shows 56-bit word timing, two-phase clocks, serial ISA/IS and DATA activity, SYNC and display-control signals. The final scheduler must be able to represent every behaviorally relevant transition inside those word times rather than executing one whole microinstruction atomically.

## Electrical net model

`src/emulation/net.rs` models a shared wire with:

- `Drive::Low`, `Drive::High`, `Drive::HighZ`;
- passive `Floating`, `PullUp` and `PullDown` bias;
- resolved `Low`, `High`, `Floating` or `Contention` state;
- named drivers so bus ownership can be audited.

This matters because ISA and DATA are shared serial buses. A functional emulator can silently overwrite a value; an electrical emulator must expose illegal simultaneous drive as a regression failure or trace anomaly.

The current resolver is a generic digital primitive. Exact HP bus polarity, passive bias and “only actively drive one state” behavior must be established from source evidence and scope traces before being encoded as HP-specific rules.

## Deterministic device scheduling

Devices obey one scheduling contract:

1. **resolve** all nets from already-committed drives;
2. **snapshot/sample** the same resolved inputs for every device;
3. **evaluate** each device for the current tick;
4. **collect/stage** proposed output drives;
5. **commit** all drives together;
6. advance time only when the scheduled event actually advances the timing coordinate.

No chip may observe another chip merely because it happened to be earlier in an implementation container. This rule is central to cycle accuracy and repeatable traces. Physical output ownership is validated once during HP-67 machine composition: each `Hp67Driver` can issue only one opaque owner token, which the device then reuses across runtime evaluations without a per-edge ownership ledger.

The generic `ElectricalScheduler` remains the model-independent reference implementation of that contract. It intentionally uses ordered maps and boxed devices for clarity and arbitrary topologies. The HP-67 production machine does **not** use that representation in its edge hot path: `src/machines/hp67/electrical.rs` implements the same contract with fixed `Hp67Net` and `Hp67Driver` indices, fixed typed pending-drive entries with slot markers, cached resolved levels and direct driver counters. Normal HP-67 ticks therefore require no map construction, no driver-name lookup, no whole-net-set re-resolution and no heap allocation. This specialization is an implementation detail only; it must never alter source-backed electrical ordering, passive bias, contention visibility or trace semantics.

## HP-67 machine composition

`src/machines/hp67/wiring.rs` contains only hardware identities and signal names for which we have source evidence. The current confirmed inventory includes:

- 1820-2530 ACT;
- 1818-0231 and 1818-0232 ROM/RAM;
- 1818-0268 ROM/display anode driver;
- 1818-0550 and 1818-0551 ROM/RAM;
- 1820-1749 display cathode driver;
- 1820-1751 card reader controller (CRC);
- 1826-0322 card-reader sense amplifier;
- 1858-0050 / CA3082 transistor array.

The first named nets include PHI1, PHI2, ISA, DATA, SYNC, RCD, STR, F1, F2 and keyboard-column lines KC1–KC5. More pins are added only when wiring and semantics are verified.

## What “cycle accurate” means for this project

The target requires all of the following:

- the original ROM words are fetched through emulated bus timing;
- ACT registers and ALU operations occur at the correct serial bit/word phases;
- branches, return stack, status and pointer state follow the hardware microarchitecture;
- RAM/ROM chips respond on the same shared buses and at the correct phases;
- display strobe/cathode/anode activity is generated by the machine, not by formatting a string;
- key presses close electrical contacts and are discovered by the emulated scan path;
- RUN/W/PRGM and power state enter through hardware-facing signals, not direct semantic state changes;
- CRC/card-reader behavior is driven by its actual interface and timing;
- logic-analyzer traces can be compared against documented or captured real-machine traces.

Instruction-level shortcuts may exist only as optional debug accelerators and must never be the source of truth for the fidelity mode.

## Display integration target

Today `classic_display.rs` converts temporary text into LED artwork. That is transitional.

The final path is:

```text
ACT/ROM0 -> ISA/STR/RCD/anode/cathode electrical activity
        -> per-segment on-time integration
        -> optical persistence / brightness
        -> photographed display renderer
```

The renderer then knows only which physical LEDs emitted how much energy during the last visual integration window. It does not know the number `0.00` as text.

## Keyboard integration target

Today the UI emits semantic `KeyAction` values. That is also transitional.

The final input path is:

```text
mouse down on photographed key
    -> physical key contact closure
    -> keyboard/cathode scan electrical state
    -> ACT/PIK/CRC-visible key information
    -> microcode decides what the key means
```

This keeps prefix handling, key timing and unusual hardware behavior inside the emulated calculator instead of the GUI.

## Multi-calculator modularity

The generic emulation layer must not know any HP-67 part number or key. Calculator-specific composition lives under `src/machines/<model>/`.

Shared chips can later move to families such as `src/chips/woodstock/` once at least two machines prove the abstraction. We intentionally avoid premature “universal calculator” interfaces before the HP-67 behavior is understood well enough to know what is truly shared.

## Current architectural debt

`src/hp67.rs` still contains the temporary live-machine bridge that runs instruction-boundary architectural execution alongside the structural serial path. That bridge remains a correctness oracle until timed ACT/RAM device state becomes authoritative; it must not survive as the source of truth in the final fidelity mode. The measured architectural fallback cost is small compared with the electrical path, so removal is driven by correctness/ownership rather than micro-optimization.

The generic map-backed `ElectricalScheduler` is retained as reusable reference infrastructure, not as the HP-67 production scheduler. The HP-67-specific dense fabric now owns the fixed topology needed for M14B and later device scheduling.


### M14A dense scheduler performance decision

The 2026-09-22 matched-observation benchmark validated the typed pending-entry + slot-marker scheduler representation: full staged PHI measured 1.221 us/word (262.05x the observed ~320 us HP-67 word time) on the validation host, versus 2.954 us/word before the representation change. The project therefore keeps the zero-copy resolved snapshot and direct typed pending entries as the M14B scheduler foundation. Further optimization should be driven by measured regressions after real DATA/RAM/STR/RCD devices are added, rather than by synthetic PHI microbenchmark tuning alone.


### M14A scheduler exit state

The 2026-09-22 full validation gate, diagnostic PAC suite and matched-PHI regression passed with composition-time driver ownership and commit/time separation enabled. The final M14A staged-PHI benchmark measured 1.171 us/word (273.27x the observed ~320 us physical word time) on the validation host, while real-firmware dual execution measured 0.955 us/word (335.24x). M14A therefore exits with the dense scheduler architecture accepted for M14B. Remaining DATA/RAM/STR/RCD edge placement and propagation details stay source-blocked and must not be inferred from this performance result.

### Electrical commit versus timing progression

The HP-67 dense fabric deliberately separates atomic electrical commit from `Tick` advancement. A commit is an observability boundary for staged drives; it is not by itself a PHI edge or a statement about elapsed physical time. The current four-transition PHI scaffold explicitly advances one tick after each PHI commit, preserving today's trace exactly. This separation leaves room for future evidence-backed zero-time settling or propagation events without corrupting the serial bit/word coordinate.


### M14B DATA phase foundation

M14B begins with a logical DATA phase engine rather than an invented electrical DATA protocol. Direct HP-67 evidence fixes the 56-bit stream phase (b2 = serial bit 0; bits 54/55 wrap to following-word b0/b1), so `data.rs` models that cross-word lifetime explicitly and can reconstruct back-to-back frames. It does not yet drive `Hp67Net::Data`: DATA polarity, passive bias, PHI launch/sample edges, propagation and physical RAM chip/address mapping remain source-blocked.

Performance is treated as an architectural invariant during this transition. New DATA work is benchmarked in isolation after the established M14A rows before it is allowed into production execution; an integration that materially regresses the validated M14A rows is rejected or redesigned rather than accepted as the price of fidelity.


### M14B live RAM DATA integration

After the fused-loop benchmark validated no material runtime regression, the desktop live machine adopted the fused logical DATA path for addresses that are actually backed by `ActRamImage`. The pre-instruction ACT/RAM payload is selected before the architectural fallback mutates state; the same structural b0..b55 loop advances DATA phase; the prior frame completes at following-word b1; and the reconstructed payload is checked against the expected RAM transfer. Ordinary non-DATA words continue through the unchanged structural fast path.

This integration is intentionally narrower than an electrical RAM model. It is a WORKING APPROXIMATION that removes the false single-word DATA assumption and exercises real firmware through the proven cross-word phase. `Hp67Net::Data` ownership/polarity, PHI launch/sample timing, propagation and physical 1818-* RAM mapping remain SOURCE-BLOCKED. CRC/card DATA ports are excluded from this RAM slice.

### M14I structural successor and flow authority

The live bridge restores oracle PC/bank/delayed-ROM/return-stack mutations before transport. `ActSerialFlowResultImage` independently derives the successor from immutable pre-instruction inputs; its preview feeds the concurrent address/bank selection and its completed b55 result becomes final authority after exact oracle comparison. Early preview and completed-word commit are scheduling/ownership approximations, not internal physical write-edge claims. Exact flow timing remains SOURCE-BLOCKED; unrelated prelude/peripheral effects still use the architectural bridge.

### M14J remaining boundary authority

Structural boundary images own P-change history, previous carry, ordinary carry reset and arithmetic condition instruction-state selection outside M14F. Explicit masks preserve M14E arithmetic carry and M14I implied-GOTO completion. Oracle-owned fields are restored before structural transport; final commit requires b55 and exact agreement. Internal physical latch edges remain SOURCE-BLOCKED.

### M14K RAM control authority

Remaining low-nibble RAM address selectors and logical installed-block clear now use independent completed-word result images. Oracle address/block changes are restored before transport. The selected DATA target remains fixed by its pre-instruction plan, and DATA destination completion remains at the following-word b0/b1 logical tail. Block clear preserves the installed/absent topology and neighbouring memory. This closes logical final-state authority without inventing physical chip mapping or a clear-block electrical sequence.

### M14L CRC control authority

CRC set/test-and-clear and ACT S3 results are independently derived from frozen pre-instruction inputs. Oracle flag/S3 mutations are restored before transport; the complete image commits after b55. Internal and external flag state stay separate, with external levels preserved during clear. This does not claim physical F2 pulse or CRC latch timing. CRC DATA payloads, buffers and card electronics retain their existing ownership.

## M14M CRC logical data authority

CRC data-port reads/writes bind an independent `ActSerialCrcDataResultImage` before b0. The live bridge restores oracle C/FIFO/READY/F7 effects before transport and compares the frozen payload/storage result before commit after b55. FIFO wrap, capacity, read duplication into both C halves and upper-C write packing preserve the existing semantic contract. Only FIFO storage and READY/F7 are applied; other control flags and external inputs retain their owners. A changed pre-state rejects commit. This is a WORKING APPROXIMATION for logical completed-word authority; it does not transport CRC bits on DATA, infer a 28-bit flux order, or extend installed-RAM phase evidence to CRC ports. CRC electrical DATA ownership/edges and sense timing remain SOURCE-BLOCKED.

## M14N complete ACT authority coverage

The flow image now restores the oracle's implied-GOTO completion latch as well as PC/bank/stack state. `ActSerialAbsentRamReadImage` isolates the existing absent-slot zero-read compatibility rule: restore oracle C before transport, compare zero result and commit only after b55. Binding rejects installed RAM and CRC read ports, repeated/late topology binding, incomplete commits and changed topology/pre-state. It does not create a responder or DATA frame for missing RAM; physical absent-address behavior remains SOURCE-BLOCKED.

The live bridge verifies the entire pre-instruction ACT contract before transport, using its immutable snapshot, prior previous-carry and flow state. Any leaked oracle mutation is a hard contextual error. Whole ACT/RAM/CRC differential coverage spans all 1024 words, six address contexts, both carry states and normal/implied-GOTO state. Installed RAM results are compared only after the documented next-word b0/b1 tail. Unknown specials and ROM self-test remain rejected. This closes semantic ownership gaps without proving internal latch edges or removing the oracle.

## M14O live authority performance measurement

`authority_benchmark_tests.rs` calls the actual live cycle rather than reconstructing a historical dual composition. Its ignored release benchmark reports real-firmware idle and labelled synthetic RAM/CRC load medians with 224-transition/execution-count invariants. Boot and warm-up are untimed; rounds rotate path order and preserve device state. Synthetic CRC fixture refill/drain cost is included and must not be interpreted as physical card throughput. Existing electrical/DATA fixture rows remain comparative references, not current live authority throughput.

### M15F clocked DATA tail scheduling

Live installed-RAM DATA no longer uses the M14C early unclocked tail or M15E
pin replay. The shared following-word b0/b1 prefix now carries display/IS and
resolved DATA through eight PHI transitions. C/RAM completion occurs after
that prefix and before binding the next instruction's inputs. The paired
transport resumes at b2, preserves the first two ROM0 samples and IS ownership,
and completes 216 more transitions. A complete live word still has 56 cells,
224 edges and 224 optional trace rows.

ACT b0/b1 execution coordinates remain result-image bookkeeping after binding;
this change does not establish internal ACT write edges. DATA output circuitry,
physical chip mapping and exact within-cell launch/sample/propagation remain
unresolved. Reference phase fixtures retain their legacy early-tail APIs, which
no longer schedule production RAM authority.

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
