# `src/program_library.rs`

## Purpose

Provide the built-in HP-67 magnetic-card program library exposed by the desktop emulator.

## Why it exists

The checked-in `.hpp` corpus should be loadable as real magnetic media without requiring users to browse individual files or depend on the process working directory. The library also needs one stable place to associate each physical program card with its pack metadata and optional PDF-derived visual artwork.

## Relationships

The module consumes `TeenixHppImport`, `Hp67MagneticCard`, and `Hp67CardTrack` from the reusable HP-67 magnetic-media core. It is owned by the desktop binary and supplies `PROGRAM_LIBRARY` entries to `src/app.rs`. Presentation metadata uses `ProgramCardArtwork` from `src/ui/program_card.rs`, while optional PNG artwork is loaded by the app and remains separate from magnetic payload bytes.

## Responsibilities

Embed all checked-in HP-67 `.hpp` parts with `include_bytes!`; group related parts into one physical-card entry; decode every part through the Teenix payload parser; preserve the distinction between firmware header class and physical card side; reject duplicate physical tracks or accidental mismatched grouping; expose pack, title, reference, PDF source, artwork-cache path, and track count; and provide a procedural fallback face when no extracted PNG exists.

## Implementation

Filename grouping is only a catalog convenience. When an entry is loaded, every embedded `.hpp` is decoded by `TeenixHppImport`. For ordinary continuation media the header-derived mapping remains the default, but header class is not universally a physical-side number. HP documents SD1-12A English-SI Conversions as one physical card with two independent one-pass sides; both Teenix payloads therefore have program header 3 and distinct A1/A2 artwork IDs. The catalog explicitly binds those two payloads to opposite physical tracks. Entries with one valid part leave the opposite physical track genuinely unrecorded.

Standard Pac and Games Pac entries point at `programs/HP67/_artwork/<reference>.png` and record the pack PDF used as the source. Missing artwork never changes card behavior. The catalog regression locks the current corpus at 59 physical program entries backed by 90 checked-in `.hpp` parts and decodes every entry through the production parser.
