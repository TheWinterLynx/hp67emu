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
