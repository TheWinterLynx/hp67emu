# src/hp67/is_owner_tests.rs

## Purpose

Verify actual live wrong-owner rejection, pending DATA-tail non-consumption, reset after a partial fault and traced/untraced firmware parity.

## Why it exists

A sole active-high IS contributor can produce a valid binary level while being the wrong chip or acting outside its window.

## Relationships

Tests fetch.rs shared source validation, ROM-selected identity, the strict resolver and live production adapter. Source boundaries are documented in docs/research/M16F_IS_WINDOW_OWNERS_2026-10-26.md.

## Responsibilities

Keep wrong-source and passive-zero cases distinct; prove actual pin payloads remain authoritative; preserve 56 cells/224 transitions and precise fault coordinates.

## Implementation

Literal display/address/return windows are checked for all 56 coordinates, all 13 sources and all five physical ROM profiles. Aggregate reference identity remains separate. Tests preserve polarity/overlap/phase priority, transmit altered legitimate bits, exercise actual autonomous bank selection, stop prefix samples before DATA consumption, and verify native reset/parity. Optional capture retains the invalid source and exact two-edge failure image. No analog output-enable, propagation or physical latch timing is inferred.
