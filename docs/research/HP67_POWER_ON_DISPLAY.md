# HP-67 power-on display transient

## Scope

This note records the evidence and implementation boundary for the visible HP-67 startup display. It is intentionally separate from the final PHI/RCD/STR electrical scheduler. No startup digit pattern is injected into the renderer or ROM0 decoder.

## Direct HP-67 timing evidence

Tony Nixon's *Notes on HP's Classic Calculators*, HP-67 section, reports from physical logic-analyser/scope captures:

- page 73: signals begin stabilizing roughly 330 us after switch-on; SYNC becomes active and ROM instruction fetching begins roughly 35 ms after switch-on;
- page 73: while reset is active, the IS bus follows PHI2 and SYNC is high; this is not enough evidence to choose a ROM0 sampling edge or fabricate a pre-SYNC display byte;
- page 76: one 56-bit HP-67 instruction cycle takes about 320 us; fifteen STR pulses form a complete display refresh of about 4.8 ms;
- pages 76-77: ROM0 display data occupies IS bits 0..7 and code `0x00` decodes as digit `0`; the shared sign slot uses bits 0/1 rather than the normal seven-segment digit decoder.

Source: https://literature.hpcalc.org/community/classic-notes.pdf

Sydney Smith's HP-67 startup trace independently shows the firmware constructing `B=03000000000022` (display `0.00`) immediately before the startup path executes `display off` and `display toggle` around octal addresses `00162` and `00163`.

Source: https://www.sydneysmith.com/wordpress/1190/hp67-flags/

The HP-97 service manual is useful corroboration for the same ACT generation: its power-on preset circuit exists specifically to reset the ACT into a defined logic state. It does not, however, document a literal startup display byte, so no such byte is assumed here.

## Emulator behavior

`ResetHold` now starts with a blank `HardwareDisplayFrame`. During the measured pre-SYNC interval the emulator does not claim a display pattern because the currently reviewed evidence does not fix the PHI-relative ROM0 sampling behavior while IS follows PHI2.

Once real firmware words begin, every visible slot is produced only by the normal structural path: current ACT A/B state -> `display_byte_from_act_registers()` -> resolved IS b0..b7 -> ROM0 decode -> current cathode slot. There is no `RESET_DISPLAY_CODE`, no prebuilt zero frame and no timer that changes the display to zeroes. The existing architectural reset state has A=B=0, so its first transported display bytes are naturally `0x00`; if the real startup zero row appears, it therefore comes from machine state and the normal decoder.

The architectural `display_enable` boolean is a semantic instruction-boundary convenience and its reset value is not accepted as evidence of the physical pre-initialization output gate. Until firmware executes its first explicit display-control instruction, structural display words are therefore allowed to reach ROM0. After the first `display off` or `display toggle`, the firmware-controlled `display_enable` state is honored normally. This models the observed fact that the real display emits before software has completed display initialization without hardcoding what it emits.

## Fidelity rule

If future pin-accurate ACT/ROM0 work causes the startup zero row to disappear, do not restore it with a special-case frame. Investigate the missing reset, serializer, SYNC, RCD/STR or display-gate behavior and update the electrical model only when supported by reviewed evidence.

## Limits

The first-valid-SYNC/ignored-SYNC detail seen in the power-on capture is not yet promoted into exact reset sequencing. Nor do the coarse durations define PHI1/PHI2 pulse widths, edge-relative IS timing, exact RCD/STR overlap, LED current decay, analogue persistence or the physical reset state of an internal display-enable latch. Those remain separate hardware-fidelity milestones.
