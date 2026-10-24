# src/hp67/clock_integrity_tests.rs

## Purpose

Verify that live PHI faults cannot partially mutate firmware or consume pending RAM DATA.

## Why it exists

A binary resolved clock level alone does not prove ACT ownership or authorize a receiver.

## Relationships

Uses the HP-67 electrical backplane, actual shared transports and retained device state.

## Responsibilities

Reject electrical faults without advancing time or consuming device input; preserve recovery and the complete PHI topology.

## Implementation

Two tests reject every foreign PHI source before inputs/authority mutation and inject a fault before an actual firmware RAM tail. After removing the driver, retry is compared with an unfaulted real-firmware machine for 32 further cycles. No custom opcode replaces the real command stream.
