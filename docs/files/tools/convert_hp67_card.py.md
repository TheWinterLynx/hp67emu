# `tools/convert_hp67_card.py`

## Purpose

Convert between legacy Teenix `.hpp` compatibility payloads and the project's native
`.hp67card` physical-card container, inspect either format, and construct native cards from
raw-byte program listings.

## Why it exists

The repository still contains historical HPP program sources, but runtime ownership is moving to
the native format. Conversion must preserve the 34 x 28-bit logical records exactly and make the
places where HPP loses physical-card information explicit instead of silently treating HPP as the
canonical media model.

## Relationships

The implementation mirrors `src/machines/hp67/magnetic_card.rs`: `HP67CARD` v1 is 250 bytes,
each recorded logical track is 34 x 28 bits packed into 119 bytes, and Teenix data carries 21 dummy
nibbles, 238 real nibbles and a seven-nibble checksum mirror. Program-listing conversion mirrors
`program_bytes_from_track()` in `src/program_library.rs` in the inverse direction.

## Responsibilities

Decode and validate Teenix XOR-0x55 envelopes; preserve all 34 logical records; auto-place ordinary
HPP payloads by header class; permit explicit Track 1/Track 2 placement for exceptional same-header
media; preserve native recorded and write-protected flags where the destination supports them;
export one compatibility HPP per recorded native track with caller-provided metadata; convert
`<step> <hex-byte>` program listings into native card records; inspect file structure; and provide
a fixture-backed self-test.

## Implementation

`hpp-to-card` writes native media directly. `card-to-hpp` is intentionally a compatibility
export and warns when native write protection or physical track identity cannot be represented by
HPP. `listing-to-card` splits each opcode low-nibble first, maps each 14-nibble RAM image to CRC
records in the HP-67 record-pair order, computes record 34 as the 28-bit running sum of records
1-33 in human numbering, packs records MSB-first and writes one `HP67CARD` v1 container.

The detailed byte/nibble diagrams and command examples are in `docs/HP67_CARD_FORMATS.md`.
