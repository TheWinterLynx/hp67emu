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


### M16A PHI windows and compiled-out recorder — delivery 4/15

Recorder-only snapshot/record code and the legacy capture regression module are gated by pin-capture; strict import/export and analysis are always available. PinClockObservation is exported so the window auditor uses validated actual edge rows and explicit word/bit coordinates.
See `docs/PIN_CAPTURE_v1.1.md` and `docs/research/M16A_PHI_WINDOWS_2026-10-24.md`. The owner gate tests both feature configurations.


M16G appends MOTOR to PIN_NAMES for full fifteen-pin export. Existing sparse named-column import already preserves absent MOTOR as unknown (None), so legacy captures remain distinguishable from actual Off/High/Floating observations. See docs/PIN_CAPTURE_v1.2.md.


M16H appends MTRS to sixteen exported named pins. Existing named-column imports preserve absent MTRS as None for both level and source masks; re-export leaves it unknown. The v1 extensible metadata and earlier indices remain unchanged.
