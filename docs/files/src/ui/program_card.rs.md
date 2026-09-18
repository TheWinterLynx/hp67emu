# `src/ui/program_card.rs`

## Purpose

Render the HP-67 program-card presentation layer for the top-down desktop UI.

## Why it exists

The real HP-67 separates magnetic reading from the passive reference-card holder above A-E. A top-down emulator should preserve that distinction: the lateral reader can be represented by a right-edge interaction/animation while the visible card artwork belongs in the front-panel holder.

## Relationships

Used by `src/panel.rs` for card interaction and rendering and by `src/app.rs` for the current card presentation state. It consumes no calculator RAM, CRC state or keyboard semantics.

## Responsibilities

Represent card artwork metadata; draw the visible holder card above A-E; expose a narrow right-edge reader hotspot; animate only the card portion still outside the case during insertion; and report reader/window clicks back to the app. The module must not remap A-E, load program memory or synthesize magnetic-card electrical behavior.

## Implementation

`ProgramCardArtwork` stores a title, card reference and separate five-position primary/shifted label rows. The initial fixture is the SD-14A Moon Rocket Lander card with `CNTRL` and `RESTART` over A and B. `ProgramCardView` supplies holder presence plus optional read-animation progress. Source-photo coordinates keep the holder above the A-E row and the reader interaction at the case's right edge. During insertion the painter is clipped to the area outside the right case edge so the top-down view never invents a second visible reader slot.
