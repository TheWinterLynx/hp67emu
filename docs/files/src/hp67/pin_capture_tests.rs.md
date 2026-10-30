# src/hp67/pin_capture_tests.rs

## Purpose

Verify that diagnostic pin recording preserves the real live machine.

## Why it exists

Full electrical observations and calibrated external recordings need a strict boundary that cannot confuse model ticks with measured time.

## Relationships

Works with the HP-67 backplane, pin_trace modules, native CLI and live adapter. Existing bus_trace IS-only format remains independent.

## Responsibilities

Preserve deterministic observation, source/domain/provenance, bounded input/output and explicit coverage without inventing physical timing.

## Implementation

Two native regressions compare recording on/off for 512 firmware words including contact events and complete ACT/RAM/CRC/display results, require every PHI edge and real RAM owner coverage, and import/export the full capture. A contended STR word retains its electrical prefix and unresolved observation; reset discards the recorder.

See `docs/PIN_CAPTURE_v1.0.md`. No timing profile is installed and no hardware accuracy claim follows from synthetic agreement.


## M16L ownership audit

M16L captures actual firmware after PROGRAM/RUN selector changes and physical card insertion, audits each 256-word capture and requires all 14336 cell checkpoints with scoped integrity success. No synthetic replacement instruction stream is used.
