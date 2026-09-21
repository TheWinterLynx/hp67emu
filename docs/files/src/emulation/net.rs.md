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
Active outputs are kept in a compact sorted `Vec<(DriverId, Drive)>`, which matches the tiny fan-in of calculator nets and preserves deterministic `DriverId` ordering without tree traversal on every pin transition. Cached low/high driver counts make `level()` constant-time while retaining explicit High-Z removal, passive bias and contention semantics. Tests cover floating/pulls/release/contention, replacement of an existing driver's level, cached-resolution correctness and stable driver ordering.
