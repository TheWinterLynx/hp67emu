# `src/research/nonpareil_obj.rs`

## Purpose

Parses textual Woodstock object files emitted by Nonpareil's official `uasm` and normalizes them into hp67emu's sparse `(bank, pc, word)` ROM corpus.

## Why it exists

Nonpareil's HP-67 sources are symbolic assembly. Reimplementing or copying Nonpareil's assembler would add unnecessary licensing and correctness risk. The official assembler already emits a compact object format, so hp67emu only needs a strict importer for that output before comparing Nonpareil against Teenix and x11-calc.

## Relationships

Uses `src/research/rom_corpus.rs` for validated 10-bit storage. `src/bin/rom_compare.rs` exposes the importer through `import-nonpareil-obj`. The module is research-only and is not part of the electrical HP-67 implementation.

## Responsibilities

Accept current Nonpareil `uasm` Woodstock object records, decode optional bank masks, parse octal PC/opcode fields, expand multi-bank masks, reject unsupported HP-67 banks, reject addresses outside the 4K logical PC space, reject words outside ten bits, and preserve duplicate-location failures from `RomCorpus`.

## Implementation

Current upstream `uasm` writes Woodstock records as an optional bank mask followed by octal address and octal opcode, for example `[0]0001:1743` or `[1]2000:0432`. The importer also accepts historical prefix-less records as bank 0. `#` metadata/comment records and blank lines are ignored. A mask such as `[01]` writes the same word to both banks. Tests cover current bank-prefixed output, multi-bank expansion, historical prefix-less records, invalid octal text, range failures and duplicate addresses.
