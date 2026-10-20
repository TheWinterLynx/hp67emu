# src/hp67/autonomous_ram_tests.rs

## Purpose

Regress production live autonomous RAM and oracle independence.

## Why it exists

Live command observation without independent read storage was insufficient fidelity.

## Relationships

Uses real firmware, default production RAM, explicit reference composition and the existing differential oracle.

## Responsibilities

Compare complete live ACT/CRC/RAM state, all 64 device registers, traces/ticks, reset and rejection of corrupted oracle RAM.

## Implementation

Two thousand firmware words in each transport must match the reference while device RAM matches every logical slot. Another real-firmware run alters only oracle memory before an installed read; the actual RAM response must remain original and fail the next-b1 authority comparison without committing C. Physical RCL then Digit0 contacts exercise a real recall beyond idle if boot has not already supplied a read.


## M15R autonomous RAM DATA identity

Autonomous RAM reads now publish through Hp67Driver::RamAggregate, a distinct
logical aggregate slot appended after all existing driver indices. Reference
ROM/RAM fixtures retain StructuralRomResponder. This is device identity, not a
new physical chip map. The shared source commit selects the RAM owner without
clearing foreign contributions, so a rogue ROM/reference drive remains visible
and can cause contention. No extra time edge or intermediate resolution.

The DATA path remembers autonomous receive ownership across body and next-word
b0/b1 tail. Its sole-owner guard requires RamAggregate for live reads; replacing
it with another named source at the same valid level fails before receive state
changes. Reset reconstructs the path and the RAM-capable transport binds its
owner again. The standalone RAM fixture also uses this identity. Exact physical
DATA output stage/polarity, per-IC mapping and PHI edges remain unresolved.
See `docs/research/M15R_RAM_DATA_DRIVER_2026-10-20.md`.
