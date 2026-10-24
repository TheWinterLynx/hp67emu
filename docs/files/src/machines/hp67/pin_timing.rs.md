# src/machines/hp67/pin_timing.rs

## Purpose

Audit observed setup/hold separations at every PHI edge and signal stability across PHI1 low.

## Why it exists

A single provisional sample checkpoint cannot establish receiver timing or hide ambiguous/censored observations. Diagnostic work must not impose per-transition recording checks in the normal build.

## Relationships

Uses pin_trace, validated PHI topology, timing.rs named edges and the native CLI. Runtime record hooks are controlled separately by the pin-capture feature.

## Responsibilities

Preserve source/time domains, ordered observations and strict failure/coverage reporting without inventing hardware timing or removing electrical work.

## Implementation

Uses validated PinClockObservation row indices to find nearest observed binary changes with partition-point lookup. Unknown/uncaptured boundaries remain censored; a pin and clock change in one publication is explicitly ambiguous. Each pin reports all four edge margins, unresolved coverage, PHI1 interior/boundary changes and phase-value disagreements. Ordinal delta ordering is retained without conversion to nanoseconds. These are observed separations, never chip timing requirements or causal propagation delays.

See `docs/PIN_CAPTURE_v1.1.md`. Default and diagnostic builds both pass through the owner local gate before integration.
