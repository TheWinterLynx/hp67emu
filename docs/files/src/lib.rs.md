# `src/lib.rs`

## Purpose
Defines the reusable hp67emu library crate.

## Why it exists
Cycle-accurate emulation must be headless-testable and reusable independently of the desktop GUI. A library boundary provides that separation. The project also needs instruction-level reference models and local firmware-research tooling that can validate the electrical implementation without becoming part of the UI or emulated hardware.

## Relationships
Exports `emulation` for generic electrical/timing primitives, `machines` for calculator-specific compositions, `reference` for behavioural oracle models such as the independent Woodstock semantic model, and `research` for local ROM provenance/normalization utilities. The desktop binary will eventually consume the electrical machine library rather than owning calculator behaviour itself.

## Responsibilities
Expose stable top-level namespaces, keep reusable logic outside egui-specific binary modules, and maintain visible separation between production electrical emulation, semantic reference/oracle code, and research/import tooling.

## Implementation
Contains only module exports and crate-level documentation. Architectural regression tests prohibit GUI/image dependencies from entering the emulation and machine subtrees, while the documentation contract requires every Rust source file to have a corresponding design document.
