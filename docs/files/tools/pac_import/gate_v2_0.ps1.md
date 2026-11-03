# `tools/pac_import/gate_v2_0.ps1`

## Purpose

Support reproducible local HP-67 PAC import, artwork repair and publication.

## Why it exists

Keep the reviewed workflow versioned with the emulator rather than in detached download bundles.

## Relationships

See `docs/PAC_LIBRARY_v2.2.md`; the tools operate on the native media catalog and image atlas, never machine execution state.

## Responsibilities

Preserve magnetic media, original source references and user work; fail on invalid inputs and report progress.

## Implementation

This file retains the local import/repair implementation or provides the full-name catalog/publication adapter. Rust compilation and tests run only in the owner local gate.
