# src/machines/hp67/magnetic_card.rs

## Purpose

Model the HP-67 magnetic medium as one physical card containing two independent user-visible logical tracks selected end-for-end, and provide host-side import/export formats without leaking file-format concepts into the emulated reader hardware.

## Why it exists

A physical HP magnetic card is not one isolated side payload. Keeping the card as the owned object is required so that the same card can leave the reader, be rotated 180 degrees in its plane, and be inserted by the opposite end to expose the other logical track. HP's engineering documentation also makes an important distinction below this layer: each logical side/track is itself encoded on two parallel physical magnetic tracks, one for zero transitions and one for one transitions. That lower layer now lives in card_flux.rs.

## Relationships

The card transport owns an Hp67MagneticCard while it is inside the reader and chooses the active logical Hp67MagneticTrack from CardInsertionEnd. The CRC still sees timed 28-bit words. card_flux.rs models the documented physical Zero/One flux-track pair underneath one logical track but does not yet impose a record-to-bit serialization order. The UI/application layer may import Teenix .hpp, .hp67raw or .hp67card files and then hands the resulting physical media object to the transport. Artwork remains in ui/program_card.rs and is intentionally not part of magnetic media.

## Responsibilities

- represent Track 1 and Track 2 independently;
- preserve unrecorded versus recorded state;
- preserve write protection independently for each track;
- expose exactly 34 CRC-visible 28-bit records for recorded tracks;
- define the logical 238-byte .hp67raw image for two recorded tracks;
- define the versioned .hp67card container that also preserves unrecorded/protected state;
- decode Teenix .hpp files into one clean magnetic track and infer its physical track from the card header;
- keep physical Zero/One flux encoding and analogue timing details out of this logical media layer.

## Implementation

Hp67MagneticCard contains two Hp67MagneticTrack values. Here "Track 1 / Track 2" follows the HP user-facing logical terminology for the two end-for-end 952-bit streams; it must not be confused with the two physical Zero/One flux tracks used to encode each stream at the head. TrackMedia distinguishes Unrecorded from Recorded([u32; 34]). CardInsertionEnd maps End1 to Track1 and End2 to Track2 and supports the physical 180-degree rotation operation through opposite().

The .hp67raw representation is exactly 238 bytes: two contiguous 952-bit logical CRC-record streams. Bits are packed record-by-record in canonical logical order, most-significant bit first within each 28-bit word. This is an emulator interchange convention, not a claim about magnetic flux transition order. It is an emulator logical interchange format, not a magnetic flux capture. Because raw data cannot encode an unrecorded state, export fails unless both tracks are recorded.

The .hp67card v1 container is 250 bytes: an eight-byte HP67CARD magic, version, one flags byte per track, one reserved byte and two 119-byte logical payload slots. Flags preserve recorded/unrecorded and write-protected state. Artwork, title and description are deliberately absent.

TeenixHppImport XOR-decodes bytes with 0x55, validates the NeWe envelope and calculator identifier, strips the historical three dummy 28-bit records, validates and removes the duplicate checksum record, reconstructs each seven-nibble record in the documented high-nibble-first order, reconstructs the 34 real words and uses the high header nibble as the authority for Track 1 versus Track 2. Teenix metadata remains host/import metadata and never reaches the transport or CRC.

Host dirty-state support: `Hp67MagneticCard::dirty()` reports whether either logical track has been modified by firmware writes, and `mark_clean()` clears both per-track dirty flags after a successful host persistence operation. Serialization itself remains pure and does not clear dirty state; only the host action that actually succeeds in writing the file does so.
