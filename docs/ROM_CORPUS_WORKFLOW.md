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
cargo run --bin rom_compare -- verify-startup .research\x11-hp67.tsv
```

The extractor does not copy x11-calc source into this repository. `verify-startup` checks three words observed directly on a physical HP-67 at power-on: bank 0 `0x000=0x000`, `0x001=0x3e3`, and branch target `0x0f8=0x11a`.

## 2. Nonpareil corpus

Sources include `ncd/67-97/67.asm`, `6797.asm`, `67b1.asm` and `6797cr.asm`.

These files are symbolic assembly rather than a simple raw binary. We do not parse labels or reproduce Nonpareil's assembler inside hp67emu. First use the upstream Nonpareil assembler/tooling to produce an address/opcode listing, then normalize each bank with `import-pairs`.

```powershell
cargo run --bin rom_compare -- import-pairs nonpareil-bank0.txt .research\nonpareil-bank0.tsv 0 8
cargo run --bin rom_compare -- import-pairs nonpareil-bank1.txt .research\nonpareil-bank1.tsv 1 8
cargo run --bin rom_compare -- merge .research\nonpareil-hp67.tsv .research\nonpareil-bank0.tsv .research\nonpareil-bank1.tsv
cargo run --bin rom_compare -- verify-startup .research\nonpareil-hp67.tsv
```

`merge` permits overlapping locations only when their words are identical. A disagreement is a hard error so that assembler/page-layout mistakes cannot be silently overwritten.

## 3. Teenix physical ROM-reader corpus

The preferred provenance candidate is Tony Nixon's `ROMreader.zip`, because the accompanying HP Museum discussion states that it contains HP-67 ROM files produced by the physical ROM-reader project.

The separate Teenix HP-67 emulator-module files `cal67.pfl`, `cal67b.pfl` and `cal6713.pfl` have been inspected locally. They are structured binary files of roughly 88-93 KiB with a common header pattern, so they are not simple flat 8192-word raw ROM images. Their format remains unproven and decoding them is **not** a prerequisite while the physical ROM-reader archive is available.

The next Teenix step is therefore:

1. preserve `ROMreader.zip` unchanged and hash it;
2. extract it locally;
3. inventory the HP-67 files supplied by the ROM-reader project;
4. identify the file format from its bundled help/source before writing a parser;
5. map every image to the physical part number and logical bank/address range;
6. normalize the result to TSV;
7. run `verify-startup` against the normalized physical dump.

The `.pfl` files may still be inspected for research purposes:

```powershell
cargo run --bin rom_compare -- inspect D:\path\to\cal67.pfl
```

`inspect` now reports byte-distribution facts such as 7-bit ratio, printable ratio, entropy and common byte values, but deliberately makes no format claim.

## 4. Compare corpora

```powershell
cargo run --bin rom_compare -- compare .research\x11-hp67.tsv .research\nonpareil-hp67.tsv
cargo run --bin rom_compare -- compare .research\teenix-physical-hp67.tsv .research\x11-hp67.tsv
cargo run --bin rom_compare -- compare .research\teenix-physical-hp67.tsv .research\nonpareil-hp67.tsv
```

The comparator reports populated counts and matches/mismatches separately for every bank/page, followed by exact conflicting locations in hexadecimal and octal. Exit code 0 means all populated locations in the two normalized corpora agree exactly and have identical presence; exit code 1 means at least one word or presence differs.

## Acceptance rule

The working canonical ROM set should be selected only after:

1. the physical Teenix ROM-reader data has a documented decoder and chip/bank mapping;
2. source archive and extracted files have SHA-256 provenance recorded;
3. Teenix physical dump, x11-calc and assembled Nonpareil images have been normalized independently;
4. all differences have been explained rather than overwritten;
5. `verify-startup` passes against the physically observed startup words;
6. later instruction-flow and electrical traces agree with the selected corpus.

If all three independent corpora agree word-for-word, that gives very strong confidence in the firmware image. It still does not make emulator implementations evidence for PHI1/PHI2 or electrical bus timing; those remain validated against hardware/service evidence.
