# src/machines/hp67/card_flux.rs

## Purpose

Represent the documented HP self-clocking magnetic encoding used underneath one logical HP-67 card track without prematurely inventing head analogue behavior or CRC-word serialization order.

## Why it exists

HP user documentation calls the two end-for-end card streams "side 1 / side 2" or "track 1 / track 2", but Hewlett-Packard's engineering description shows that each such logical stream is physically recorded as two parallel magnetic tracks. One physical track carries a flux reversal for every logical 1 and the other carries a flux reversal for every logical 0. Exactly one transition therefore occurs in each bit cell and supplies the clock as well as the data.

Keeping this physical encoding separate prevents Hp67MagneticTrack from being mistaken for one literal magnetic stripe and gives later M13 work a correct boundary for head, sense-amplifier and flux timing.

## Relationships

magnetic_card.rs owns the logical post-CRC 952-bit Track 1 / Track 2 media selected by CardInsertionEnd. card_flux.rs models the two physical flux tracks that encode one of those logical streams. card_transport.rs still moves 28-bit CRC-visible records and does not yet serialize those records into timed bit cells. crc.rs remains above the magnetic head/sense boundary.

## Responsibilities

- distinguish the logical card track from its two physical recording tracks;
- represent the physical Zero and One flux tracks explicitly;
- encode one logical bit as exactly one flux reversal on exactly one physical track;
- round-trip a supplied 952-bit serial stream through the self-clocking flux representation;
- avoid claiming an absolute magnetic polarity;
- avoid claiming an as-yet-unverified temporal bit order inside each 28-bit CRC record.

## Implementation

Hp67PhysicalFluxTrack has Zero and One variants. Hp67FluxCell::encode(false) creates a reversal on the Zero track and Hp67FluxCell::encode(true) creates a reversal on the One track. The representation makes the invariant "exactly one reversal per bit cell" unrepresentable as an invalid public state.

Hp67SelfClockingFluxPair contains exactly HP67_CARD_BITS_PER_TRACK cells, currently 952. from_serial_bits() accepts an already ordered logical bit stream and serial_bits() reconstructs it losslessly. This is deliberately not yet wired directly to [u32; 34]: the HP engineering sources establish the two-track encoding, but this slice does not assert the still-unpinned time order in which the CRC shifts the 28 bits of each record onto the magnetic stream.

The model stores reversal events rather than absolute north/south polarity. That is the evidenced information needed by the self-clocking scheme and avoids inventing an initial magnetization state.
