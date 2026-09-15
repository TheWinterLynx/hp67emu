# `src/research/mod.rs`

## Purpose

Defines the namespace for local research tooling that imports, decodes and compares external HP-67 firmware corpora and emulator containers.

## Why it exists

Firmware provenance work must remain separate from the production electrical emulator. The project needs utilities for comparing independent ROM sources and decoding research containers without making those utilities, or copyrighted ROM bytes, part of the emulated machine.

## Relationships

Exports `research::rom_corpus` for normalized ROM comparison and `research::teenix` for the Teenix `NeWe` XOR container. It is exposed by `src/lib.rs` so local command-line research tools such as `src/bin/rom_compare.rs` can reuse the same tested logic.

## Responsibilities

Keep research/import/decoding code clearly separated from `emulation`, `machines` and `reference`. Do not place device behaviour or timing assumptions here.

## Implementation

The module re-exports small, source-specific helpers with explicit evidence boundaries: ROM corpus normalization/comparison and Teenix outer-container decoding. Internal `.pfl` source grammar remains separate until proven.
