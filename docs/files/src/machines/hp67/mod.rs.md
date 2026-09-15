# `src/machines/hp67/mod.rs`

## Purpose
Defines the HP-67-specific machine module boundary.

## Why it exists
The HP-67 combines a particular ACT, ROM/RAM set, display drivers, serial-word timing, IS/ISA bus behavior and card-reader electronics. Those choices must be explicit and isolated from generic simulation infrastructure.

## Relationships
Exports `wiring` hardware vocabulary, `timing` serial-word coordinates, `isa` ROM-fetch serialization helpers, `fetch` ACT↔ROM serial transport endpoints and the initial `machine` backplane shell. Later sibling modules will model the full ACT, physical ROM/RAM chips, display, keyboard, CRC and card transport.

## Responsibilities
Provide one coherent HP-67 namespace, re-export high-value public types/constants, and prevent GUI details or prematurely generalized Woodstock assumptions from entering machine code.

## Implementation
Declares `fetch`, `isa`, `machine`, `timing` and `wiring`; re-exports the HP-67 electrical backplane, structural serial fetch endpoints/pipeline latch, 4/14/56 geometry, evidenced b16..b27 address and b46..b55 ROM-word windows, IS wired-high serializers, chip/net vocabulary and `CHIPSET`.
