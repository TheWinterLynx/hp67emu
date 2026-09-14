# `src/research/rom_corpus.rs`

## Purpose

Provides a firmware-neutral representation of the HP-67's two-bank, four-page, 8192-word physical ROM corpus and tools to normalize and compare independent firmware sources.

## Why it exists

The emulator should not choose a canonical ROM because one emulator happens to contain a convenient copy. We need reproducible comparison between physical Teenix dumps, Nonpareil-derived listings and x11-calc while keeping original firmware outside the repository unless redistribution is separately approved.

## Relationships

Used by `src/bin/rom_compare.rs`. Its address layout deliberately matches `src/reference/rom.rs`: two banks, four 1024-word pages per bank and 10-bit microinstructions. It does not depend on the semantic CPU or electrical scheduler.

## Responsibilities

Parse x11-calc's complete `i_rom[]` array, normalize address/opcode pair listings, serialize a stable TSV interchange format, validate 10-bit words and physical addresses, and report exact differences by bank/page/PC. It must never guess an undocumented binary format such as Teenix `.pfl`.

## Implementation

`RomCorpus` stores 8192 optional word slots. Flat index `bank * 4096 + pc` mirrors x11-calc's verified `rom[pc | (BANK << 12)]` access. The x11 parser accepts C octal/decimal/hex literals and requires exactly 8192 words. The comparator distinguishes equal words, mismatches and words present in only one corpus, with per-page summaries. Unit tests cover x11 bank mapping, 10-bit validation, TSV round trips, pair imports and difference classification.
