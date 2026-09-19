# `src/ui/program_card.rs`

## Purpose

Render the HP-67 program-card presentation layer for the top-down desktop UI.

## Why it exists

The real HP-67 separates magnetic reading from the passive reference-card holder above A-E. A top-down emulator should preserve that distinction: the lateral reader can be represented by a right-edge interaction/animation while the visible card artwork belongs in the front-panel holder.

## Relationships

Used by `src/panel.rs` for card interaction and rendering and by `src/app.rs` for the current card presentation state. It consumes no calculator RAM, CRC state or keyboard semantics.

## Responsibilities

Represent card artwork metadata; draw the visible holder card above A-E; expose a narrow right-edge reader hotspot; animate the same physical card from the right reader mouth through the hidden case to the left exit; leave a clickable tab protruding from the left side; when selected, start a second right-side insertion into the holder with the shell/lip occluding the hidden travel; and report reader/left-exit/window interactions back to the app. The module must not remap A-E, load program memory or synthesize magnetic-card electrical behavior.

## Implementation

`ProgramCardArtwork` stores a title, card reference and separate five-position primary/shifted label rows. The initial fixture is the SD-14A Moon Rocket Lander card with `CNTRL` and `RESTART` over A and B. `ProgramCardPhase` owns the UI-only presentation states `Idle`, `ReadingFromRight`, `ParkedLeft`, `InsertingWindowFromRight` and `InWindow`. The card has one physical size in every phase: 71.1 × 11.4 mm, mapped through the photographed 81 mm case width at the same source-pixel scale used elsewhere by the panel. During reading, one fixed-size card rectangle moves right-to-left while two clip regions expose only the portions outside the right reader mouth and left exit mouth. Because the real 71.1 mm card is shorter than the modeled span between those two mouths, there can be a physically valid interval in which the card is completely hidden inside the calculator. At the end, a 10.5 mm-equivalent tab remains clickable on the left. After the user clicks the left-exit tab, the presentation treats the physical card as having been picked up and reinserted into the passive holder from the calculator's right side. The holder is an aperture, not a resizer: the same 71.1 × 11.4 mm card extends behind its lip and is clipped by `CARD_WINDOW`. A complete card begins outside the physical right case edge and slides horizontally toward `CARD_WINDOW`; the span between the case edge and the holder window is clipped away so the card passes beneath the shell/lip rather than floating over it. Only the outside-right portion and the portion already inside `CARD_WINDOW` are visible. The final holder aperture is constrained to source x=166..764, exactly the photographed A-E row span, so it cannot paint across the white side trim. Card label anchors are calibrated to the photographed A-E key centres at source x=211, 338, 465, 592 and 719 rather than evenly dividing the physical card into fifths. No second front-facing reader slot is drawn.


Physical-size evidence used by this renderer: HP magnetic-card specifications list 7.11 cm × 1.14 cm (2.8 × 0.45 in). The supplied Moon Rocket Lander reference shows an asymmetric six-edge silhouette: a large upper-left diagonal and matching lower-right diagonal, plus three short rectangular marks/notches along the top edge. The renderer now follows that photographed silhouette at approximately 4.2 mm diagonal depth. The card body is a near-black olive rather than the earlier medium olive.


Holder fidelity correction: `CARD_WINDOW` is now source x=132..798 and y=344..457. Its 113-source-pixel height exceeds the 1:1 card height (~111.5 source pixels), so the top edge, three white marks and bottom labels remain visible. Horizontally it exposes more than 95% of the real card width; the small remaining side occlusion represents the card sitting under the holder lips rather than resizing the card.
