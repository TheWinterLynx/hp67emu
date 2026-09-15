# `src/machines/hp67/display.rs`

## Purpose

Defines the first source-backed structural model of the HP-67 display hardware path: ROM0 (`1818-0268`) receiving the eight-bit display code on IS, decoding it to LED anodes, and the `1820-1749` cathode scan responding to RCD/STR events.

## Why it exists

Direct HP-67 logic-analyser evidence in Tony Nixon's *Notes on HP's Classic Calculators* fixes several display facts that no longer need to remain semantic guesses: display data occupies IS bit times `b0..b7` LSB-first; ROM0 decodes the byte; STR occurs during bit 7; RCD resets the cathode sequence; the refresh has fifteen STR slots; slot order is exponent units, exponent tens, shared signs, mantissa digits 11 through 1, then a duplicate exponent-units slot; and the ROM0 decode table is directly observed.

This layer deliberately stops short of inventing exact PHI-relative launch/sample edges, pulse widths, transistor propagation or LED current. It gives later electrical-device work a tested display protocol boundary instead of deriving visible text in the UI.

## Relationships

Consumes `display_data_serial_bit()` from `timing.rs` and resolved `LogicLevel` values from the generic electrical layer. `Rom0DisplayEndpoint` is intended to sit on the same resolved IS net already used by the structural ACT/ROM fetch path. `CathodeDriver1820_1749` consumes future RCD/STR falling-edge events. Segment masks are suitable input to a later LED-energy accumulator and ultimately `ui/classic_display.rs`; they are not formatted display strings.

## Responsibilities

Reconstruct one complete display byte from IS `b0..b7`; reject floating/contentious/incomplete input; decode the direct HP-67 ROM0 codes `00..0E`, blank `0F`, observed blank `20`, decimal point `30`, and `4x` blanking; special-case scan slot 3 using the documented two sign bits; preserve the fifteen observed scan roles; and expose structural RCD reset / STR advance behavior without claiming final edge timing.

## Implementation

`Hp67SegmentMask` uses bits A..G/DP. `Rom0DisplayEndpoint` samples one resolved IS bit per evidenced display bit time and only produces a byte after all eight bits arrive. `decode_rom0_display_byte()` hard-fails unknown code values instead of filling gaps. For shared signs, bit 0 low energizes segment E for the mantissa-negative path and bit 1 high energizes segment G for the exponent-negative path. `CathodeDriver1820_1749` tracks structural scan slots 1..15; RCD falling resets to slot 1 and each supplied STR falling edge consumes the current role then advances/wraps the slot counter.

The direct HP-67 capture assigns `0x0A=o` and `0x0C=r`. A reviewed semantic Nonpareil character table has those two code numbers reversed; this implementation follows the direct HP-67 hardware capture and records the discrepancy rather than silently reconciling it.
