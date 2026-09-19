# `src/machines/hp67/crc.rs`

## Purpose

Provides an independent architectural bring-up model of the HP-67 card-reader controller (CRC) control/flag interface.

## Why it exists

Real HP-67 firmware reaches CRC opcodes during ordinary power-on/idle execution. Treating those words as unknown ACT specials would stop long firmware runs for the wrong reason. At the same time, card transport, magnetic sensing and DATA-bus timing are not yet modeled, so the bring-up layer must separate the already-understood control opcodes from the still-pending physical card path.

## Relationships

`architectural.rs` composes this CRC control core with the independent ACT core. The semantic `reference::crc` implementation remains test/oracle material only and is not imported here. Later physical CRC/device code can replace this architectural control scaffold while preserving the same firmware-visible flag behavior.

## Responsibilities

Decode the documented CRC control opcode families, maintain twelve internal flags and twelve external flag inputs, implement set-flag and test-and-clear semantics, expose the HP-67 PROGRAM switch plus card-present external contact and named firmware-visible buffer/motor/write flag identities, and provide the single 28-bit read-buffer latch fed by the timed transport. Reject out-of-range opcodes, flags and card words explicitly.

## Implementation

`decode_crc_opcode()` recognizes the two documented selector families that cover CRC flags 0 through 11. `CrcArchitecturalCore::execute_opcode()` sets internal flags or returns the combined internal/external condition for test-and-clear while clearing only the internal latch. The RUN/W-PRGM input is external flag 1 and the physical card-present contact is external flag 10. Internal flag 9 remains the firmware-owned motor request; external contacts participate in `fs?c` tests without being cleared by the firmware test. The CRC core now also owns a one-record 28-bit read buffer. `present_read_word()` is the transport/sense boundary: it validates width, latches the newest record and raises `buffer_ready`; firmware test-and-clear may clear the flag without consuming the buffered record, and `take_read_word()` consumes it when the architectural `0x9B` read port executes. The write-side `0x99` path remains intentionally unimplemented.
