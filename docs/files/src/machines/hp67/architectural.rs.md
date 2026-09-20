# `src/machines/hp67/architectural.rs`

## Purpose

Composes the independent ACT architectural core, temporary architectural RAM image and CRC control core into one HP-67 instruction-boundary bring-up machine.

## Why it exists

After the ACT opcode space was implemented, the next long-run firmware stop occurred on octal `1000`, which is not an ACT special at all: it is a CRC control instruction. Long firmware runs therefore need correct chip ownership at instruction boundaries rather than forcing every 10-bit word through the ACT decoder. This layer resolves that ownership while keeping the final electrical machine work separate.

## Relationships

Uses `act.rs` for Woodstock ACT behavior, `crc.rs` for CRC flag/control instructions and the temporary `ActRamImage` until physical 1818-* RAM timing is connected. `hp67_poweron_smoke.rs` drives this composed machine with words reconstructed from the serial IS/ISA fetch path. Reference implementations are not imported.

## Responsibilities

Route THEN-GOTO target words to the ACT regardless of bit pattern, route recognized CRC control words to the CRC while still applying the ACT's universal instruction-boundary work, latch true CRC flag tests into ACT status bit S3, preserve the HP-67 bank-zero fetch rule, accept external PROGRAM and card-present hardware contacts, service CRC read port `0x9B` from the transport-fed 28-bit buffer and CRC write port `0x99` into the two-buffer write FIFO.

## Implementation

For an ordinary ACT word, `execute_word()` delegates to `ActArchitecturalCore`. For a CRC control opcode it executes an ACT NOP-equivalent boundary cycle so PC/carry/P-history/delayed-ROM behavior still advances correctly, then applies the CRC side effect. A true CRC test represents the CRC pulsing the ACT F2 input, which the architectural composition latches into S3; a false CRC test does not clear S3, because firmware explicitly owns the preceding `0 -> s 3` where required. A pending THEN-GOTO state bypasses CRC decode because that 10-bit ROM word is branch data rather than an opcode. `set_card_present()` drives only CRC external flag 10, just as `set_program_mode()` drives flag 1. Reads targeting CRC address `0x9B` now execute the real ACT RAM-read-class boundary bookkeeping, consume one 28-bit CRC buffered record, and duplicate its seven nibbles into C[0..6] and C[7..13]. The operation is surfaced as `Hp67ArchitecturalOperation::CrcDataRead`. A read with no buffered record fails transactionally with `ReadBufferEmpty`. Writes targeting `0x99` now execute the ACT RAM-write-class boundary bookkeeping, pack only C[13:7] into one 28-bit word and enqueue it in the CRC write FIFO. The low seven C nibbles, including the `0x99` address artifact used by the compact firmware sequence, are intentionally ignored. A write outside firmware write mode fails transactionally with `WriteModeInactive`.
