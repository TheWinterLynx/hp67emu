# `src/bin/rom_subset.rs`

## Purpose

Verifies that every populated word in one normalized HP-67 ROM corpus exists and matches exactly in a second, broader reference corpus.

## Why it exists

The current Teenix HP-67 listing normalizes to the 5120 physically populated words used by the calculator, while x11-calc exposes a full 8192-entry two-bank address space. A normal full-corpus comparison correctly reports the additional x11-calc locations, but those right-only words should not obscure whether all 5120 Teenix words agree. This tool performs that directional subset check without weakening the strict full comparator.

## Relationships

Uses `src/research/rom_corpus.rs` to load normalized TSV corpora and reuse the existing exact comparison engine. It is a local research binary and is not part of the production emulator, reference CPU, scheduler or UI.

## Responsibilities

Treat the first input as the required subset; fail if any populated subset location is absent from the reference or has a different 10-bit word; report reference-only locations as informational; print precise bank/page/PC diagnostics; return exit status 0 for a complete subset match, 1 for firmware disagreement and 2 for malformed input or I/O errors.

## Implementation

The binary calls the existing symmetric `compare` function, classifies `Mismatch` and `MissingRight` records as subset failures, and counts `MissingLeft` records only as reference-only coverage. It deliberately does not alter either corpus or infer aliases for unpopulated physical ROM regions.
