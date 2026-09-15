# `src/bin/rom_compare.rs`

## Purpose

Provides the local command-line entry point for HP-67 firmware extraction, normalization, merging, Teenix container decoding, inspection, evidence checking and comparison.

## Why it exists

ROM provenance must be checked locally against files obtained from their original sources. A dedicated tool makes that process reproducible without embedding third-party firmware in `hp67emu` or depending on GitHub Actions. The current 2026 Teenix HP-67 module uses XOR-obfuscated `NeWe` containers, so the tool also needs a lossless way to expose their textual payload for further grammar analysis.

## Relationships

Uses `src/research/rom_corpus.rs` for normalized ROM parsing/storage/comparison and `src/research/teenix.rs` for the Teenix `NeWe` outer container. It is a developer tool only and is not called by the desktop emulator or the cycle-accurate machine. The `verify-startup` command checks normalized corpora against three instruction words observed on a physical HP-67 by Tony Nixon while monitoring the real machine at power-on.

## Responsibilities

Expose `extract-x11`, `import-pairs`, `merge`, `compare`, `verify-startup`, `decode-teenix`, `preview-teenix` and `inspect`; write normalized TSV files; decode validated Teenix payloads without changing them; combine sparse bank/page corpora only when overlaps agree; print bank/page comparison summaries and precise mismatches; return a non-zero process status when corpora differ or contradict physical startup evidence. Decoding the outer Teenix container must not silently infer the internal `.pfl` assembler/source grammar.

## Implementation

The CLI uses only the Rust standard library plus the tested research modules in this crate. `extract-x11` parses x11-calc's complete 8192-entry HP-67 `i_rom[]`; `import-pairs` normalizes explicit address/opcode listings; `merge` combines corpora and fails on conflicting overlaps; `compare` returns exit code 0 for identical corpora and 1 for differences; `verify-startup` checks bank 0 locations `0x000`, `0x001` and `0x0f8` against `0x000`, `0x3e3` and `0x11a`. `decode-teenix` validates the `NeWe` header/count then writes the exact XOR-decoded payload, while `preview-teenix` prints a numbered prefix of that payload. `inspect` recognizes and validates a `NeWe` signature when present and otherwise reports byte-distribution facts. Malformed input or I/O failures return 2.
