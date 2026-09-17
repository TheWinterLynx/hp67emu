# `src/lib.rs`

## Purpose
Defines the reusable hp67emu library crate.

## Why it exists
Cycle-accurate emulation must be headless-testable and reusable independently of the desktop GUI. A library boundary provides that separation while keeping production electrical emulation and independent semantic reference models available to tests without coupling them to the UI.

## Relationships
Exports `emulation` for generic electrical/timing primitives, `machines` for calculator-specific compositions, and `reference` for independent behavioural oracle models. Firmware provenance and historical source-comparison material live only in Markdown documentation; no source-specific research/import module is part of the library crate.

## Responsibilities
Expose stable top-level namespaces, keep reusable logic outside egui-specific binary modules, and maintain visible separation between production electrical emulation and semantic reference/oracle code.

## Implementation
Contains only module exports and crate-level documentation. Architectural regression tests prohibit GUI/image dependencies from entering the emulation and machine subtrees, while the documentation contract requires every Rust source file to have a corresponding design document.
