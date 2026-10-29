# `src/machines/hp67/wiring.rs`

## Purpose
Defines the verified HP-67 chip inventory and named electrical nets used by the machine model.

## Why it exists
Part numbers and signal names are hardware facts, not implementation details. Centralizing them provides a reviewed vocabulary before chip behavior is written and prevents guessed wiring from spreading across modules.

## Relationships
Consumed by `machine.rs` and future HP-67 chip modules. Evidence for each entry is tracked in `docs/HARDWARE_SOURCES.md`.

## Responsibilities
Enumerate confirmed chips, expose part numbers, enumerate currently confirmed core nets, define stable dense electrical driver slots for the production fabric, and keep uncertain signals out until verified.

## Implementation
`Hp67Chip` maps enum variants to physical part numbers and `CHIPSET` provides the inventory. `Hp67Driver` provides dense stable slots for those physical output owners plus an explicitly temporary aggregate ROM responder and external hardware stimulus. `Hp67Net` names PHI1/PHI2, ISA, DATA, SYNC, RCD, STR, F1/F2 and KC1–KC5. The net enum is `repr(u8)` with a tested dense `index()`/`COUNT` contract so the HP-67 hot path can use fixed arrays instead of tree maps without changing electrical semantics. Tests catch duplicate part numbers, driver/net index-order drift and accidental removal of required core nets.

## M15J five-ROM composition

The aggregate driver documentation now reflects its actual remaining roles:
reference ROM fixtures and the live logical-level RAM DATA bridge. Live IS fetch
uses the existing five physical ROM driver slots; no new guessed pin is added.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.


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


M16G appends Motor at index 14 without changing prior indices or driver identities. HP-67 logic PCB sheet 2 labels the barred MOT/Motor CRC output; HP-97 service section 2-36 corroborates grounding MOTOR to activate the sense-amplifier/motor stage. See M16G_RESOLVED_MOTOR_2026-10-26.md. No bias or transistor stage is inferred.


M16H appends Mtrs at index 15, preserving all previous net/driver indices. Sixteen nets times thirteen drivers gives 208 staging slots, within the existing u8 marker range. MTRS uses External physical stimulus; generic fabric default remains Floating.
