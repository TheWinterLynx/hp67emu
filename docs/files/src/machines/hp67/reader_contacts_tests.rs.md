# src/machines/hp67/reader_contacts_tests.rs

## Purpose
Regressions for reader input ownership, head-edge write permission and source-authoritative transport.

## Why it exists
WPS polarity is easy to invert: grounded means write permitted, not protection asserted. A level change after head closure must not silently reclassify that pass.

## Relationships
Exercises reader_contacts, actual delayed IS/SYNC CRC commands, owned CRC buffers, card transport, physical card metadata and named sparse capture parsing.

## Responsibilities
Cover both WPS states before/at/after HDS closure, all foreign contributors on both nets/polarities, source loss/replacement/tampering, mismatch atomicity, source-based write inhibit and service-style grounded bypass, and legacy optional observations.

## Implementation
Seven ordinary tests include all four metadata/permission combinations: a resolved blocked WPS prevents writing even on writable media, while grounded WPS permits writing even on clipped metadata. Write mode and READY acknowledgement use real CRC serial command words. A feature-only test verifies one complete pair per capture publication, no intermediate pair or extra PHI time, changed-only rows and export/import parity. Legacy input has two valid rows.

Service note 97-101 also requires opening/reclosing grounded WPS between passes. The receiver consumes the accepted WPS closure and requires an observed open before a later head closure can grant permission. A dedicated regression rejects held-ground reuse and accepts a real reopen/reclose. Reset begins armed as an explicit bridge initialization policy; no physical power-on waveform is claimed.
