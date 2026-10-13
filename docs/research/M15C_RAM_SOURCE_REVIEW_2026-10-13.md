# M15C RAM/source recovery — 2026-10-13

## Starting state

Owner reported M15B FULL GATE GREEN at
`fc9a84d86c9205a5287c40ff33044ae393f8601e`; integrated that exact head into
main. Diagnostic 12/12 remains unchanged in outcomes and cycle counts.

## Performance review

M14Z versus M15B owner medians in us/word: raw PHI 0.358 -> 0.407;
full structural 0.725 -> 0.830; live idle 0.909 -> 1.048. Elapsed increases
are 13.7%, 14.5% and 15.3%. Source diff leaves the raw PHI path unchanged;
these separate-session measurements cannot isolate M15B guard overhead.
The live/raw-PHI ratio changed about 1.4%, but this normalization is a rough
control, not a matched A/B result or proof of no regression. Do not optimize
away the IS guard on this evidence. Revisit with matched runs if needed.

## Retrieved primary research sources

- https://www.teenix.org/ (retrieved 2026-10-13), CalCom download.
- https://www.teenix.org/CalCom.zip
- https://www.teenix.org/ROMreader.zip (linked by the hardware investigator).
- https://www.hpmuseum.org/forum/thread-18327-page-2.html,
  posts 21, 22, 34 and 35. Retrieved page through HTTP; search-service page
  retrieval failed, so conclusions use the retrieved full HTML, not snippets.

Archives inspected locally without executing their Windows programs, flashing
hardware or copying copyrighted firmware into this repository.

## CalCom scanner route

Archive SHA-256:
`26d6664dc50e8f9de0eade617e33b41aafe262f2c3f6bebc74cf3bee779e9d27`.
`ROMscanner.pdf` SHA-256:
`9c2c5b096ec67bdc575f887a1e07c2d1a3c96fd0ccdc3ce6b84c2217406b0399`.
`ROMscan.zip` SHA-256:
`daaa1aa6eddf731b3ac09d2931b6d8167a1b271dbd3d259008f24beb1ec9aaf7`.
The nested archive contains PCB/Gerber files, not original signal samples.

Printed manual p9 describes optional display/DATA capture; p12 describes
address, code, display and 14-nibble DATA listing columns; p13 gives an HP-67
ENTER-key acquisition procedure with SYNC/DATA/PHI2/IS connections. This is
an acquisition route for independent firmware/bus comparison. These listing
records are decoded word observations, not per-edge PHI1/PHI2 timing samples.
A procedure or example listing alone does not prove a capture's provenance.
The archive does not supply an original timestamped HP-67 acquisition identified
as such. Do not substitute scanner/PIC-generated timing for native ACT timing.

## ROM/RAM reader route

The reader help requires user IC definitions containing ROM start/size and RAM
start/size. It describes a RAM write/read test using byte patterns 0x5A and 0xAA
in seven-byte 56-bit words. The package does not expose HP-67 per-chip definition
files under chip part numbers, nor a physical HP-67 ROM/RAM dump identified by
chip. `OCM.dat` is not assumed to be that database: trial XOR 0x55 yields a
mnemonic table beginning `NeWe`, and no 1818-* chip identifiers were found in
that decoded table. That inspection is a format observation, not a complete
reverse engineering of the Windows executable or proof that no embedded data
could exist elsewhere.

`ROMread..hex` is the reader PIC programming payload identified by the help's
reprogramming section; do not interpret it as calculator ROM microcode.

## Crucial model distinction

Forum post 21 (2022-06-29) discusses **HP-97 1818-0229** responding at RAM
0x20..0x2F instead of the writer's expected 0x30..0x3F. It is not an HP-67
1818-0232 observation. Post 22 lists HP-97 ROM start addresses only. Post 35
(2022-08-28) independently identifies HP-67 1818-0232 serving code at 0x0FC6
and reports faulty RAM in 1818-0231/0232; it does not assign RAM base addresses
for the four HP-67 ROM/RAM chips. Do not derive RAM partitioning from ROM order.

## Outcome and next boundary

No production hardware claim or timing change is justified by this retrieval.
Physical HP-67 RAM-to-chip assignment, DATA polarity, handoff, launch/sample
edges and propagation remain SOURCE-BLOCKED. Next useful evidence is original
HP-67 IC definitions/dumps with readout provenance, or timestamped multi-channel
hardware acquisition with sampling/skew metadata. Existing capture/comparison
code remains sufficient; no new generic trace tools or Pac tests are added.

## ROMreader archive inventory

| Entry | Bytes | SHA-256 |
|---|---:|---|
| `OCM.dat` | 46823 | `5c970ffe7131078e787c6453a5d534fe3c52caa51a4cfaa200d3672df2dd909e` |
| `readme.txt` | 992 | `cd771a47806cbed7005e3989fefa6bb09833f025c25a030030790f65c48637e3` |
| `pcbBot.pdf` | 23795 | `3a7a31dfd07cf47f55a58d1662a462fe770500c1fb801fb132f713181376423b` |
| `pcbBotOLay.pdf` | 29005 | `e1029e3943e35f392b809592fda5f589d3042a30fb8925f7ff6d2b402c01f477` |
| `pcbTop.pdf` | 23702 | `28fa33679568e317f3ee00d54a97298a52cc9824464490a2ef63a9684ca0a6d8` |
| `pcbTopOLay.pdf` | 44066 | `8f517797ffaacbda0e61ef85e146f0fae9489097d66a24e36fa2fcea05ea96d1` |
| `schem.pdf` | 31181 | `a11e21941908cd756dd3519c802d7af582debb8e32dbd098dcab4a6f0e1a5107` |
| `ROM Reader Help.pdf` | 826258 | `d00a57d2c800526746ea816b2a9be39a43816035f5bf14377d8128f6c0f7c7fb` |
| `ROMread..hex` | 7134 | `255cb90ff6125ab96cfa3ad22dc958c600c00346e73628c0e496f7e720b708eb` |
| `HPRR.exe` | 11839960 | `44aeb7652477fd0a796002e63c3763fdc7d757f827cadab1ea7ef9c996661c44` |
| `rmrdrPCB.zip` | 43022 | `48013c1bbc85e4b11a1119ec350378d1bcde7e79cd17d879e22760a47fd106a5` |

Archive SHA-256: `5a4fffcf7208ebba132eb12c9ad6a8b2792346784be10b74fe1aacdec9693e47`.
