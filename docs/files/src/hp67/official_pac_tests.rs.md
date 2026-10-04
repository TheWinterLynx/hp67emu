# `src/hp67/official_pac_tests.rs`

## Purpose

Validate Moving Average from the built-in Standard Pac with real firmware and physical inputs.

## Why it exists

Custom diagnostics exercise synthetic cases; official application examples add continuous real-program state and two-track corpus coverage.

## Relationships

Test-only child of `src/hp67.rs`, using the existing library importer, diagnostic boot/card/segment helpers and M12 keyboard dispatch helper. Evidence and limits are recorded in `docs/research/M14P_OFFICIAL_PAC_ACCEPTANCE_2026-10-04.md`.

## Responsibilities

Require both recorded tracks, load them through firmware Crd transport, drive nine bounded functional checkpoints and compare stable raw segment frames with independent expected values.

## Implementation

The normal test checks both required tracks exist. The explicitly ignored release acceptance boots once and keeps program/window state across all checkpoints. It uses physical key contacts and observes no-key RUN wait plus S2/S15 clear and full segment equality. It never writes calculator state or computes application answers. M14P retains continuous in-memory use for its final 225 input. M14Q reuses the first eight checkpoints, saves through B/Crd to blank media, persists native bytes, drops the writer, boots a new reader and reloads program/data before three recovery checks. Only media crosses the machine boundary. HP-97 printer behavior, analog power reset and magnetic timing are excluded. See `docs/research/M14Q_OFFICIAL_PAC_DATA_RECOVERY_2026-10-04.md`.
