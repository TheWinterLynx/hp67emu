# src/machines/hp67/clock_integrity_tests.rs

## Purpose

Exercise PHI ownership and phase failures through electrical and transport boundaries.

## Why it exists

A binary resolved clock level alone does not prove ACT ownership or authorize a receiver.

## Relationships

Uses the HP-67 electrical backplane, actual shared transports and retained device state.

## Responsibilities

Reject electrical faults without advancing time or consuming device input; preserve recovery and the complete PHI topology.

## Implementation

Six always-enabled tests cover every foreign driver in four phases on both clocks, missing/replaced/wrong ACT levels, staged visibility, six composition entry points, all receiving device lanes, and checked/raw traversal equality over fifteen words. Pin and endpoint images remain unchanged on rejected operations. These are fault-injection simulator contracts, not physical timing measurements.
