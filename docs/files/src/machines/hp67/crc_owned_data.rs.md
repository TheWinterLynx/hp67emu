# src/machines/hp67/crc_owned_data.rs

## Purpose

Retain both CRC read and write FIFO pairs independently of the architectural observer.

## Why it exists

Production CRC payloads must be sourced from retained device buffers rather than observer storage.

## Relationships

Works with crc.rs, act_serial_crc_data.rs, card_transport.rs, serial_crc_control.rs and the live adapter.

## Responsibilities

Preserve strict comparisons, two-buffer capacity, FIFO ordering, completed-word publication, card identity and reset/reference composition.

## Implementation

Fixed-size CrcDataState storage implements record ingress, write draining and side-effect-free port staging. Invalid range/mode/empty/full staging preserves state; transport overflow asserts existing F7. Completed result images reject stale pre-state. Independent control supplies handshake latches, and an agreeing transport observer gates mirror publication. Four regressions cover raw opcode/address/qualifier combinations, FIFO wrap and rejection, poisoned mirrors and all 34 read/write records at 95/100/105 percent speed.

Logical 28-bit records and b55 publication remain a working approximation. Magnetic order, CRC DATA wire ownership/phase and exact physical edges remain source-blocked.
