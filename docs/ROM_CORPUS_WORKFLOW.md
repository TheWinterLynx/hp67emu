# HP-67 ROM corpus comparison workflow

This document defines the reproducible local workflow used to decide whether independently sourced HP-67 firmware images agree. Convenience is never enough to make a corpus canonical.

## Canonical normalized format

`hp67emu` normalizes firmware evidence to a sparse TSV with explicit bank, 12-bit PC and 10-bit word:

```text
# hp67emu-rom-corpus-v1
bank	pc	word
0	0x000	0x000
0	0x001	0x3e3
...
```

The comparison space is two banks × 4096 logical PCs = 8192 possible 10-bit words. Normalized files derived from copyrighted firmware remain local research artefacts unless redistribution is reviewed separately.

## 1. x11-calc corpus

Source: `mike632t/x11-calc`, `src/x11-calc-67.c`.

x11-calc embeds an 8192-entry `int i_rom[]`. `rom_compare extract-x11` parses C octal/decimal/hex literals, validates every word as 10-bit and refuses an image that is not exactly 8192 words.

```powershell
cargo run --bin rom_compare -- extract-x11 D:\path\to\x11-calc\src\x11-calc-67.c .research\x11-hp67.tsv
cargo run --bin rom_compare -- verify-startup .research\x11-hp67.tsv
```

`verify-startup` checks the three words observed directly on a physical HP-67: bank 0 `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`.

## 2. Nonpareil corpus

Sources include `ncd/67-97/67.asm`, `6797.asm`, `67b1.asm` and `6797cr.asm`.

Use upstream Nonpareil tooling to obtain address/opcode listings, then normalize them:

```powershell
cargo run --bin rom_compare -- import-pairs nonpareil-bank0.txt .research\nonpareil-bank0.tsv 0 8
cargo run --bin rom_compare -- import-pairs nonpareil-bank1.txt .research\nonpareil-bank1.tsv 1 8
cargo run --bin rom_compare -- merge .research\nonpareil-hp67.tsv .research\nonpareil-bank0.tsv .research\nonpareil-bank1.tsv
cargo run --bin rom_compare -- verify-startup .research\nonpareil-hp67.tsv
```

`merge` rejects conflicting overlaps.

## 3. Teenix ROM-reader provenance

The 2022 HP Museum announcement for `ROMreader.zip` explicitly says that the ZIP included HP-97 and HP-67 ROM files produced with the physical reader. This is valuable historical provenance.

The archive downloaded and extracted on 2026-09-15, however, did **not** expose obvious HP-67 ROM dumps in a filename search for `67`, `1818` or `rom`. That search returned only:

- `ROM Reader Help.pdf` — 826258 bytes;
- `ROMread..hex` — 7134 bytes.

The `.hex` file is expected to be reader-controller firmware, not calculator microcode, but that must be confirmed from its contents. Therefore the current ZIP may have changed since the 2022 release. We no longer assume the currently downloadable archive contains the historical ROM payload.

The correct next procedure is:

1. list **every file** in the extracted archive with size and SHA-256;
2. inspect the first records of `ROMread..hex` and confirm whether it is Intel HEX for the reader controller;
3. inspect `ROM Reader Help.pdf` for the output-file naming/format used by the reader;
4. search for an archived 2022 copy if the current ZIP no longer ships the HP-67/97 data files;
5. only then write a parser for the actual physical-reader output format.

The separate Teenix HP-67 emulator files `cal67.pfl`, `cal67b.pfl` and `cal6713.pfl` are structured ~88-93 KiB containers and are not treated as flat raw ROM images.

## 4. Compare software corpora now

We do not need to wait for the historical physical dump to compare the two independent open-source firmware corpora:

```powershell
cargo run --bin rom_compare -- compare .research\x11-hp67.tsv .research\nonpareil-hp67.tsv
```

Every mismatch must be explained. Agreement plus the three physical startup words provides a strong provisional firmware baseline while the raw physical-reader corpus is being recovered.

## 5. Add the physical corpus later

Once a physical-reader dump with defensible provenance is available:

```powershell
cargo run --bin rom_compare -- compare .research\physical-hp67.tsv .research\x11-hp67.tsv
cargo run --bin rom_compare -- compare .research\physical-hp67.tsv .research\nonpareil-hp67.tsv
```

The comparator reports populated counts and exact mismatches by bank/page/PC. Exit code 0 means identical presence and values; exit code 1 means at least one word differs.

## Acceptance rule

A working canonical ROM set requires:

1. x11-calc and Nonpareil independently normalized;
2. both passing `verify-startup`;
3. all x11-calc ↔ Nonpareil differences explained;
4. physical-reader provenance recovered and mapped to chip/bank/address when available;
5. the physical corpus compared against both software corpora;
6. later instruction-flow and electrical traces agreeing with the selected image.

Firmware agreement does not establish PHI1/PHI2 or bus timing. Electrical timing remains validated against hardware/service evidence.
