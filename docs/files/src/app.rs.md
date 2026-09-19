# `src/app.rs`

## Purpose

Run the desktop HP-67 application loop around the photorealistic panel and live structural machine.

## Why it exists

Input sampling, elapsed-time advancement and painting need one deterministic ordering. Physical key contact must reach the machine before the elapsed firmware work for the frame is executed.

## Relationships

Owns `Hp67State`, `Hp67LiveMachine`, the embedded front-panel texture, `Hp67Panel`, the small visual correction layers for top keys/sliders, and the UI-only program-card presentation state.

## Responsibilities

Create the versioned-firmware live machine; process power/mode switch events; reset on OFF-to-ON; continuously drive the live CRC PROGRAM input from the RUN/PRGM switch position; translate the panel's held `KeyAction` into a physical `Hp67Key`; release the machine keyboard whenever no key is held or power is off; advance firmware from real elapsed time; manage the UI-only card phase machine from right-side magnetic reading through left-side parking and user-triggered re-insertion from the right into the passive holder; and repaint continuously while powered or while a card phase is animating.

## Implementation

The frame first paints the currently available hardware display and collects panel interaction. Mechanical events are applied, the RUN/PRGM state is passed to `Hp67LiveMachine::set_program_mode()`, the current key contact is passed to `Hp67LiveMachine::set_key_contact()`, then elapsed time is calculated and the live machine advances. Resetting `last_live_tick` on power transitions preserves the original zero-elapsed first powered frame. No UI key callback performs calculator-level work. The M13 card phase machine advances only visual state (`Idle` → `ReadingFromRight` → `ParkedLeft` → `InsertingWindowFromRight` → `InWindow`) and likewise does not alter CRC state, program RAM or A-E semantics.


M13 artwork loading: the exact HP badge is stored as `assets/hp67-card-logo.png`, cropped from the supplied original-card reference. `Hp67App` decodes it once at startup into a dedicated `TextureHandle` and passes that texture into the program-card renderer; the logo is no longer approximated with procedural line geometry.


M13 magnetic-media boundary: the app now owns an optional complete Hp67MagneticCard independently from the printed ProgramCardArtwork. Dropped .hpp files are decoded by TeenixHppImport in the host layer and merged into Track 1 or Track 2 according to the card header. Dropped .hp67raw and .hp67card images are also accepted directly. The reader itself never sees those file formats.

Reader clicks hand the complete card plus the current CardInsertionEnd to Hp67LiveMachine::insert_magnetic_card(). When the selected track has crossed the head, take_completed_magnetic_card() returns the same physical card object. A double-click on the protruding left card rotates the physical orientation by switching End1 and End2 before the next reader insertion; a normal click still moves the card to the passive A-E holder.

The magnetic-media slot intentionally initializes empty until verified media is imported. The Moon Rocket Lander artwork is therefore never silently treated as recorded magnetic content.