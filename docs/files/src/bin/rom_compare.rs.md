# `src/bin/rom_compare.rs`

## Purpose

Provides the local command-line entry point for HP-67 firmware extraction, normalization, inspection and comparison.

## Why it exists

ROM provenance must be checked locally against files obtained from their original sources. A dedicated tool makes that process reproducible without embedding third-party firmware in `hp67emu` or depending on GitHub Actions.

## Relationships

Uses `src/research/rom_corpus.rs` for all parsing and comparison logic. It is a developer tool only and is not called by the desktop emulator or the cycle-accurate machine.

## Responsibilities

Expose `extract-x11`, `import-pairs`, `compare` and `inspect` commands; write normalized TSV files; print bank/page comparison summaries and precise mismatches; return a non-zero process status when corpora differ. Binary inspection reports facts only and must not infer the unknown Teenix `.pfl` format.

## Implementation

The CLI uses only the Rust standard library. `compare` returns exit code 0 for identical corpora and 1 for differences; malformed input or I/O failures return 2. Difference output includes bank, page, hexadecimal PC, octal PC and both word encodings, capped to a readable number while preserving the total difference count.
