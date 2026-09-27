# `src/machines/hp67/act_serial_special_register.rs`

## Purpose

Builds the completed-word structural result image for the focused ACT non-arithmetic register-transfer family introduced in M14G.

## Why it exists

M14E moved arithmetic A/B/C/carry final-state authority away from the instruction-boundary executor and M14F did the same for the focused P/status condition family. Several fixed ACT special instructions still let the architectural executor directly mutate working registers, stack registers, memory registers, F and key-derived A digits. M14G removes that causal dependency without inventing unsupported internal write timing.

## Relationships

Uses `ActSerialExecution` only to identify the active special/peripheral word and preserve the complete `b0..b55` structural lifetime. It reads immutable pre-instruction A/B/C/Y/Z/T/M1/M2/F/key/P values from `ActSerialStateSnapshot`. `ActSerialEndpoint` owns one optional `ActSerialSpecialRegisterResultImage` and marks it complete only when the bound structural execution reaches b55. The live bridge then compares its final register image against `ActArchitecturalCore` semantics before committing it.

## Responsibilities

Decode the M14G fixed register-transfer family; preserve the full pre-instruction source image; compute clear, copy, exchange, stack-transfer, A-rotate, F/A0 and key-to-A results independently from the architectural executor; complete the C-digit side of load-constant while leaving its P mutation to M14F; expose every migrated register result; and reject non-M14G words.

## Implementation

The image starts as an exact copy of the pre-instruction register state. `complete_word()` applies only the decoded special-register transformation after the structural execution has completed b55. Supported direct opcodes are clear working stack, C/M1 and C/M2 exchange/copy, the four stack-register transfers, F-to-A0 and A0/F exchange, key-to-A and A rotate-right. The `0o30` family writes the four-bit operand into `C[P]` only when the captured P is within the fourteen-digit word; M14F separately owns the P decrement for the same instruction.

The completed-word handoff is a **WORKING APPROXIMATION**. M14G establishes final-state causal authority but does not claim that the physical 1820-2530 writes these registers at b55, at a digit boundary, or on any particular PHI edge. Exact internal serial routing and visibility remain SOURCE-BLOCKED.
