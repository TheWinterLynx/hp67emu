# `tools/generate_embedded_card_artwork.py`

## Purpose

Generate the embedded full-resolution PNG lookup for scanned program cards.

## Why it exists

A 499 × 80 atlas face discards fine lettering present in the canonical 1680 × 269 source. Embed the original PNGs so portable executables can retain that detail.

## Relationships

Reads artwork paths in `src/program_library.rs`; writes `assets/embedded-card-artwork.rs`, included by that module. `src/app.rs` decodes the selected PNG. The embedded PDF gate invokes `--check`.

## Responsibilities

Deduplicate source paths, reject unexpected locations and emit one `include_bytes!` per asset. Do not alter image bytes, magnetic cards or PDF extraction.

## Implementation

Uses only the Python standard library. Default root is the script's parent repository; `--repo` selects another checkout. Run without arguments after changing catalog artwork paths. `--check` compares ordered path/include pairs while tolerating rustfmt whitespace; Rust compilation verifies included asset existence and the focused application regression verifies image dimensions and representative decode. Current catalog has 349 artwork entries sharing 232 distinct PNG files. Images decode at selection, never per frame; atlas allocation is deferred until needed.
