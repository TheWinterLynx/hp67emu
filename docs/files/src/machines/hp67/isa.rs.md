# `src/machines/hp67/isa.rs`

## Purpose
Encodes the evidenced HP-67 IS/ISA serial-bus behavior for ROM0 display data and ROM fetch without yet pretending the complete ACT/ROM devices exist.

## Why it exists
The HP-67 does not move display data, a 12-bit address or a 10-bit microinstruction through parallel buses. Tony Nixon's HP-67 logic-analyser work shows the ACT emitting the eight-bit ROM0 display code LSB-first during b0..b7, serializing the 12-bit ROM address LSB-first during b16..b27, and the selected ROM returning its 10-bit word LSB-first during b46..b55. The same hardware investigation indicates that the shared IS line is loosely biased low and active devices pull it high rather than actively driving zero. These details need to be represented once, explicitly, before ACT, ROM0 and ROM/RAM device code is completed.

## Relationships
Uses the canonical `b0..b55` windows defined by `src/machines/hp67/timing.rs` and the generic electrical `Drive` type from `src/emulation/net.rs`. `src/machines/hp67/machine.rs` gives the physical IS net its passive pull-down bias. `display.rs` reconstructs the ROM0 byte from b0..b7, while `fetch.rs` uses the display, address and ROM-word helpers together on the same resolved structural word. Future ACT and ROM/RAM devices will call the same helpers when their electrical serializers replace the current bring-up harness.

## Responsibilities
Represent the eight-bit ROM0 display byte, 12-bit ROM-address and 10-bit ROM-word serial streams, serialize all three least-significant bit first, translate logical one/zero to HP-67 wired-high drive/release behavior, and ensure the display, ACT-address and ROM-response roles occupy their evidenced non-overlapping windows.

## Implementation
`wired_high_drive()` maps one to `Drive::High` and zero to `Drive::HighZ`, relying on the machine's passive pull-down for the observed low state. `act_display_drive()` contributes only during b0..b7; `act_address_drive()` contributes only during b16..b27; `rom_word_drive()` contributes only during b46..b55. Unit tests verify LSB-first display serialization with `0x30`, use the measured HP-67 example address `0x07B` and returned word `0x04C`, and prove that all three active serial roles are mutually non-overlapping. Exact PHI launch/sample edges are deliberately not encoded here because the reviewed evidence fixes the bit cells but the final machine-readable edge convention remains open.
