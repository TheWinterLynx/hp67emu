# src/machines/hp67/crc_transaction_tests.rs

## Purpose

Verify CRC prepared transactions against the exact pre-M16K full-copy algorithm.

## Why it exists

Performance-oriented representation changes must preserve every retained bit and rejection boundary.

## Relationships

Child of serial_crc_control; exercises resolved IS/SYNC, the shared DATA path and the existing provisional PHI schedule.

## Responsibilities

Compare every cell, rejected phase/replay, all opcode/qualifier pairs, discarded preparations and fault/retry; keep benchmarks explicit and separate from correctness.

## Implementation

The old M16J algorithm is test-only. Four flag/contact patterns traverse consecutive 1024-word streams under both qualifiers; CRC images match before/after every cell and rejection. DATA faults at every cell and final-return faults cannot publish frames or execute latches. An ignored release benchmark rotates the two kernels for seven rounds of 50000 words, uses equal black-box boundary counts, checks full final images/224 edges, and reports medians without a speed assertion. It is a kernel comparison, not whole-live A/B or physical evidence.


## M16K v2 fixture correction

The final-bit fault/retry fixture uses opcode 0o1000, independently decoded as SET flag 4 by crc.rs and its pre-existing opcode-family regression. v1 incorrectly asserted flag 0 after success. v2 fixes that expectation without changing production behavior: all twelve flags and the receive/result images must remain unpublished on the precise b55 InvalidInstruction error, both algorithms must preserve identical complete state, and a clean retry must publish exactly flag 4/result plus the next received frame after 448 total PHI transitions. No assertion is deleted or fault check weakened. Owner runtime validation remains pending.
