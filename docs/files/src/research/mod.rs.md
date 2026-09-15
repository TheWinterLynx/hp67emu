# `src/research/mod.rs`

## Purpose

Defines the namespace for local research tooling that imports, decodes, assembles and compares external HP-67 firmware corpora and emulator containers.

## Why it exists

Firmware provenance work must remain separate from the production electrical emulator. The project needs utilities for comparing independent ROM sources and decoding research containers without making those utilities, or copyrighted ROM bytes, part of the emulated machine.

## Relationships

Exports `research::rom_corpus` for normalized ROM comparison, `research::teenix` for the Teenix `NeWe` XOR container, `research::woodstock_asm` for strict mnemonic-to-word assembly, and `research::teenix_hp67` for current HP-67 listing analysis/normalization. It is exposed by `src/lib.rs` so local command-line research tools such as `src/bin/rom_compare.rs` can reuse the same tested logic.

## Responsibilities

Keep research/import/decoding/assembly code clearly separated from `emulation`, `machines` and `reference`. Do not place production device behaviour or electrical timing assumptions here.

## Implementation

The module re-exports small helpers with explicit evidence boundaries: ROM corpus normalization/comparison, Teenix outer-container decoding, documented Woodstock text assembly and HP-67-specific listing topology validation. Unknown source grammar remains a hard error rather than being guessed.
