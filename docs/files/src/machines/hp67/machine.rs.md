# `src/machines/hp67/machine.rs`

## Purpose
Provides the first concrete HP-67 electrical backplane shell and exposes the current 56-bit serial-word coordinate.

## Why it exists
Before ACT/ROM/display chips can be implemented, they need a common set of explicit nets, deterministic time, evidenced passive behavior and one shared serial-word position on which to interact. This module creates that composition point without pretending the complete CPU or final measured edge timing already exists.

## Relationships
`machine.rs` now owns the HP-67-specific `Hp67ElectricalFabric` from `electrical.rs`, while continuing to expose the backplane API used by structural fetch/display code. `src/machines/hp67/isa.rs` translates logical IS bits to active-high/release drive values against the passive bias installed by the dense fabric. Future ACT/ROM/RAM/display devices will stage their outputs through the same fabric instead of a map-backed generic scheduler.

## Responsibilities
Instantiate every declared HP-67 net, expose resolved levels/driving, expose the zero-copy dense begin-evaluation/stage/commit scheduler boundary for M14B devices, maintain simulation tick, model the evidenced weak-low idle behavior of IS/ISA, provide a temporary PHI1/PHI2 scaffold with the directly observed HP-67 active-low pin polarity, and keep that scaffold aligned with the project `b0..b55` machine-word convention.

## Implementation
Stores one `Hp67ElectricalFabric`, whose net/driver state is fixed-size and directly indexed. This removes generic driver lookup, vector mutation and per-tick map allocation from the production electrical path while keeping explicit `HighZ`, passive bias and contention semantics. HP-67 probing reported that IS is loosely pulled low by internal circuitry and actively driven only high, so `Hp67Net::Isa` begins with pull-down bias. Direct page-70 captures show PHI1 and PHI2 normally high between alternating low-going pulses, so the fabric seeds both ACT clock outputs high in the initial interphase. Other nets without equivalent evidence remain floating.

`advance_clock_edge()` advances the same four-transition topology but publishes only the one PHI pin that physically changes on each transition: PHI1 falling, PHI1 rising, PHI2 falling, PHI2 rising. The untouched clock line remains driven at its previous level. PHI updates address the ACT's dense driver slot directly. A single monotonic transition counter in the dense fabric is now the canonical HP-67 backplane time coordinate; PHI phase, b0..b55, digit and word index are derived exactly from that counter instead of advancing duplicate `TwoPhaseClock`, `Hp67ClockPhase` and `Hp67WordTiming` state. This changes representation only: the observable edge sequence and electrical net levels remain identical. Four temporary transitions currently make one serial bit; 56 bit times make one machine word. Pulse widths, dead time and bus launch/sample edges remain open. Tests guarantee initial evidenced idle levels, active-low non-overlap, the repeated PHI1-low / both-high / PHI2-low / both-high ordering, and exact 56-bit wrapping.
