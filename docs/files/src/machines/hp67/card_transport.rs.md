# src/machines/hp67/card_transport.rs

## Purpose

Model the HP-67 magnetic-card transport cadence between one physical card, the selected end-for-end logical track, and the CRC 28-bit read/write buffers. The selected logical stream is physically represented by the separate Zero/One flux-track pair in card_flux.rs.

## Why it exists

The HP-67 firmware does not treat motor_on as equivalent to data availability. HP documentation separates card-present/motor control, the head switch, magnetic sensing and the CRC buffer; the HP Journal further gives a nominal 28 ms interval between CRC-visible 28-bit records. This transport layer keeps those physical concerns separate from the CRC flag/control model and from host file formats.

## Relationships

The transport owns an Hp67MagneticCard while it is inside the reader. CardInsertionEnd selects logical Track 1 or Track 2 from magnetic_card.rs. card_flux.rs describes the two physical parallel flux tracks that encode one logical stream, but the transport still operates at 28-bit CRC-record granularity. The selected Hp67MagneticTrack feeds CrcArchitecturalCore::present_read_word() in crc.rs and receives queued write words in write mode. Hp67LiveMachine in src/hp67.rs owns the firmware-synchronized card-present/head lifecycle. The transport never parses Teenix .hpp, .hp67raw or .hp67card files and never owns artwork.

## Responsibilities

Carry one complete physical card through the reader; expose only the logical track selected by the insertion end; gate record movement on motor enable and head-active state; support the documented 95-105% reader-speed range with 100% as the default; present recorded words to the CRC at the speed-adjusted record cadence; let an unrecorded logical track cross the head without inventing zero records; drain queued write words at the same cadence; materialize an initially unrecorded track when it is written; enforce write protection independently on the selected logical track; preserve the non-selected track unchanged; and release the same physical card after the 34th record position.

## Implementation

Hp67CardTransport stores Option<Hp67MagneticCard> plus Option<CardInsertionEnd>, next-record position, partial record time, Hp67CardSpeed and head/startup-handshake state. active_track() resolves the selected logical card track from the insertion end. insert_card() accepts the whole card and an explicit end; take_completed_card() returns that same card so the caller can rotate it 180 degrees and reinsert the opposite end.

advance_us() advances only while a card is present, the firmware-owned motor flag is on and the modeled head switch is active. The nominal interval is HP67_NOMINAL_CARD_RECORD_US = 28_000 at 100% speed. Hp67CardSpeed accepts only 95 through 105 percent, matching HP's documented +/-5% reader-speed range. record_interval_us() exposes the corresponding rounded human-readable cadence (26,667 us at 105%, 29,474 us at 95%), but transport timing does not round every record independently: record_phase_units accumulates elapsed microseconds multiplied by the configured speed percentage against an exact 2,800,000-unit threshold, preserving fractional phase between records and avoiding artificial drift across a 34-record pass. Exact motor acceleration, head-switch position, record-to-bit serialization, absolute magnetic polarity and sense-amplifier electrical behavior remain later M13 work.

For a recorded track, read cadence presents each of the 34 28-bit words to the CRC in order. For TrackMedia::Unrecorded, the same 34 physical record positions pass the head but no synthetic data-ready event or zero word is generated.

In write mode the CRC pair of 28-bit buffers remains authoritative. While motor/head are active and the selected track is writable, firmware 0x99 writes enqueue records and the transport commits one FIFO record every nominal 28 ms. The first committed record materializes an initially unrecorded track. A clipped/protected selected track raises the F7 status path and remains unchanged; protection on the other track has no effect.

Read-side buffering follows the HP hardware description: the CRC can retain two 28-bit records while the card continues moving. The transport may enqueue a second record while firmware processes the first but never silently overwrites either one.

Closing the modeled head switch first raises the existing CRC buffer_ready startup/readiness event without advancing the magnetic position. Firmware acknowledges that event in Scr3341; only then does record_stream_active() become true and the 28 ms logical record cadence begin. The same startup sequence is shared by read and write paths.
