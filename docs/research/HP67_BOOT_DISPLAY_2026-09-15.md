# HP-67 boot display convergence — 2026-09-15

## Purpose

Record the evidence used to turn the real-microcode power-on idle checkpoint into a structural display regression without pretending that the instruction-boundary ACT model is already the final bit-serial 1820-2530 implementation.

## Real-firmware checkpoint

Running `hp67_poweron_smoke` with the normalized current Teenix HP-67 corpus reaches the documented no-key wait loop after 281 executed machine cycles with:

- `A = 0000FFFFFFF100`
- `B = 03000000000022`
- `C = 00000000000000`
- `display_14_digit = true`
- `display_enable = true`

The independent HP-67 microcode trace published by Sydney Smith reaches the same display-formatting routine, records `B=03000000000022`, and explicitly notes that the display is then `0.00`:

- https://www.sydneysmith.com/wordpress/1190/hp67-flags/

This is an architectural-state cross-check, not an electrical timing source.

## Direct HP-67 display evidence

Tony Nixon's *Notes on HP's Classic Calculators* contains HP-67-specific logic-analyser observations on pages 67 and 75-77:

- the ACT/ROM0 display data is carried on IS during word bits `b0..b7`, LSB first;
- ROM0 `1818-0268` decodes that byte and supplies STR;
- ACT supplies RCD to the `1820-1749` cathode driver;
- the observed 15-slot order is exponent units, exponent tens, shared signs, mantissa digits 11 down to 1, then a duplicate exponent-units slot;
- the first/last slot while displaying `0.00` is measured as display code `0x20` (blank);
- `0x30` is the decimal-point code;
- `0x00..0x09` are numeric digits, `0x0f` is blank, and the shared-sign slot uses the first two display-code bits.

Source:

- https://literature.hpcalc.org/community/classic-notes.pdf

The same document describes the 14-nibble numerical register order from the least-significant end as exponent units, exponent tens, sign, then mantissa digits 1 through 11.

## Bring-up composition

The instruction-boundary bridge in `src/machines/hp67/display_snapshot.rs` maps the observed physical scan slots to the architectural register indices:

| Scan slot | Role | ACT register index |
| --- | --- | ---: |
| 1 | exponent units | 0 |
| 2 | exponent tens | 1 |
| 3 | shared signs | 2 |
| 4 | mantissa digit 11 | 13 |
| ... | ... | ... |
| 14 | mantissa digit 1 | 3 |
| 15 | exponent units duplicate | 0 |

For this bring-up bridge only, the already-latched A/B nibbles are composed as:

`display_code = (B[n] << 4) | A[n]`

That composition is kept outside the future electrical ACT device. It is justified here as a diagnostic bridge because it simultaneously reproduces the independently observed power-on state and the direct HP-67 display captures:

- slot 1: `A[0]=0`, `B[0]=2` -> `0x20`, matching the measured blank exponent-units code;
- slot 2: `A[1]=0`, `B[1]=2` -> `0x20`;
- slot 3: `A[2]=1`, `B[2]=0` -> `0x01`, which means positive mantissa and positive exponent in the measured sign decode;
- slot 4: `A[13]=0`, `B[13]=0` -> `0x00`, digit zero;
- slot 5: `A[12]=0`, `B[12]=3` -> `0x30`, the measured decimal-point code;
- slots 6-7 -> `0x00`, two more zero digits;
- the remaining unused mantissa positions contain `A=0xf`, `B=0` -> `0x0f`, blank;
- slot 15 repeats slot 1 -> `0x20`.

The complete expected boot scan is therefore:

`20 20 01 00 30 00 00 0F 0F 0F 0F 0F 0F 0F 20`

After ROM0 decode the only visible outputs are zero, decimal point, zero, zero: `0.00`.

## What this proves

The regression now joins four independently meaningful layers:

1. real HP-67 microcode reaches the documented idle loop;
2. that firmware produces the independently corroborated A/B display state;
3. the bring-up bridge serializes each display byte over the resolved weak-low/active-high IS model at `b0..b7`;
4. the source-backed ROM0 decoder and 15-slot cathode scan yield the expected `0.00` pattern.

## What this does not prove

This is **not** yet the final electrical display implementation. In particular it does not claim:

- exact PHI1/PHI2 launch or sample edges for the ACT display bits;
- exact STR pulse edge/width or propagation delay;
- exact RCD/STR overlap ordering at slots 15/1;
- transistor-level sign routing;
- LED current, decay or brightness integration;
- that production ACT code may compose whole A/B nibbles in one host operation.

The production 1820-2530 must eventually generate the same `b0..b7` stream from its bit-serial state and the physical ROM0/cathode devices must consume it on measured edges. The bridge is a regression oracle for that later implementation, not a shortcut around it.
