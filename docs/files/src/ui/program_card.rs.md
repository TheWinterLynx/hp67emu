# `src/ui/program_card.rs`

## Purpose

Render the HP-67 program-card presentation layer for the top-down desktop UI.

## Why it exists

The real HP-67 separates magnetic reading from the passive reference-card holder above A-E. A top-down emulator should preserve that distinction: the lateral reader can be represented by a right-edge interaction/animation while the visible card artwork belongs in the front-panel holder.

## Relationships

Used by `src/panel.rs` for card interaction and rendering and by `src/app.rs` for the current card presentation state. It consumes no calculator RAM, CRC state or keyboard semantics. `ProgramCardView::opposite_track_requested` is a presentation hint computed by the app from live firmware display state plus host media availability.

## Responsibilities

Represent card artwork metadata; draw the visible holder card above A-E; expose a narrow right-edge reader hotspot; animate the same physical card from the right reader mouth through the hidden case to the left exit using externally supplied transport progress; leave a clickable tab protruding from the left side; present a `Crd`-specific opposite-end reinsertion hint when requested by the app; when normally selected, start a second right-side insertion into the holder with the shell/lip occluding the hidden travel; and report reader/left-exit/window interactions plus a distinct left-exit double-click back to the app. The module must not remap A-E, load program memory or synthesize magnetic-card electrical behavior.

## Implementation

`ProgramCardArtwork` stores a title, card reference and separate five-position primary/shifted label rows. The initial fixture is the SD-14A Moon Rocket Lander card with `CNTRL` and `RESTART` over A and B. `ProgramCardPhase` owns the UI-only presentation states `Idle`, `ReadingFromRight`, `ParkedLeft`, `InsertingWindowFromRight` and `InWindow`. The card has one physical size in every phase: 71.1 × 11.4 mm, mapped through the photographed 81 mm case width at the same source-pixel scale used elsewhere by the panel. During reading, one fixed-size card rectangle moves right-to-left while two clip regions expose only the portions outside the right reader mouth and left exit mouth. Because the real 71.1 mm card is shorter than the modeled span between those two mouths, there can be a physically valid interval in which the card is completely hidden inside the calculator. At the end, a 10.5 mm-equivalent tab remains clickable on the left. After the user clicks the left-exit tab, the presentation treats the physical card as having been picked up and reinserted into the passive holder from the calculator's right side. A double-click is reported separately so the app can rotate that same physical card 180 degrees in its plane and, when real firmware is displaying `Crd`, immediately reinsert the opposite end into the magnetic reader through the same `ReadingFromRight` motion. The holder is an aperture, not a resizer: the same 71.1 × 11.4 mm card extends behind its lip and is clipped by `CARD_WINDOW`. A complete card begins outside the physical right case edge and slides horizontally toward `CARD_WINDOW`; the span between the case edge and the holder window is clipped away so the card passes beneath the shell/lip rather than floating over it. Only the outside-right portion and the portion already inside `CARD_WINDOW` are visible. The holder aperture is source x=138..796 and y=345..454; the same physical card is clipped by that aperture while the original photographed casing remains untouched outside it, so the card appears underneath the real frame instead of floating on top. Card label anchors are calibrated to the photographed A-E key centres at source x=211, 338, 465, 592 and 719 rather than evenly dividing the physical card into fifths. No second front-facing reader slot is drawn.


Physical-size evidence used by this renderer: HP magnetic-card specifications list 7.11 cm × 1.14 cm (2.8 × 0.45 in). The supplied Moon Rocket Lander reference shows an asymmetric six-edge silhouette: a large upper-left diagonal and matching lower-right diagonal, plus three short rectangular top marks. The renderer now follows that photographed silhouette at approximately 4.2 mm diagonal depth. The card body is a near-black olive rather than the earlier medium olive.


Holder fidelity correction: `CARD_WINDOW` is source x=138..796 and y=345..454. Its 109-source-pixel height is slightly smaller than the ~111.5-source-pixel physical card, so only about 1.2 source pixels are hidden at each horizontal edge while the top marks and bottom labels remain visible. Horizontally the opening exposes about 94% of the physical card width, leaving the ends visibly tucked under the photographed side frame without resizing the card.


`ProgramCardArtwork` now owns each card's `top_marks` pattern and whether the HP logo is present. The three SD-14A positions are measured from the supplied reference rather than treated as a global physical code. The classic HP badge at the left is texture-mapped from `assets/hp67-card-logo.png`, a transparent crop taken directly from the supplied original-card scan.

No source found so far assigns electrical or magnetic meaning to the small top marks. The documented magnetic information lives on the two tracks, while write protection is achieved by cutting the corresponding card edge/corner. Until stronger evidence appears, the emulator treats top marks strictly as per-card artwork metadata and reproduces them from scans.


Holder masking is now based on the photographed frame itself rather than synthetic painted lips. Measurements from the emulator capture map the pale left rail to about source x=131, its inner black edge to x=138, the right inner edge to x=793 and the pale right rail to about x=800. Therefore the card is painted only through `CARD_WINDOW = (138,345)..(796,454)`; the original `assets/hp67.png` remains untouched and visible around that aperture, so the card cannot paint over the casing. The opening clips only ~1.2 source pixels from each physical top/bottom edge and ~20 source pixels from each end, while preserving the real 71.1 × 11.4 mm card rectangle underneath.

The HP badge is no longer generated procedurally. `assets/hp67-card-logo.png` is a transparent crop taken directly from the supplied SD-13A reference and is texture-mapped onto every artwork that requests the HP logo.


Final right-edge registration now follows the photographed rail slope instead of a vertical clip. The card is allowed to render to source x=796, then the original `hp67.png` is restored over it in 109 one-source-pixel horizontal bands. The restored boundary interpolates from x=787 at the top of the holder to x=792 at the bottom, matching the visible white rail inclination. The physical card centre remains fixed at source x=465.5 with a -1.5 source-pixel window-centre compensation, so title/logo/A-E registration remains unchanged.


The reader hotspot now distinguishes primary click from secondary click. Primary click requests insertion of already prepared media; secondary click requests a new blank physical card through the app. The renderer itself never creates magnetic records. At the left exit the `Crd` continuation hint is valid for both reading a recorded opposite track and writing a writable unrecorded opposite track; the app determines that distinction from the live CRC write-mode flag.


`GENERIC_MAGNETIC_CARD` is a neutral physical-card face with no title, reference, labels, top marks or HP-logo claim. It is used for host media whose visual identity is not known. `MOON_ROCKET_LANDER_CARD` remains the measured SD-14A artwork fixture and is no longer used as the universal magnetic-card skin.


Opposite-end presentation is now physical rather than symbolic. `ProgramCardView::rotated_180` rotates all orientation-bearing artwork by 180 degrees: top-edge marks move to their mirrored bottom positions, text is emitted through egui 0.27.2 `TextShape::with_angle(PI)`, and the HP logo mesh is rotated about its center. The card outline itself is invariant under the 180-degree transformation. Tests lock the point/mark transform as an involution.
