# src/pin_trace_cli_tests.rs

## Purpose

Validate the complete native CLI capture/audit/compare and output safety contract.

## Why it exists

Full electrical observations and calibrated external recordings need a strict boundary that cannot confuse model ticks with measured time.

## Relationships

Works with the HP-67 backplane, pin_trace modules, native CLI and live adapter. Existing bus_trace IS-only format remains independent.

## Responsibilities

Preserve deterministic observation, source/domain/provenance, bounded input/output and explicit coverage without inventing physical timing.

## Implementation

Two temporary-directory regressions exercise real firmware capture, ordinal audit, exact scoped comparison, refusal to overwrite any input/existing output, argument bounds and distinct mismatch/incomplete return codes. Test-owned temporary files are removed; gates do not create persistent file logs.

See `docs/PIN_CAPTURE_v1.0.md`. No timing profile is installed and no hardware accuracy claim follows from synthetic agreement.
