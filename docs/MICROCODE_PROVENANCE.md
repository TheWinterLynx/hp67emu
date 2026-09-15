# HP-67 microcode provenance and verification

Research snapshot: 2026-09-15.

## Separate Teenix sources correctly

There are two different Teenix source lines and they must not be conflated:

- historical physical ROM-reader project: https://www.teenix.org/ROMreader.zip
- current HP-67 emulator module: https://www.teenix.org/HP67.zip
- current MultiCalc host package: https://www.teenix.org/MultiCalc.zip
- physical-reader provenance discussion: https://www.hpmuseum.org/forum/thread-18327-page-2.html

The Teenix web page currently marks the **HP-67 emulator module as updated 10 May 2026** (`Updated 100526`) and **MultiCalc as updated 11 May 2026**. Therefore the locally downloaded HP-67 module is a current 2026 artifact, not a stale legacy package from the 2022 ROM-reader work.

The 2022 HP Museum post separately states that the then-published `ROMreader.zip` contained ROM files for both HP-97 and HP-67 obtained with the physical ROM-reader project. The archive currently served at that URL appears to have different contents, so the 2022 physical-dump statement must be treated independently from the present archive.

## Current HP-67 module status

The locally observed current-module files are:

- `cal67.pfl`: 92866 raw bytes;
- `cal6713.pfl`: 92893 raw bytes;
- `cal67b.pfl`: 87845 raw bytes.

Their outer format is proven. Every byte is XORed with `0x55`; after decoding, the first line is `NeWe`, the second line is the decimal byte count of the following text payload. The three current HP-67 `.pfl` files exhibit exactly that structure.

Observed decoded headers are:

```text
cal67.pfl   -> NeWe\r92855\r...
cal6713.pfl -> NeWe\r92882\r...
cal67b.pfl  -> NeWe\r87834\r...
```

The decoded `cal67.pfl` payload is not a flat binary ROM image. It is a source-style Woodstock microcode listing. For the current file the grammar needed for exact reconstruction is now established:

- blank lines do not consume ROM;
- `// ...` lines are comments;
- `org $1400` switches to physical bank 1 at logical PC `0x400`;
- both `Lxxxx:` and `Hxxxx:` prefixes are hexadecimal address anchors;
- every remaining non-empty record is one Woodstock microinstruction.

The complete current listing contains 5127 decoded text lines but exactly **5120 microinstructions**: all 4096 bank-0 locations plus bank-1 PC `0x400..0x7ff`. The strict analyzer reports:

```text
candidate instructions: 5120
unknown instructions  : 0
label mismatches      : 0
overflow instructions : 0
```

`extract-teenix-hp67` therefore reconstructs exactly 5120 explicit 10-bit `(bank, pc, word)` entries from the current Teenix source without guessing or silently discarding text.

The current Teenix page says that where available calculators run original microcode, while also warning that some microcode information across the project was unreliable or unavailable and was repaired with best guesses. Therefore the 2026 HP-67 module is a strong current firmware reference, but its reconstructed words still require independent comparison before being treated as canonical physical firmware.

## Independent firmware corpora

Use the following corpora together:

- current Teenix HP-67 module, updated 10 May 2026: https://www.teenix.org/HP67.zip
- Nonpareil HP-67 disassembly and machine definition: https://github.com/brouhaha/nonpareil
- x11-calc HP-67 implementation and embedded 8192-entry ROM corpus: https://github.com/mike632t/x11-calc
- Panamatik HP-67 emulator: https://www.panamatik.de/html/hp-67.html
- Sydney Smith HP67u/HP67w and address-level analyses: https://www.sydneysmith.com/wordpress/hp67-main/
- Tony Nixon, *Notes on HP's Classic Calculators*: https://literature.hpcalc.org/community/classic-notes.pdf

The reviewed x11-calc baseline is commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`. Its first two words are octal `00000, 01743`, matching Nonpareil's symbolic reset entry. Agreement between independent emulators is useful evidence, but none of them becomes an electrical-timing authority merely by agreeing.

## Physical startup evidence

Tony Nixon published a direct capture from a physical HP-67 while monitoring SYNC and IS during switch-on. The first observed execution path includes:

- address `0x000`: word `0x000` — no operation;
- address `0x001`: word `0x3e3` — conditional branch to `0x0f8` when carry is clear;
- address `0x0f8`: word `0x11a` — `0 -> c[w]`.

The reconstructed Teenix 2026 corpus passes all three checkpoints exactly:

```text
bank 0 pc 0x000 = 0x000 (0000 octal)
bank 0 pc 0x001 = 0x3e3 (1743 octal)
bank 0 pc 0x0f8 = 0x11a (0432 octal)
```

This is currently our strongest direct firmware checkpoint because it ties decoded source, reconstructed words and observed real-hardware execution together. The same physical-reader discussion records a later call from `0x0068` into the `1818-0232` ROM at `0x0fc6`, which remains a future instruction-flow trace target.

## Normalized corpus tooling

`src/research/rom_corpus.rs` and `src/bin/rom_compare.rs` normalize sources to explicit `(bank, pc, 10-bit word)` coordinates. The x11-calc extractor requires all 8192 words, sparse address/opcode listings can be imported with an explicit radix, sparse corpora can be merged only when overlaps agree, and every mismatch is reported by bank/page/PC.

`src/research/teenix.rs` removes and validates the Teenix XOR/text envelope. `src/research/woodstock_asm.rs` assembles only documented/recognized Woodstock mnemonics, and `src/research/teenix_hp67.rs` handles the proven current HP-67 listing grammar and physical population map.

Because Teenix has 5120 physically populated words while x11-calc exposes a complete 8192-entry two-bank logical array, `src/bin/rom_subset.rs` performs a directional comparison: every Teenix word must exist and match in the broader reference, while reference-only addresses remain informational. `docs/research/compare_teenix_x11.ps1` automates this check against the pinned x11-calc baseline and records local SHA-256 provenance.

The complete procedure is documented in `docs/ROM_CORPUS_WORKFLOW.md`.

## HP-67 / HP-97 relationship

Tony Nixon's notes record that HP-67 and HP-97 microcode is identical from ROM address `$400` through `$FFF`. The HP-97 Service Manual is therefore a useful corroborating source, but this does not imply that every electrical detail of the two calculators is identical:

- https://literature.hpcalc.org/community/hp97-sm-en.pdf

## Acceptance procedure for a canonical corpus

1. Preserve every downloaded archive unchanged and record SHA-256, URL and acquisition date.
2. Treat current `HP67.zip` (10 May 2026) as the latest Teenix HP-67 module.
3. Decode its `.pfl` outer `NeWe` container with the tested XOR-`0x55` decoder and preserve the exact plaintext payload locally.
4. Require the current `cal67.pfl` listing to parse to exactly 5120 understood microinstructions with zero unknowns, zero address-anchor mismatches and zero unsupported locations.
5. Require the reconstructed Teenix corpus to pass the physical startup checkpoints at `0x000`, `0x001` and `0x0f8`.
6. Independently normalize x11-calc at the reviewed commit and Nonpareil.
7. Compare Teenix against broader corpora directionally over all 5120 physically populated words; any missing or differing word is a hard failure.
8. Compare x11-calc and Nonpareil symmetrically where their population maps are intended to match.
9. Treat the historical 2022 ROM-reader dump statement separately from the current `ROMreader.zip` contents.
10. If a historical or newer physical HP-67 dump is recovered, map each image to physical part number, bank and address range and use that as the strongest firmware provenance source.
11. Cross-check the shared HP-67/97 region where independent HP-97 data is available.
12. Record verified hashes/mappings/reports in the repository; do not commit copyrighted ROM bytes until redistribution is reviewed separately.

## Repository policy

The emulator core must support externally supplied ROM images. Public availability does not automatically grant redistribution rights. Raw HP ROM bytes are therefore not embedded in the repository until licensing/redistribution is reviewed.

## First microcode milestone

The first execution target remains:

`power/reset -> PHI1/PHI2 -> ACT address activity -> ROM response on ISA -> ACT captures a 10-bit instruction -> next 56-bit cycle`

Unknown or unverified microinstructions must stop with PC/word/tick diagnostics rather than silently using guessed behaviour.
