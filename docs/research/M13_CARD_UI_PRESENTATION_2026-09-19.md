# M13 magnetic-card UI presentation slice — 2026-09-19

## Scope

This M13 slice establishes the visual/mechanical presentation boundary and the first firmware-visible card-control contact while magnetic data transport remains deliberately unimplemented.

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

Implemented control boundary:

- external CRC `card_present` (flag 10) is modeled as a persistent physical contact;
- real HP-67 firmware detects it from the idle loop and issues internal CRC `motor_on` (flag 9), covered by a live-firmware regression.

Still not implemented:

- `buffer_ready` generation from card transport;
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
- `InsertingWindowFromRight`: after the exposed left end is selected, the card is treated as picked up and reintroduced from the right; the case hides the travel until it appears through the holder aperture;
- `InWindow`: the passive reference artwork remains above A-E until removed.

These states are presentation only. They are deliberately separate from future CRC/card-electronics states so later M13 work can drive motor, card-presence and data timing without making the artwork layer authoritative.


## Physical card geometry

The visual card is locked to 71.1 mm × 11.4 mm (2.8 × 0.45 in). The renderer derives source pixels per millimetre from the HP-67 case width already calibrated in the project (81 mm represented by 792 source pixels), so the card is not independently scaled.

The same fixed physical rectangle is used for reader insertion, left-side emergence, parked-left state, holder insertion and final holder display. `CARD_WINDOW` is only an aperture; it clips the physical card and never changes its width or height.

Photographic references show a rectangular card with small rounded/chamfered corners. The previous arrow-shaped procedural nose was removed. The current 0.9 mm corner chamfer is an optical approximation to that photographed corner treatment, not a claim of a separately documented manufacturing radius.
