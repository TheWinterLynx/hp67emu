# HP-67 power-on display transient

## Scope

This note records the evidence and implementation boundary for the visible HP-67 startup display. It is intentionally separate from the final PHI/RCD/STR electrical scheduler.

## Direct HP-67 timing evidence

Tony Nixon's *Notes on HP's Classic Calculators*, HP-67 section, reports from physical logic-analyser/scope captures:

- page 73: signals begin stabilizing roughly 330 us after switch-on; SYNC becomes active and ROM instruction fetching begins roughly 35 ms after switch-on;
- page 76: one 56-bit HP-67 instruction cycle takes about 320 us; fifteen STR pulses form a complete display refresh of about 4.8 ms;
- pages 76-77: ROM0 display data occupies IS bits 0..7 and code `0x00` decodes as digit `0`; the shared sign slot uses bits 0/1 rather than the normal seven-segment digit decoder.

Source: https://literature.hpcalc.org/community/classic-notes.pdf

Sydney Smith's HP-67 startup trace independently shows the firmware constructing `B=03000000000022` (display `0.00`) immediately before the startup path executes `display off` and `display toggle` around octal addresses `00162` and `00163`.

Source: https://www.sydneysmith.com/wordpress/1190/hp67-flags/

## Emulator behavior

Before valid firmware fetch, the production UI now presents a reset-phase frame obtained by passing display code `0x00` through the existing ROM0 decoder for every structural scan slot. This is not a formatted number string. It naturally gives zeroes in digit-capable positions and the decoder-defined state of the shared sign slot.

The reset frame is retained during the measured ~35 ms pre-SYNC interval and while firmware runs until an explicit display-control instruction establishes software ownership. Firmware is paced at the measured coarse 320 us/word interval. After display control is established, each shared structural word updates only its actual current scan slot.

## Limits

The first-valid-SYNC/ignored-SYNC detail seen in the power-on capture is not yet promoted into exact reset sequencing. Nor do these coarse durations define PHI1/PHI2 pulse widths, edge-relative IS timing, exact RCD/STR overlap, LED current decay, or analogue persistence. Those remain separate hardware-fidelity milestones.
