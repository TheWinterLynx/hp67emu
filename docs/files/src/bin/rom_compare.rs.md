# `src/bin/rom_compare.rs`

## Purpose

Provides the local command-line entry point for HP-67 firmware extraction, normalization, merging, Teenix container/listing analysis, inspection, evidence checking and comparison.

## Why it exists

ROM provenance must be checked locally against files obtained from their original sources. A dedicated tool makes that process reproducible without embedding third-party firmware in `hp67emu` or depending on GitHub Actions. The current 2026 Teenix HP-67 module uses XOR-obfuscated `NeWe` containers whose decoded payload is a Woodstock source-style listing.

## Relationships

Uses `src/research/rom_corpus.rs` for normalized ROM parsing/storage/comparison, `src/research/teenix.rs` for the Teenix `NeWe` outer container, and `src/research/teenix_hp67.rs` plus `src/research/woodstock_asm.rs` to analyse/assemble the HP-67 listing. It is a developer tool only and is not called by the desktop emulator or cycle-accurate machine. `verify-startup` checks normalized corpora against three words observed directly on physical HP-67 hardware.

## Responsibilities

Expose `extract-x11`, `import-pairs`, `merge`, `compare`, `verify-startup`, `decode-teenix`, `preview-teenix`, `analyze-teenix-hp67`, `extract-teenix-hp67` and `inspect`; write normalized TSV files; decode validated Teenix payloads losslessly; report every unknown mnemonic, label mismatch and trailing non-ROM record before extraction; combine sparse corpora only when overlaps agree; print precise differences; return non-zero status when evidence disagrees.

## Implementation

`extract-x11` parses x11-calc's 8192-entry HP-67 image. `decode-teenix` and `preview-teenix` validate the `NeWe` envelope before exposing payload text. `analyze-teenix-hp67` counts candidate microinstructions against the expected 5120 physical words, lists unknown mnemonics, validates every `Lxxxx:` anchor, and prints any additional non-empty records after the physical ROM slots verbatim with source line and candidate ordinal. `extract-teenix-hp67` only writes TSV after the entire listing structure is understood, then immediately applies the physical startup check. `compare` returns 0 for identical corpora and 1 for differences; malformed input/I/O errors return 2.
