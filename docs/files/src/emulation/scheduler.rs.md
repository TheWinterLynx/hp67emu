# `src/emulation/scheduler.rs`

## Purpose

Implements the generic resolve → snapshot → evaluate → commit loop for deterministic electrical simulation.

## Why it exists

Shared buses and multi-chip propagation cannot depend on the order in which Rust happens to iterate device objects. Every IC must see the same resolved wire state for a simulation instant and its new output drives must become visible only after every device has evaluated that instant.

## Relationships

Uses `emulation::device` for the `ElectricalDevice`, `NetReader` and `DriveSink` contracts, `emulation::net` for resolved wires and named drivers, and `emulation::clock::Tick` for deterministic time. It remains the model-independent reference implementation of the scheduling contract. The HP-67 production machine uses the fixed-topology dense fabric in `src/machines/hp67/electrical.rs` so its edge hot path does not pay map/boxing/allocation costs.

## Responsibilities

Own installed nets and electrical devices, capture one immutable resolved input snapshot per tick, buffer all device output drives, reject duplicate driver identities, commit outputs atomically after evaluation, and fail immediately on bus contention with tick/net/driver diagnostics.

## Implementation

`ElectricalScheduler` intentionally stores nets in a `BTreeMap` and devices in a boxed vector because arbitrary model-independent topology and auditability are its priorities, not HP-67 hot-path throughput. `step()` first rejects pre-existing contention, snapshots all resolved levels, evaluates every device against that snapshot into per-device drive buffers, merges those buffers while checking driver identity collisions, commits the complete pending drive set, advances the tick, and verifies the resulting nets are contention-free. Tests prove one-tick propagation, device-order independence, deterministic contention diagnostics and collision rejection before commit.
