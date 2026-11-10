# Live pacing and reset integrity regressions

## Purpose
Prove partition-independent host duration accumulation and non-destructive reset rejection.

## Why it exists
Quantizing each host call independently loses fractional time. Resetting an occupied
transport must not silently destroy physical media.

## Relationships
Child test module of `src/hp67.rs`, using actual embedded firmware, the production
live pipeline, card/contact APIs and the focused `core_audit_` gate.

## Responsibilities
Compare whole-duration and fragmented-duration execution, reset the fractional
pacing debt explicitly, and prove a rejected reset leaves a machine able to follow
an untouched control through subsequent firmware words.

## Implementation
500 ns and irregular partitions span reset hold and 400 actual firmware words.
Assertions compare tick/cycle counts, timing debt, ACT/reference/serial state, CRC,
autonomous RAM, media and display. Reset tests compare with a fresh instance just
below the reset-hold threshold. Occupied-reader tests cover both insertion ends,
retained key contact and subsequent execution, then explicit physical-card recovery
followed by successful electronic reset. They do not certify physical reset edges.

## Audit regressions, 2026-11-09
Early held/released keys run beyond 2,000 words while retaining the key latch.
Public pacing rejects all further elapsed time after an injected PHI failure until
reset. Recovery writes an actual transport record, faults reader contacts, rejects
normal recovery/reset, then checks exact media, dirty state and insertion end via
the consuming recovery API. These tests are added, not claimed executed here.

## Independent processor clock (2026-11-10)

Independent-clock regressions cover reset hold, nanosecond debt, full PHI word counts, batch partitioning, real firmware with inserted media in both orientations and modes, and shared fail-stop handling. These test the optional host acceleration contract, not physical clock equivalence.
