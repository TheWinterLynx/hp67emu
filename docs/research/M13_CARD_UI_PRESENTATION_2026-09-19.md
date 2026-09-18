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

The lateral reader is represented as a right-edge interaction. During reading, the same physical card moves from right to left behind the top-down case: only the portions outside the right reader mouth or left exit mouth are rendered. The card is longer than the distance between both mouths, so the leading edge starts emerging on the left before the trailing edge fully disappears on the right. When reading completes, a small clickable tab remains protruding from the left side. The user must click that exposed end; this represents picking up the read card. The second animation then starts with the complete card at the calculator's right side and inserts it horizontally into the passive holder above A-E. The calculator shell/lip occludes the intermediate span: only the part still outside the physical right case edge and the part already inside the holder window are rendered.

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


## Presentation phase machine

The UI presentation uses five explicit states:

- `Idle`: no card visible; the right-edge reader hotspot accepts insertion;
- `ReadingFromRight`: the card traverses the hidden lateral reader path from right to left;
- `ParkedLeft`: only the physically exposed left end remains visible and clickable;
- `InsertingWindowFromRight`: the user-selected card animates from the left exit into the holder;
- `InWindow`: the passive reference artwork remains above A-E until removed.

These states are presentation only. They are deliberately separate from future CRC/card-electronics states so later M13 work can drive motor, card-presence and data timing without making the artwork layer authoritative.
