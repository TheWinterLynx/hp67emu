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
Stores the fixed HP-67 net set in `[Net; Hp67Net::COUNT]`, indexed by the tested dense `Hp67Net` representation, plus `TwoPhaseClock` and `Hp67WordTiming`. This removes generic tree lookup from every PHI/net operation while keeping the same resolved-net semantics. HP-67 probing reported that IS is loosely pulled low by internal circuitry and actively driven only high, so `Hp67Net::Isa` begins with `Bias::PullDown`. Direct page-70 captures show PHI1 and PHI2 normally high between alternating low-going pulses, so the backplane now seeds both clock outputs high in the initial interphase instead of leaving them electrically floating. Other nets without equivalent evidence remain floating.

`advance_clock_edge()` advances the same four-transition topology but publishes only the one PHI pin that physically changes on each transition: PHI1 falling, PHI1 rising, PHI2 falling, PHI2 rising. The untouched clock line remains driven at its previous level. A single monotonic transition counter is now the canonical HP-67 backplane time coordinate; PHI phase, b0..b55, digit and word index are derived exactly from that counter instead of advancing duplicate `TwoPhaseClock`, `Hp67ClockPhase` and `Hp67WordTiming` state. This changes representation only: the observable edge sequence and electrical net levels remain identical. Four temporary transitions currently make one serial bit; 56 bit times make one machine word. Pulse widths, dead time and bus launch/sample edges remain open. Tests guarantee initial evidenced idle levels, active-low non-overlap, the repeated PHI1-low / both-high / PHI2-low / both-high ordering, and exact 56-bit wrapping.
