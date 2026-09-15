# `src/research/teenix_hp67.rs`

## Purpose

Analyses and normalizes the decoded current Teenix HP-67 microcode listing into the repository's sparse `(bank, pc, word)` ROM corpus format.

## Why it exists

The current `cal67.pfl` is a `NeWe` XOR container whose decoded payload is source-style Woodstock assembly. The HP-67 has 4096 bank-0 words plus the populated 1024-word bank-1 window at PC `0x400..0x7ff`; a strict parser lets us turn the Teenix 2026 listing into a corpus only when every microinstruction is understood and every address anchor agrees.

## Relationships

Uses `src/research/woodstock_asm.rs` to assemble each mnemonic and `src/research/rom_corpus.rs` to store normalized words. It is called by `src/bin/rom_compare.rs` through the `analyze-teenix-hp67` and `extract-teenix-hp67` commands. It is not used by the production emulator or electrical scheduler.

## Responsibilities

Count payload/blank/instruction lines; map the expected 5120-word HP-67 physical ROM order; validate optional `Lxxxx:` hexadecimal address labels; report unknown mnemonics, address mismatches and overflow; refuse extraction until the entire listing is understood; normalize a clean listing into a `RomCorpus`.

## Implementation

Every non-empty payload line is treated as one candidate microinstruction. The first 4096 candidates map to bank 0 PCs `0x000..0xfff`; the next 1024 map to bank 1 PCs `0x400..0x7ff`. Optional labels are validation anchors rather than authorities that silently move the parser. Analysis returns detailed issue records with source line and expected location. Normalization is all-or-nothing and only proceeds when the instruction count is exactly 5120, all mnemonics assemble, all labels match and there is no overflow.
