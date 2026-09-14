# `src/reference/differential.rs`

## Purpose

Provides the reusable instruction-boundary harness that compares the semantic HP-67 oracle with a timed or electrical implementation.

## Why it exists

The final emulator will execute one Woodstock microinstruction through many low-level scheduler ticks. Tests therefore need a stable bridge between two different notions of progress: one atomic semantic word and many electrical ticks. Without a common harness, each ACT regression would duplicate timeout, boundary detection and state-diff logic.

## Relationships

Uses `reference::hp67::Hp67Reference` as the semantic oracle, `reference::rom::RomImage` for semantic fetch, and `reference::snapshot` for architectural comparison. The future electrical ACT/machine will implement `InstructionBoundaryTarget` once it can expose a monotonic completed-instruction counter and reconstruct architectural state.

## Responsibilities

Verify initial synchronization, advance the semantic model by exactly one word, advance the timed target tick-by-tick until exactly one instruction completes, detect stalls or skipped boundaries, and report a precise `StateDiff` if architectural state diverges.

## Implementation

`InstructionBoundaryTarget` deliberately requires only three operations: read the completed-instruction count, advance one smallest target tick, and capture an architectural snapshot. `step_and_compare()` records the starting counter, steps the semantic HP-67, waits up to a caller-supplied tick budget for the target's next boundary, rejects counter jumps larger than one, and compares the resulting snapshots. Unit tests exercise successful delayed boundaries, synchronization mismatches and timeout handling using a semantic target that behaves like a multi-tick machine; no fake electrical timing is encoded in the production path.
