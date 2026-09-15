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

Teenix states that original microcode is used where available while also warning that some information elsewhere in the project required best-guess repair. Therefore Teenix remains a strong current firmware source but is independently cross-checked before becoming canonical.

## Independent firmware corpora

Use these corpora together:

- current Teenix HP-67 module, updated 10 May 2026;
- Nonpareil HP-67 disassembly and machine definition;
- x11-calc HP-67 implementation and embedded 8192-entry ROM corpus;
- Panamatik HP-67 emulator;
- Sydney Smith HP67u/HP67w analyses;
- Tony Nixon, *Notes on HP's Classic Calculators*.

The reviewed x11-calc baseline is commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`.

The current Nonpareil research baseline is commit `c347bc1ab20170c253512042f7aac0d952f304ea`. Nonpareil's HP-67 calculator definition references official `uasm` output from `67.asm`, `6797.asm` and `67b1.asm`. Current `uasm` emits Woodstock object records as optional bank mask plus octal address and octal opcode, for example `[0]0001:1743`. hp67emu now imports that upstream object format directly instead of reimplementing Nonpareil's symbolic assembler.

## Physical startup evidence

Tony Nixon published a direct capture from a physical HP-67 while monitoring SYNC and IS during switch-on. The observed startup path includes:

- address `0x000`: word `0x000` — no operation;
- address `0x001`: word `0x3e3` — conditional branch to `0x0f8` when carry is clear;
- address `0x0f8`: word `0x11a` — `0 -> c[w]`.

Both the reconstructed Teenix 2026 corpus and the pinned x11-calc corpus pass all three checkpoints exactly:

```text
bank 0 pc 0x000 = 0x000 (0000 octal)
bank 0 pc 0x001 = 0x3e3 (1743 octal)
bank 0 pc 0x0f8 = 0x11a (0432 octal)
```

This is currently the strongest direct firmware checkpoint because it connects decoded source, reconstructed 10-bit words and real-hardware execution. The same physical-reader discussion records a later call from `0x0068` into the `1818-0232` ROM at `0x0fc6`, which remains a future instruction-flow trace target.

## Teenix 2026 ↔ x11-calc complete comparison

The comparison used:

- Teenix `cal67.pfl` SHA-256 `C8563E7982F38DEE9B6004B1194727AA2942DEB72D327C4D2F008CFDD057804B`;
- x11-calc `src/x11-calc-67.c` SHA-256 `3EBB5E8C014ACAC9013CC1D0CBC96734EC17BC6D6903E9D988B5AFC89B919B43`;
- x11-calc commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`.

The first run exposed a local hp67emu assembler-table swap between `0o1110 = down rotate` and `0o1310 = c -> stack`: 5091 words matched and 29 differed solely by that pair. Teenix's documented table and x11-calc's decoder agreed on the correct mapping, so hp67emu was fixed and regressions were added.

The clean rerun produced:

```text
matching subset words : 5120
value mismatches      : 0
missing in reference  : 0
reference-only words  : 3072 (informational)

SUBSET MATCH: all 5120 populated subset words exist and match exactly.
```

Thus every physically populated HP-67 microcode location reconstructed from current Teenix agrees bit-for-bit with the pinned x11-calc image. The 3072 x11-only locations lie outside the Teenix physical-population subset and are informational. Full provenance and the diagnostic first-run mismatch set are preserved in `docs/research/TEENIX_X11_COMPARISON_2026-09-15.md`.

## Normalized corpus tooling

`src/research/rom_corpus.rs` and `src/bin/rom_compare.rs` normalize sources to explicit `(bank, pc, 10-bit word)` coordinates. x11-calc extraction requires all 8192 words, sparse corpora can be merged only when overlaps agree, and exact differences are reported by bank/page/PC.

`src/research/teenix.rs` validates/removes the Teenix XOR envelope. `src/research/woodstock_asm.rs` assembles only recognized/documented Woodstock mnemonics. `src/research/teenix_hp67.rs` handles the proven current Teenix HP-67 listing grammar and physical population map.

`src/research/nonpareil_obj.rs` parses the object format produced by official Nonpareil `uasm`; `src/bin/nonpareil_rom.rs` merges those objects into the same normalized corpus. `docs/research/compare_nonpareil.ps1` pins the Nonpareil source revision, records source/object hashes, invokes local official `uasm`, verifies physical startup and performs Teenix/Nonpareil plus optional x11/Nonpareil comparisons.

`src/bin/rom_subset.rs` performs directional comparisons when one corpus has narrower physical coverage. The complete procedure is in `docs/ROM_CORPUS_WORKFLOW.md`.

## HP-67 / HP-97 relationship

Tony Nixon's notes record that HP-67 and HP-97 microcode is identical from ROM address `$400` through `$FFF`. The HP-97 Service Manual is therefore a useful corroborating source, but this does not imply that every electrical detail of the two calculators is identical.

## Acceptance procedure for a canonical corpus

1. Preserve downloaded archives unchanged and record SHA-256, URL and acquisition date.
2. Treat current `HP67.zip` as the latest Teenix HP-67 module.
3. Decode its `.pfl` envelope and require the listing to parse to exactly 5120 understood words with zero unknowns/address errors.
4. Require Teenix to pass the direct physical startup checkpoints.
5. Independently normalize x11-calc at the reviewed commit — completed.
6. Require all 5120 Teenix words to agree with x11-calc — completed, exact match.
7. Assemble Nonpareil `67.asm`, `6797.asm` and `67b1.asm` with official `uasm`, normalize the object output and compare it against Teenix/x11-calc.
8. Require every normalized candidate to pass the direct physical startup checkpoints.
9. Treat the historical 2022 ROM-reader claim separately from the current archive contents.
10. If a physical HP-67 dump is recovered, map each image to part number/bank/address and use it as the strongest firmware provenance source.
11. Cross-check the shared HP-67/97 region where independent HP-97 data is available.
12. Record hashes/mappings/reports; do not commit copyrighted ROM bytes until redistribution is reviewed.

## Repository policy

The emulator core must support externally supplied ROM images. Public availability does not automatically grant redistribution rights. Raw HP ROM bytes are not embedded in the repository until licensing/redistribution is reviewed.

## First microcode milestone

The first execution target remains:

`power/reset -> PHI1/PHI2 -> ACT address activity -> ROM response on ISA -> ACT captures a 10-bit instruction -> next 56-bit cycle`

Unknown or unverified microinstructions must stop with PC/word/tick diagnostics rather than silently using guessed behaviour.
