# `src/documentation.rs`

## Purpose

Expose the calculator handbook and complete PAC manuals through Help and selected-card details.

## Why it exists

Readers need the application instructions alongside the magnetic media.

## Relationships

Called by app.rs; uses the desktop PDF viewer or packaged browser URLs.

## Responsibilities

Present full manual names, disable unavailable PDFs, keep external document opening separate from emulation.

## Implementation

Desktop lookup checks the executable documentation folder and development checkout. PDF signatures are checked. Availability is cached in UI memory and refreshed explicitly; no per-frame disk reads or PDF decoding in the machine loop. Browser availability comes from documentation/available.rs and Trunk copies the folder.



Native manuals now use bytes embedded in the executable and open inside the integrated viewer. Program reference + manual identity selects a reviewed 1-based physical PDF page; a missing section is labelled full manual, never claimed as a program jump. Browser builds retain a #page link to the packaged PDF.

Page expectation correction: the reviewed hp67-pac-standard-en.pdf has MOVING AVERAGE / printed section 01-01 on physical PDF page 14. PDF page 10 is FORMAT OF USER INSTRUCTIONS. The generated source-backed section link was already correct; the test and render smoke expectation now use PDF page 14 (zero-based render index 13).

Ayuda → Uso del emulador opens a separate window for card import and speed guidance. Its open state is independent of the manual browser and does not affect emulation.
