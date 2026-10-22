# src/hp67/received_execution_tests.rs

## Purpose

Verify production ACT-owned instruction execution through the live firmware bridge.

## Why it exists

A host-side prefetch mirror must not become the source of a production instruction
or let an invalid serial return manufacture a qualified complete command.

## Relationships

Exercises ActSerialEndpoint, the resolved IS/SYNC backplane, shared transports,
ACT execution lifetime and live oracle/reference composition.

## Responsibilities

Lock ordered reception, atomic qualification, one-word command latency, pending
instruction preservation and unchanged electrical scheduling.

## Implementation

Compare production, traced production and explicit reference firmware while keyboard contacts cause RAM tails and implied GOTO. Require complete ACT/CRC agreement, the executed word matching the prior device frame, 224 transitions and complete trace length. Preserve reset composition. Poison the pipeline with a DATA tail pending and poison the qualifier mirror; both fail before clock advancement, tail consumption or ACT mutation.

The b46..b55 window and following-word execution use the existing hardware
contract. The PHI checkpoint remains provisional; no new physical timing claim.
