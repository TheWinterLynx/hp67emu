# HP-67 program media formats

This document describes the three representations that are easy to confuse in this project:

- a human-readable user-program listing,
- the legacy Teenix `.hpp` compatibility file,
- the native hp67emu `.hp67card` physical-card container.

The canonical project format is `.hp67card`. Teenix `.hpp` is import/export compatibility only.

## Representation layers

```text
Human program listing
"001  FA  LBL A"
"002  12  2"
        |
        | one stored-program opcode byte per step
        v
112 program bytes for one program half
[ FA 12 B1 D2 19 00 F1 ... 00 ]
        |
        | low nibble first, then high nibble
        v
224 nibbles = 16 x 14-nibble RAM-register images
        |
        | each 14-nibble image becomes two 7-nibble CRC records
        | first 7 nibbles -> later record of the pair
        | second 7 nibbles -> earlier record of the pair
        v
34 x 28-bit logical card records
+----------+--------------------------------------+----------+
| record 0 | records 1..32                        | record33 |
| header   | 112 user-program bytes               | checksum |
+----------+--------------------------------------+----------+
        |
        | each 28-bit record packed MSB-first, no byte padding
        v
119-byte logical track payload
        |
        | Track 1 and Track 2 are stored independently
        v
250-byte HP67CARD v1 file
```

The magnetic reader model stops at the 34 x 28-bit logical-record boundary. The lower physical
Zero/One flux-track encoding is modeled separately and is not serialized by `.hp67card`.

## Native `.hp67card` v1

A native file represents one physical card, including both end-for-end logical tracks and their
recorded/write-protected state.

```text
byte offset      size        meaning
-----------      ----        -----------------------------------------------
0x00             8           ASCII "HP67CARD"
0x08             1           version = 1
0x09             1           Track 1 flags
0x0A             1           Track 2 flags
0x0B             1           reserved = 0
0x0C             119         Track 1 logical payload
0x83             119         Track 2 logical payload
                          ===============
                               250 bytes total
```

Track flag bits are:

```text
bit 0 (0x01)  recorded
bit 1 (0x02)  write protected
bits 2..7     must be zero
```

If a track is unrecorded, its 119-byte slot is ignored. This is why `.hp67card` can represent a
blank side while `.hp67raw` cannot.

One recorded track is exactly:

```text
34 records x 28 bits = 952 bits = 119 bytes
```

Records are serialized consecutively. Within each 28-bit word, bit 27 is written first and bit 0
last. There is no four-bit or byte padding between records, so record boundaries alternate between
byte-aligned and half-byte-aligned positions.

```text
record 0                         record 1
27                           0   27                           0
+-----------------------------+ +-----------------------------+
| b27 b26 ... b2 b1 b0        | | b27 b26 ... b2 b1 b0        |
+-----------------------------+ +-----------------------------+
 \_____________________________ ______________________________/
                               V
continuous bit stream -> bytes b7..b0, b7..b0, ...
```

This packing is an hp67emu logical interchange convention. It does not claim to be the order of
flux transitions on a real magnetic card.

## Legacy Teenix `.hpp`

An HPP file represents one compatibility payload, not one complete physical card. The entire file
is XOR-obfuscated byte-for-byte with `0x55`. After XOR decoding, the envelope is plain text:

```text
NeWe<EOL>
<body-length><EOL>
67<EOL>
bitmap-name.bmp<EOL>
card name<EOL>
<nibble data><EOL>
```

`body-length` is the byte length beginning at the calculator line and including the final line
ending. The importer accepts LF, CRLF and legacy CR-only line endings.

The nibble data contains exactly 266 nibbles:

```text
+----------------------+-----------------------------+----------------------+
| 21 dummy nibbles     | 238 real nibbles            | 7 mirror nibbles     |
| 3 historical records | 34 records x 7 nibbles      | duplicate record 34  |
+----------------------+-----------------------------+----------------------+
       ignored                 imported                    validated
```

Each real 28-bit record is written least-significant nibble first. For example:

```text
28-bit word:       0x03090222
hex nibbles:       0 3 0 9 0 2 2 2   (human high -> low view)
HPP serialization: 2 2 2 0 9 0 3     (low -> high)
compact HPP text:  "2220903"
```

The final seven HPP nibbles are not an additional logical card record. They must duplicate the
seven nibbles of real record 34. The importer checks that mirror.

The data area may be compact hexadecimal or whitespace-separated decimal nibble values. Both are
accepted by hp67emu.

The high nibble of logical record 1 (zero-based word 0) is the Teenix header class:

```text
header 1 or 3 -> compatibility importer infers Track 1
header 2 or 4 -> compatibility importer infers Track 2
```

That inference is a Teenix compatibility convention, not a universal physical-side identity.
Known media such as SD1-12A contain two independent header-3 programs on opposite physical sides.
For such files, conversion back to native media must explicitly specify `--track1` and
`--track2`.

HPP also has no field for native per-track write protection or an explicit unrecorded opposite
track. Converting `.hp67card -> .hpp -> .hp67card` is therefore record-lossless for each exported
recorded track but is not guaranteed to preserve all physical-card metadata unless the caller
supplies the missing placement/protection information.

## From a program listing to card records

A listing line such as:

```text
001  FA  LBL A
002  12  2
003  B1  GSB 1
004  D2  GTO 2
005  19  9
006  00  R/S
007  F1  LBL 1
```

contains the stored-program byte in the middle column. The mnemonic is presentation; the raw byte
is what is stored.

The first seven bytes above become fourteen nibbles by taking the low nibble before the high
nibble:

```text
FA -> A F
12 -> 2 1
B1 -> 1 B
D2 -> 2 D
19 -> 9 1
00 -> 0 0
F1 -> 1 F

14-nibble sequence:
A F 2 1 1 B 2 | D 9 1 0 0 1 F
```

The HP-67 program-card RAM/register ordering reverses the two seven-nibble halves when assigning
the CRC record pair:

```text
first  7 nibbles A F 2 1 1 B 2 -> logical record 2 -> 0x2B112FA
second 7 nibbles D 9 1 0 0 1 F -> logical record 1 -> 0xF10019D
```

The same operation is repeated sixteen times:

```text
program nibbles 0..13    -> records 2,1
program nibbles 14..27   -> records 4,3
program nibbles 28..41   -> records 6,5
...
program nibbles 210..223 -> records 32,31
```

This consumes exactly records 1 through 32. Record 0 is the card header and record 33 is the card
checksum record.

For the synthetic diagnostics, `listing-to-card` requires the full 28-bit header explicitly rather
than inventing the lower 24 header bits. Typical project examples are `0x03100222` for a one-pass
program, or `0x03000222` followed by `0x04000222` for a two-pass program. Those values are
project/corpus conventions; the converter does not infer them from mnemonics.

The checksum written by `listing-to-card` is:

```text
record[33] = (record[0] + record[1] + ... + record[32]) & 0x0fffffff
```

Unspecified listing steps are filled with opcode `0x00` (`R/S`). For header class 3, listing
steps are numbered 001..112. For header class 4, they are numbered 113..224.

After the 34 records are complete, each record is packed MSB-first into the 119-byte native track
slot, the native recorded/protected flags are set, and the result is wrapped in the 250-byte
`HP67CARD` container.

## Converter

The converter lives at `tools/convert_hp67_card.py`.

Normal Teenix media whose header classes already identify opposite tracks can be converted with:

```powershell
py .\tools\convert_hp67_card.py hpp-to-card side1.hpp side2.hpp -o program.hp67card
```

For exceptional media where both HPP payloads carry the same header class, physical placement is
explicit:

```powershell
py .\tools\convert_hp67_card.py hpp-to-card --track1 side-a.hpp --track2 side-b.hpp -o program.hp67card
```

Native media can be exported for compatibility:

```powershell
py .\tools\convert_hp67_card.py card-to-hpp program.hp67card --out-dir .\converted --stem program
```

That creates one HPP file per recorded native track. HPP-only metadata can be supplied with
`--calculator`, `--bitmap`, `--name`, `--line-ending` and `--decimal-nibbles`. Native
write-protect state cannot be represented and produces a warning.

A raw-byte listing generated by the emulator can be turned directly into native media:

```powershell
py .\tools\convert_hp67_card.py listing-to-card --track1 listing.txt --header1 0x03100222 -o program.hp67card
```

For a two-pass 224-step program:

```powershell
py .\tools\convert_hp67_card.py listing-to-card --track1 first.txt --header1 0x03000222 --track2 second.txt --header2 0x04000222 -o program.hp67card
```

The structural inspector is:

```powershell
py .\tools\convert_hp67_card.py inspect program.hp67card
```

And the converter regression checks HPP -> native against committed SD1-15A, native -> HPP -> records
against SD-15C, and listing -> native against CD-01:

```powershell
py .\tools\convert_hp67_card.py self-test
```
