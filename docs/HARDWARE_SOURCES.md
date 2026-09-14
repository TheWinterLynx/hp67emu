# Hardware evidence and source policy

## Purpose

Cycle accuracy is only meaningful if implementation claims can be traced to hardware evidence. This document records the source hierarchy and what each source is allowed to establish.

## Source hierarchy

### Tier A — direct HP-67 hardware evidence

**HP-67 logic PCB schematic**

- https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf
- Directly identifies the 1820-2530 ACT and signal names including PHI1, PHI2, ISA, DATA, RCD, SYNC, F1/F2 and KC lines.
- Use for HP-67 wiring and pin/net identity.

**HP-67 physical board/chip inventory**

- https://www.keesvandersanden.nl/calculators/hp67_inside.php
- Confirms the HP-67 chip set and board layout: 1818-0231, 1818-0232, 1818-0268, 1818-0550, 1818-0551, 1820-1749, 1820-1751, 1820-2530, 1826-0322 and 1858-0050.
- Use for part inventory. Treat reverse-engineered prose as secondary to a service schematic when they differ.

### Tier B — detailed HP-family timing/service evidence

**Notes on HP's Classic Calculators / Teenix**

- https://literature.hpcalc.org/community/classic-notes.pdf
- Contains measured/reverse-engineered timing and HP-67-specific sections, including power-on observations, 56-bit bus behavior, ROM0 display decode, RCD/STR behavior and serial bus notes.
- Particularly useful for scope-derived waveforms and gaps not covered by an HP service manual.
- Because it is reverse engineering rather than an original HP design specification, uncertain details must be cross-checked where practical.

**HP-97 Service Manual**

- https://literature.hpcalc.org/community/hp97-sm-en.pdf
- The HP-97 is not the HP-67, but it is a close programmable/card-reader relative and service documentation describes ACT/ROM/CRC/display bus theory, SYNC, ISA/DATA timing and display scanning.
- Use as corroboration only where the relevant chips/signals are known to be shared or equivalent. Never copy HP-97-only behavior into HP-67 code without an HP-67-specific check.

### Tier C — architecture and microcode research

**Nonpareil microcode-level simulation**

- https://github.com/brouhaha/nonpareil
- https://nonpareil.brouhaha.com/microcode_simulation.pdf
- Strong reference for how HP calculator microcode simulation can be structured and for Woodstock-family concepts.
- It does not by itself prove an HP-67-specific pin timing or opcode variant.

**Sydney Smith HP-67 / Woodstock microcode articles**

- https://www.sydneysmith.com/wordpress/articles/
- Contains HP-67 startup, keypress, flag, card-reader and Woodstock microcode analyses.
- Use to form hypotheses/test cases, then anchor exact implementation to raw ROM and hardware traces when possible.

**HP Museum HP-67 ROM-reader discussion**

- https://www.hpmuseum.org/forum/thread-18327.html
- Documents practical investigation of HP-67 ROM/RAM bus behavior and hardware ROM reading.
- Useful secondary evidence for electrical bus ownership and ROM provenance work.

## Claims we will not guess

The following must have a source citation or a captured regression trace before production implementation depends on them:

- exact PHI1/PHI2 frequency, pulse width and dead time;
- ISA and DATA passive level and active-drive polarity;
- exact SYNC bit positions and width;
- RCD and STR edge placement;
- ACT reset sequencing;
- opcode semantics that differ across chipset generations;
- keyboard matrix wiring and mode-switch flag path;
- CRC/card-reader timing and bit encoding;
- sign-digit transistor routing details.

Unknown behavior is represented as an explicit TODO or unimplemented error, not filled with a plausible value.

## ROM provenance

The emulator is intended to execute real microcode, but source availability and redistribution rights are separate questions. Before any ROM bytes are committed to the repository:

1. record origin and extraction method;
2. compute SHA-256;
3. map the image to physical part number/bank/address range;
4. compare against at least one independent known-good dump where possible;
5. decide whether the repository can legally redistribute it.

The core must support externally supplied ROM images so development is not blocked by redistribution policy.
