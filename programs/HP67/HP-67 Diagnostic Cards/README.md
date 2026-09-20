# HP-67 Diagnostic Cards

This directory contains diagnostic media that is kept separate from the synthetic Custom
Diagnostic Pacs.

## SD-15C

`SD-15C-Diagnostic-Program.hp67card` is the exact 250-byte native card supplied for the project on
2026-09-20. It is preserved byte-for-byte with SHA-256:

`f626047e875d919bb22bf7b25bb0d58218955b07ff504f2f546239a2f1fdf75a`

It contains two recorded program tracks with header classes 3 and 4. Their first record headers are
`0x03090222` and `0x04090222`; both 28-bit record-34 checksums validate. The Program Library
loads this native container directly. No Teenix wrapper is required or generated.

The decoded user-program listing places `LBL A` at step 035 and `LBL 8` at step 113. Loading the
library entry therefore exercises the original native media contents rather than the separately
checked-in SD1-15A Standard Pac card.
