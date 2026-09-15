# HP-67 56-bit word and SYNC evidence — 2026-09-15

## Scope

This note separates timing facts we can already encode from electrical details that still need a direct HP-67 waveform or stronger primary/service evidence. The implementation must not turn family-level diagrams into HP-67 facts merely because the signal names look similar.

## Evidence that is strong enough to encode now

The Woodstock/HP-67 datapath uses a 56-bit serial machine word arranged as fourteen 4-bit digit times. `hp67emu` therefore adopts a single canonical trace coordinate `b0..b55`, grouped as digit 0 bit 0 through digit 13 bit 3. This is a numbering convention for traces and devices, not yet an assertion about which PHI edge samples each signal.

Tony Nixon's *Notes on HP's Classic Calculators* contains an HP-67-specific oscilloscope trace of the key-wait loop showing PHI1, PHI2, Sync and Is together. The accompanying decoded sequence explicitly reports Sync present for ordinary fetched instructions and absent for the word following an `IF` test, where the full 10-bit ROM output becomes the implied-GOTO address. The shown loop includes examples such as `$0088 if s3 = 1` followed by `$0089 then go to $2D4` with Sync suppressed for the target word. This directly corroborates the semantic behavior already modeled by `InstructionState::ThenGoto`.

The 2022 HP Museum physical-ROM-reader discussion also records that, on the HP-67 ROM ICs, `Is` and `Data` are bidirectional while `Sync`, `Phi1` and `Phi2` are inputs to the ROM. That is useful bus-ownership evidence, but it still does not identify every drive/release edge within a bit time.

## Evidence used only as a coordinate aid

Jacques Laporte's measured Classic-series ROM-dump work labels one complete serial machine word `b0..b55`, and separately shows a Woodstock/HP-25 ISA waveform where address and instruction share the ISA line. His text explicitly notes that the Woodstock time windows differ from the Classic arrangement. Consequently we reuse the convenient `b0..b55` numbering convention, but **do not** copy the Classic address window (`b19..b26`) or instruction window (`b45..b54`) into HP-67 code.

The historical Tom Napier *An HP-67 Anatomy Lesson* series is repeatedly cited by later researchers as describing the HP-67 machine time word and capturing ISA/DATA with external shift registers. Recovering/scanning those exact pages remains a high-priority primary reverse-engineering source because it may resolve the remaining bit-slot questions directly for the HP-67.

## Current implementation boundary

`src/machines/hp67/timing.rs` now encodes only:

- 4 serial bits per digit;
- 14 digit times per machine word;
- 56 serial bit times per machine word;
- canonical trace coordinates `b0..b55`;
- deterministic wrapping from bit 55 to bit 0.

The existing temporary PHI scaffold has four scheduler slots (`PHI1 high`, dead time, `PHI2 high`, dead time). For deterministic bring-up, one complete four-slot scaffold period advances one serial bit coordinate. This does **not** claim equal real durations; it merely keeps the abstract PHI scaffold and the 56-bit coordinate in lockstep until measured timing replaces the placeholder durations.

## Deliberately unresolved

Do not yet encode any of the following as production electrical truth:

- exact HP-67 ISA address bit window inside `b0..b55`;
- exact HP-67 ISA instruction bit window inside `b0..b55`;
- ISA bit order on the wire;
- DATA valid/drive/release windows;
- exact SYNC start/end bit positions and active-edge relationship;
- PHI1/PHI2 pulse widths, dead time or absolute frequency for the 1820-2530 target;
- the edge on which ACT or ROM samples/drives each serial line;
- RCD or STR edge placement.

Those points remain blocked on direct HP-67 waveform/service evidence and will be introduced one verified fact at a time.

## Sources

- Tony Nixon, *Notes on HP's Classic Calculators*, HP-67 Woodstock section and key-wait waveform: https://literature.hpcalc.org/community/classic-notes.pdf
- HP Museum, *Reading HP67 ROMs* discussion and physical bus observations: https://www.hpmuseum.org/forum/thread-18327.html
- HP Museum continuation with physical startup trace: https://www.hpmuseum.org/forum/thread-18327-page-2.html
- Jacques Laporte archived hardware notes / ROM dump measurements: https://archived.hpcalc.org/laporte/ROM%20DUMP.htm
- Jacques Laporte hardware overview including Woodstock ISA waveform comparison: https://archived.hpcalc.org/laporte/HP35%20Hardware%20basic%20design.htm
- Napier bibliography entry confirming the three-part *An HP-67 Anatomy Lesson* series: PPC Journal V5N7 pp. 7-8, V5N8 pp. 14-17, V5N10 pp. 25-27.
