# HP-67 microcode provenance and verification

Research snapshot: 2026-09-15.

## Separate Teenix sources correctly

There are two different Teenix source lines and they must not be conflated:

- historical physical ROM-reader project: https://www.teenix.org/ROMreader.zip
- current HP-67 emulator module: https://www.teenix.org/HP67.zip
- current MultiCalc host package: https://www.teenix.org/MultiCalc.zip
- physical-reader provenance discussion: https://www.hpmuseum.org/forum/thread-18327-page-2.html

The Teenix web page marks the HP-67 emulator module as updated **10 May 2026** and MultiCalc as updated **11 May 2026**. The locally downloaded HP-67 module is therefore a current 2026 artifact, separate from the 2022 ROM-reader project.

The 2022 HP Museum post separately states that the then-published `ROMreader.zip` contained ROM files for HP-97 and HP-67 obtained with the physical reader. The archive currently served at that URL appears to have different contents, so the historical physical-dump claim remains separate evidence.

## Current Teenix HP-67 module

Observed current files:

- `cal67.pfl`: 92866 raw bytes;
- `cal6713.pfl`: 92893 raw bytes;
- `cal67b.pfl`: 87845 raw bytes.

Their outer format is proven. Every byte is XORed with `0x55`; after decoding, the first line is `NeWe`, the second line is the decimal byte count of the following text payload. Observed decoded headers are:

```text
cal67.pfl   -> NeWe\r92855\r...
cal6713.pfl -> NeWe\r92882\r...
cal67b.pfl  -> NeWe\r87834\r...
```

The decoded `cal67.pfl` payload is a source-style Woodstock microcode listing, not a flat binary ROM. Its required grammar is established:

- blank lines do not consume ROM;
- `// ...` lines are comments;
- `org $1400` switches to physical bank 1 at logical PC `0x400`;
- `Lxxxx:` and `Hxxxx:` are hexadecimal address anchors;
- every remaining non-empty record is one Woodstock microinstruction.

The listing contains 5127 decoded text lines but exactly **5120 microinstructions**: all 4096 bank-0 locations plus bank-1 PC `0x400..0x7ff`. The strict analyzer reports:

```text
candidate instructions: 5120
unknown instructions  : 0
label mismatches      : 0
overflow instructions : 0
```

`extract-teenix-hp67` reconstructs exactly 5120 explicit 10-bit `(bank, pc, word)` entries without guessing or silently discarding text.

## Independent firmware corpora

The three fully normalized corpora now used as the software baseline are:

- current Teenix HP-67 module, updated 10 May 2026;
- x11-calc at commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`;
- Nonpareil at commit `c347bc1ab20170c253512042f7aac0d952f304ea`, assembled with its own official `uasm` from `67.asm`, `6797.asm` and `67b1.asm`.

Nonpareil is deliberately assembled with the upstream microassembler rather than with hp67emu's Teenix parser, so that agreement does not depend on one local mnemonic table.

## Direct physical firmware evidence

Tony Nixon published a direct physical HP-67 capture of the startup sequence while monitoring SYNC and IS. The observed path includes:

- address `0x000`: word `0x000` — no operation;
- address `0x001`: word `0x3e3` — conditional branch to `0x0f8` when carry is clear;
- address `0x0f8`: word `0x11a` — `0 -> c[w]`.

All three normalized software corpora reproduce those words exactly.

The same physical investigation records a later call from `0x0068` into the `1818-0232` ROM at `0x0fc6`. The reconciled firmware contains:

```text
0x0067 : delayed select rom 15  -> 0x3f4 (1764 octal)
0x0068 : jsb $00c6              -> 0x319 (1431 octal)
```

`tests/physical_startup_flow.rs` now executes those two words through the independent Rust Woodstock reference model and requires return address `0x0069` plus target `0x0fc6`, matching the reported physical control flow.

## Complete triple-corpus result

The exact 2026 comparison result is:

```text
Teenix 2026  ↔ x11-calc   : 5120 / 5120 identical
Teenix 2026  ↔ Nonpareil  : 5120 / 5120 identical
Nonpareil    ↔ x11-calc   : 5120 / 5120 identical

value mismatches : 0
missing words    : 0
```

The x11-calc image contains 3072 additional logical-array locations outside the physically populated Teenix/Nonpareil HP-67 subset; those positions are informational and are not treated as firmware mismatches.

The first Teenix ↔ x11-calc run exposed a local hp67emu assembler-table swap between `0o1110 = down rotate` and `0o1310 = c -> stack`: 5091 words matched and 29 differed solely by that pair. Teenix's documented table and x11-calc's decoder agreed on the correct mapping, hp67emu was fixed, and the clean rerun reached 5120/5120. The diagnostic first-run mismatch set remains preserved in `docs/research/TEENIX_X11_COMPARISON_2026-09-15.md`.

The complete three-source hashes and final comparison are preserved in `docs/research/HP67_TRIPLE_CORPUS_COMPARISON_2026-09-15.md`.

## Normalized corpus tooling

`src/research/rom_corpus.rs` and `src/bin/rom_compare.rs` normalize sources to explicit `(bank, pc, 10-bit word)` coordinates. `src/bin/rom_subset.rs` performs directional comparisons when one corpus has narrower physical coverage.

`src/research/teenix.rs` validates/removes the Teenix XOR envelope. `src/research/woodstock_asm.rs` assembles only recognized/documented Woodstock mnemonics. `src/research/teenix_hp67.rs` handles the proven Teenix listing grammar.

`src/research/nonpareil_obj.rs` parses official Nonpareil `uasm` object records, `src/bin/nonpareil_rom.rs` merges them, and `docs/research/compare_nonpareil.ps1` pins/builds/validates upstream `uasm` and performs the cross-comparisons.

## Timing/electrical boundary after firmware closure

Firmware identity is now strong enough that remaining startup failures should no longer be explained away as “possibly the wrong ROM image.” The next fidelity work therefore moves to serial timing and chip interaction.

`src/machines/hp67/timing.rs` defines only the verified structural geometry of the HP-67 serial machine word: fourteen 4-bit digit times, 56 serial bit times, canonical coordinates `b0..b55`, and deterministic word wrapping. It intentionally does not assign unverified ISA/DATA/SYNC bit windows or physical sampling edges.

The evidence boundary and currently unresolved timing details are recorded in `docs/research/HP67_WORD_TIMING_EVIDENCE_2026-09-15.md` and `docs/HARDWARE_SOURCES.md`.

## HP-67 / HP-97 relationship

Tony Nixon's notes record that HP-67 and HP-97 microcode is identical from ROM address `$400` through `$FFF`. The HP-97 Service Manual remains useful corroboration, but this does not imply that every electrical detail of the two calculators is identical.

## Canonical-corpus status

For software-corpus purposes, the physically populated 5120-word HP-67 image is now a **provisional canonical baseline** because three independently represented sources agree bit-for-bit and all available direct physical firmware checkpoints agree with them.

It remains provisional rather than final physical provenance because the historical/raw physical ROM-reader dump has not yet been recovered and mapped to chip part numbers. If such a dump is recovered, it becomes the strongest corpus and must be compared against the current baseline.

## Repository policy

The emulator core must support externally supplied ROM images. Public availability does not automatically grant redistribution rights. Raw HP ROM bytes are not embedded in the repository until licensing/redistribution is reviewed.

## First electrical microcode milestone

The first low-level execution target is now:

`power/reset -> PHI1/PHI2 -> b0..b55 word timing -> ACT address activity -> ROM response on ISA -> ACT captures one 10-bit instruction -> next 56-bit word`

Exact ISA/DATA/SYNC windows, pulse widths and sampling edges must come from HP-67-specific waveform/service evidence. Unknown electrical behavior remains explicit rather than being filled with plausible Woodstock-family values.
