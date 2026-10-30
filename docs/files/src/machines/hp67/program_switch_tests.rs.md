# src/machines/hp67/program_switch_tests.rs

## Purpose
Regressions for resolved program selection and atomic freezing of both physical CRC inputs.

## Why it exists
Program contact test-and-clear must not clear a physical selector, and the two contact inputs must not partially update if the second sample fails.

## Relationships
Exercises program_switch, MTRS contact source, independent CRC command receiver and oracle, F2 publication and named sparse capture parsing.

## Responsibilities
Cover all opcodes/qualifiers and four PROGRAM/card-present combinations, frozen-word lifetime, all foreign contributors/polarities, missing/replaced/tampered sources, observer mismatch/late freeze and legacy unknown observations.

## Implementation
Five ordinary tests include 8192 actual delayed IS/SYNC command cases comparing an independent CRC oracle and resolved F2; every word retains 56 cells/224 transitions. Legacy input has two valid rows and omitted PGMSW stays unknown. A feature-only test proves changed-only selector capture publication without advancing time and exact export/import parity.
