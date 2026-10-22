# src/machines/hp67/serial_crc_control.rs

## Purpose

Own the twelve CRC control latches and receive their commands from resolved IS/SYNC.

## Why it exists

Move CRC control causality from the ACT-bound result image into an independently
received and retained peripheral state, with a reviewable live acceptance path.

## Relationships

Uses the resolved electrical backplane, strict raw command receiver, CRC semantic
decoder and architectural comparison mirror. The live DATA participant schedules
reception; logical FIFO and card transport retain their explicit READY/F7 port.

## Responsibilities

Preserve one-word command latency, qualified reception, unrelated latches and
external contacts; reject faults and mismatches; preserve 56 cells/224 PHI
transitions and the existing production/reference boundary.

## Implementation

A fixed-size Copy image samples all 56 coordinates, independently collects b46..b55 through the strict command receiver, and executes the previous qualified frame at completed b55. Invalid phase, order or pins preserve the complete endpoint. External contacts freeze only before b0 after mirror verification. The buffer port can change READY/F7 only. Tests cover all 1024 words, both SYNC qualifiers, every internal/external boolean combination, sequential set/test/clear, persistent contacts, pin faults, retries and shared DATA failure atomicity.

Completed-word control timing remains a WORKING APPROXIMATION. Exact CRC F2,
PHI latch edges, output circuitry, magnetic sensing and DATA serialization remain
SOURCE-BLOCKED. No new physical timing value is introduced.
