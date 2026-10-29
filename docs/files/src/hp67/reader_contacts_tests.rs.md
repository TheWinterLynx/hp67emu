# src/hp67/reader_contacts_tests.rs

## Purpose
Native acceptance of resolved HDS/WPS through the real live firmware and card lifecycle.

## Why it exists
Faulty reader sources must not consume pending RAM tails, input events, CRC buffers or physical media. Selected insertion end must determine the physical corner contact, while CRC owns the retained permission.

## Relationships
Pairs production and explicit reference machines, the real ROM, card transport and retained CRC receiver. The existing full 34-record write/read regression is additionally strengthened.

## Responsibilities
Cover pending-tail recovery, all foreign sources before word/card mutation, per-end corner selection, rearming/reset/reference release, prevention of observer substitution and real-firmware F7 without writing either protected end.

## Implementation
Five tests retain 56 cells/224 transitions per completed word. Both selected protected ends are driven through real program-mode firmware and compared word-for-word with the independent reference until F7. Manual head events are explicit test fixtures for the existing logical contact boundary, not measured insertion geometry.
