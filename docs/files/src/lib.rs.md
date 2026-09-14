# `src/lib.rs`

## Purpose
Defines the reusable hp67emu library crate.

## Why it exists
Cycle-accurate emulation must be headless-testable and reusable independently of the desktop GUI. A library boundary provides that separation. The project also needs an instruction-level reference model that can validate the electrical implementation without becoming part of the UI.

## Relationships
Exports `emulation` for generic electrical/timing primitives, `machines` for calculator-specific compositions, and `reference` for behavioural oracle models such as the independent Woodstock semantic decoder/state. The desktop binary will eventually consume the electrical machine library rather than owning calculator behavior itself.

## Responsibilities
Expose stable top-level namespaces, keep reusable logic outside egui-specific binary modules, and maintain a visible separation between production electrical emulation and semantic reference/oracle code.

## Implementation
Contains only module exports and crate-level documentation. Architectural regression tests prohibit GUI/image dependencies from entering the emulation, machine and reference subtrees.
