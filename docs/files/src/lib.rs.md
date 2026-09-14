# `src/lib.rs`

## Purpose
Defines the reusable hp67emu library crate.

## Why it exists
Cycle-accurate emulation must be headless-testable and reusable independently of the desktop GUI. A library boundary provides that separation.

## Relationships
Exports `emulation` for generic electrical/timing primitives and `machines` for calculator-specific compositions. The desktop binary will eventually consume this library rather than owning calculator behavior itself.

## Responsibilities
Expose stable top-level namespaces and keep reusable logic outside egui-specific binary modules.

## Implementation
Contains only module exports and crate-level documentation. Architectural regression tests prohibit GUI/image dependencies from entering its core subtrees.
