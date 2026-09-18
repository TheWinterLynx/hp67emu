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

`ProgramCardArtwork` stores a title, card reference and separate five-position primary/shifted label rows. The initial fixture is the SD-14A Moon Rocket Lander card with `CNTRL` and `RESTART` over A and B. `ProgramCardPhase` owns the UI-only presentation states `Idle`, `ReadingFromRight`, `ParkedLeft`, `InsertingWindowFromRight` and `InWindow`. During reading, one card rectangle moves right-to-left while two clip regions expose only the portions outside the right reader mouth and left exit mouth; its physical width is deliberately long enough that the leading edge begins to emerge before the trailing edge vanishes. At the end, a 105-source-pixel tab remains clickable on the left. After the user clicks the left-exit tab, the presentation treats the physical card as having been picked up and reinserted into the passive holder from the calculator's right side. A complete card begins outside the physical right case edge and slides horizontally toward `CARD_WINDOW`; the span between the case edge and the holder window is clipped away so the card passes beneath the shell/lip rather than floating over it. Only the outside-right portion and the portion already inside `CARD_WINDOW` are visible. No second front-facing reader slot is drawn.
