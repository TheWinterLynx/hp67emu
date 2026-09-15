# `src/machines/hp67/timing.rs`

## Purpose
Defines the HP-67 serial machine-word coordinate system used while the electrical timing model is being built.

## Why it exists
The Woodstock datapath operates on a 56-bit serial word made from fourteen 4-bit digit times. The project needs one stable `b0..b55` numbering convention before ISA, DATA, SYNC, RCD and display timing can be attached to individual bit positions. At the same time, the exact HP-67 edge placement and physical pulse widths are not yet sufficiently evidenced to encode them as facts.

## Relationships
Used by `src/machines/hp67/machine.rs` alongside the generic `TwoPhaseClock` scaffold. `docs/HARDWARE_SOURCES.md` and the timing research notes define which details are physically supported and which remain open. Future ACT, ROM/RAM and trace modules will consume these coordinates rather than inventing their own numbering.

## Responsibilities
Define 4 bits per digit, 14 digits per word and 56 bits per word; expose the current word, bit, digit and bit-within-digit coordinates; and advance those coordinates deterministically from the temporary four-subphase PHI scaffold.

## Implementation
`Hp67WordTiming` starts at word 0, bit 0, subphase 0. Four scheduler subphases currently form one complete temporary PHI1/dead/PHI2/dead period and therefore advance one serial bit. Bit 55 wraps to bit 0 and increments the machine-word counter. This four-slot relationship is explicitly a scheduler scaffold: it does not claim equal physical subphase duration or commit ISA/DATA/SYNC to any edge. Unit tests lock the 14×4 geometry and the exact wrap behavior.
