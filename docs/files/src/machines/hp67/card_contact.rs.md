# src/machines/hp67/card_contact.rs

## Purpose
Resolved MTRS card-present contact source and CRC-facing digital receiver.

## Why it exists
Insertion grounds MTRS in the service evidence. Production must consume the conductor instead of importing the architectural card-present flag.

## Relationships
Uses existing External driver, appended Mtrs net, serial_crc_control, live adapter and pin capture. HDS/head readiness remains separate.

## Responsibilities
Publish physical input changes, validate sole source including agreeing foreign drives, reject tampered/missing/replacement source and preserve it for inspection.

## Implementation
Attach normalizes open High. set_closed checks the previous publication before changing it, and changes only on actual physical events. Receiver accepts Low as closed only with one External contribution. High/Low normalization and word-frozen reception remain working approximations; no passive bias, bounce, voltage or PHI timing is invented.
