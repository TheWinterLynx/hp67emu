# `src/machines/hp67/machine.rs`

## Purpose
Provides the first concrete HP-67 electrical backplane shell and exposes the current 56-bit serial-word coordinate.

## Why it exists
Before ACT/ROM/display chips can be implemented, they need a common set of explicit nets, deterministic time, evidenced passive behavior and one shared serial-word position on which to interact. This module creates that composition point without pretending the complete CPU or final measured edge timing already exists.

## Relationships
`machine.rs` now owns the HP-67-specific `Hp67ElectricalFabric` from `electrical.rs`, while continuing to expose the backplane API used by structural fetch/display code. `src/machines/hp67/isa.rs` translates logical IS bits to active-high/release drive values against the passive bias installed by the dense fabric. Future ACT/ROM/RAM/display devices will stage their outputs through the same fabric instead of a map-backed generic scheduler.

## Responsibilities
Instantiate every declared HP-67 net, expose resolved levels/driving, expose the zero-copy dense begin-evaluation/stage/commit scheduler boundary plus one-time driver ownership for M14B devices, maintain simulation tick, model the evidenced weak-low idle behavior of IS/ISA, provide a temporary PHI1/PHI2 scaffold with the directly observed HP-67 active-low pin polarity, and keep that scaffold aligned with the project `b0..b55` machine-word convention.

## Implementation
Stores one `Hp67ElectricalFabric`, whose net/driver state is fixed-size and directly indexed. This removes generic driver lookup, vector mutation and per-tick map allocation from the production electrical path while keeping explicit `HighZ`, passive bias and contention semantics. HP-67 probing reported that IS is loosely pulled low by internal circuitry and actively driven only high, so `Hp67Net::Isa` begins with pull-down bias. Direct page-70 captures show PHI1 and PHI2 normally high between alternating low-going pulses, so the fabric seeds both ACT clock outputs high in the initial interphase. Other nets without equivalent evidence remain floating.

`advance_clock_edge()` advances the same four-transition topology but publishes only the one PHI pin that physically changes on each transition: PHI1 falling, PHI1 rising, PHI2 falling, PHI2 rising. The untouched clock line remains driven at its previous level. PHI updates address the ACT's dense driver slot directly. A single monotonic transition counter in the dense fabric is now the canonical HP-67 backplane time coordinate; PHI phase, b0..b55, digit and word index are derived exactly from that counter instead of advancing duplicate `TwoPhaseClock`, `Hp67ClockPhase` and `Hp67WordTiming` state. This changes representation only: the observable edge sequence and electrical net levels remain identical. Four temporary transitions currently make one serial bit; 56 bit times make one machine word. Pulse widths, dead time and bus launch/sample edges remain open. Tests guarantee initial evidenced idle levels, active-low non-overlap, the repeated PHI1-low / both-high / PHI2-low / both-high ordering, and exact 56-bit wrapping.


`commit_staged()` and `advance_clock_edge()`/timing progression are intentionally separate. The backplane no longer equates every electrical commit with one PHI transition.

## M15B IS sample policy

Exposes the fabric's cached IS drive-validity check and exceptional driver mask
to structural transport. These methods distinguish same-level active-high
overlap and invalid active-low drive from the resolved logic level. They add
no timing progression, passive bias or electrical DATA behavior.

## M15E DATA bridge surface

Exposes one-observation DATA source handoff and cached active-contributor counts
to the phase engine. No passive DATA bias, chip selection or time progression
is inferred. Core floating resolution and existing PHI topology are unchanged.

## M15G resolved display controls

Forwards `active_driver_count(net)` to the fabric so the display level bridge
can reject same-level extra contributors as well as resolved contention.

The pulse bridge is a WORKING APPROXIMATION: STR is Low in b7 and High in
other cells; RCD shares that cell only in slot 15. Publishing before the four
scaffold PHI transitions and restoring High at b8 are not physical edge/width
claims. No extra tick or cathode-to-ACT feedback is added.

## M15H provisional PHI receiver scheduling

Adds internal launch/sample phase predicates combining ordinal phase with both
resolved PHI levels. Phase count alone cannot authorize a receiver if a clock
pin is unresolved. Neither predicate advances time or changes electrical bias.

This is a WORKING APPROXIMATION with a source-backed stable interval, not
a measured latch-edge/propagation claim. Full words remain 56 cells and 224
transitions. See `docs/research/M15H_PHI_RECEIVER_SCHEDULING_2026-10-16.md`.

## M15I atomic serial publication

Forwards the shared serial image to the dense fabric without advancing time.
The transport validates the launch phase before this internal bridge is called;
PHI stepping and receiver checkpoint validation remain separate operations.

The launch boundary and PHI1-rising sample checkpoint retain M15H
WORKING APPROXIMATION status. No propagation delay, physical output stage or
unknown latch edge is promoted to proven hardware behavior.

## M15J five-ROM composition

Forwards typed aggregate/five-ROM IS launch images to the fabric.
`driver_drive(net, driver)` exposes committed contributions for read-only ownership
validation, including distinguishing the live physical sources from the aggregate
reference slot. Neither operation adds timing or changes resolver semantics.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.


## M15R autonomous RAM DATA identity

Autonomous RAM reads now publish through Hp67Driver::RamAggregate, a distinct
logical aggregate slot appended after all existing driver indices. Reference
ROM/RAM fixtures retain StructuralRomResponder. This is device identity, not a
new physical chip map. The shared source commit selects the RAM owner without
clearing foreign contributions, so a rogue ROM/reference drive remains visible
and can cause contention. No extra time edge or intermediate resolution.

The DATA path remembers autonomous receive ownership across body and next-word
b0/b1 tail. Its sole-owner guard requires RamAggregate for live reads; replacing
it with another named source at the same valid level fails before receive state
changes. Reset reconstructs the path and the RAM-capable transport binds its
owner again. The standalone RAM fixture also uses this identity. Exact physical
DATA output stage/polarity, per-IC mapping and PHI edges remain unresolved.
See `docs/research/M15R_RAM_DATA_DRIVER_2026-10-20.md`.


### M15Y pin-derived display circuit (delivery 2/15)

Exposes the crate-internal joint display-control commit to structural composition. It delegates to the fixed-topology fabric and does not change time or erase external drivers.
The b7 checkpoint, one-cell pulse bridge, whole-word cathode application and nominal segment dwell remain WORKING APPROXIMATION. See `docs/research/M15Y_DISPLAY_CIRCUIT_2026-10-23.md`.


### M15Z full-pin capture and physical observation audit — delivery 3/15

Opt-in pin recording observes committed immediate drives, joint IS/SYNC/DATA and display bundles, staged commit success and all PHI transitions. Each saved row preserves every declared net level and named high/low ownership mask. Delta order distinguishes multiple publications at one ordinal tick; a bundle has one snapshot. Disabled capture performs no pin enumeration/allocation. Overflow is explicit and cannot export truncated success. Power reset replaces the backplane and clears the recorder.
Ordinal ticks are never converted to physical time. See `docs/PIN_CAPTURE_v1.0.md` and `docs/research/M15Z_ELECTRICAL_CAPTURE_2026-10-23.md`.


### M16A PHI windows and compiled-out recorder — delivery 4/15

The pin-capture feature compiles in both recorder storage and every publication hook. Without it the backplane contains only the electrical fabric and no record_pins calls/method, rather than checking an optional observer at every PHI edge. Public start/take APIs explicitly refuse/return None in normal builds. Matrix regressions lock default storage/absence of side effects and diagnostic capture/atomic controls. All 224 PHI transitions, resolved nets and device work remain intact.
See `docs/PIN_CAPTURE_v1.1.md` and `docs/research/M16A_PHI_WINDOWS_2026-10-24.md`. The owner gate tests both feature configurations.


## M16B clock ownership integrity — delivery 5/15

Hp67ClockFault exposes ordinal tick, net, expected/resolved level, ACT contribution and exceptional active-driver mask. validate_clock checks both ownership and the active-low phase image; try_advance_clock_edge refuses before mutation. The existing raw scaffold stepping API remains available for low-level fixtures; production compositions validate before entry and shared launch/sample guards require sole ACT ownership. No widths, dead time or latch edges are calibrated.
