# `src/machines/hp67/mod.rs`

## Purpose
Defines the HP-67-specific machine module boundary.

## Why it exists
The HP-67 combines a particular ACT, ROM/RAM set, display drivers and card-reader electronics. Those choices must be explicit and isolated from generic simulation infrastructure.

## Relationships
Exports `wiring` hardware vocabulary and the initial `machine` backplane shell. Later sibling modules will model ACT, ROM/RAM, display, keyboard, CRC and card transport.

## Responsibilities
Provide one coherent HP-67 namespace, re-export high-value public types, and prevent GUI details from entering machine code.

## Implementation
Declares `machine` and `wiring` and re-exports `Hp67ElectricalBackplane`, `Hp67Chip`, `Hp67Net` and `CHIPSET`.
