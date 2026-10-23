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


### M15W owned ACT registers

The result_c accessor exposes the frozen logical C result for the retained ACT register file. Existing completion, pre-state comparison and FIFO commit guards remain authoritative. This does not introduce magnetic serialization or physical DATA timing.
Exact silicon register-write edges remain unproven. See `docs/research/M15W_ACT_OWNED_REGISTERS_2026-10-23.md`.


### M15X retained CRC DATA storage

One port classifier can stage from either reference or retained CRC data sources. Production uses begin_owned; no host FIFO is imported. The immutable image validates prior state and b55 completion before committing owned storage. Existing ACT C/FIFO result comparisons remain unchanged.
This closes logical storage ownership, not physical CRC DATA serialization or PHI timing. See `docs/research/M15X_CRC_OWNED_DATA_2026-10-23.md`.
