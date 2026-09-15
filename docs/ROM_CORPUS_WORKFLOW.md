# HP-67 ROM corpus comparison workflow

This document defines the reproducible local workflow used to decide whether independently sourced HP-67 firmware images agree. Convenience is never enough to make a corpus canonical.

## Canonical normalized format

`hp67emu` normalizes firmware evidence to a sparse TSV with explicit bank, 12-bit PC and 10-bit word:

```text
# hp67emu-rom-corpus-v1
bank\tpc\tword
0\t0x000\t0x000
0\t0x001\t0x3e3
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

## 3. Teenix 2026 HP-67 module

The Teenix page marks HP-67 as updated **10 May 2026** and MultiCalc as updated **11 May 2026**, so the current HP-67 module is a high-priority corpus rather than a legacy artefact.

The outer `.pfl` format is now established. Teenix's current *Classic Notes* documents a file convention where each byte is XORed with `0x55`; after decoding the first line is `NeWe`, the second line is the decimal byte count of the remaining text. The current `cal67.pfl`, `cal6713.pfl` and `cal67b.pfl` match that convention exactly.

Observed decoded headers:

```text
cal67.pfl   -> NeWe\r92855\r...
cal6713.pfl -> NeWe\r92882\r...
cal67b.pfl  -> NeWe\r87834\r...
```

The declared lengths exactly match the remaining payload sizes, and each observed payload begins with `L0000:\tno operation`.

Use the tested decoder rather than manually XORing files:

```powershell
cargo run --bin rom_compare -- inspect .\cal67.pfl
cargo run --bin rom_compare -- preview-teenix .\cal67.pfl 80
cargo run --bin rom_compare -- decode-teenix .\cal67.pfl .research\cal67.decoded.txt
```

`decode-teenix` validates both `NeWe` and the exact declared payload length before writing the plaintext. This proves the **outer container only**. The next stage is to establish the internal textual assembler/listing grammar and then translate that source to 10-bit words with tests. Do not infer opcodes merely from mnemonic text until address, bank and directive syntax are understood.

## 4. Teenix physical ROM-reader provenance

Keep the historical physical-reader project separate from the 2026 emulator module. Tony Nixon's 2022 HP Museum announcement for `ROMreader.zip` explicitly says that the ZIP included HP-97 and HP-67 ROM files produced with the physical reader.

The archive downloaded and extracted on 2026-09-15 did **not** expose obvious HP-67 ROM dumps in a filename search for `67`, `1818` or `rom`. That search returned only `ROM Reader Help.pdf` and `ROMread..hex`. Therefore current archive contents and historical physical provenance must not be conflated.

Physical startup observations from the same research remain direct evidence even while the historical dump is being located.

## 5. Compare corpora

Once Teenix's decoded textual payload has been converted to explicit `(bank, pc, word)` data, compare all three software corpora:

```powershell
cargo run --bin rom_compare -- compare .research\x11-hp67.tsv .research\nonpareil-hp67.tsv
cargo run --bin rom_compare -- compare .research\teenix-2026-hp67.tsv .research\x11-hp67.tsv
cargo run --bin rom_compare -- compare .research\teenix-2026-hp67.tsv .research\nonpareil-hp67.tsv
```

Every mismatch must be explained. Agreement plus the three physical startup words provides a strong provisional firmware baseline while the raw physical-reader corpus is being recovered.

## Acceptance rule

A working canonical ROM set requires:

1. current Teenix `.pfl` containers decoded losslessly and their internal source/listing grammar proven;
2. x11-calc and Nonpareil independently normalized;
3. all three candidates passing `verify-startup` after normalization;
4. all Teenix ↔ x11-calc ↔ Nonpareil differences explained;
5. physical-reader provenance recovered and mapped to chip/bank/address when available;
6. the physical corpus compared against all software corpora;
7. later instruction-flow and electrical traces agreeing with the selected image.

Firmware agreement does not establish PHI1/PHI2 or bus timing. Electrical timing remains validated against hardware/service evidence.
