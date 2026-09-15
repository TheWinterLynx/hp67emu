# `src/machines/hp67/mod.rs`

## Purpose
Defines the HP-67-specific machine module boundary.

## Why it exists
The HP-67 combines a particular ACT, ROM/RAM set, display drivers, serial-word timing and card-reader electronics. Those choices must be explicit and isolated from generic simulation infrastructure.

## Relationships
Exports `wiring` hardware vocabulary, `timing` serial-word coordinates and the initial `machine` backplane shell. Later sibling modules will model ACT, ROM/RAM, display, keyboard, CRC and card transport.

## Responsibilities
Provide one coherent HP-67 namespace, re-export high-value public types/constants, and prevent GUI details or prematurely generalized Woodstock assumptions from entering machine code.

## Implementation
Declares `machine`, `timing` and `wiring`; re-exports `Hp67ElectricalBackplane`, `Hp67WordTiming`, the 4/14/56 serial geometry constants, `Hp67Chip`, `Hp67Net` and `CHIPSET`.
