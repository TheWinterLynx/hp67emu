# `src/machines/hp67/card_transport.rs`

## Purpose

Model the HP-67 magnetic-card transport cadence between the physical reader/head boundary and the CRC 28-bit read/write buffers.

## Why it exists

The HP-67 firmware does not treat `motor_on` as equivalent to data availability. HP documentation separates card-present/motor control, the head switch, magnetic sensing and the CRC buffer; the HP Journal further gives a nominal 28 ms interval between CRC-visible 28-bit records. This transport layer keeps those physical concerns separate from the CRC flag/control model and from card-file persistence.

## Relationships

Feeds `CrcArchitecturalCore::present_read_word()` in `crc.rs`; is owned by `Hp67LiveMachine` in `src/hp67.rs`; uses the same 320 us observed firmware word time when advanced by the live machine. It does not decode firmware, alter ACT state, render the UI, or own the eventual card-file adapter.

## Responsibilities

Represent one inserted HP-67 card side as 34 validated 28-bit records; gate record movement on both motor enable and the head-active condition; accumulate nominal transport time; present each read record to the CRC every 28,000 us; drain queued write records to the inserted card at the same nominal cadence; preserve FIFO order; enforce per-side write protection; mark modified media dirty; and stop after the 34th record.

## Implementation

`Hp67CardSide` owns a fixed 34-word array, per-side write-protect state and dirty state, and rejects values wider than the CRC 28-bit record mask. `Hp67CardTransport` tracks the inserted side, next record, partial record time and head-active state. `advance_us()` advances only while a side is present, the firmware-owned motor flag is on and the head switch is active. The nominal interval is `HP67_NOMINAL_CARD_RECORD_US = 28_000`, source-backed by the November 1976 HP Journal description of the HP-67/97 card reader. Exact motor acceleration, head-switch position, ±5% speed variation, flux-transition timing and sense-amplifier electrical behavior remain later M13 work.


In write mode the CRC pair of 28-bit buffers is authoritative. While motor/head are active and the side is writable, the transport signals that capacity is available; firmware `0x99` writes enqueue records, and the transport commits one FIFO record every nominal 28 ms. A protected side raises the F7 status path used by firmware and no media word is modified.


Completed-media ownership: `take_completed_side()` succeeds only after all 34 record positions have crossed the head. It resets transport position/timing and releases the inserted `Hp67CardSide` without guessing when the separate external card-present switch mechanically opens. The caller remains responsible for that contact event.


Read-side buffering follows the HP hardware description: the CRC alternates between two 28-bit buffers while the card continues moving. The transport therefore may enqueue a second record while firmware processes the first, but it may not silently overwrite either one. Tests that advance multiple record intervals without an ACT-side consumer are intentionally invalid once both buffers are occupied.


Head-startup handshake: closing the modeled head switch does not immediately start record cadence. The transport first raises one CRC `buffer_ready` readiness event without consuming or producing a card record. Firmware acknowledges/clears that event in `Scr3341`; only after that acknowledgement does `record_stream_active()` become true and the 28 ms record clock begin. This same startup handshake is required in read and write mode, which follows directly from `Scr3341` being shared by both paths.
