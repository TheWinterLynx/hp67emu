# HP-67 microcode provenance and verification

Research snapshot: 2026-09-14.

## Preferred source

The current preferred starting point for the original HP-67 microcode is Tony Nixon's Teenix ROM-reader archive:

- https://www.teenix.org/ROMreader.zip
- provenance discussion: https://www.hpmuseum.org/forum/thread-18327-page-2.html

In that discussion Nixon states that the archive contains ROM files for both HP-97 and HP-67 produced by his physical ROM-reader project. That makes it more valuable to this project than a listing transcribed from an emulator or reconstructed from program behaviour.

The exact filenames, encoding, chip mapping and SHA-256 values inside the archive are deliberately **not** asserted here until we inspect the downloaded archive locally. We will not invent those details.

## Independent cross-checks

Use at least two independent references before accepting our interpretation of a raw word or opcode family:

- Teenix HP-67 emulator module: https://www.teenix.org/HP67.zip
- Panamatik HP-67 emulator, documented as running original HP-67 microcode: https://www.panamatik.de/html/hp-67.html
- Sydney Smith HP67u/HP67w microcode emulators and address-level analyses: https://www.sydneysmith.com/wordpress/hp67-main/
- Sydney Smith article index: https://www.sydneysmith.com/wordpress/articles/
- Tony Nixon, *Notes on HP's Classic Calculators*: https://literature.hpcalc.org/community/classic-notes.pdf

Reference emulators are not sources of electrical truth. They are useful for detecting our own decoding mistakes and for finding interesting execution paths to verify against raw ROM and hardware traces.

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
5. Determine the word encoding without modifying the source image.
6. Map images to physical HP part numbers and logical banks/address ranges.
7. Check total implemented address space against the HP-67 ROM map.
8. Compare the `$400-$FFF` region against an independently obtained HP-97 dump where formats permit.
9. Decode known startup locations and compare with published execution traces.
10. Run our decoder against Sydney Smith and Teenix/Panamatik address-level observations.
11. Store the verified hashes and mapping in the repository, but not necessarily the copyrighted ROM payload itself.

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