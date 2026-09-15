# `src/machines/hp67/crc.rs`

## Purpose

Provides an independent architectural bring-up model of the HP-67 card-reader controller (CRC) control/flag interface.

## Why it exists

Real HP-67 firmware reaches CRC opcodes during ordinary power-on/idle execution. Treating those words as unknown ACT specials would stop long firmware runs for the wrong reason. At the same time, card transport, magnetic sensing and DATA-bus timing are not yet modeled, so the bring-up layer must separate the already-understood control opcodes from the still-pending physical card path.

## Relationships

`architectural.rs` composes this CRC control core with the independent ACT core. The semantic `reference::crc` implementation remains test/oracle material only and is not imported here. Later physical CRC/device code can replace this architectural control scaffold while preserving the same firmware-visible flag behavior.

## Responsibilities

Decode the documented CRC control opcode families, maintain twelve internal flags and twelve external flag inputs, implement set-flag and test-and-clear semantics, and expose the HP-67 program-mode external flag. Reject out-of-range opcodes and flags explicitly.

## Implementation

`decode_crc_opcode()` recognizes the two documented selector families that cover CRC flags 0 through 11. `CrcArchitecturalCore::execute_opcode()` sets internal flags or returns the combined internal/external condition for test-and-clear while clearing only the internal latch. The RUN/W-PRGM input is represented as external flag 1. Card data registers at addresses `0x99` and `0x9b` are declared here but intentionally not serviced by this module; crossing those ports remains a later physical-device milestone.
