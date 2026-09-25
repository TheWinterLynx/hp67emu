# `src/machines/hp67/act_serial_result.rs`

## Purpose

Maintains the A/B/C/carry result image for every Woodstock arithmetic opcode executed across the structural `b0..b55` ACT word, while preserving a separate replay helper for differential validation.

## Why it exists

M14D removed the architectural executor from the causal final ADD/SUB result path. M14E extends the same authority boundary to clear, copy, exchange, shift and zero/nonzero test operations so all 32 arithmetic opcodes can obtain their final A/B/C/carry result from the structural traversal. Exact internal ACT register-write and PHI-edge timing is still source-blocked, so the image remains a word-completion authority bridge rather than a claim about physical internal write edges.

## Relationships

Uses `ActSerialExecution` and `ActSerialArithmeticAction` for operation/field routing, `ActSerialStateSnapshot` for immutable pre-instruction operands, `ActSerialDigitAluResult` for ADD/SUB digit results, and `ActSerialExecution::arithmetic_field_selects_digit()` to source adjacent digits for shifts without duplicating field-selection rules. `ActSerialEndpoint` owns one incremental image during the actual shared structural word traversal. `evaluate()` remains an independent replay path compared against `Hp67ArchitecturalMachine`.

## Responsibilities

Initialize A/B/C from the pre-instruction snapshot; initialize the correct carry/test seed for each operation; accept one structural checkpoint per selected four-bit digit; apply clear/copy/exchange routing from the immutable snapshot; source left/right shifts only from adjacent digits inside the same selected field; propagate ADD/SUB carry or borrow; accumulate zero/nonzero tests; preserve registers for destination-less compares and test operations; expose final carry and processed-digit count; and return no image for non-arithmetic words.

## Implementation

`ActSerialArithmeticResultImage::begin()` now accepts every decoded arithmetic action. ADD/SUB start from their opcode carry/borrow seed, `TestZero` starts true, and all other families start false to match the architectural instruction-boundary carry result. At each selected digit-end checkpoint, `record_digit_checkpoint()` updates only the private image. Clear/copy/exchange read source digits from the immutable pre-state. Shift-left takes the preceding selected source digit or zero at the field boundary; shift-right takes the following selected source digit or zero at the field boundary. Test operations fold nonzero/zero state into `final_chain`. ADD/SUB continue to consume `ActSerialDigitAluResult`.

`evaluate()` replays the same `b0..b55` coordinate independently and is regression-checked against the architectural oracle for all 32 arithmetic operations, all eight fields, representative P values including invalid P, and decimal/hex modes. The production endpoint accumulates its own image during the real shared-word traversal, so the replay helper is not the causal live path.

The final live commit still occurs only after the completed structural word. That is a **WORKING APPROXIMATION** for internal ACT timing: M14E establishes final-state causal authority for the arithmetic family without asserting a physical nibble-write or PHI edge.
