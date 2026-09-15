# Teenix 2026 vs x11-calc HP-67 ROM comparison — 2026-09-15

## Provenance

The first complete local directional comparison used:

- Teenix `cal67.pfl` SHA-256: `C8563E7982F38DEE9B6004B1194727AA2942DEB72D327C4D2F008CFDD057804B`
- x11-calc reviewed commit: `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`
- x11-calc `src/x11-calc-67.c` SHA-256: `3EBB5E8C014ACAC9013CC1D0CBC96734EC17BC6D6903E9D988B5AFC89B919B43`

Both reconstructed corpora independently passed the physical HP-67 startup checkpoints at bank 0 PC `0x000`, `0x001` and `0x0f8`.

## First comparison result

The Teenix corpus contained 5120 physically populated words. The x11-calc logical corpus contained 8192 words. Directional comparison produced:

```text
matching subset words : 5091
value mismatches      : 29
missing in reference  : 0
reference-only words  : 3072 (informational)
```

Every one of the 29 mismatches was exclusively the pair:

```text
0x248 = 0o1110
0x2c8 = 0o1310
```

with one corpus containing one value and the other corpus containing the other. No third opcode value occurred in the mismatch set.

Exact mismatch locations from the first run:

```text
bank 0 pc 0x2d8: Teenix 0x2c8, x11 0x248
bank 0 pc 0x2d9: Teenix 0x2c8, x11 0x248
bank 0 pc 0x2da: Teenix 0x2c8, x11 0x248
bank 0 pc 0x589: Teenix 0x248, x11 0x2c8
bank 0 pc 0x59b: Teenix 0x248, x11 0x2c8
bank 0 pc 0x6e0: Teenix 0x248, x11 0x2c8
bank 0 pc 0x6ed: Teenix 0x248, x11 0x2c8
bank 0 pc 0x880: Teenix 0x248, x11 0x2c8
bank 0 pc 0x8a2: Teenix 0x248, x11 0x2c8
bank 0 pc 0x900: Teenix 0x248, x11 0x2c8
bank 0 pc 0x93a: Teenix 0x248, x11 0x2c8
bank 0 pc 0x946: Teenix 0x248, x11 0x2c8
bank 0 pc 0x989: Teenix 0x248, x11 0x2c8
bank 0 pc 0xac0: Teenix 0x248, x11 0x2c8
bank 0 pc 0xb28: Teenix 0x248, x11 0x2c8
bank 0 pc 0xb2d: Teenix 0x248, x11 0x2c8
bank 0 pc 0xb60: Teenix 0x248, x11 0x2c8
bank 0 pc 0xcd9: Teenix 0x248, x11 0x2c8
bank 0 pc 0xd3d: Teenix 0x248, x11 0x2c8
bank 0 pc 0xe78: Teenix 0x248, x11 0x2c8
bank 0 pc 0xea6: Teenix 0x248, x11 0x2c8
bank 0 pc 0xef7: Teenix 0x2c8, x11 0x248
bank 0 pc 0xef8: Teenix 0x2c8, x11 0x248
bank 0 pc 0xef9: Teenix 0x2c8, x11 0x248
bank 0 pc 0xfad: Teenix 0x248, x11 0x2c8
bank 0 pc 0xffc: Teenix 0x248, x11 0x2c8
bank 1 pc 0x55d: Teenix 0x2c8, x11 0x248
bank 1 pc 0x55e: Teenix 0x2c8, x11 0x248
bank 1 pc 0x55f: Teenix 0x2c8, x11 0x248
```

## Diagnosis

The mismatch was caused by a local `hp67emu` assembler-table error, not by a demonstrated firmware disagreement.

Teenix's published Woodstock instruction table defines:

```text
down rotate -> 1001001000 -> 0x248 -> 0o1110
c -> stack  -> 1011001000 -> 0x2c8 -> 0o1310
```

The reviewed x11-calc CPU decoder at the pinned commit independently uses the same mapping (`case 01110` for `down rotate`, `case 01310` for `c -> stack`). The initial `hp67emu` textual assembler had those two mnemonic encodings reversed. The semantic `reference::woodstock` execution behaviour itself already corresponded to the correct operations; regression tests now lock both opcode identities explicitly.

The assembler mapping was corrected after this first run. A clean rerun is required before declaring the 5120-word Teenix subset identical to x11-calc.
