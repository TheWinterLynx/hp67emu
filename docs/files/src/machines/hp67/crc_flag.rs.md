# src/machines/hp67/crc_flag.rs

## Purpose

Connect independently decoded CRC control to resolved active-low F2 and ACT reception.

## Why it exists

CRC condition values previously crossed directly into ACT S3, bypassing the named electrical net and conductor validation.

## Relationships

Uses the independent CRC IS/SYNC control receiver, the HP-67 backplane and retained ACT register port. The architectural result is comparison-only on production.

## Responsibilities

Preserve sticky S3 and command pipeline semantics, source identity, resolved levels, reset/reference separation and all 224 PHI transitions.

## Implementation

Hp67CrcFlagBridge retains only its last source assertion. attach establishes the explicit inactive-High logical bridge. Read-only preflight validates sole CRC ownership, binary resolution and unchanged committed source. begin_word releases the last delta pulse; publish consumes only the autonomous CRC result at a completed word. sample_crc_f2 converts an active Low from the strict resolved pin to ACT assertion. No passive bias or measured output circuitry/pulse duration is inferred.

Wiring/active-low net evidence and explicit boundary/push-pull approximations are recorded in `docs/research/M16C_RESOLVED_CRC_FLAG_2026-10-25.md`.
