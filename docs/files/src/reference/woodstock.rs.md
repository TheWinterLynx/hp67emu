# `src/reference/woodstock.rs`

## Purpose

Provides a compact, UI-independent Woodstock architectural state and 10-bit microinstruction decoder for differential validation of the future electrical ACT implementation.

## Why it exists

Nonpareil already demonstrates a mature instruction-level Woodstock model. Re-deriving the basic opcode shape, register widths and field layout inside every low-level test would waste effort. This file captures those architectural facts in a fresh Rust structure while keeping them explicitly separate from cycle/electrical timing.

## Relationships

`src/reference/mod.rs` exports this module. Future tests will compare `ArchitecturalState` snapshots with state reconstructed from the timed ACT/device simulation in `src/emulation` and `src/machines/hp67`. `docs/NONPAREIL_ANALYSIS.md` records the upstream analysis and the licensing boundary.

## Responsibilities

Define the 14-digit register shape, status/stack/page constants, field selection, broad opcode classes and an architectural snapshot type. Reject opcodes wider than the real 10-bit instruction word instead of silently truncating them. Remain free of egui, image handling and host presentation state.

## Implementation

`decode()` uses the two least-significant opcode bits to classify the 1024-word instruction space into special, JSB, arithmetic and GOTO forms. Arithmetic words expose their five-bit operation selector and three-bit field selector. `Field::digit_range()` translates Woodstock field names into indices within a 14-nibble word. `ArchitecturalState` holds the registers and control state that differential tests will eventually compare at microinstruction boundaries. The file deliberately does not execute instructions or model electrical timing yet.
