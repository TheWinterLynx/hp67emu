# src/machines/hp67/act_received_tests.rs

## Purpose

Prove ACT instruction causality through actual resolved IS/SYNC reception.

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

Exhaustively receive all 1024 words with both qualifiers and begin the next execution from the retained frame. Reject duplicate/out-of-order samples, invalid PHI checkpoint, IS/SYNC faults and pending-frame overwrite without receiver mutation. Demonstrate that changing an observed pin changes the executed word and that bad qualifier binding or an active execution preserves the pending frame.

The b46..b55 window and following-word execution use the existing hardware
contract. The PHI checkpoint remains provisional; no new physical timing claim.
