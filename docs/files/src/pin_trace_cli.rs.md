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


### M16A PHI windows and compiled-out recorder — delivery 4/15

Adds --timing-pins <input.tsv> <IS,DATA,SYNC,...> [new-report.txt] in all native builds. The audit reports observed setup/hold separations and phase windows, marks unresolved/coincident observations with exit 3, and never installs a timing profile. Full-pin capture tests run in the explicit diagnostic build; offline timing CLI tests run in both modes.
See `docs/PIN_CAPTURE_v1.1.md` and `docs/research/M16A_PHI_WINDOWS_2026-10-24.md`. The owner gate tests both feature configurations.


## M16L ownership audit

Adds --integrity-pins <input.tsv> [new-report.txt] in all native builds, strict input and create_new output behavior, with scoped 0/2/3 statuses. It audits production source contracts, not complete physical accuracy. Malformed input remains an error. Scope and source-blocked limits are included in every report.
