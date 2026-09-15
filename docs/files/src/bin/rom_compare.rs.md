# `src/bin/rom_compare.rs`

## Purpose

Provides the local command-line entry point for HP-67 firmware extraction, normalization, merging, inspection, evidence checking and comparison.

## Why it exists

ROM provenance must be checked locally against files obtained from their original sources. A dedicated tool makes that process reproducible without embedding third-party firmware in `hp67emu` or depending on GitHub Actions.

## Relationships

Uses `src/research/rom_corpus.rs` for parsing, normalized storage and comparison logic. It is a developer tool only and is not called by the desktop emulator or the cycle-accurate machine. The `verify-startup` command checks normalized corpora against three instruction words observed on a physical HP-67 by Tony Nixon while monitoring the real machine at power-on.

## Responsibilities

Expose `extract-x11`, `import-pairs`, `merge`, `compare`, `verify-startup` and `inspect` commands; write normalized TSV files; combine sparse bank/page corpora only when overlapping words agree; print bank/page comparison summaries and precise mismatches; return a non-zero process status when corpora differ or contradict physical startup evidence. Binary inspection reports facts only and must not infer the unknown Teenix `.pfl` format.

## Implementation

The CLI uses only the Rust standard library. `extract-x11` parses x11-calc's complete 8192-entry HP-67 `i_rom[]`; `import-pairs` normalizes explicit address/opcode listings; `merge` combines two or more normalized corpora and fails on conflicting overlapping words; `compare` returns exit code 0 for identical corpora and 1 for differences; `verify-startup` checks bank 0 locations `0x000`, `0x001` and `0x0f8` against the physically observed words `0x000`, `0x3e3` and `0x11a`; malformed input or I/O failures return 2. `inspect` additionally reports byte-distribution facts such as 7-bit ratio, printable ratio, entropy and common byte values without claiming a decoder.
