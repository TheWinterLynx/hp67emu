# `src/research/mod.rs`

## Purpose

Defines the namespace for local research tooling that imports and compares external HP-67 firmware corpora.

## Why it exists

Firmware provenance work must remain separate from the production electrical emulator. The project needs utilities for comparing independent ROM sources without making those utilities, or copyrighted ROM bytes, part of the emulated machine.

## Relationships

Exports `research::rom_corpus`. It is exposed by `src/lib.rs` so local command-line research tools such as `src/bin/rom_compare.rs` can reuse the same tested normalization and comparison logic.

## Responsibilities

Keep research/import code clearly separated from `emulation`, `machines` and `reference`. Do not place device behaviour or timing assumptions here.

## Implementation

The module currently re-exports only `rom_corpus`. Additional source-analysis helpers may be added here when they have a reusable, testable purpose.
