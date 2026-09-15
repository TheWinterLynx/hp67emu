# `src/machines/hp67/isa.rs`

## Purpose
Encodes the evidenced HP-67 IS/ISA serial-bus behavior needed for ROM fetch without yet pretending the complete ACT/ROM devices exist.

## Why it exists
The HP-67 does not fetch a 12-bit address or 10-bit microinstruction through parallel buses. Tony Nixon's HP-67 logic-analyser work shows the ACT serializing the 12-bit ROM address LSB-first during bit times 16..27 and the selected ROM returning its 10-bit word LSB-first during bit times 46..55. The same hardware investigation indicates that the shared IS line is loosely biased low and active devices pull it high rather than actively driving zero. These details need to be represented once, explicitly, before ACT and ROM device code is added.

## Relationships
Uses the canonical `b0..b55` windows defined by `src/machines/hp67/timing.rs` and the generic electrical `Drive` type from `src/emulation/net.rs`. `src/machines/hp67/machine.rs` gives the physical IS net its passive pull-down bias. Future ACT and ROM/RAM devices will call these helpers when driving that net. ROM0 display-anode traffic on IS is a separate behavior and is intentionally not folded into these fetch helpers.

## Responsibilities
Represent the 12-bit ROM-address and 10-bit ROM-word masks, serialize both least-significant bit first, translate logical one/zero to HP-67 wired-high drive/release behavior, and ensure the ACT address and ROM response fetch roles occupy their evidenced non-overlapping windows.

## Implementation
`wired_high_drive()` maps one to `Drive::High` and zero to `Drive::HighZ`, relying on the machine's passive pull-down for the observed low state. `act_address_drive()` contributes only during b16..b27; `rom_word_drive()` contributes only during b46..b55. Unit tests use the measured HP-67 example address `0x07B` and returned word `0x04C`, verify LSB-first serialization, and prove that the two fetch contributors never actively overlap. Exact PHI launch/sample edges are deliberately not encoded here because the current reviewed text identifies the bit times but not yet a machine-readable edge convention.
