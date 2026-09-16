# `src/machines/hp67/act_serial_execution.rs`

## Purpose
Represents the lifetime of one executing HP-67 ACT word across the canonical 56 serial bit coordinates `b0..b55`.

## Why it exists
The architectural ACT core currently produces correct instruction-boundary state for real firmware, but a physical 1820-2530 executes over a complete serial machine word. Moving toward cycle accuracy requires execution to have an explicit intra-word lifetime before individual register, ALU and carry transitions are migrated to source-backed bit timing.

## Relationships
Uses the existing `ActInstructionState` to distinguish ordinary instructions from THEN-GOTO data, the ten-bit ROM width from `isa.rs`, and the 14x4 word geometry from `timing.rs`. It complements the structural `ActSerialEndpoint`; it does not replace the architectural ACT semantics or invent PHI-relative mutation timing.

## Responsibilities
Decode the currently executing ten-bit word into its Woodstock instruction class, expose the next `b0..b55` coordinate together with digit and bit-within-digit coordinates, enforce strictly sequential advancement, and reject words wider than the physical ten-bit ROM word.

## Implementation
`ActSerialExecution` starts at `b0`, advances exactly once for every serial bit coordinate and becomes complete after `b55`. `ActSerialWordClass` preserves arithmetic operation/field information, branch offsets and the special/peripheral opcode while treating a word following an IF/test as `ThenGotoData`. The object deliberately performs no A/B/C/carry mutation yet: those transitions remain blocked until HP-67/1820-2530 evidence fixes the required bit-cell or PHI-relative commit rule. Tests verify the full 56-bit lifetime, arithmetic decoding, THEN-GOTO handling, ordering errors and ten-bit width enforcement.
