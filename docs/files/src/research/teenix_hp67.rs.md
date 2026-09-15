# `src/research/teenix_hp67.rs`

## Purpose

Analyses and normalizes the decoded current Teenix HP-67 microcode listing into the repository's sparse `(bank, pc, word)` ROM corpus format.

## Why it exists

The current `cal67.pfl` is a `NeWe` XOR container whose decoded payload is source-style Woodstock assembly. The listing is not a bare sequence of instructions: the bank transition contains `//` comments, an `org $1400` directive and `Hxxxx:` address anchors. A strict parser must understand those source records without counting them as ROM words.

## Relationships

Uses `src/research/woodstock_asm.rs` to assemble each mnemonic and `src/research/rom_corpus.rs` to store normalized words. It is called by `src/bin/rom_compare.rs` through the `analyze-teenix-hp67` and `extract-teenix-hp67` commands. It is not used by the production emulator or electrical scheduler.

## Responsibilities

Count payload, blank, comment, directive and instruction lines separately; track the combined Teenix source address; interpret bit 12 as the ROM bank selector and the low 12 bits as logical Woodstock PC; validate observed `Lxxxx:` and `Hxxxx:` anchors; report unknown mnemonics, address mismatches and unsupported locations; refuse extraction until all 5120 physical HP-67 microinstructions are understood.

## Implementation

The address cursor starts at `$0000`. Blank lines and `//` comments consume no ROM location. `org $xxxx` changes the combined source address without emitting a word; the observed `org $1400` therefore starts bank 1 at logical PC `0x400`. Instructions at combined `$0000..$0fff` map to bank 0, while `$1400..$17ff` map to the populated bank-1 window. Both `Lxxxx:` and `Hxxxx:` prefixes are treated as hexadecimal address anchors whose numeric PC must agree with the current cursor; the prefix is preserved for diagnostics but is not used by itself to infer the bank. Normalization remains all-or-nothing and `RomCorpus::set` rejects duplicate locations.
