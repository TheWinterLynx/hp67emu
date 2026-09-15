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

The reviewed baseline is commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`.

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

The outer `.pfl` format is established: XOR every byte with `0x55`; the decoded first line is `NeWe`; the decoded second line is the decimal byte count of the remaining text. Current `cal67.pfl`, `cal6713.pfl` and `cal67b.pfl` match that convention exactly.

The decoded `cal67.pfl` payload is a Woodstock source-style listing. Its complete grammar needed for this file is now understood by the strict parser:

- blank lines do not consume ROM;
- `// ...` comment lines do not consume ROM;
- `org $1400` switches to physical bank 1 at logical PC `0x400`;
- both `Lxxxx:` and `Hxxxx:` prefixes are hexadecimal address anchors;
- all remaining non-empty records are Woodstock microinstructions assembled to 10-bit words.

The current file contains 5127 decoded text lines but exactly 5120 microinstructions: all 4096 bank-0 locations plus bank-1 PC `0x400..0x7ff`. The strict analyzer reports 5120 candidates, 0 unknown instructions, 0 label mismatches and 0 overflow. Extraction produces exactly 5120 normalized words and all three direct physical startup checkpoints pass.

```powershell
cargo run --bin rom_compare -- analyze-teenix-hp67 .\cal67.pfl
cargo run --bin rom_compare -- extract-teenix-hp67 .\cal67.pfl .research\teenix-2026-hp67.tsv
```

Extraction is all-or-nothing. A source comment, unknown pseudo-op, address-anchor mismatch, unsupported `org`, or word-count error prevents corpus generation.

The lossless outer-container commands remain useful:

```powershell
cargo run --bin rom_compare -- inspect .\cal67.pfl
cargo run --bin rom_compare -- preview-teenix .\cal67.pfl 80
cargo run --bin rom_compare -- decode-teenix .\cal67.pfl .research\cal67.decoded.txt
```

## 4. Directional comparison: Teenix vs x11-calc

Teenix represents the physically populated HP-67 microcode as 5120 words. x11-calc exposes a full two-bank 8192-entry logical array. Therefore a symmetric full comparison will intentionally report 3072 x11-only address slots even if every physically populated Teenix word agrees.

Use `rom_subset` for this specific evidence question:

```powershell
cargo run --bin rom_subset -- .research\teenix-2026-hp67.tsv .research\x11-hp67.tsv
```

The first file is the required subset. Every populated Teenix location must exist in x11-calc and contain the identical 10-bit word. Reference-only x11-calc locations are reported but do not fail the subset check. Value mismatches or missing reference words return exit code 1.

For a fully reproducible local cross-check, `docs/research/compare_teenix_x11.ps1` downloads only the pinned x11-calc HP-67 source file at commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`, records SHA-256 provenance, extracts both corpora, checks x11-calc against the physical startup words and finally runs the 5120-word subset comparison:

```powershell
& .\docs\research\compare_teenix_x11.ps1
```

No external ROM or source payload is committed to this repository; downloaded source and generated TSVs remain under `.research`.

## 5. Teenix physical ROM-reader provenance

Keep the historical physical-reader project separate from the 2026 emulator module. Tony Nixon's 2022 HP Museum announcement for `ROMreader.zip` explicitly says that the ZIP included HP-97 and HP-67 ROM files produced with the physical reader.

The archive downloaded and extracted on 2026-09-15 did **not** expose obvious HP-67 ROM dumps in a filename search for `67`, `1818` or `rom`. That search returned only `ROM Reader Help.pdf` and `ROMread..hex`. Therefore current archive contents and historical physical provenance must not be conflated.

Physical startup observations from the same research remain direct evidence even while the historical dump is being located.

## 6. Compare all corpora

Once x11-calc and Nonpareil are both normalized, compare equivalent populated regions among all three software corpora. Use `rom_subset` when one corpus intentionally has narrower physical coverage and `rom_compare compare` when two corpora are expected to have identical population maps.

```powershell
cargo run --bin rom_compare -- compare .research\x11-hp67.tsv .research\nonpareil-hp67.tsv
cargo run --bin rom_subset -- .research\teenix-2026-hp67.tsv .research\x11-hp67.tsv
cargo run --bin rom_subset -- .research\teenix-2026-hp67.tsv .research\nonpareil-hp67.tsv
```

Every actual word mismatch must be explained. Agreement plus the three physical startup words provides a strong provisional firmware baseline while the raw physical-reader corpus is being recovered.

## Acceptance rule

A working canonical ROM set requires:

1. current Teenix `.pfl` container decoded losslessly and the complete listing accepted by the strict analyzer;
2. current Teenix listing normalized to 5120 explicit physical words with all address anchors agreeing;
3. x11-calc and Nonpareil independently normalized;
4. all candidates passing `verify-startup` wherever those addresses are present;
5. all Teenix ↔ x11-calc ↔ Nonpareil word differences over equivalent populated regions explained;
6. physical-reader provenance recovered and mapped to chip/bank/address when available;
7. the physical corpus compared against all software corpora;
8. later instruction-flow and electrical traces agreeing with the selected image.

Firmware agreement does not establish PHI1/PHI2 or bus timing. Electrical timing remains validated against hardware/service evidence.
