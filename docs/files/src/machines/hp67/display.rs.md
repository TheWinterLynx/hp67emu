# `src/machines/hp67/display.rs`

## Purpose

Defines the source-backed structural HP-67 display path: ROM0 (`1818-0268`) receives the eight-bit display code on IS, decodes it to LED anodes, emits an explicit STR event, and the `1820-1749` cathode driver consumes ROM0 STR plus ACT RCD downstream.

## Why it exists

Direct HP-67 logic-analyser evidence fixes several display facts that no longer need to remain semantic guesses: display data occupies IS bit times `b0..b7` LSB-first; ROM0 decodes the byte; STR occurs at the end of that display window; RCD resets the cathode sequence; the refresh has fifteen STR slots; slot order is exponent units, exponent tens, shared signs, mantissa digits 11 through 1, then a duplicate exponent-units slot; and the ROM0 decode table is directly observed.

This layer deliberately stops short of inventing exact PHI-relative launch/sample edges, pulse widths, transistor propagation or LED current. It gives later electrical-device work a tested display-control boundary rather than deriving visible text in the UI.

## Relationships

Consumes `display_data_serial_bit()` from `timing.rs` and resolved `LogicLevel` values from the generic electrical layer. `Rom0DisplayEndpoint` sits on the same resolved IS net used by the structural ACT/ROM fetch path. `Rom0StrEvent` represents the ROM0-owned falling STR event after one complete display byte. `CathodeDriver1820_1749` consumes that STR event together with the ACT-owned coarse RCD event; it never feeds a scan slot back into ACT.

## Responsibilities

Reconstruct one complete display byte from IS `b0..b7`; reject floating/contentious/incomplete input; decode the direct HP-67 ROM0 codes; special-case the shared sign slot; preserve the fifteen observed scan roles; emit explicit structural STR events; consume ACT RCD downstream; and reject cathode phase or RCD mismatches instead of silently resynchronizing.

## Implementation

`Hp67SegmentMask` uses bits A..G/DP. `Rom0DisplayEndpoint` samples one resolved IS bit per evidenced display bit time and only produces a byte after all eight bits arrive. `str_falling_event()` emits a `Rom0StrEvent` only after a complete byte and valid scan slot. `CathodeDriver1820_1749::apply_control_edges()` validates that the ROM0 STR slot matches the downstream cathode phase, requires RCD only at the final slot, and returns to slot 1 after the observed slot-15 STR/RCD boundary without inventing ordering inside their physical overlap.

The direct HP-67 capture assigns `0x0A=o` and `0x0C=r`. A reviewed semantic Nonpareil character table has those two code numbers reversed; this implementation follows the direct HP-67 hardware capture and records the discrepancy rather than silently reconciling it.
