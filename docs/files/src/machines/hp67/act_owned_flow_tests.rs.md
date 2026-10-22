# src/machines/hp67/act_owned_flow_tests.rs

## Purpose

Verify ACT-owned flow state, independent input selection and completed-word latches.

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

Exercise all 1024 words across PC, carry, instruction/data and delayed-ROM contexts while every host flow field is poisoned, retaining explicit unsupported-oracle exclusions. Independently received flow instructions cover calls, returns, dispatch, pages, bank changes and implied GOTO. Check successor preview before b16, unchanged latches through b54, b55 commit and bank normalization. Invalid execution coordinates cannot commit early.

This tests logical ownership, not physical silicon write edges or PHI calibration.
