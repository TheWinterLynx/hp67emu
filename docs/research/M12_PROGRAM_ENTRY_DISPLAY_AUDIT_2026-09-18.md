# M12 program-entry and display-path audit — 2026-09-18

## Scope

This audit was triggered by the M12 end-to-end PROGRAM regression reaching a structural ROM0 error `UnknownDisplayCode { scan_slot: 6, code: 80 }` (`0x50`). No new ROM0 character code was accepted merely to make the regression pass. The program-entry path and display transport were rechecked against HP documentation, direct HP-67 measurements and independent microcode emulators before changing production code.

## What the failing run had already proven

The failure occurred after the M12 helper had entered `1 ENTER 2 + R/S` in PROGRAM mode. For each of those five physical keys the helper had already required:

- real firmware execution of `keys -> a`;
- the exact physical key scan code in ACT A[2:1];
- a persistent change in the modeled HP-67 RAM image; and
- return to the no-key firmware wait.

The `0x50` failure occurred later, after switching back toward RUN and entering `press_live_key_to_target()`. It is therefore not evidence that PROGRAM storage failed.

## HP and hardware evidence

The HP-29C service manual describes the same Woodstock ACT family and states that once per 56-bit word time the ACT sends ROM0 a character code over IS/IA; ROM0 decodes digits, decimal/minus, blank and the Error letters.

Tony Nixon's direct HP-67 logic-analyser measurements in *Notes on HP's Classic Calculators* place the ROM0 display byte on IS at b0..b7, LSB first. The capture explicitly identifies `$20` as **Blank** and records the HP-67 ROM0 decode set, including `$00..$0F`, `$30` and `$4x`. `$50` is not in the measured decode set.

The same hardware material shows ACT, ROM0 and the cathode driver connected through IS/ISA, STR and RCD without a separate ACT-to-ROM0 DISPLAY-enable pin. DISPLAY-off therefore cannot be modeled as permission for arbitrary A/B working-register contents to become a visible ROM0 character.

## Independent emulator cross-checks

Pinned Nonpareil `c347bc1ab20170c253512042f7aac0d952f304ea`:

- `ncd/67-97/67.ncd.tmpl` maps HP-67 RAM as four 16-register blocks, `0x00..0x3F`, and maps PRGM/RUN to CRC flag 1.
- `src/proc_woodstock.c` applies `display off` and `display toggle` directly to the ACT display-enable latch before the display scan executed later in the same cycle. With DISPLAY disabled its scan emits no visible segments.
- `src/crc.c` implements a successful CRC flag test as a pulse on ACT F2. A false result does not directly clear ACT S3.
- `ncd/67-97/67.asm` explicitly executes `0 -> s 3` before relevant CRC tests and routes a key through `display off`, `keys -> a`, program-code construction and the `insert` path.

Pinned x11-calc `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983` independently defines HP-67 `MEMORY_SIZE 64`, corroborating the `0x00..0x3F` RAM map.

Greg Sydney-Smith's HP67u microcode notes independently show the HP-67 wait loop, program RAM layout and real microcode paths, including DISPLAY OFF around key/function processing.

## Corrections made

1. The earlier M12 experiment that made a false CRC test forcibly clear ACT S3 was reverted. Source-backed behavior is: a true CRC test pulses ACT F2 and therefore sets S3; false produces no F2 pulse. Firmware owns explicit `0 -> s 3` instructions.
2. ROM0 `0x50` remains invalid. No decoder relaxation was made.
3. ACT structural display serialization now emits the measured HP-67 ROM0 blank code `$20` while `display_enable == false`, instead of serializing arbitrary live A/B nibbles.
4. The already documented power-on exception is retained: before firmware has executed its first explicit display-control instruction, the live machine permits the observed startup display traffic despite the architectural reset value of the DISPLAY latch.

## Still not claimed

This remains a structural word/bit model. The exact PHI-relative launch/sample edge for display data, the intra-word instant at which an ACT result bit commits, and exact STR/RCD overlap timing remain M14/M15 electrical-fidelity work. The M12 correction does not claim those unresolved timings.


## Follow-up after repeated cycle-4430 failure

The first DISPLAY-off correction was insufficient: the M12 regression still produced the same `slot 6 / 0x50` ROM0 error at live cycle 4430. That repeated result disproves DISPLAY-off as the complete explanation because the failing transport is occurring with the display path active.

A stronger hardware discrepancy was then found in Hewlett-Packard's HP-29C Service Manual, paragraph 2-45. The Woodstock ACT does **not** transmit four A bits followed by four B bits. HP states that it sends the four A bits for a character and then **three bits recoded from the four-bit B-register character modifier**. Therefore the current structural serializer's direct high-nibble composition from B is not source-backed and can manufacture impossible ROM0 bytes such as `0x50`.

Independent high-level Woodstock implementations agree on the role of B rather than treating it as a raw character-code high nibble: Nonpareil and NP25 decode B as display-format/punctuation state, while x11-calc's HP-67 renderer accepts only specific B formats for number, blank and decimal output.

The exact Woodstock B(4)->display(3) recoding table has not yet been established from a primary source. Production behavior is therefore not being guessed. Commit `9172d3b` adds diagnostics only: on a ROM0 decode failure it reports the executing word, post-execution PC, DISPLAY latch, selected display register index, and the selected A/B nibbles from both the pre-instruction serial snapshot and post-instruction architectural state. One rerun can then distinguish missing B recoding from an instruction-boundary timing error, or show that both are present.


## Cycle 4430 decisive trace

The diagnostic rerun produced:

```text
executing_word=Some(90)
post_pc=0152
display_enable=true
register_index=Some(11)
pre_a_b=Some((5, 0))
post_a_b=Some((0, 5))
```

Rust printed the word as decimal 90, which is octal `0132`. The selected digit changes from A/B `(5,0)` before the instruction to `(0,5)` after it. The observed bad ROM0 byte `$50` is therefore exactly the post-instruction pair serialized backward into the same word's b0..b7 display window.

This distinguishes the immediate M12 failure from the separate B-recoding gap. Regardless of the final B(4)->modifier(3) network, the same-word display source cannot be the fully committed post-instruction architectural A/B state. The existing `ActSerialStateSnapshot` already exists specifically to prevent that temporal inversion for arithmetic; M12 now extends that ownership to display A/B and DISPLAY-enable state.

A regression reproduces the exact `0132` condition: snapshot A/B `(5,0)`, mutate architectural A/B to `(0,5)`, then require the structural display transport to emit the pre-instruction value. The HP-documented B recoding remains open and is not inferred from this trace.
