# src/machines/hp67/pin_trace.rs

## Purpose

Capture all committed HP-67 pins and owners, with a strict versioned import/export boundary.

## Why it exists

Full electrical observations and calibrated external recordings need a strict boundary that cannot confuse model ticks with measured time.

## Relationships

Works with the HP-67 backplane, pin_trace modules, native CLI and live adapter. Existing bus_trace IS-only format remains independent.

## Responsibilities

Preserve deterministic observation, source/domain/provenance, bounded input/output and explicit coverage without inventing physical timing.

## Implementation

Owns explicit time/source domains, bit anchor/provenance, bounded snapshot rows, owner-mask validation, input limits and serialization. Recording deduplicates unchanged same-tick images, retains ownership-only changes and emits delta order; overflow blocks export. Normalized physical rows use integer nanoseconds and order zero, contain no internal driver masks and may mark pins unobserved. Unknown, floating and contention remain distinct.

See `docs/PIN_CAPTURE_v1.0.md`. No timing profile is installed and no hardware accuracy claim follows from synthetic agreement.
