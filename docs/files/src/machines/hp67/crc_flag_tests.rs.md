# src/machines/hp67/crc_flag_tests.rs

## Purpose

Verify causal CRC command-to-pin behavior and electrical fault rejection.

## Why it exists

CRC condition values previously crossed directly into ACT S3, bypassing the named electrical net and conductor validation.

## Relationships

Uses the independent CRC IS/SYNC control receiver, the HP-67 backplane and retained ACT register port. The architectural result is comparison-only on production.

## Responsibilities

Preserve sticky S3 and command pipeline semantics, source identity, resolved levels, reset/reference separation and all 224 PHI transitions.

## Implementation

Four tests transmit every CRC command/qualifier/contact combination through real IS/SYNC and 224 PHI edges per word, prove the one-word command pipeline and internal test/clear behavior, reject every foreign contributor and replaced/missing/changed CRC source, and preserve source state at invalid publication phases. Fixtures do not inject completed CRC results.

Wiring/active-low net evidence and explicit boundary/push-pull approximations are recorded in `docs/research/M16C_RESOLVED_CRC_FLAG_2026-10-25.md`.


## M16C gate v2 fixture correction

The owner's v1 gate exposed a fixture error: flag 0 (READY) has TEST/CLEAR
but no SET opcode. The internal pulse test now explicitly locks that absence
and covers SET -> TEST -> repeated TEST for every settable flag 1..11, each
through independent IS/SYNC reception and the existing one-word pipeline.
READY remains supplied by the buffer/transport handshake; the decoder and
production source/ACT behavior are unchanged. Owner full gate remains pending.
