# Host card import

## Purpose
Decode a single physical card or an explicitly paired HPP batch with bounded input.

## Why it exists
Display titles do not identify physical media. Sequential title-based merging could
retain an unrelated opposite track. Unbounded reads could allocate arbitrarily.

## Relationships
Called by app.rs before UI/media publication; delegates byte parsing to the existing
magnetic_card format parsers. No electrical core depends on this host module.

## Responsibilities
One file creates fresh media. Exactly two HPP files dropped together request a
pair with opposite header-derived tracks, independent of order or title.
Reject same-track, mixed-format and larger batches without UI mutation.
Files with exceptional same-class headers require explicit placement using the
existing converter --track1/--track2 options; native cards preserve that placement.

## Implementation
Reads at most limit+1 bytes from each opened stream, rejecting excess even when
metadata is missing or changes. Limits are 250 bytes native, 238 raw, and 1 MiB HPP
(including text metadata/whitespace); the HPP cap is host policy, not hardware.
Existing parsers still enforce exact payload shape. Pairing operates on temporary
owned results; only a fully validated batch is returned. Length warnings are
retained; differing titles on an explicit pair leave presentation name unset.
Tests cover title collisions, order-independent pairing, same-track rejection,
finite/infinite bounded streams, invalid batches, malformed second disk input,
native round-trip and oversized native input. No firmware timing is changed.
