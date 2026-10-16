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
