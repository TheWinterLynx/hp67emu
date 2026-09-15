# `src/machines/hp67/machine.rs`

## Purpose
Provides the first concrete HP-67 electrical backplane shell and exposes the current 56-bit serial-word coordinate.

## Why it exists
Before ACT/ROM/display chips can be implemented, they need a common set of explicit nets, deterministic time and one shared serial-word position on which to interact. This module creates that composition point without pretending the CPU or measured edge timing already exists.

## Relationships
Builds on generic `Net`/`TwoPhaseClock` primitives plus HP-67-specific `Hp67Net` and `Hp67WordTiming`. Future chip instances and scheduler integration will live around this backplane.

## Responsibilities
Instantiate every declared HP-67 net, expose resolved levels/driving, maintain simulation tick, provide a temporary PHI1/PHI2 scaffold, and keep that scaffold aligned with the project `b0..b55` machine-word convention.

## Implementation
Stores a `BTreeMap<Hp67Net, Net>`, `TwoPhaseClock` and `Hp67WordTiming`. `advance_clock` drives PHI1/PHI2 through a named scaffold driver and advances one timing subphase. Four temporary subphases currently make one serial bit; 56 bit times make one machine word. This is a structural scheduler convention only: final pulse durations and ISA/DATA/SYNC edge placement remain blocked on direct evidence. Tests guarantee initial floating nets, non-overlapping phases and exact 56-bit wrapping.
