# src/hp67/owned_flow_tests.rs

## Purpose

Verify owned ACT flow registers in the actual live firmware composition.

## Why it exists

A checked host flow mirror must not remain the source of the ACT's PC, return
storage or successor address. These tests prove causal separation and lifecycle.

## Relationships

Uses ActSerialEndpoint, ActSerialFlowResultImage, resolved IS/SYNC, ROM address
reception and the composed architectural oracle. Native tests use real firmware.

## Responsibilities

Lock own state, preview/completion chronology, bank normalization, mirror guards,
reset and unchanged 56-cell/224-transition transport.

## Implementation

Compare traced/untraced production against reference firmware with physical key contacts and require the actual ROM received address to match owned PC. Poison each host flow field with DATA pending and require failure before any clocks, tail, pipeline or ACT mutation. Isolate startup serialization below the mirror guard and prove a poisoned host PC/bank cannot drive the address. Verify reset clears and preserves production/reference ownership.

This tests logical ownership, not physical silicon write edges or PHI calibration.
