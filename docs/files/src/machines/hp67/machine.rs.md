# `src/machines/hp67/machine.rs`

## Purpose
Provides the first concrete HP-67 electrical backplane shell.

## Why it exists
Before ACT/ROM/display chips can be implemented, they need a common set of explicit nets and deterministic time on which to interact. This module creates that composition point without pretending the CPU already exists.

## Relationships
Builds on generic `Net`/`TwoPhaseClock` primitives and HP-67-specific `Hp67Net`. Future chip instances and scheduler integration will live around this backplane.

## Responsibilities
Instantiate every declared HP-67 net, expose resolved levels/driving, maintain simulation tick, and currently provide a temporary PHI1/PHI2 scaffold.

## Implementation
Stores a `BTreeMap<Hp67Net, Net>` plus `TwoPhaseClock`. `advance_clock` drives PHI1/PHI2 through a named scaffold driver. The scaffold is explicitly temporary: final clock behavior moves into ACT/timing logic after measured timing is established. Tests guarantee initial floating nets and non-overlapping phases.
