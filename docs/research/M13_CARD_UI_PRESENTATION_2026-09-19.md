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

The lateral reader is represented as a right-edge interaction. During reading, the same physical card moves from right to left behind the top-down case: only the portions outside the right reader mouth or left exit mouth are rendered. The card retains its real 71.1 mm length. Since that is shorter than the current modeled span between the two side mouths, the card can be fully hidden inside the case for part of the motorized transit before it emerges on the left. When reading completes, a small clickable tab remains protruding from the left side. The user must click that exposed end; this represents picking up the read card. The second animation then starts with the complete card at the calculator's right side and inserts it horizontally into the passive holder above A-E. The calculator shell/lip occludes the intermediate span: only the part still outside the physical right case edge and the part already inside the holder window are rendered.

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

The supplied pre-recorded card references show a six-edge profile with a large upper-left diagonal and the opposite lower-right diagonal. The renderer uses an approximately 4.2 mm end chamfer on the locked 71.1 × 11.4 mm card. These diagonal cuts are consistent with HP's documented write-protection method for the corresponding card edges.


## Holder registration correction

The holder now uses source x=132..798 and y=344..457 so the complete 1:1 card height remains available. The card is not shrunk to that aperture. Instead the aperture clips the physical card and explicit side lips plus thin edge shadows are painted over it, restoring the visual layering of card-below-frame.

The text/tick anchors are no longer generic 20% card intervals. Their physical-card fractions are calibrated so that, in the final holder position, they land on the actual photographed A-E key centres at source x=211, 338, 465, 592 and 719. The title/reference band is lowered slightly and the primary-label band raised slightly to keep all artwork comfortably inside the visible aperture.


## Reference-card silhouette and holder correction

The supplied SD-14A Moon Rocket Lander reference was used as the visual oracle for the card face. Its outline is not a symmetric rounded rectangle: the upper-left corner is cut diagonally and the lower-right corner carries the opposing diagonal, while the other two corners remain square. The photographed diagonal depth corresponds to roughly 4.2 mm on a 71.1 × 11.4 mm card. Three short rectangular light marks are visible along the SD-14A top edge and are now rendered explicitly from card-specific metadata.

The earlier holder clip was only 79 source pixels high, so it necessarily removed the physical card's top and bottom from a ~111.5-source-pixel 1:1 card. The holder aperture is now x=132..798, y=344..457. This exposes the complete physical height, including the top markers, and more than 95% of the physical width. The card itself never changes size between reader transit, left exit, holder insertion and final holder state. Its palette was also changed from medium olive to a near-black olive to match the supplied original-card and calculator photographs.


## Per-card artwork catalog strategy

Future Standard Pac and Application Pac cards will use the same physical renderer and differ only through metadata: card reference, title, A-E primary labels, f+A through f+E shifted labels, measured top-mark positions and logo visibility. Magnetic track contents remain a separate object and will never be inferred from artwork. This lets us recreate the remaining cards from high-quality scans without duplicating geometry or UI code.

The small top marks are deliberately not assigned a reader or binary meaning. Available HP documentation clearly describes magnetic data as two tracks and write protection as cutting the corresponding edge/corner, but does not document these artwork marks as part of the reader protocol. Their patterns therefore remain scan-derived presentation data unless a primary source establishing another purpose is found.


## Exact logo asset and photographed holder mask

The procedural HP badge was rejected after visual comparison with the supplied original card. A transparent crop of the actual badge from the supplied SD-13A scan is now committed as `assets/hp67-card-logo.png`; the app loads it once and the card renderer texture-maps those exact pixels without reconstructing the glyph.

The previous synthetic holder lips did not reproduce the front-panel layering. Measurements from the emulator screenshot establish the photographed aperture at source x=138..793: outside that range the original chassis/rail pixels must remain on top. Vertically the measured usable opening is y=345..454. Rendering is now clipped to that aperture instead of painting artificial frame pieces. This keeps the original casing photograph authoritative and the physical 71.1 × 11.4 mm card underneath it. A-E artwork fractions were recalibrated after the half-pixel holder-center change so their final source positions remain exactly 211, 338, 465, 592 and 719.
