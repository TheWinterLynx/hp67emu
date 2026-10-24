# src/machines/hp67/pin_timing_tests.rs

## Purpose

Prove phase-sensitive timing analysis with labelled synthetic waveforms.

## Why it exists

A single provisional sample checkpoint cannot establish receiver timing or hide ambiguous/censored observations. Diagnostic work must not impose per-transition recording checks in the normal build.

## Relationships

Uses pin_trace, validated PHI topology, timing.rs named edges and the native CLI. Runtime record hooks are controlled separately by the pin-capture feature.

## Responsibilities

Preserve source/time domains, ordered observations and strict failure/coverage reporting without inventing hardware timing or removing electrical work.

## Implementation

Four tests use explicit nonuniform clock and pin events to verify all setup/hold ranges, censored constant signals, unobserved DATA, same-publication ambiguity, PHI1-window transitions, ordinal same-tick order, anchors and invalid clock/input rejection. No synthetic value is asserted as HP-67 hardware timing.

See `docs/PIN_CAPTURE_v1.1.md`. Default and diagnostic builds both pass through the owner local gate before integration.
