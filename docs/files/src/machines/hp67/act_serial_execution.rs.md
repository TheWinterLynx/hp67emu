# `src/machines/hp67/act_serial_execution.rs`

## Purpose
Represents the lifetime of one executing HP-67 ACT word across the canonical 56 serial bit coordinates `b0..b55` and identifies the arithmetic field selected at each coordinate.

## Why it exists
The architectural ACT core currently produces correct instruction-boundary state for real firmware, but a physical 1820-2530 executes over a complete serial machine word. Moving toward cycle accuracy requires execution to have an explicit intra-word lifetime and field gate before individual register, ALU and carry transitions are migrated to source-backed bit timing.

## Relationships
Uses the existing `ActInstructionState` to distinguish ordinary instructions from THEN-GOTO data, the ten-bit ROM width from `isa.rs`, and the 14x4 word geometry from `timing.rs`. It complements the structural `ActSerialEndpoint`; it does not replace the architectural ACT semantics or invent PHI-relative mutation timing.

## Responsibilities
Decode the currently executing ten-bit word into its Woodstock instruction class, expose the next `b0..b55` coordinate together with digit and bit-within-digit coordinates, resolve whether that coordinate belongs to the instruction's P/WP/XS/X/S/M/W/MS arithmetic field, enforce strictly sequential advancement, and reject words wider than the physical ten-bit ROM word.

## Implementation
`ActSerialExecution` starts at `b0`, advances exactly once for every serial bit coordinate and becomes complete after `b55`. `ActSerialWordClass` preserves arithmetic operation/field information, branch offsets and the special/peripheral opcode while treating a word following an IF/test as `ThenGotoData`. For arithmetic words, `arithmetic_coordinate(p)` returns `ActSerialArithmeticCoordinate` containing operation, field, digit, bit-within-digit and the field-selection result for the current P value. The field mapping is the Woodstock P/WP/XS/X/S/M/W/MS mapping already used by the architectural model; it is a serial selection gate only and deliberately performs no A/B/C/carry mutation. Tests cover all-word execution, W, M, P and WP selection behavior, invalid-P fallbacks, non-arithmetic words, THEN-GOTO handling, ordering errors and ten-bit width enforcement.
