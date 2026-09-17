# `src/machines/hp67/firmware.rs`

## Purpose

Provide the HP-67 machine with a firmware source that is compiled into the executable instead of opened from a TSV or other external file at runtime.

## Why it exists

The emulator is intended to ship as a self-contained executable. Research corpora remain useful for provenance and reproducible verification, but a released HP-67 executable must not depend on `.research/teenix-2026-hp67.tsv`, an environment variable, or network access when it starts.

## Relationships

`build.rs` consumes the locally validated normalized HP-67 corpus while Cargo is compiling and generates a direct-address ROM table in `OUT_DIR`. This module includes that generated table and implements `Hp67RomWordSource`, so the existing structural ACT/ROM fetch path can use the same interface without knowing whether firmware came from a research file or an embedded table.

## Responsibilities

Expose the verified populated-word count; preserve the HP-67 page/bank fallback rule; select the requested ROM bank; return 10-bit words for structural fetch; and guarantee that normal runtime fetch performs no filesystem or network I/O.

## Implementation

The generated image uses 8192 `u16` slots, one for every `(bank, pc)` address, with `0xffff` as the impossible 10-bit missing-word sentinel. `EmbeddedHp67Rom` keeps only the currently requested bank in a `Cell<u8>`. `read_word()` resolves the effective bank from the generated page-bank mask and performs a direct array lookup. The build-time generator asserts the 5120-word corpus size and the established startup anchors before writing the generated Rust table.
