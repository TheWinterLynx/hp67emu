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

Source: `mike632t/x11-calc`, `src/x11-calc-67.c`. The reviewed baseline is commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`.

x11-calc embeds an 8192-entry `int i_rom[]`. `rom_compare extract-x11` parses C octal/decimal/hex literals, validates every word as 10-bit and refuses an image that is not exactly 8192 words.

```powershell
cargo run --bin rom_compare -- extract-x11 D:\path\to\x11-calc\src\x11-calc-67.c .research\x11-hp67.tsv
cargo run --bin rom_compare -- verify-startup .research\x11-hp67.tsv
```

`verify-startup` checks the three words observed directly on a physical HP-67: bank 0 `0x000=0x000`, `0x001=0x3e3`, `0x0f8=0x11a`.

## 2. Nonpareil corpus

The HP-67 calculator definition references three instruction object files produced from:

- `ncd/67-97/67.asm`;
- `ncd/67-97/6797.asm`;
- `ncd/67-97/67b1.asm`.

The current research baseline is Nonpareil commit `c347bc1ab20170c253512042f7aac0d952f304ea`. We deliberately use Nonpareil's own `uasm` to assemble those symbolic sources rather than duplicating or translating its assembler.

At that commit `uasm` writes Woodstock object records as an optional bank mask followed by **octal** address and **octal** 10-bit opcode, for example:

```text
[0]0001:1743
[1]2000:0432
```

`research::nonpareil_obj` parses that exact upstream object format, and `nonpareil_rom` merges several `.obj` files into the standard hp67emu TSV while rejecting conflicting overlaps.

There is an important executable-name collision on Windows: the widely distributed `uasm64.exe` identifies itself as **“UASM ... Masm-compatible assembler”** and is an x86/x64 MASM-compatible assembler. It is unrelated to Eric Smith's Nonpareil calculator microassembler and cannot assemble these `.asm` files. The comparison script now probes the executable banner and refuses that program rather than passing Nonpareil options to it.

Current Nonpareil documentation states that precompiled binaries are not published. `compare_nonpareil.ps1` therefore follows this order:

1. use an explicitly supplied `-Uasm` only if its banner identifies it as `uasm microassembler`;
2. otherwise try `tools\nonpareil-uasm.exe`, `nonpareil-uasm` or `uasm` and validate the same banner;
3. otherwise, if WSL is available, clone the pinned Nonpareil source and build the official `uasm` locally from that exact commit with `gcc`, `flex` and `bison`, then run the resulting Linux binary under WSL to generate the three `.obj` files.

The WSL build helper is `docs/research/build_nonpareil_uasm_wsl.sh`. It keeps the upstream GPL source and generated binary under `.research`; none of them are committed to hp67emu.

Run the complete comparison with:

```powershell
& .\docs\research\compare_nonpareil.ps1
```

If WSL is present but its build tools are missing, install them once inside the WSL distribution:

```bash
sudo apt-get update && sudo apt-get install -y build-essential flex bison
```

The script pins the full upstream checkout, records source/object/uasm SHA-256 hashes, normalizes the three objects, verifies the physical startup words, compares the complete Teenix 5120-word subset against Nonpareil and, when an x11 TSV is already present in `.research`, compares Nonpareil against x11-calc too.

The standalone normalization command is:

```powershell
cargo run --bin nonpareil_rom -- .research\nonpareil-hp67.tsv 67.obj 6797.obj 67b1.obj
```

No Nonpareil source/object payload is committed to hp67emu.

## 3. Teenix 2026 HP-67 module

The Teenix page marks HP-67 as updated **10 May 2026** and MultiCalc as updated **11 May 2026**, so the current HP-67 module is a high-priority corpus rather than a legacy artefact.

The outer `.pfl` format is established: XOR every byte with `0x55`; the decoded first line is `NeWe`; the decoded second line is the decimal byte count of the remaining text. Current `cal67.pfl`, `cal6713.pfl` and `cal67b.pfl` match that convention exactly.

The decoded `cal67.pfl` payload is a Woodstock source-style listing. Its grammar needed for exact reconstruction is established:

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

## 4. Proven Teenix 2026 ↔ x11-calc comparison

Teenix represents the physically populated HP-67 microcode as 5120 words. x11-calc exposes a full two-bank 8192-entry logical array, so `rom_subset` asks the correct directional question: every populated Teenix location must exist in x11-calc and contain the same word, while x11-only locations are informational.

The reproducible script is:

```powershell
& .\docs\research\compare_teenix_x11.ps1
```

Using Teenix `cal67.pfl` SHA-256 `C8563E7982F38DEE9B6004B1194727AA2942DEB72D327C4D2F008CFDD057804B` and x11-calc source SHA-256 `3EBB5E8C014ACAC9013CC1D0CBC96734EC17BC6D6903E9D988B5AFC89B919B43` at the pinned commit, the corrected final result is:

```text
matching subset words : 5120
value mismatches      : 0
missing in reference  : 0
reference-only words  : 3072 (informational)

SUBSET MATCH: all 5120 populated subset words exist and match exactly.
```

The first run exposed a local hp67emu assembler-table swap between `0o1110 = down rotate` and `0o1310 = c -> stack`. That bug was corrected and locked with regressions before the clean rerun. The full provenance and first-run mismatch list are preserved in `docs/research/TEENIX_X11_COMPARISON_2026-09-15.md`.

Thus Teenix 2026 and x11-calc are reconciled bit-for-bit over all 5120 physically populated HP-67 microcode locations.

## 5. Teenix physical ROM-reader provenance

Keep the historical physical-reader project separate from the 2026 emulator module. Tony Nixon's 2022 HP Museum announcement for `ROMreader.zip` explicitly says that the ZIP included HP-97 and HP-67 ROM files produced with the physical reader.

The archive downloaded and extracted on 2026-09-15 did **not** expose obvious HP-67 ROM dumps in a filename search for `67`, `1818` or `rom`. That search returned only `ROM Reader Help.pdf` and `ROMread..hex`. Therefore current archive contents and historical physical provenance must not be conflated.

Physical startup observations from the same research remain direct evidence while the historical dump is being located.

## 6. Compare all corpora

Once official Nonpareil objects are normalized, compare equivalent populated regions among all three software corpora. Use `rom_subset` when one corpus intentionally has narrower physical coverage and `rom_compare compare` when two corpora are expected to have identical population maps.

```powershell
cargo run --bin rom_subset -- .research\teenix-2026-hp67.tsv .research\nonpareil-hp67.tsv
cargo run --bin rom_subset -- .research\nonpareil-hp67.tsv .research\x11-hp67.tsv
```

Every actual word mismatch must be explained. The already proven Teenix ↔ x11 identity plus the three physical startup words is a strong provisional firmware baseline; Nonpareil is the next independent symbolic/assembler cross-check.

## Acceptance rule

A working canonical ROM set requires:

1. current Teenix `.pfl` container decoded losslessly and the complete listing accepted by the strict analyzer;
2. current Teenix listing normalized to 5120 explicit physical words with all address anchors agreeing;
3. x11-calc independently normalized and all 5120 Teenix words matching exactly — **completed**;
4. Nonpareil assembled with official `uasm`, normalized and compared against the same populated region;
5. all candidates passing `verify-startup` wherever those addresses are present;
6. every Teenix ↔ x11-calc ↔ Nonpareil word difference over equivalent populated regions explained;
7. physical-reader provenance recovered and mapped to chip/bank/address when available;
8. the physical corpus compared against all software corpora;
9. later instruction-flow and electrical traces agreeing with the selected image.

Firmware agreement does not establish PHI1/PHI2 or bus timing. Electrical timing remains validated against hardware/service evidence.
