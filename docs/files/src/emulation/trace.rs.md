# `src/emulation/trace.rs`

## Purpose
Provides a minimal logic-analyzer-style trace container.

## Why it exists
Cycle accuracy must be validated against waveforms, not only final calculator results. Capturing `(tick, net, level)` samples creates a deterministic comparison surface for future golden traces.

## Relationships
Uses generic `Tick` and `LogicLevel`. Future scheduler/probe code will populate traces; HP-67 tests will compare them with source-backed captures.

## Responsibilities
Store ordered resolved-net observations, expose samples for comparison/export, and support reset between runs.

## Implementation
An append-only `Vec<TraceSample<N>>` with `push`, `samples` and `clear`. Serialization/trace-file format is intentionally deferred until M1 defines the canonical format.
