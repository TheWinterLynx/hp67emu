# `src/research/teenix.rs`

## Purpose

Decodes and validates the lightweight Teenix `NeWe` container used by the current HP-67 `.pfl` files and documented for other Teenix text-backed data files.

## Why it exists

The 2026 Teenix HP-67 module is a current and valuable firmware corpus, but its `.pfl` files initially looked binary. Their raw prefix becomes `NeWe` when XORed with `0x55`, and the following decoded decimal length exactly matches the remaining payload size. Capturing this outer format in a small tested decoder lets the project inspect the real textual payload without guessing ROM words or embedding copyrighted firmware.

## Relationships

This is a research-only helper exported through `src/research/mod.rs`. `src/bin/rom_compare.rs` uses it for local inspection/decoding of developer-supplied Teenix files. It is not part of the electrical HP-67 machine, the semantic reference model, or the UI.

## Responsibilities

Recognize the `NeWe` signature, XOR-decode bytes with `0x55`, parse CR/LF/CRLF header lines, validate the declared payload byte count exactly, expose raw payload bytes and UTF-8 text, and reject malformed containers with explicit errors. It must not infer the internal grammar of a `.pfl` payload until that grammar is separately established.

## Implementation

`decode_container` XORs the full file, requires the first decoded line to be `NeWe`, parses the second line as a decimal byte count, and checks that it equals the exact number of remaining decoded bytes. Unit tests cover the observed raw `NeWe` signature, CR and CRLF headers, payload extraction, incorrect lengths and wrong magic. No third-party firmware bytes are stored in the test fixtures.
