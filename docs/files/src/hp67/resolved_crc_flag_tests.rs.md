# src/hp67/resolved_crc_flag_tests.rs

## Purpose

Validate resolved F2 in real firmware, reset, fault recovery and diagnostic capture.

## Why it exists

CRC condition values previously crossed directly into ACT S3, bypassing the named electrical net and conductor validation.

## Relationships

Uses the independent CRC IS/SYNC control receiver, the HP-67 backplane and retained ACT register port. The architectural result is comparison-only on production.

## Responsibilities

Preserve sticky S3 and command pipeline semantics, source identity, resolved levels, reset/reference separation and all 224 PHI transitions.

## Implementation

Three always-enabled regressions compare 2400 real-firmware cycles with the independent reference composition, reject all foreign F2 sources before inputs/clocks, and retry a fault before a real pending RAM tail against an unfaulted machine. One feature-only test waits for a real CRC assertion then captures 512 full words, CRC driver masks and TSV roundtrip. No synthetic opcode replaces production command reception.

Wiring/active-low net evidence and explicit boundary/push-pull approximations are recorded in `docs/research/M16C_RESOLVED_CRC_FLAG_2026-10-25.md`.
