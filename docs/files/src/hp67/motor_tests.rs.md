# src/hp67/motor_tests.rs

## Purpose

Verify production MOTOR preflight, pending RAM-tail recovery, real card-present motor start, reset and explicit reference separation.

## Why it exists

Transport previously accepted the logical motor latch directly, bypassing the CRC MOTOR conductor and source validation.

## Relationships

Uses independent IS/SYNC CRC control, the named Motor backplane net and the existing word-based card transport. Evidence and limits are in docs/research/M16G_RESOLVED_MOTOR_2026-10-26.md.

## Responsibilities

Preserve CRC control pipeline and prior-interval cadence, reject missing/replaced/agreeing/contended/tampered sources without overwriting evidence, retain outputs through idle and leave physical timing/circuitry uncertainty explicit.

## Implementation

Hp67MotorOutput remembers its held source level. Strict sampling requires one CRC contribution and Low/High resolution; Low means On. Preflight precedes inputs/clocks/tails. Completed CRC control publishes changed output only; transport consumes its pre-word resolved sample for the elapsed interval. Reset establishes normalized inactive High, while explicit reference fixtures release the source. No PHI edge is added. Tests cover all 1024 commands/both qualifiers/initial states, capture and legacy unknown columns, real firmware and recovery.

## Occupied-reset regression contract

The real-firmware motor-start test now checks reset rejection while the card is
retained, preserving transport, tick and motor-on state in both production and
reference compositions. It explicitly recovers the exact blank card through the
power-off API before resetting electronics. Recovery itself leaves the CRC motor
output on; the following reset must still establish inactive production MOTOR
and the released reference net. This updates the obsolete destructive-reset test
sequence without relaxing the media-preservation guard or motor assertions.
