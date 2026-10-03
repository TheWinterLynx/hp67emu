# `src/machines/hp67/act_serial_crc_data.rs`

## Purpose

Provides completed-word logical CRC data-port payload and FIFO authority.

## Why it exists

The architectural composition still causally consumed/queued CRC records and wrote C. An independent staged result removes those mutations from the live structural word.

## Relationships

Uses frozen ACT state, the structural word class and scoped CRC FIFO storage. The endpoint binds it with CRC controls before b0 and completes it after b55. The live adapter restores oracle mutations, compares expected payload/C/storage and commits before physical card transport advances. RAM address selection remains M14K-owned; installed RAM uses its existing M14C cross-word path.

## Responsibilities

Classify fixed and selected CRC read/write ports, exclude implied-GOTO payloads, duplicate read nibbles into both C halves, pack writes from upper C, preserve FIFO order/handshakes, reject empty/full/inactive access and reject stale or incomplete commits. Preserve external inputs and unrelated control latches.

## Implementation

A Copy image captures pre/result C and scoped FIFO storage plus READY/F7. `stage_data_word` computes a storage transition independently from architectural take/queue methods without mutating the live CRC. Failed staging is transactional; the composition already rolls failed instructions back. Restore reverses only migrated oracle effects; commit requires completed execution and exact unchanged pre-state. Differential tests compare all 256 addresses and 1024 words, including wrapped FIFO heads and unrelated state. Live regression checks selected/fixed reads/writes and both buffers. The logical b55 handoff is WORKING APPROXIMATION, not electrical DATA or magnetic timing evidence. CRC DATA polarity/phase/driver enables, intra-record serialization and sense/PHI edges remain SOURCE-BLOCKED.
