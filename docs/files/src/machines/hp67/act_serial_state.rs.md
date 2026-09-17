# `src/machines/hp67/act_serial_state.rs`

## Purpose
Captures the ACT register and control state that belongs to one serially executing HP-67 microinstruction before the instruction-boundary fallback mutates the architectural machine.

## Why it exists
The current bring-up architecture still computes complete Woodstock instruction effects at an instruction boundary while the structural transport gives that same instruction an explicit `b0..b55` lifetime. A serial ALU must not read A/B/C, P or decimal mode after those architectural effects have already been applied. A dedicated immutable snapshot preserves the correct pre-instruction inputs without inventing an internal PHI write edge.

## Relationships
Reads `ActArchitecturalState` at the start of an instruction and uses `ActSerialRegister`/`ActSerialOperand` from `act_serial_execution.rs`. It is intended to become owned by the ACT serial endpoint as arithmetic operations migrate from the architectural fallback into the physical b0..b55 path.

## Responsibilities
Preserve A, B and C nibbles, P and decimal/hex mode for one executing word; expose register digits and LSB-first register/operand bits; reject invalid digit/bit coordinates; and keep serial reads independent from later mutations of the architectural state.

## Implementation
`ActSerialStateSnapshot::capture()` copies only the ACT state required by the current arithmetic datapath work. `register_digit()` returns a masked four-bit digit, `register_bit()` resolves one of its four LSB-first serial bits, `operand_bit()` also handles the explicit zero operand, and `radix()` reports 10 or 16 from the captured mode. Tests prove that later architectural mutation cannot alter the snapshot and verify register-bit, zero-operand, bounds and radix behavior.
