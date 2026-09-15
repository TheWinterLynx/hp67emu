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

The locally observed current-module files `cal67.pfl`, `cal67b.pfl` and `cal6713.pfl` are structured binary files of roughly 88-93 KiB. They are not flat 8192-word ROM images, but because they come from the HP-67 module updated on 10 May 2026 they are now a **high-priority Teenix corpus to decode**, not something to defer as obsolete.

The current Teenix page says that where available calculators run original microcode, while also warning that some microcode information across the project was unreliable or unavailable and was repaired with best guesses. Therefore the 2026 HP-67 module is a strong current semantic/microcode reference, but its contents still require independent comparison before being treated as canonical physical firmware.

## Independent firmware corpora

Use the following corpora together:

- current Teenix HP-67 module, updated 10 May 2026: https://www.teenix.org/HP67.zip
- Nonpareil HP-67 disassembly and machine definition: https://github.com/brouhaha/nonpareil
- x11-calc HP-67 implementation and embedded 8192-entry ROM corpus: https://github.com/mike632t/x11-calc
- Panamatik HP-67 emulator: https://www.panamatik.de/html/hp-67.html
- Sydney Smith HP67u/HP67w and address-level analyses: https://www.sydneysmith.com/wordpress/hp67-main/
- Tony Nixon, *Notes on HP's Classic Calculators*: https://literature.hpcalc.org/community/classic-notes.pdf

x11-calc's first two words are octal `00000, 01743`, matching Nonpareil's symbolic reset entry. Agreement between independent emulators is useful evidence, but none of them becomes an electrical-timing authority merely by agreeing.

## Physical startup evidence

Tony Nixon published a direct capture from a physical HP-67 while monitoring SYNC and IS during switch-on. The first observed execution path includes:

- address `0x000`: word `0x000` — no operation;
- address `0x001`: word `0x3e3` — conditional branch to `0x0f8` when carry is clear;
- address `0x0f8`: word `0x11a` — `0 -> c[w]`.

This is currently our strongest direct firmware checkpoint because it ties decoded words and control flow to a real HP-67. `rom_compare verify-startup` checks normalized corpora against those three points. The same discussion records a later call from `0x0068` into the `1818-0232` ROM at `0x0fc6`, which is a future instruction-flow trace target.

## Normalized corpus tooling

`src/research/rom_corpus.rs` and `src/bin/rom_compare.rs` normalize sources to explicit `(bank, pc, 10-bit word)` coordinates. The x11-calc extractor requires all 8192 words, sparse address/opcode listings can be imported with an explicit radix, sparse corpora can be merged only when overlaps agree, and every mismatch is reported by bank/page/PC.

The complete procedure is documented in `docs/ROM_CORPUS_WORKFLOW.md`.

## HP-67 / HP-97 relationship

Tony Nixon's notes record that HP-67 and HP-97 microcode is identical from ROM address `$400` through `$FFF`. The HP-97 Service Manual is therefore a useful corroborating source, but this does not imply that every electrical detail of the two calculators is identical:

- https://literature.hpcalc.org/community/hp97-sm-en.pdf

## Acceptance procedure for a canonical corpus

1. Preserve every downloaded archive unchanged and record SHA-256, URL and acquisition date.
2. Treat current `HP67.zip` (10 May 2026) as the latest Teenix HP-67 module and decode its `.pfl` structure from Teenix tooling/documentation or observable module behavior, not guesswork.
3. Treat the historical 2022 ROM-reader dump statement separately from the current `ROMreader.zip` contents.
4. Independently normalize x11-calc and Nonpareil.
5. Require every normalized corpus to pass the physical startup checkpoints at `0x000`, `0x001` and `0x0f8`.
6. Compare current Teenix HP-67 module ↔ x11-calc ↔ Nonpareil word-for-word once `.pfl` decoding is proven.
7. If a historical or newer physical HP-67 dump is recovered, map each image to physical part number, bank and address range and use that as the strongest firmware provenance source.
8. Cross-check the shared HP-67/97 region where independent HP-97 data is available.
9. Record verified hashes/mappings/reports in the repository; do not commit copyrighted ROM bytes until redistribution is reviewed separately.

## Repository policy

The emulator core must support externally supplied ROM images. Public availability does not automatically grant redistribution rights. Raw HP ROM bytes are therefore not embedded in the repository until licensing/redistribution is reviewed.

## First microcode milestone

The first execution target remains:

`power/reset -> PHI1/PHI2 -> ACT address activity -> ROM response on ISA -> ACT captures a 10-bit instruction -> next 56-bit cycle`

Unknown or unverified microinstructions must stop with PC/word/tick diagnostics rather than silently using guessed behaviour.
