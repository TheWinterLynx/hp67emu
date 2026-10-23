# src/machines/hp67/act_owned_registers.rs

## Purpose

Retain the complete non-flow ACT register file independently of the comparison mirror.

## Why it exists

Execution inputs must come from retained device state rather than a host oracle snapshot.

## Relationships

Uses the fetch endpoint, existing structural result images and the live adapter. See the M15W research note and existing M14/M15 authority contracts.

## Responsibilities

Preserve comparison guards, explicit input causality, reset/reference composition and the 56-cell/224-transition schedule.

## Implementation

Captures A/B/C/Y/Z/T/M1/M2/F, pointer history, carry history, radix, status, instruction qualification, keyboard code, display controls and RAM address. Combines them with owned flow for execution binding and display sources. Structural images update registers at completed b55. Keyboard, reconstructed RAM C and independent CRC conditions use explicit guarded ports. Tests cover supported nonperipheral words across pointer/radix/carry/qualifier/address contexts, unchanged storage through b54, invalid port phases, sticky inputs and poisoned source mirrors.

Completed-word publication is a working approximation; these tests do not prove physical PHI edges, propagation, RAM chip mapping or magnetic serialization.
