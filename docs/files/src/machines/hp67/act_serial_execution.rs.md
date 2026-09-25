# `src/machines/hp67/act_serial_execution.rs`

## Purpose
Represents the lifetime of one executing HP-67 ACT word across the canonical 56 serial bit coordinates `b0..b55` and makes the arithmetic datapath routing for each Woodstock arithmetic opcode explicit.

## Why it exists
The architectural ACT core currently produces correct instruction-boundary state for real firmware, but a physical 1820-2530 executes over a complete serial machine word. Moving toward cycle accuracy requires execution to have an explicit intra-word lifetime, field selection and ALU/register routing before individual register, ALU and carry transitions are migrated to source-backed bit timing.

## Relationships
Uses the existing `ActInstructionState` to distinguish ordinary instructions from THEN-GOTO data, the ten-bit ROM width from `isa.rs`, and the 14x4 word geometry from `timing.rs`. Its arithmetic routing mirrors the already-validated operation semantics in `act.rs`; it does not replace the architectural ACT semantics or invent PHI-relative mutation timing.

## Responsibilities
Decode the currently executing ten-bit word into its Woodstock instruction class, expose the next `b0..b55` coordinate together with digit and bit-within-digit coordinates, identify whether that coordinate belongs to the active arithmetic field, decode every arithmetic operation `0x00..0x1f` into explicit A/B/C/zero source and destination routing, enforce strictly sequential advancement, and reject words wider than the physical ten-bit ROM word.

## Implementation
`ActSerialExecution` starts at `b0`, advances exactly once for every serial bit coordinate and becomes complete after `b55`. `ActSerialWordClass` preserves arithmetic operation/field information, branch offsets and the special/peripheral opcode while treating a word following an IF/test as `ThenGotoData`.

`ActSerialArithmeticCoordinate` combines the current operation, field, decoded `ActSerialArithmeticAction`, digit, bit-within-digit and field-selection state. `decode_serial_arithmetic_action()` covers all 32 Woodstock arithmetic operations and distinguishes clear/copy/exchange, add/subtract with their architectural carry seed, left/right shifts and zero/nonzero tests. Compare operations use subtraction routing with no destination, matching the architectural core without pretending that a register write occurs.

The object deliberately performs no live A/B/C/carry mutation. The routing says what data path an operation requires, not which PHI edge commits it. M14E adds `arithmetic_field_selects_digit()` so the structural result image can source the adjacent pre-instruction digit required by left/right shifts without duplicating field semantics. Those physical transitions remain blocked until HP-67/1820-2530 evidence fixes the required bit-cell or PHI-relative commit rule. Tests verify the full 56-bit lifetime, all 32 arithmetic routes, representative add/compare/increment/decrement routing, direct arbitrary-digit field membership, THEN-GOTO handling, ordering errors and ten-bit width enforcement.
