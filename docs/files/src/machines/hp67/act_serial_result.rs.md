# `src/machines/hp67/act_serial_result.rs`

## Purpose

Builds an independent A/B/C result image for source-backed ACT ADD/SUB execution by replaying the complete b0..b55 serial coordinate from a pre-instruction snapshot.

## Why it exists

The structural ACT path now owns a pre-instruction A/B/C/P/radix snapshot and a carry/borrow chain, but the architectural core still commits the complete instruction at an instruction boundary. Before removing that fallback, the serial path needs an independently testable final-state oracle that proves its digit traversal produces the same register and carry result without assigning an unsupported PHI-relative write edge.

## Relationships

Uses `ActSerialExecution` and `ActSerialArithmeticAction` for operation/field routing, `ActSerialStateSnapshot` for immutable pre-instruction operands and ADD/SUB digit evaluation, and the canonical `BITS_PER_DIGIT`/`BITS_PER_WORD` timing coordinates. Tests compare the completed image directly with `Hp67ArchitecturalMachine` for representative decimal and hexadecimal arithmetic.

## Responsibilities

Start from the captured A/B/C registers; replay all 56 serial bit coordinates; update an image only at the structural end of selected four-bit digit cells; propagate carry/borrow between selected digits; preserve registers for compare-only subtract operations; expose the final carry/borrow and number of processed digits; and return no image for arithmetic operations that are not yet implemented by this ADD/SUB slice.

## Implementation

`ActSerialArithmeticResultImage::evaluate()` creates a fresh serial execution for the arithmetic word, walks b0 through b55, and asks the snapshot for each selected ADD/SUB digit result. Destination digits are written only to the private result image. This write is a structural bookkeeping boundary and is not presented as a physical ACT register-write edge. Decimal and hexadecimal arithmetic reuse the same source-backed digit rules already exercised by `ActSerialStateSnapshot`.

The tests compare the serial image against the architectural core for decimal multi-digit addition, hexadecimal wrap, decimal borrow propagation and destination-less subtract/compare. This establishes a second implementation path for arithmetic final-state validation before the architectural fallback is removed.
