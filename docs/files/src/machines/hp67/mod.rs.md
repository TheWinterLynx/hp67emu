# `src/machines/hp67/mod.rs`

## Purpose
Defines the HP-67-specific machine module boundary.

## Why it exists
The HP-67 combines a particular ACT, ROM/RAM set, display drivers, serial-word timing, IS/ISA bus behavior and card-reader electronics. Those choices must be explicit and isolated from generic simulation infrastructure.

## Relationships
Exports `wiring` hardware vocabulary, `timing` serial-word coordinates, `isa` ROM-fetch serialization helpers, `fetch` ACT<->ROM serial transport endpoints, `display` for the structural ROM0/anode and cathode scan path, `act` for the independent complete instruction-boundary ACT bring-up core, `act_serial_execution` for the explicit b0..b55 execution lifetime and arithmetic routing, `act_serial_state` for immutable pre-instruction A/B/C/P/radix snapshots, `crc` for the independent CRC control interface, `architectural` for the composed ACT+RAM+CRC bring-up machine, and the initial electrical `machine` backplane shell. Later device work will replace architectural/structural scaffolds with fully timed chips while retaining the same machine boundary.

## Responsibilities
Provide one coherent HP-67 namespace, re-export high-value public types/constants, and prevent GUI details or prematurely generalized Woodstock assumptions from entering machine code.

## Implementation
Declares `act`, `act_serial_execution`, `act_serial_state`, `architectural`, `crc`, `display`, `fetch`, `isa`, `machine`, `timing` and `wiring`; re-exports the HP-67 electrical backplane, structural serial fetch runner/endpoints/pipeline latch, complete ACT architectural state/core/RAM scaffold, explicit intra-word ACT execution lifetime/arithmetic routing/pre-instruction state snapshot, CRC control decoder/state, ACT+CRC architectural composition, ROM0 display-byte receiver/decoder, explicit `Rom0StrEvent`, downstream `CathodeScanError`/`1820-1749` cathode state, temporary `PowerOnAct*` compatibility aliases, 4/14/56 geometry, evidenced `b0..b7` display window, `b16..b27` address and `b46..b55` ROM-word windows, IS wired-high serializers, chip/net vocabulary and `CHIPSET`.
