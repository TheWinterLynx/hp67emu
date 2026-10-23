# src/hp67/display_circuit_tests.rs

## Purpose

Validate the complete live display lane through firmware, physical key contacts, RAM prefixes and optics.

## Why it exists

Downstream display selection must arise from the cathode device and resolved pins, rather than an out-of-band ACT slot annotation in host code.

## Relationships

Works with display.rs, display_exposure.rs, fetch.rs and the live adapter. Presentation consumes masks and completed exposure.

## Responsibilities

Preserve scan roles, strict ownership/order, atomic rejection, reset and the existing fifteen-slot optical policy.

## Implementation

Three regressions compare production/reference and traced/untraced execution over boot and RCL/key contacts, decode a separate mask oracle, require DATA-tail coverage and 224 transitions per word, and verify reset. ACT scan poisoning fails before clocks/tails. Extra STR/RCD owners fail without changing published display state. Altering the host diagnostic frame cannot feed the circuit or optical exposure.

Physical STR/RCD widths, exact PHI edges, internal propagation and LED current remain unproven.
