# HP-67 ROM corpus comparison workflow

This document defines the reproducible local workflow used to decide whether independently sourced HP-67 firmware images agree. It does **not** make any ROM corpus canonical merely because it is convenient to load.

## Canonical normalized format

`hp67emu` research tooling normalizes all sources to a sparse TSV file with three fields:

```text
# hp67emu-rom-corpus-v1
bank	pc	word
0	0x000	0x000
0	0x001	0x3e3
...
```

The physical comparison space is two banks × 4096 logical PCs = 8192 possible 10-bit words. PC is always the 12-bit logical address `0x000..0xfff`; bank is kept as a separate coordinate.

No original ROM payload is committed merely by generating a normalized file. Normalized files derived from copyrighted firmware remain local research artifacts unless redistribution is reviewed separately.

## 1. x11-calc corpus

Source: `mike632t/x11-calc`, `src/x11-calc-67.c`.

x11-calc embeds `int i_rom[ROM_SIZE]` and defines `ROM_SIZE` as octal `020000`, i.e. 8192 entries. Its processor uses the extra address bit as ROM bank selection, so the array naturally consists of bank 0 followed by bank 1. `rom_compare extract-x11` parses C octal/decimal/hex literals, validates every word as 10-bit and refuses an array that is not exactly 8192 words.

```powershell
cargo run --bin rom_compare -- extract-x11 D:\path\to\x11-calc\src\x11-calc-67.c .research\x11-hp67.tsv
```

The extractor does not copy x11-calc source into this repository.

## 2. Nonpareil corpus

Sources include `ncd/67-97/67.asm`, `6797.asm`, `67b1.asm` and `6797cr.asm`.

These files are symbolic assembly rather than a simple raw binary. We do not parse labels or reproduce Nonpareil's assembler inside hp67emu. First use the upstream Nonpareil assembler/tooling to produce an address/opcode listing, then normalize each bank with `import-pairs`.

```powershell
cargo run --bin rom_compare -- import-pairs nonpareil-bank0.txt .research\nonpareil-bank0.tsv 0 8
cargo run --bin rom_compare -- import-pairs nonpareil-bank1.txt .research\nonpareil-bank1.tsv 1 8
cargo run --bin rom_compare -- merge .research\nonpareil-hp67.tsv .research\nonpareil-bank0.tsv .research\nonpareil-bank1.tsv
```

`merge` permits overlapping locations only when their words are identical. A disagreement is a hard error so that assembler/page-layout mistakes cannot be silently overwritten.

## 3. Teenix physical-reader corpus

The preferred provenance candidate remains Tony Nixon's ROM-reader archive. The locally observed `cal67.pfl`, `cal67b.pfl` and `cal6713.pfl` files are binary. Their layout has **not** yet been established, so hp67emu deliberately does not guess an endian, record structure or word packing.

Use the fact-only inspection command first:

```powershell
cargo run --bin rom_compare -- inspect D:\path\to\cal67.pfl
cargo run --bin rom_compare -- inspect D:\path\to\cal67b.pfl
cargo run --bin rom_compare -- inspect D:\path\to\cal6713.pfl
```

Once the `.pfl` layout is proven from Teenix source/documentation or an independent decoder, add a dedicated parser with regression fixtures before converting those files to normalized TSV.

## 4. Compare corpora

```powershell
cargo run --bin rom_compare -- compare .research\x11-hp67.tsv .research\nonpareil-hp67.tsv
cargo run --bin rom_compare -- compare .research\teenix-hp67.tsv .research\x11-hp67.tsv
cargo run --bin rom_compare -- compare .research\teenix-hp67.tsv .research\nonpareil-hp67.tsv
```

The comparator reports populated counts and matches/mismatches separately for every bank/page, followed by exact conflicting locations in hexadecimal and octal. Exit code 0 means all populated locations in the two normalized corpora agree exactly and have identical presence; exit code 1 means at least one word or presence differs.

## Acceptance rule

The working canonical ROM set should be selected only after:

1. Teenix `.pfl`/reader data has a documented decoder and chip/bank mapping.
2. Source archive and extracted files have SHA-256 provenance recorded.
3. Teenix, x11-calc and assembled Nonpareil images have been normalized independently.
4. All differences have been explained rather than overwritten.
5. Reset/startup words and known published traces agree with the selected corpus.

If all three independent corpora agree word-for-word, that gives very strong confidence in the firmware image. It still does not make emulator implementations evidence for PHI1/PHI2 or electrical bus timing; those remain validated against hardware/service evidence.
