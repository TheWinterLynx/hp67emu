# `src/program_library.rs`

## Purpose

Provide the built-in HP-67 magnetic-card program library exposed by the desktop emulator.

## Why it exists

The checked-in `.hpp` corpus should be loadable as real magnetic media without requiring users to browse individual files or depend on the process working directory. The library also needs one stable place to associate each physical program card with its pack metadata and optional PDF-derived visual artwork.

## Relationships

The module consumes `TeenixHppImport`, `Hp67MagneticCard`, and `Hp67CardTrack` from the reusable HP-67 magnetic-media core. It is owned by the desktop binary and supplies `PROGRAM_LIBRARY` entries to `src/app.rs`. Presentation metadata uses `ProgramCardArtwork` from `src/ui/program_card.rs`; PDF-backed Standard Pac and Games Pac entries also map to rows in the embedded raster artwork atlas. Artwork remains presentation-only and separate from magnetic payload bytes.

## Responsibilities

Embed all checked-in HP-67 `.hpp` parts with `include_bytes!`; group related parts into one physical-card entry; decode every part through the Teenix payload parser; preserve the distinction between firmware header class and physical card side; reject duplicate physical tracks or accidental mismatched grouping; expose pack, title, reference, PDF source, artwork-cache path, and track count; provide a procedural fallback face when no extracted PNG exists; and derive a read-only human program listing directly from the same decoded magnetic payload used by the live reader.

## Implementation

Filename grouping is only a catalog convenience. When an entry is loaded, every embedded `.hpp` is decoded by `TeenixHppImport`. For ordinary continuation media the header-derived mapping remains the default, but header class is not universally a physical-side number. HP documents SD1-12A English-SI Conversions as one physical card with two independent one-pass sides; both Teenix payloads therefore have program header 3 and distinct A1/A2 artwork IDs. The catalog explicitly binds those two payloads to opposite physical tracks. Entries with one valid part leave the opposite physical track genuinely unrecorded.

The 15 Standard Pac and 20 Games Pac physical cards are backed by source-faithful card-face crops extracted from their checked-in HP pack PDFs and packed into `assets/hp67-card-artwork-atlas.png`. `artwork_atlas_row()` gives each of those 35 PDF-backed catalog entries one unique atlas row, and a regression requires the complete 0..34 mapping with no duplicates or omissions. The older `artwork_path` remains only as a filesystem fallback; artwork never changes card behavior. Demo Pac 1 has no checked-in pack PDF in the current source set, so those entries deliberately retain procedural/generic presentation rather than inventing artwork. The catalog regression locks the current corpus at 59 physical program entries backed by 90 checked-in `.hpp` parts and decodes every entry through the production parser. `program_listing()` reconstructs the 112 user-program bytes carried by a program side from the CRC-visible 34×28-bit track without touching calculator state: record 0 remains the firmware card header, records 1..32 are paired in the same 14-nibble register order restored by firmware, and record 33 is outside the 112-byte user-program payload. Header 3 listings begin at step 001 and header 4 continuation listings at step 113; documented independent two-sided header-3 media such as SD1-12A therefore display each side from 001 rather than being falsely joined. Each listing line preserves the raw internal hexadecimal byte beside a source-backed mnemonic so presentation never hides the actual stored code.
