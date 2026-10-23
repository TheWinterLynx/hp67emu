# src/machines/hp67/display_circuit.rs

## Purpose

Own the downstream cathode counter, anode latch and complete-scan exposure from pin-derived ROM0 transactions.

## Why it exists

Downstream display selection must arise from the cathode device and resolved pins, rather than an out-of-band ACT slot annotation in host code.

## Relationships

Works with display.rs, display_exposure.rs, fetch.rs and the live adapter. Presentation consumes masks and completed exposure.

## Responsibilities

Preserve scan roles, strict ownership/order, atomic rejection, reset and the existing fifteen-slot optical policy.

## Implementation

Accepts only ordered receiver transactions, validates one strobe per 224 ordinal transitions, derives scan roles from the retained cathode counter, decodes the actual byte, and applies RCD overlap and nominal dwell atomically. Replay, skipped strobes, decode, RCD and exposure errors leave all circuit state unchanged. The existing enable/startup bridge remains an explicit input. Five embedded tests cover scans, raw reset, replay, strict named owners and electrical faults at every coordinate.

Physical STR/RCD widths, exact PHI edges, internal propagation and LED current remain unproven.
