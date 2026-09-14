# `src/emulation/net.rs`

## Purpose
Models shared digital electrical wires with explicit drivers, high impedance, passive bias and contention.

## Why it exists
ISA/DATA and other calculator buses are electrically shared. A cycle-accurate emulator must distinguish “nobody drives”, “one side drives”, and illegal opposite simultaneous drives instead of letting Rust assignment order hide them.

## Relationships
Used by calculator backplanes and `scheduler.rs`. `device.rs` defines how devices read resolved levels and publish drives; the scheduler uses `active_drives()` to report exactly which named outputs caused a conflict.

## Responsibilities
Track named drivers, resolve net level deterministically, implement release/high-Z behavior, expose contention, and provide stable active-driver diagnostics.

## Implementation
A `BTreeMap<DriverId, Drive>` holds active outputs. Resolution scans for active high/low drives, reports contention when both exist, otherwise returns the driven level or configured bias. `active_drives()` exposes active outputs in deterministic `DriverId` order for scheduler errors and traces. Tests cover floating/pulls/release/contention and stable driver ordering.
