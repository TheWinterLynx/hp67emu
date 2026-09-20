# `src/machines/hp67/crc.rs`

## Purpose

Provides an independent architectural bring-up model of the HP-67 card-reader controller (CRC) control/flag interface.

## Why it exists

Real HP-67 firmware reaches CRC opcodes during ordinary power-on/idle execution. Treating those words as unknown ACT specials would stop long firmware runs for the wrong reason. At the same time, card transport, magnetic sensing and DATA-bus timing are not yet modeled, so the bring-up layer must separate the already-understood control opcodes from the still-pending physical card path.

## Relationships

`architectural.rs` composes this CRC control core with the independent ACT core. The semantic `reference::crc` implementation remains test/oracle material only and is not imported here. Later physical CRC/device code can replace this architectural control scaffold while preserving the same firmware-visible flag behavior.

## Responsibilities

Decode the documented CRC control opcode families, maintain twelve internal flags and twelve external flag inputs, implement set-flag and test-and-clear semantics, expose the HP-67 PROGRAM switch plus card-present external contact and named firmware-visible buffer/motor/write flag identities, and provide the documented pair of 28-bit read buffers plus the two-entry 28-bit write buffering used by card I/O. Reject out-of-range opcodes, flags and card words explicitly.

## Implementation

`decode_crc_opcode()` recognizes the two documented selector families that cover CRC flags 0 through 11. `CrcArchitecturalCore::execute_opcode()` sets internal flags or returns the combined internal/external condition for test-and-clear while clearing only the internal latch. The RUN/W-PRGM input is external flag 1 and the physical card-present contact is external flag 10. Internal flag 9 remains the firmware-owned motor request; external contacts participate in `fs?c` tests without being cleared by the firmware test. The CRC core now owns a two-entry 28-bit read FIFO, matching HP's documented pair of alternating buffers. `present_read_word()` enqueues without overwriting unread data and raises `buffer_ready`; a third arrival with both buffers occupied fails explicitly with `ReadBufferFull`. `take_read_word()` consumes the oldest record at architectural `0x9B` and immediately reasserts `buffer_ready` when the second buffer is already full. Write mode now owns a two-entry FIFO matching the documented pair of 28-bit CRC buffers. `queue_write_word()` accepts a packed record only while firmware flag 11 is set; `take_queued_write_word()` is the transport-facing drain. Flag 0 remains the buffer-ready handshake. Flag 7 is deliberately named `CRC_FLAG_F7_STATUS` rather than given an invented hardware name: Teenix documents opcode 1700 as the firmware status check for valid read data / writable buffer, and the pinned firmware treats a true result on this path as an error condition.
