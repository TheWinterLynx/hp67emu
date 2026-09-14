# `src/emulation/net.rs`

## Purpose
Models shared digital electrical wires with explicit drivers, high impedance, passive bias and contention.

## Why it exists
ISA/DATA and other calculator buses are electrically shared. A cycle-accurate emulator must distinguish “nobody drives”, “one side drives”, and illegal opposite simultaneous drives instead of letting Rust assignment order hide them.

## Relationships
Used by calculator backplanes and future chip models. `device.rs` defines how devices will read resolved levels and publish drives.

## Responsibilities
Track named drivers, resolve net level deterministically, implement release/high-Z behavior and expose contention for diagnostics/tests.

## Implementation
A `BTreeMap<DriverId, Drive>` holds active outputs. Resolution scans for active high/low drives, reports contention when both exist, otherwise returns the driven level or configured bias. Tests cover floating/pulls/release/contention.
