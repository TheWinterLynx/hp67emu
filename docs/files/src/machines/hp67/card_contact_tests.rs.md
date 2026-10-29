# src/machines/hp67/card_contact_tests.rs

## Purpose
Regressions for MTRS conductor causality and atomic CRC input freezing.

## Why it exists
A binary resolved level alone cannot prove source identity; a sticky physical contact must survive CRC test-and-clear without becoming a hidden host Boolean.

## Relationships
Exercises card_contact, serial_crc_control, delayed IS/SYNC command reception, F2 publication and capture parser.

## Responsibilities
Cover all opcodes/qualifiers/contact states, retained external versus clearable internal flag, frozen-word policy, all foreign drivers/polarities, source replacement, mirror mismatch, phase rejection and legacy unknown observations.

## Implementation
Actual words advance 56 cells and 224 PHI transitions. All 4096 opcode/contact/qualifier cases compare an independent CRC oracle and resolved F2. A feature-only test records closed/open at ordinal 224/448, no extra rows for unchanged publication and no extra time for physical publication. Capture ordinals are not calibrated timing.

M16H v2 corrects the legacy fifteen-pin fixture to contain two valid rows with a PHI1 transition. It explicitly checks that the first, single-row prefix is rejected by the unchanged parser, then checks unknown MTRS level/ownership on both accepted rows and export/import parity. The owner v1 run passed the other four focused library tests but stopped here before later stages.
