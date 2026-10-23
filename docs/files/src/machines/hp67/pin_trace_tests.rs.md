# src/machines/hp67/pin_trace_tests.rs

## Purpose

Exercise full-pin capture, physical-domain separation and strict electrical analysis.

## Why it exists

Full electrical observations and calibrated external recordings need a strict boundary that cannot confuse model ticks with measured time.

## Relationships

Works with the HP-67 backplane, pin_trace modules, native CLI and live adapter. Existing bus_trace IS-only format remains independent.

## Responsibilities

Preserve deterministic observation, source/domain/provenance, bounded input/output and explicit coverage without inventing physical timing.

## Implementation

Seven regressions cover joint bundles and same-level ownership changes, a synthetic nonuniform 2/3/5/7 ns waveform, clock faults, malformed metadata/columns/masks/coordinates, explicit alignment and incomplete observations, DATA wrap/SYNC/display events and real capture limits. Every generated waveform is labelled as synthetic/model data; no hardware fixture is fabricated.

See `docs/PIN_CAPTURE_v1.0.md`. No timing profile is installed and no hardware accuracy claim follows from synthetic agreement.
