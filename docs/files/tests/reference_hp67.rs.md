# `tests/reference_hp67.rs`

## Purpose

Provides cross-module regression tests for the integrated HP-67 semantic reference path.

## Why it exists

Unit tests inside the individual reference modules verify local behaviour, but several important HP-67 properties only matter when ROM fetch, Woodstock execution and calculator-specific quirks are combined. The P-wrap label-search case is also a named regression target inherited from the Nonpareil analysis and must remain visible rather than disappearing inside a generic helper.

## Relationships

Uses `reference::hp67::Hp67Reference`, `reference::rom::RomImage` and Woodstock instruction state. It complements the module-local tests in `src/reference` and will later sit beside semantic-vs-electrical differential regressions.

## Responsibilities

Keep the HP-67 label-search P-wrap target around octal address `06132` under test without introducing a PC-specific execution hack, and prove that a caller-supplied in-memory ROM fixture can drive the integrated HP-67 reference machine without filesystem I/O.

## Implementation

The first test executes two `INC P` words ending at the historical `06132` test site and verifies that the following P==0 conditional uses the generic transient-wrap rule. The second constructs a two-word ROM page in memory, steps the integrated HP-67 reference through it and verifies that the binary-mode instruction changes ACT state as expected.
