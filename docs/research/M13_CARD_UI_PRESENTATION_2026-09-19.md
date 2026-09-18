# M13 magnetic-card UI presentation slice — 2026-09-19

## Scope

This first M13 slice establishes the visual/mechanical presentation boundary for HP-67 magnetic cards before any CRC or magnetic-data implementation is connected.

The HP-67 has two physically distinct card uses:

- a lateral motorized reader/writer path;
- a passive front-panel reference-card holder above A-E.

The emulator uses a strict top-down view. It therefore must not invent a second front-facing reader slot.

## Architecture decision

Magnetic card contents and card artwork are separate concerns.

The visible card artwork is UI metadata only:

- title;
- reference number;
- five A-E labels;
- five shifted f+A through f+E labels.

It does not alter key identities. A-E continue to generate their existing physical HP-67 keycodes and firmware remains responsible for deciding default-function versus user-label behavior.

The lateral reader is represented as a right-edge interaction. During the temporary read animation, only the part of the card still outside the calculator case is rendered. Once the animation completes, the same artwork is placed in the passive holder above A-E.

## Initial fixture

The first visual fixture is the SD-14A Moon Rocket Lander card:

- A: `CNTRL`;
- B: `RESTART`;
- C-E: blank in the visible primary row.

This is intentionally artwork only. No magnetic payload is fabricated for this slice.

## Fidelity boundary

This slice does not claim a working HP-67 card reader.

Specifically it does not yet:

- assert CRC `card_present`;
- drive `motor_on`;
- provide `buffer_ready` data;
- emulate the 1820-1751 CRC card path;
- model sense-amplifier timing;
- read/write card tracks;
- mutate program RAM from the UI.

Those are the next M13 electrical/firmware slices. This presentation layer exists so real card data can later be attached without conflating metadata with machine state.
