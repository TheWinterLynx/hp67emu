# HP-67 triple firmware corpus comparison — 2026-09-15

## Result

Three independently obtained/produced HP-67 firmware corpora agree exactly over all 5120 physically populated microcode locations:

```text
Teenix 2026   ↔ x11-calc   : 5120 / 5120 exact
Teenix 2026   ↔ Nonpareil  : 5120 / 5120 exact
Nonpareil     ↔ x11-calc   : 5120 / 5120 exact
value mismatches           : 0
missing populated locations: 0
```

x11-calc exposes an 8192-entry logical two-bank array, so its additional 3072 locations are reference-only address slots and are not treated as physically populated HP-67 words.

All three normalized corpora also pass the direct physical startup checkpoints:

```text
bank 0 pc 0x000 = 0x000 (0000 octal)
bank 0 pc 0x001 = 0x3e3 (1743 octal)
bank 0 pc 0x0f8 = 0x11a (0432 octal)
```

## Teenix 2026 provenance

Current HP-67 module `cal67.pfl`:

```text
SHA256=C8563E7982F38DEE9B6004B1194727AA2942DEB72D327C4D2F008CFDD057804B
```

The strict local decoder reconstructs exactly 5120 10-bit words from its source-style Woodstock listing with 0 unknown instructions, 0 label mismatches and 0 overflow.

## x11-calc provenance

Reviewed commit:

```text
9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983
```

Reviewed `src/x11-calc-67.c`:

```text
SHA256=3EBB5E8C014ACAC9013CC1D0CBC96734EC17BC6D6903E9D988B5AFC89B919B43
```

Its embedded HP-67 ROM array normalizes to 8192 logical entries. Every one of the 5120 physically populated Teenix locations is present and identical.

## Nonpareil provenance

Pinned Nonpareil commit:

```text
c347bc1ab20170c253512042f7aac0d952f304ea
```

The official Nonpareil `uasm` was built from that exact source revision under WSL and used to assemble the three HP-67 source components. A missing upstream `src/wasm.h` was supplied only in the temporary build tree with the minimal declaration documented in upstream issue #24; no Nonpareil source is vendored into hp67emu.

Build/tool provenance:

```text
uasm SHA256=AA3992928B124593495458981CEE87586079A132CBA77A8076C27EF421BFFAEC
```

Source/object hashes:

```text
67.asm   SHA256=2A3102C7CB8FB90A2AC733B702AAEA044F501B2327054E411B4D37BA67292099
67.obj   SHA256=0954BAF7D2264A4F760AE49DBE3C7C1FB32E859E8A3DE7FAD7C248790FC63C88
6797.asm SHA256=CC865C032AACDF0E3B06DB2650478D9273A6A4D7C2364E39DFE586A47E7A18D5
6797.obj SHA256=4CFE1ED1864C488C79FBA4E0C3345D40ABBFEB44990D378F4C87839A394D3B4F
67b1.asm SHA256=D77CF379E956141F06E8A7A6079A6536458D7E5B97D86B33503079246F7766E4
67b1.obj SHA256=5D953BA5F50508A0492BC9F27030961EB020EC992993A40200AC8623D8A4247D
```

Population contributed by the three official objects:

```text
67.obj   : 1024 words
6797.obj : 3072 words
67b1.obj : 1024 words
merged   : 5120 non-conflicting words
```

## Interpretation

This establishes a strong provisional HP-67 firmware baseline: three different representation/tooling paths converge on the same 5120 10-bit words. It does **not** establish electrical timing, ISA/DATA bit order, PHI1/PHI2 widths, bus ownership or display scan timing. Those remain hardware-evidence questions.

The next firmware-adjacent checkpoint is control flow observed on physical hardware. The repository now includes a regression for the documented sequence at PCs `0x0067/0x0068`: delayed select ROM 15 followed by JSB `0x00c6` must resolve to target `0x0fc6` with return address `0x0069`.
