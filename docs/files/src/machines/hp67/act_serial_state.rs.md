# `src/machines/hp67/act_serial_state.rs`

## Purpose
Captures the ACT register and control state that belongs to one serially executing HP-67 microinstruction before the instruction-boundary fallback mutates the architectural machine, and resolves source-backed ADD/SUB inputs and digit results for the active serial coordinate.

## Why it exists
The current bring-up architecture still computes complete Woodstock instruction effects at an instruction boundary while the structural transport gives that same instruction an explicit `b0..b55` lifetime. A serial ALU must not read A/B/C, P or decimal mode after those architectural effects have already been applied. A dedicated immutable snapshot preserves the correct pre-instruction inputs without inventing an internal PHI write edge.

## Relationships
Reads `ActArchitecturalState` at the start of an instruction and combines it with `ActSerialExecution`, `ActSerialArithmeticCoordinate`, `ActSerialArithmeticAction`, `ActSerialRegister` and `ActSerialOperand` from `act_serial_execution.rs`. Its ADD/SUB digit arithmetic deliberately mirrors the already-validated `add_range()` and `sub_range()` semantics in `act.rs`; it remains read-only and does not claim physical write timing.

## Responsibilities
Preserve A, B and C nibbles, P and decimal/hex mode for one executing word; expose register and operand digits plus their LSB-first bits; use the captured P value for arithmetic field selection; resolve source bits, radix and architectural carry seed for ADD/SUB cells; evaluate one selected ADD/SUB digit with explicit carry/borrow chaining; reject invalid coordinates; and keep serial reads independent from later architectural mutations.

## Implementation
`ActSerialStateSnapshot::capture()` copies only the ACT state required by the current arithmetic datapath work. `register_digit()` and `operand_digit()` expose masked source nibbles, while `register_bit()` and `operand_bit()` expose their four LSB-first serial bits. `radix()` reports 10 or 16 from the captured mode. `arithmetic_coordinate()` applies the snapshot's pre-instruction P value to the active serial execution.

`alu_inputs()` accepts only selected ADD/SUB coordinates and returns `ActSerialAluInputs` containing the two source bits, radix, initial carry and full arithmetic coordinate. `alu_digit_result()` evaluates the complete selected digit using caller-supplied carry/borrow input. Decimal ADD applies the same +6 BCD adjustment as the architectural core, hexadecimal ADD wraps at 16, and SUB applies the same radix-based borrow correction as `sub_range()`. The returned `ActSerialDigitAluResult` exposes source digits, result digit, chain output and radix but does not mutate A/B/C or assign a PHI-relative write edge. Tests cover snapshot isolation, operand extraction, field gating, decimal and hexadecimal addition, decimal subtraction, increment carry seeding, invalid coordinates and radix behavior.
