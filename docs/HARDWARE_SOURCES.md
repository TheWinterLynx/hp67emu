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
- Confirms an inspected HP-67 built with the 1820-2530 ACT plus 1818-0231, 1818-0232, 1818-0268, 1818-0550, 1818-0551, 1820-1749, 1820-1751, 1826-0322 and 1858-0050.
- Use for the physical target inventory. Treat reverse-engineered prose as secondary to a service schematic when they differ.

**Physical HP-67 ROM-reader / bus-monitor capture**

- https://www.hpmuseum.org/forum/thread-18327-page-2.html
- Tony Nixon monitored a physical HP-67's SYNC and IS buses at power-on and published the first observed instructions: `0x000 = 0x000`, `0x001 = 0x3e3`, branch target `0x0f8 = 0x11a`.
- In the same hardware experiment he removed a failed `1818-0232`, replaced its ROM function with a microcontroller and observed the calculator call into that ROM at `0x0fc6` from `0x0068`, execute five instructions and return.
- These are direct machine-level trace anchors. Use them as startup/fetch regressions and as future logic-analyzer targets; do not infer unreported PHI edge placement from the prose alone.

**HP-67 key-wait PHI/SYNC/IS waveform**

- https://literature.hpcalc.org/community/classic-notes.pdf
- The HP-67 Woodstock section includes an HP-67-specific waveform showing PHI1, PHI2, Sync and Is together while the calculator runs its key-wait loop.
- The decoded trace shows Sync present for ordinary fetched instructions and intentionally suppressed for the 10-bit word following an `IF`, where that ROM word is consumed as the implied-GOTO address. This directly corroborates the HP-67 THEN-GOTO fetch rule.
- Use this as direct HP-67 evidence for the semantic role of Sync. The currently available text/image does not yet justify hard-coding exact bit-slot boundaries or PHI sampling edges.

### Tier B — detailed HP-family timing/service evidence

**Notes on HP's Classic Calculators / Teenix**

- https://literature.hpcalc.org/community/classic-notes.pdf
- Contains measured/reverse-engineered timing and HP-67-specific sections, including power-on observations, 56-bit bus behavior, ROM0 display decode, RCD/STR behavior and serial bus notes.
- Also documents the HP-67/97 banked ROM map: bank 0 spans the normal 4K address space; HP-67 bank 1 is populated only for the equivalent of PC `0x400..0x7ff`, while entering the low `0x000..0x3ff` region forces bank 0.
- Particularly useful for scope-derived waveforms and gaps not covered by an HP service manual.
- Because it is reverse engineering rather than an original HP design specification, uncertain details must be cross-checked where practical.

**HP-97 Service Manual**

- https://literature.hpcalc.org/community/hp97-sm-en.pdf
- The HP-97 is not the HP-67, but it is a close programmable/card-reader relative and service documentation describes ACT/ROM/CRC/display bus theory, SYNC, ISA/DATA timing and display scanning.
- Its logic-PCA replacement-parts table lists 1820-1596 as the replacement ACT and explicitly permits 1820-2530 when 1820-1596 is unavailable. This is strong HP-origin evidence that the two ACT revisions are intended to be compatible in this hardware family.
- Use as corroboration only where the relevant chips/signals are known to be shared or equivalent. Never copy HP-97-only behavior into HP-67 code without an HP-67-specific check.

**Jacques Laporte serial-bus measurements**

- https://archived.hpcalc.org/laporte/ROM%20DUMP.htm
- https://archived.hpcalc.org/laporte/HP35%20Hardware%20basic%20design.htm
- Establishes a useful `b0..b55` trace-numbering convention and includes measured Classic-series bus windows plus a Woodstock/HP-25 ISA waveform comparison.
- Laporte explicitly notes that the Woodstock address/instruction windows differ from the Classic arrangement. Therefore these pages support the project bit-numbering convention and family-level hypotheses, but Classic bit windows must not be copied into HP-67 code.

### Tier C — architecture and microcode research

**Nonpareil microcode-level simulation**

- https://github.com/brouhaha/nonpareil
- https://nonpareil.brouhaha.com/microcode_simulation.pdf
- Strong reference for how HP calculator microcode simulation can be structured and for Woodstock-family concepts.
- Nonpareil's HP-67 definition names the older 1820-1596/MK6216N ACT while its Woodstock variant table assigns the same unusual P-wrap behaviour to both 1820-1596 and 1820-2530. Combined with HP's replacement note above, we treat 1820-1596 semantic behaviour as a valid compatibility reference while targeting 1820-2530 as the physical ACT revision in this emulator.
- It does not by itself prove an HP-67-specific pin timing or opcode variant.

**x11-calc HP-67 emulator**

- https://github.com/mike632t/x11-calc
- `src/x11-calc-cpu.c` explicitly executes one complete instruction per `v_processor_tick()`, so this is not a PHI/bit-cycle electrical timing reference.
- It is still valuable as a second semantic implementation: it records HP-67-specific P-pointer behaviour, implicit bank-0 selection in the low ROM region, HP-67 CRC/card operations, the complete 35-key mapping, an embedded 8192-entry ROM corpus and diagnostic-card programs.
- Use for behavioural and ROM corpus cross-checks only. See `docs/X11_CALC_ANALYSIS.md`.

**Sydney Smith HP-67 / Woodstock microcode articles**

- https://www.sydneysmith.com/wordpress/articles/
- Contains HP-67 startup, keypress, flag, card-reader and Woodstock microcode analyses.
- Use to form hypotheses/test cases, then anchor exact implementation to raw ROM and hardware traces when possible.

**HP Museum HP-67 ROM-reader discussion**

- https://www.hpmuseum.org/forum/thread-18327.html
- Documents practical investigation of HP-67 ROM/RAM bus behavior and hardware ROM reading.
- Tony Nixon states that HP-67 ROM `Is` and `Data` pins are bidirectional while Sync, Phi1 and Phi2 are inputs to the ROM. Use this as bus-direction evidence, while leaving exact intra-bit drive/release timing open.
- Useful secondary evidence for electrical bus ownership and ROM provenance work; specific physical captures promoted above to Tier A should be treated as direct evidence points.

**Tom Napier, “An HP-67 Anatomy Lesson”**

- PPC Journal V5N7 pp. 7-8, V5N8 pp. 14-17 and V5N10 pp. 25-27.
- Later researchers cite this series as a direct 1978 investigation of the HP-67 machine time word using external shift registers on ISA/DATA.
- Recovering the exact scans/pages is a priority because they may resolve HP-67-specific serial bit windows without borrowing them from another calculator family.

## ACT revision policy

The production physical target is the **1820-2530 ACT** because it is identified by the HP-67 schematic and by an inspected HP-67 board. The older **1820-1596** remains an important semantic/reference revision because Nonpareil uses it for the HP-67 definition and HP service documentation establishes 1820-2530 as a compatible substitute in the closely related HP-97 logic assembly.

This resolves the apparent part-number conflict without claiming that every manufactured HP-67 necessarily contains the same ACT revision. If later primary HP-67 production documentation shows revision-dependent electrical behaviour, that difference must be represented explicitly rather than hidden under one generic Woodstock implementation.

## Claims we will not guess

The following must have a source citation or a captured regression trace before production implementation depends on them:

- exact PHI1/PHI2 frequency, pulse width and dead time;
- ISA and DATA passive level and active-drive polarity;
- exact HP-67 ISA address and instruction bit windows within `b0..b55`;
- exact SYNC bit positions, width and sampling edge;
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
