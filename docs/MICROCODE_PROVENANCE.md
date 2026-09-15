# HP-67 microcode provenance and verification

Research snapshot: 2026-09-15.

## Preferred evidence path

Tony Nixon's Teenix ROM-reader project remains the strongest provenance lead because the 2022 HP Museum announcement explicitly says the published `ROMreader.zip` contained ROM files for both HP-97 and HP-67 read with the physical ROM-reader project:

- historical project URL: https://www.teenix.org/ROMreader.zip
- provenance discussion: https://www.hpmuseum.org/forum/thread-18327-page-2.html

However, the **currently downloaded archive inspected on 2026-09-15 does not obviously match the 2022 contents described in that post**. A filename search over the extracted archive for `67`, `1818` or `rom` returned only:

- `ROM Reader Help.pdf` — 826258 bytes;
- `ROMread..hex` — 7134 bytes, apparently reader-controller firmware rather than an HP-67 ROM image.

Therefore the project must not claim that the current Teenix download contains the HP-67 ROM dumps until a complete archive inventory proves otherwise. The likely explanations are that the archive changed after 2022 or that any ROM payload uses unexpected filenames. Historical provenance remains useful, but current-file contents must be verified independently.

The separate Teenix HP-67 emulator-module files `cal67.pfl`, `cal67b.pfl` and `cal6713.pfl` are structured binary files of roughly 88-93 KiB, not simple flat 8192-word ROM images. Their format remains unproven and is not used as a firmware authority.

## Independent firmware corpora

Until a physical-reader dump is actually recovered, use at least two independent software corpora and the physical startup trace together:

- Nonpareil HP-67 disassembly and machine definition: https://github.com/brouhaha/nonpareil
- x11-calc HP-67 implementation and embedded 8192-entry ROM corpus: https://github.com/mike632t/x11-calc
- Teenix HP-67 emulator module: https://www.teenix.org/HP67.zip
- Panamatik HP-67 emulator: https://www.panamatik.de/html/hp-67.html
- Sydney Smith HP67u/HP67w and address-level analyses: https://www.sydneysmith.com/wordpress/hp67-main/
- Tony Nixon, *Notes on HP's Classic Calculators*: https://literature.hpcalc.org/community/classic-notes.pdf

x11-calc's first two words are octal `00000, 01743`, matching Nonpareil's symbolic reset entry. Agreement between independent emulators is useful evidence, but neither becomes an electrical-timing authority.

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
2. Inventory **all** files in the current `ROMreader.zip`, not just names containing HP-67 keywords.
3. Determine whether the current archive actually contains calculator ROM data or only reader hardware/software.
4. If it does not contain ROM data, locate an archived 2022 copy or another physical-reader export with traceable provenance.
5. Independently normalize x11-calc and Nonpareil.
6. Require both software corpora to pass the physical startup checkpoints at `0x000`, `0x001` and `0x0f8`.
7. Compare x11-calc ↔ Nonpareil word-for-word and explain every mismatch.
8. When a physical dump is recovered, map each image to physical part number, bank and address range, then compare it against both software corpora.
9. Cross-check the shared HP-67/97 region where independent HP-97 data is available.
10. Record verified hashes/mappings/reports in the repository; do not commit copyrighted ROM bytes until redistribution is reviewed separately.

## Repository policy

The emulator core must support externally supplied ROM images. Public availability does not automatically grant redistribution rights. Raw HP ROM bytes are therefore not embedded in the repository until licensing/redistribution is reviewed.

## First microcode milestone

The first execution target remains:

`power/reset -> PHI1/PHI2 -> ACT address activity -> ROM response on ISA -> ACT captures a 10-bit instruction -> next 56-bit cycle`

Unknown or unverified microinstructions must stop with PC/word/tick diagnostics rather than silently using guessed behaviour.
