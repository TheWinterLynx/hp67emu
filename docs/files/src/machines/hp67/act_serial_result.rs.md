# `src/machines/hp67/act_serial_result.rs`

## Purpose

Maintains the A/B/C/carry result image for source-backed ACT ADD/SUB execution and also provides an independent replay helper for differential validation.

## Why it exists

The structural ACT path already owns the pre-instruction A/B/C/P/radix snapshot, the complete `b0..b55` execution lifetime and the ADD/SUB carry-or-borrow chain. M14D needs that same structural traversal to produce the final arithmetic register image instead of allowing the instruction-boundary architectural executor to remain the causal source of A/B/C/carry. At the same time, exact internal ACT register-write and PHI-edge timing is still source-blocked, so the result image must not pretend that its structural digit checkpoints are physical write edges.

## Relationships

Uses `ActSerialExecution` and `ActSerialArithmeticAction` for operation/field routing, `ActSerialStateSnapshot` for immutable pre-instruction operands and digit evaluation, and `ActSerialDigitAluResult` for completed selected-digit results. `ActSerialEndpoint` owns one incremental image during the actual shared structural word traversal. `evaluate()` remains a second replay path used to compare the same final arithmetic semantics with `Hp67ArchitecturalMachine`.

## Responsibilities

Initialize A/B/C from the pre-instruction snapshot; preserve the opcode's initial carry/borrow seed even when the selected field contains no digits; accept completed selected-digit ADD/SUB results from the structural traversal; update only the routed destination register; preserve all registers for destination-less compare operations; expose final carry/borrow and processed-digit count; reject non-ADD/SUB arithmetic as outside this slice; and provide an independent full-word replay helper for regression tests.

## Implementation

`ActSerialArithmeticResultImage::begin()` identifies ADD/SUB routing from the current serial execution, copies A/B/C and initializes `final_chain` from the instruction's source-backed carry/borrow seed. During the real structural word, `ActSerialEndpoint::advance_execution_for_bit()` calls `record_digit_result()` after the fourth coordinate of each selected digit. That updates the private image and the chain but does not mutate live ACT architectural registers at that point.

`evaluate()` independently creates a fresh serial execution and replays `b0..b55` into another image. Tests compare that replay with the architectural oracle for decimal/hexadecimal ADD/SUB, compare-only subtraction and an empty selected field. M14D uses the image accumulated by the actual structural traversal as the live final A/B/C/carry source after the word has completed. That final word-boundary handoff is a **WORKING APPROXIMATION**: it removes the architectural executor from the causal arithmetic result path without claiming which internal bit or PHI edge physically writes the ACT registers.
