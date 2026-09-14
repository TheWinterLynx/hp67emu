# `src/emulation/device.rs`

## Purpose
Defines order-independent interfaces for emulated electrical devices.

## Why it exists
If one chip can observe another chip's freshly-written output only because it ran earlier in a Rust loop, host iteration order becomes fake hardware timing. Device evaluation needs stable input snapshots and delayed output commit.

## Relationships
Sits between the future scheduler and chip implementations. Uses `Tick`, `LogicLevel`, `DriverId` and `Drive` from the generic kernel.

## Responsibilities
Separate net reading from drive publication and define the contract every emulated IC/subassembly will follow.

## Implementation
`NetReader` exposes resolved input levels, `DriveSink` collects proposed output drives, and `ElectricalDevice::evaluate` consumes one tick's stable inputs to produce next outputs. The scheduler implementing resolve/snapshot/evaluate/commit is a later milestone.
