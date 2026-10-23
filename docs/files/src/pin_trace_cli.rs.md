# src/pin_trace_cli.rs

## Purpose

Provide native capture, audit and comparison commands without altering GUI or emulator semantics.

## Why it exists

Full electrical observations and calibrated external recordings need a strict boundary that cannot confuse model ticks with measured time.

## Relationships

Works with the HP-67 backplane, pin_trace modules, native CLI and live adapter. Existing bus_trace IS-only format remains independent.

## Responsibilities

Preserve deterministic observation, source/domain/provenance, bounded input/output and explicit coverage without inventing physical timing.

## Implementation

Reads bounded UTF-8 files and writes only new output paths. Captures 1..512 firmware words, audits pin topology/intervals and compares explicit word indices/pin lists. Reports preserve both domains, paths, provenance and coverage. Exit codes distinguish failure, mismatch and incomplete data. Inputs and existing reports cannot be overwritten.

See `docs/PIN_CAPTURE_v1.0.md`. No timing profile is installed and no hardware accuracy claim follows from synthetic agreement.
