# HP-67 microcode provenance and verification

Research snapshot: 2026-09-15.

## Preferred source

The current preferred starting point for the original HP-67 microcode is Tony Nixon's Teenix ROM-reader archive:

- https://www.teenix.org/ROMreader.zip
- provenance discussion: https://www.hpmuseum.org/forum/thread-18327-page-2.html

In that discussion Nixon states that the archive contains ROM files for both HP-97 and HP-67 produced by his physical ROM-reader project. That makes it more valuable to this project than a listing transcribed from an emulator or reconstructed from program behaviour.

The locally observed HP-67 Teenix files `cal67.pfl`, `cal67b.pfl` and `cal6713.pfl` are binary module/container files of roughly 88-93 KiB, not a simple 8192-word raw ROM image. Their first bytes also show a structured common header. Their internal record/word layout is not yet proven, so the project deliberately does not guess endian, packing, addresses or bank assignment. These files remain useful emulator artefacts, but they are no longer treated as the preferred raw-ROM path while `ROMreader.zip` exists.

## Independent cross-checks

Use at least two independent references before accepting our interpretation of a raw word or opcode family:

- Teenix HP-67 emulator module: https://www.teenix.org/HP67.zip
- Nonpareil HP-67 disassembly and machine definition: https://github.com/brouhaha/nonpareil
- x11-calc HP-67 implementation and embedded ROM corpus: https://github.com/mike632t/x11-calc
- Panamatik HP-67 emulator, documented as running original HP-67 microcode: https://www.panamatik.de/html/hp-67.html
- Sydney Smith HP67u/HP67w microcode emulators and address-level analyses: https://www.sydneysmith.com/wordpress/hp67-main/
- Sydney Smith article index: https://www.sydneysmith.com/wordpress/articles/
- Tony Nixon, *Notes on HP's Classic Calculators*: https://literature.hpcalc.org/community/classic-notes.pdf

x11-calc is especially useful as a third ROM corpus because `src/x11-calc-67.c` embeds an 8192-entry HP-67 ROM array. Its first words, octal `00000, 01743`, agree with Nonpareil's symbolic reset entry (`nop`, `go to reset0`). This is a useful sanity check, but x11-calc does not document physical-reader provenance for that array, so it remains a cross-check rather than the canonical source.

Reference emulators are not sources of electrical truth. They are useful for detecting our own decoding mistakes and for finding interesting execution paths to verify against raw ROM and hardware traces.

## Physical startup evidence

Tony Nixon also published a direct capture from a physical HP-67 while monitoring SYNC and IS during switch-on. The first observed execution path includes:

- address `0x000`: word `0x000` — no operation;
- address `0x001`: word `0x3e3` — conditional branch to `0x0f8` when carry is clear;
- address `0x0f8`: word `0x11a` — `0 -> c[w]`.

This is especially valuable because it ties ROM words and execution flow to a real HP-67 rather than to another emulator. `rom_compare verify-startup` checks any normalized corpus against those three evidence points. The same discussion later records a call from `0x0068` into the `1818-0232` ROM at `0x0fc6`, providing another future trace target once the instruction stream is running.

## Normalized corpus format and tooling

The repository now contains a local, firmware-free comparison path in `src/research/rom_corpus.rs` and `src/bin/rom_compare.rs`. All sources are normalized to explicit `(bank, pc, 10-bit word)` coordinates before comparison.

The x11-calc extractor requires all 8192 words and maps its flat image into two 4096-word banks. Address/opcode pair listings can be imported with an explicit radix, sparse bank/page files can be merged only when overlaps agree, and the comparator reports every mismatch or missing location by bank/page/PC. Binary `inspect` mode intentionally reports only factual properties and does not infer the unknown Teenix `.pfl` format.

The complete reproducible procedure and command examples are in `docs/ROM_CORPUS_WORKFLOW.md`.

## HP-67 / HP-97 relationship

Tony Nixon's notes record that HP-67 and HP-97 microcode is identical from ROM address `$400` through `$FFF`. We will use this as a strong cross-check because the HP-97 has a detailed service manual, but it does not imply that the complete machines are electrically identical.

The HP-97 Service Manual remains a corroborating source:

- https://literature.hpcalc.org/community/hp97-sm-en.pdf

Any code below `$400`, display/printer differences, keyboard differences or peripheral behaviour must remain independently verified.

## Acceptance procedure for the ROM corpus

When the ROM-reader archive has been downloaded locally:

1. Preserve the original archive unchanged.
2. Record archive SHA-256 and source URL/date.
3. Extract to a non-repository research directory.
4. Record every HP-67 file name, size and SHA-256.
5. Determine the ROM-reader word encoding from Teenix source/documentation or another independently verified decoder; do not infer it from desired output.
6. Map images to physical HP part numbers and logical banks/address ranges.
7. Normalize the Teenix corpus with a tested parser.
8. Run `rom_compare verify-startup` and require agreement with the physical `0x000 -> 0x001 -> 0x0f8` evidence path.
9. Extract x11-calc with `rom_compare extract-x11`.
10. Assemble/export Nonpareil's HP-67 sources to address/opcode pairs and normalize each bank with `rom_compare import-pairs`; merge them with conflict checking.
11. Compare Teenix ↔ Nonpareil, Teenix ↔ x11-calc and Nonpareil ↔ x11-calc, recording every disagreement by bank/page/address.
12. Compare the `$400-$FFF` region against an independently obtained HP-97 dump where formats permit.
13. Decode known startup locations and compare with published execution traces.
14. Run our decoder against Sydney Smith and Teenix/Panamatik address-level observations.
15. Store verified hashes, mapping and comparison reports in the repository, but not necessarily the copyrighted ROM payload itself.

## Repository policy

The emulator core must support externally supplied ROM images. Original HP firmware being publicly downloadable does not automatically give this project permission to redistribute it.

Therefore:

- raw HP ROM bytes are not embedded in the repository until redistribution has been reviewed separately;
- hashes, provenance, mapping, parsers, decoders and tests may be committed;
- tests that require original firmware should accept a local external ROM corpus;
- deterministic synthetic ROM fixtures may be committed for unit tests of the electrical/core logic.

## First microcode milestone

The first meaningful execution target is not arithmetic. It is a verified reset/fetch sequence:

`power/reset -> PHI1/PHI2 -> ACT address activity -> ROM response on ISA -> ACT captures a 10-bit instruction -> next 56-bit cycle`

Only after that trace matches the documented hardware model should we expand instruction semantics. Unknown or unverified microinstructions must stop with PC/word/tick diagnostics rather than silently behaving as a guessed opcode.
