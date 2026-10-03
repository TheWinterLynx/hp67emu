# `src/machines/hp67/act_serial_flow.rs`

## Purpose

Owns structural successor-fetch and final ACT PC, bank, delayed-ROM and return-stack state for M14I.

## Why it exists

The live fetch previously consumed the architectural executor's mutated PC. Removing that dependency requires a successor derived independently from immutable pre-instruction inputs before the concurrent address window.

## Relationships

`fetch.rs` captures the image before the oracle runs and marks it complete only after the executing word reaches b55. `src/hp67.rs` restores oracle flow mutations, uses the structural preview for successor fetch, verifies completed flow against the oracle and commits it.

## Responsibilities

Own sequential increment and address wrap, JSB, conditional GOTO, implied-GOTO payload, return, key/A dispatch, immediate/delayed ROM selection and bank switching. Preserve delayed-ROM override ordering and the incremented JSB return address. Apply low-page bank-zero selection at the same logical fetch boundary as before.

## Implementation

`ActSerialFlowState` groups only migrated fields. `ActSerialFlowResultImage::begin` independently computes their next values without executing an architectural instruction or changing live state. Fetch may consume PC/bank from this read-only preview. Completion and live commit require the full b0..b55 word. Implied-GOTO takes precedence over opcode decoding and clears its instruction-state latch at completion; other instruction-state effects remain with existing owners.

Early successor preview and completed-word commit are WORKING APPROXIMATIONS of ownership, not physical latch timings. Exact internal PC/stack/bank edges, propagation and SYNC timing remain SOURCE-BLOCKED. Exhaustive composed-oracle tests cover all ten-bit words over carry, implied-GOTO, delayed selection, stack slots and page/wrap boundaries; focused tests lock delayed-ROM JSB and payload precedence.
