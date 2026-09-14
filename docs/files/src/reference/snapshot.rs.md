# `src/reference/snapshot.rs`

## Purpose

Captures instruction-boundary Woodstock CPU state and produces compact, human-readable differences between two snapshots.

## Why it exists

Differential validation is only useful when a mismatch explains exactly which architectural state diverged. Dumping an entire machine after every instruction would make regressions noisy and difficult to diagnose, especially once the electrical ACT runs many bit-times per semantic word.

## Relationships

Exported by `src/reference/mod.rs`. Snapshots are captured from `reference::woodstock::ReferenceMachine` and will be used by the future semantic-vs-electrical differential harness. ROM execution from `reference::rom` provides natural instruction boundaries at which snapshots can be taken.

## Responsibilities

Clone CPU-visible state at a stable boundary, compare all registers and control fields, report changed status bits individually, and format PC/RAM-address values in notation that is useful when cross-referencing Woodstock listings. RAM contents are intentionally not included in the architectural CPU snapshot.

## Implementation

`ArchitecturalSnapshot::capture()` clones `ArchitecturalState`. `diff()` compares A/B/C/Y/Z/T/M1/M2, F/P/P-history, arithmetic mode, carry state, PC/bank/delayed-ROM state, return stack, instruction state, key buffer, display state, RAM address and all 16 status bits. `StateDiff` exposes both structured lines and a newline-separated `Display` representation. Registers are rendered most-significant nibble first, PC in five-digit octal and RAM addresses in hexadecimal.
