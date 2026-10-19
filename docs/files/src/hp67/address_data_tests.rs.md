# src/hp67/address_data_tests.rs

## Purpose

Regress live C-to-address DATA transport through real resolved nets.

## Why it exists

An autonomous RAM receiver needs an actual address frame; an ACT scalar latch
alone cannot address an independent device.

## Relationships

Uses Hp67LiveMachine's production traced/untraced cycle, structural snapshot,
clocked DATA phase, five-ROM transport and existing reference RAM comparisons.

## Responsibilities

Cover frozen C, address/write/read dependencies, all clock counts, DATA owner
identity, peripheral addressing without RAM writes, implied-GOTO exclusion,
tail contention, reset and real-firmware trace parity.

## Implementation

Short synthetic command sequences inject executing words through the existing
reference fixture and preserve concurrent ROM fetch. Their DATA transport is
the production path. A separate 1200-word real-firmware boot comparison requires
actual address frames, idle reachability and 224 transitions per word. The
contended-tail case checks pending/state preservation before the next command.
These are digital logical-level tests, not evidence for exact physical edges.
