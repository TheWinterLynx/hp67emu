# `tests/physical_startup_flow.rs`

## Purpose

Locks a directly observed HP-67 control-flow checkpoint against the semantic Woodstock reference machine.

## Why it exists

The firmware corpus is now independently reconciled across Teenix 2026, x11-calc and Nonpareil, but corpus identity alone does not prove that hp67emu interprets page selection and calls correctly. Physical HP-67 research reports that execution around PC `0x0068` uses a delayed ROM select followed by a JSB and enters address `0x0fc6`. This is a high-value bridge between real-hardware observation, exact 10-bit firmware words and the Rust reference semantics.

## Relationships

Uses `reference::woodstock::ReferenceMachine` only. The two embedded 10-bit words correspond to the reconciled firmware instructions at PCs `0x0067` and `0x0068`; no complete or redistributable ROM image is embedded. `docs/MICROCODE_PROVENANCE.md` records the evidence source and corpus provenance.

## Responsibilities

Verify that the instruction at `0x0067` installs delayed ROM 15, that the following JSB stores return address `0x0069`, and that delayed selection transforms the JSB target into `0x0fc6`. Fail if future changes alter delayed-ROM/JSB ordering or return-stack semantics.

## Implementation

The test starts a fresh semantic reference machine at PC `0x0067`, executes `0x3f4` (`0o1764`, delayed select ROM 15), then `0x319` (`0o1431`, JSB offset `0xc6`). It requires PC `0x0fc6`, return stack entry `0x0069`, stack pointer 1 and a consumed delayed-ROM latch. The test is deliberately instruction-boundary semantic evidence; it does not claim to validate PHI1/PHI2, ISA serialization or electrical timing.
