# `src/machines/hp67/act_serial_state.rs`

## Purpose
Captures the ACT register and control state that belongs to one serially executing HP-67 microinstruction before the instruction-boundary fallback mutates the architectural machine, and resolves source-backed ADD/SUB input bits for the active serial coordinate.

## Why it exists
The current bring-up architecture still computes complete Woodstock instruction effects at an instruction boundary while the structural transport gives that same instruction an explicit `b0..b55` lifetime. A serial ALU must not read A/B/C, P or decimal mode after those architectural effects have already been applied. A dedicated immutable snapshot preserves the correct pre-instruction inputs without inventing an internal PHI write edge.

## Relationships
Reads `ActArchitecturalState` at the start of an instruction and combines it with `ActSerialExecution`, `ActSerialArithmeticCoordinate`, `ActSerialArithmeticAction`, `ActSerialRegister` and `ActSerialOperand` from `act_serial_execution.rs`. It is intended to become owned by the ACT serial endpoint as arithmetic operations migrate from the architectural fallback into the physical b0..b55 path.

## Responsibilities
Preserve A, B and C nibbles, P and decimal/hex mode for one executing word; expose register digits and LSB-first register/operand bits; use the captured P value for arithmetic field selection; resolve the two source bits, radix and architectural carry seed needed by ADD/SUB cells; reject invalid digit/bit coordinates; and keep serial reads independent from later architectural mutations.

## Implementation
`ActSerialStateSnapshot::capture()` copies only the ACT state required by the current arithmetic datapath work. `register_digit()` returns a masked four-bit digit, `register_bit()` resolves one of its four LSB-first serial bits, `operand_bit()` also handles the explicit zero operand, and `radix()` reports 10 or 16 from the captured mode. `arithmetic_coordinate()` applies the snapshot's pre-instruction P value to the active serial execution. `alu_inputs()` accepts only selected ADD/SUB coordinates and returns `ActSerialAluInputs` containing the two source bits, radix, initial carry and full arithmetic coordinate. It deliberately does not perform decimal correction, carry propagation or a destination write until the physical ACT timing for those transitions is established. Tests prove snapshot isolation and verify register bits, zero operands, field gating, ADD/increment inputs, bounds and radix behavior.
