# `src/machines/hp67/machine.rs`

## Purpose
Provides the first concrete HP-67 electrical backplane shell and exposes the current 56-bit serial-word coordinate.

## Why it exists
Before ACT/ROM/display chips can be implemented, they need a common set of explicit nets, deterministic time, evidenced passive behavior and one shared serial-word position on which to interact. This module creates that composition point without pretending the complete CPU or final measured edge timing already exists.

## Relationships
Builds on generic `Net`/`TwoPhaseClock` primitives plus HP-67-specific `Hp67Net` and `Hp67WordTiming`. `src/machines/hp67/isa.rs` translates logical IS bits to active-high/release drive values against the passive bias installed here. Future ACT/ROM/RAM/display devices and scheduler integration will live around this backplane.

## Responsibilities
Instantiate every declared HP-67 net, expose resolved levels/driving, maintain simulation tick, model the evidenced weak-low idle behavior of IS/ISA, provide a temporary PHI1/PHI2 scaffold with the directly observed HP-67 active-low pin polarity, and keep that scaffold aligned with the project `b0..b55` machine-word convention.

## Implementation
Stores a `BTreeMap<Hp67Net, Net>`, `TwoPhaseClock` and `Hp67WordTiming`. HP-67 probing reported that IS is loosely pulled low by internal circuitry and actively driven only high, so `Hp67Net::Isa` begins with `Bias::PullDown`; nets without equivalent evidence remain floating. `advance_clock` drives PHI1/PHI2 through a named scaffold driver and advances one timing subphase. The generic clock says which phase is active; the HP-67 backplane maps an active phase to a low-going pin pulse because the direct page-70 captures show both PHI lines normally high with alternating low pulses. Four temporary subphases currently make one serial bit; 56 bit times make one machine word. Pulse widths, dead time and bus launch/sample edges remain open. Tests guarantee initial net biases, active-low non-overlap, the repeated PHI1-low / both-high / PHI2-low / both-high ordering, and exact 56-bit wrapping.
