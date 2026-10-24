# src/timing_cli_tests.rs

## Purpose

Validate offline timing commands in normal and diagnostic native builds.

## Why it exists

A single provisional sample checkpoint cannot establish receiver timing or hide ambiguous/censored observations. Diagnostic work must not impose per-transition recording checks in the normal build.

## Relationships

Uses pin_trace, validated PHI topology, timing.rs named edges and the native CLI. Runtime record hooks are controlled separately by the pin-capture feature.

## Responsibilities

Preserve source/time domains, ordered observations and strict failure/coverage reporting without inventing hardware timing or removing electrical work.

## Implementation

Shared tests cover normalized physical-time input, reports, input protection and incomplete observations. Conditional tests require an explicit refusal without output in a normal build and a real firmware capture-to-timing audit in a diagnostic build. All files are temporary and removed; the gate creates no persistent file logs.

See `docs/PIN_CAPTURE_v1.1.md`. Default and diagnostic builds both pass through the owner local gate before integration.
