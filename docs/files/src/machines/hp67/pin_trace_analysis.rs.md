# src/machines/hp67/pin_trace_analysis.rs

## Purpose

Analyze PHI topology and input-time intervals, decode serial frames and compare explicit word ranges.

## Why it exists

Full electrical observations and calibrated external recordings need a strict boundary that cannot confuse model ticks with measured time.

## Relationships

Works with the HP-67 backplane, pin_trace modules, native CLI and live adapter. Existing bus_trace IS-only format remains independent.

## Responsibilities

Preserve deterministic observation, source/domain/provenance, bounded input/output and explicit coverage without inventing physical timing.

## Implementation

Rejects low-pulse overlap, invalid order, missing/simultaneous/unresolved clock edges and zero/backwards PHI intervals. Summaries retain source/time units and input provenance; physical timestamps are not certified hardware evidence. Decodes IS address/opcode/SYNC masks and DATA b2..next b1, preserving incomplete frames. Selected-pin comparisons use explicit word indices and binary checkpoints, bounded examples, owner coverage and distinct incomplete status; durations are never silently compared or fitted.

See `docs/PIN_CAPTURE_v1.0.md`. No timing profile is installed and no hardware accuracy claim follows from synthetic agreement.
