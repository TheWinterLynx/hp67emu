# src/hp67/owned_crc_data_tests.rs

## Purpose

Verify owned CRC DATA through firmware, actual card transport and comparison-only reference composition.

## Why it exists

Production CRC payloads must be sourced from retained device buffers rather than observer storage.

## Relationships

Works with crc.rs, act_serial_crc_data.rs, card_transport.rs, serial_crc_control.rs and the live adapter.

## Responsibilities

Preserve strict comparisons, two-buffer capacity, FIFO ordering, completed-word publication, card identity and reset/reference composition.

## Implementation

Three regressions cover keyboard/traced execution/reset, corrupted FIFO mirrors rejected before clocks and pending RAM tails, and complete real-firmware write/eject/reinsert/read of all 34 records. Production ACT, CRC, media and 224-transition word timing agree with the reference at every word. Existing full regressions retain clipping and both-track behavior.

Logical 28-bit records and b55 publication remain a working approximation. Magnetic order, CRC DATA wire ownership/phase and exact physical edges remain source-blocked.


M16G strengthens compare_word with resolved MOTOR/CRC-latch agreement on every step, so the existing full real-firmware write/read of all 34 records exercises the production MOTOR gate as well as retained CRC DATA. No duplicate card roundtrip harness is added.


M16H strengthens the existing full real-firmware 34-record write/read roundtrip: compare_word checks resolved MTRS against the independently updated card-present observer after every word, including actual card exit. The expensive scenario is reused rather than duplicated.
