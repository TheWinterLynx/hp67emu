# src/hp67/autonomous_crc_tests.rs

## Purpose

Verify independent CRC control in the actual desktop firmware composition.

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

Compare production, traced production and reference firmware through boot and card-present motor activation, requiring ACT/CRC equality, independent latch agreement and 224 transitions/word. Verify reset clears production state and preserves reference composition. Reject a poisoned motor mirror before clocks or instruction changes. A real logical card track exercises transport READY and firmware CRC data consumption without injecting an opcode.

Completed-word control timing remains a WORKING APPROXIMATION. Exact CRC F2,
PHI latch edges, output circuitry, magnetic sensing and DATA serialization remain
SOURCE-BLOCKED. No new physical timing value is introduced.
