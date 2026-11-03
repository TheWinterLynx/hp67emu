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
