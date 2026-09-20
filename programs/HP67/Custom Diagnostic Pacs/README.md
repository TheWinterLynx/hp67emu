# Custom Diagnostic Pacs

Synthetic HP-67 program cards for emulator validation. They are not Hewlett-Packard programs.
Each card isolates a small advanced-programming path and halts on a known display value.

Use: power-cycle the calculator, leave RUN selected, load the card from the Program Library,
insert the opposite end if firmware shows `Crd`, then press `A`.

| Ref | Coverage | Expected after A |
| --- | --- | ---: |
| CD-01 | LBL, GSB, RTN, GTO and arithmetic after return | 7.00 |
| CD-02 | SF 0, CF 0, F? 0 true/false skip semantics | 6.00 |
| CD-03 | x=y? and x=0? conditional execution/skip | 7.00 |
| CD-04 | x<>I, STO (i), RCL (i) | 42.00 |
| CD-05 | two-level nested GSB/RTN return stack | 6.00 |
| CD-06 | two-pass program card and GTO label E in steps 113-224 | 67.00 |
| CD-07 | I-register ISZ loop, skipping GTO when I reaches zero | 3.00 |
| CD-08 | I-register DSZ loop, skipping GTO when I reaches zero | 3.00 |

Every custom diagnostic is checked in as a Teenix-compatible `.hpp` used by the Program Library
and as a native `.hp67card` containing the exact same logical tracks. The generator verifies the
28-bit card checksum and can reproduce every binary fixture with `--check`.

The Standard Pac directory also contains `SD1-15A-Diagnostic-Program.hp67card`, a lossless native
container conversion of the existing source-backed SD1-15A Teenix sides. The earlier scratch name
`SD-15C-Diagnostic-Program.hp67card` was not source-backed; the canonical Standard Pac reference
is SD1-15A.
