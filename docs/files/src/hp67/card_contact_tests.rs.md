# src/hp67/card_contact_tests.rs

## Purpose
Native regressions for physical card insertion through resolved MTRS and real firmware.

## Why it exists
The production reference mirror must not manufacture the card-present input or conceal a pending DATA-tail fault.

## Relationships
Pairs the production machine with its explicit reference fixture, shared card transport, independent CRC, actual ROM firmware and the existing long CRC data roundtrip.

## Responsibilities
Protect insertion/withdrawal/power-off against invalid sources; preserve a pending tail; reject mirror substitution; test both insertion ends, reset/reference release and MTRS-driven firmware MOTOR start.

## Implementation
Five tests exercise all foreign sources at both polarities, source removal/retry without losing media/state/time, 1200-word boot parity and real card-present execution until MOTOR. The existing full 34-record write/read compare helper additionally checks resolved MTRS after every word.
