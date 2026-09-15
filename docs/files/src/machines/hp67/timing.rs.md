# `src/machines/hp67/timing.rs`

## Purpose
Defines the HP-67 serial machine-word coordinate system and the now-evidenced IS/ISA ROM-fetch windows.

## Why it exists
The Woodstock datapath operates on a 56-bit serial word made from fourteen 4-bit digit times. The project needs one stable `b0..b55` numbering convention before ISA, DATA, SYNC, RCD and display timing can be attached to individual bit positions. Tony Nixon's HP-67 logic-analyser section in *Notes on HP's Classic Calculators* provides the first exact HP-67 fetch anchors: a 12-bit ROM address LSB-first during bit times 16..27 and a 10-bit ROM result LSB-first during bit times 46..55. The same section shows SYNC occupying those final ten bit times for a normal instruction and remaining low after an IF/test so the returned word becomes a full 10-bit implied-GOTO destination.

## Relationships
Used by `src/machines/hp67/machine.rs` alongside the generic `TwoPhaseClock` scaffold and by `src/machines/hp67/isa.rs` to serialize address and ROM words. `docs/research/HP67_ISA_TIMING.md` records the evidence and limits. Future ACT, ROM/RAM and trace modules consume these coordinates rather than inventing their own numbering.

## Responsibilities
Define 4 bits per digit, 14 digits per word and 56 bits per word; expose the current word, bit, digit and bit-within-digit coordinates; encode the evidenced address window b16..b27 and ROM-word/SYNC decision window b46..b55; classify each bit by fetch role; and advance those coordinates deterministically from the temporary four-subphase PHI scaffold.

## Implementation
`Hp67WordTiming` starts at word 0, bit 0, subphase 0. Four scheduler subphases currently form one complete temporary PHI1/dead/PHI2/dead period and therefore advance one serial bit. Bit 55 wraps to bit 0 and increments the machine-word counter. `isa_window_for_bit()` maps b16..b27 to serial address bits 0..11 and b46..b55 to serial ROM bits 0..9, both LSB-first. `sync_decision_window()` identifies those same ten return bits without deciding whether SYNC is high or low; that level depends on whether the previous instruction was a normal instruction or an IF/test. Exact physical pulse widths and precise PHI launch/sample edges remain open and are not inferred from the bit-window evidence.
