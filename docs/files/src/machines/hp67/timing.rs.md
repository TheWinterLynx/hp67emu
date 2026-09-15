# `src/machines/hp67/timing.rs`

## Purpose
Defines the HP-67 serial machine-word coordinate system and the evidenced ROM0-display and IS/ISA ROM-fetch windows.

## Why it exists
The Woodstock datapath operates on a 56-bit serial word made from fourteen 4-bit digit times. The project needs one stable `b0..b55` numbering convention before ISA, DATA, SYNC, RCD and display timing can be attached to individual bit positions. Tony Nixon's HP-67 logic-analyser material in *Notes on HP's Classic Calculators* fixes ROM0 display data at `b0..b7` LSB-first, the 12-bit ROM address at `b16..b27` LSB-first, and the 10-bit ROM result at `b46..b55` LSB-first. The same evidence shows SYNC occupying the final ten bit times for a normal instruction and remaining low after an IF/test, and places the ROM0 STR pulse in display bit 7 without yet fixing its final PHI-relative edge convention.

## Relationships
Used by `src/machines/hp67/machine.rs` alongside the generic `TwoPhaseClock` scaffold; by `src/machines/hp67/isa.rs` and `fetch.rs` for instruction transport; and by `src/machines/hp67/display.rs` for ROM0 display-byte reconstruction. `docs/research/HP67_ISA_TIMING.md` records the evidence and limits. Future ACT, ROM/RAM, display and trace modules consume these coordinates rather than inventing their own numbering.

## Responsibilities
Define 4 bits per digit, 14 digits per word and 56 bits per word; expose the current word, bit, digit and bit-within-digit coordinates; encode ROM0 display data at `b0..b7`, the ROM address at `b16..b27`, and the ROM-word/SYNC decision window at `b46..b55`; expose the coarse STR bit coordinate `b7`; classify fetch roles; and advance all coordinates deterministically from the temporary four-subphase PHI scaffold.

## Implementation
`Hp67WordTiming` starts at word 0, bit 0, subphase 0. Four scheduler subphases currently form one complete temporary PHI1/dead/PHI2/dead period and therefore advance one serial bit. Bit 55 wraps to bit 0 and increments the machine-word counter. `display_data_serial_bit()` maps `b0..b7` to display bits `0..7`; `isa_window_for_bit()` maps `b16..b27` to serial address bits `0..11` and `b46..b55` to serial ROM bits `0..9`; all are LSB-first. `sync_decision_window()` identifies the ten return bits without deciding final physical edge timing. Exact pulse widths and precise PHI launch/sample edges remain open and are not inferred from these coarse bit-window observations.
