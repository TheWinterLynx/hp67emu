# src/hp67/owned_register_tests.rs

## Purpose

Exercise retained ACT registers through the production live machine.

## Why it exists

Execution inputs must come from retained device state rather than a host oracle snapshot.

## Relationships

Uses the fetch endpoint, existing structural result images and the live adapter. See the M15W research note and existing M14/M15 authority contracts.

## Responsibilities

Preserve comparison guards, explicit input causality, reset/reference composition and the 56-cell/224-transition schedule.

## Implementation

Traced/untraced production and reference run through firmware boot and key contacts, with full ACT equality, DATA tails, reset and 224 transitions per word. Every non-flow register/latch mirror group is poisoned before clocks, including C and S3 with a pending tail. A physical-card insertion exercises logical CRC readiness and firmware consumption. Instruction qualifier faults retain the earlier M15U-specific guard.

Completed-word publication is a working approximation; these tests do not prove physical PHI edges, propagation, RAM chip mapping or magnetic serialization.
