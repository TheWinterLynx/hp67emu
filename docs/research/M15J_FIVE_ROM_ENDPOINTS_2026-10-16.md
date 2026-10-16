# M15J — five ROM serial endpoints in the live machine

Base/main: owner-validated M15I `87810d65635f616197e2dd6d6c67c65c73588434`.
Goal: replace aggregate live ROM IS response with per-chip receivers/drivers.
RAM-device ports and ACT internal execution are separate, still open milestones.

## Evidence and profile map

| Logical chip | Bank / PC window | Driver identity | Status |
| --- | --- | --- | --- |
| ROM0 | 0 / 000–3ff | 1818-0268 | HP-67 display-ROM inventory plus 1K map |
| ROM1 | 0 / 400–7ff | 1818-0550 | HP-97 table 4-3 under owner equivalence assumption |
| ROM2 | 0 / 800–bff | 1818-0551 | HP-97 table 4-3 under owner equivalence assumption |
| ROM3 | 0 / c00–fff | 1818-0232 | HP-67 physical replacement/call at 0fc6 |
| ROM4 | 1 / 400–7ff | 1818-0231 | WORKING INFERENCE by elimination from target five-chip inventory |

Primary/research sources reviewed:

- Teenix, *Notes on HP's Classic Calculators* (2025), printed page 71
  (PDF page 72), https://literature.hpcalc.org/community/classic-notes.pdf.
  SHA256 `bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
  Establishes the five 1K bank/address windows and ROM0 display role.
- HP-97 Service Manual, printed 4-11 (PDF page 69), table 4-3, and service
  note 97-102: https://www.teenix.org/97ServMan_2.pdf.
  SHA256 `7a102cd7051ed63b23b62b18f6768083f7e4abf4e607d9aa978541dfb25c5048`.
  Visually inspected; explicitly identifies 0550 as ROM1 and 0551 as ROM2.
- Direct physical ROM-reader/replacement account, posts 34–35:
  https://www.hpmuseum.org/forum/thread-18327-page-2.html.
  Identifies 0232 responding at 0fc6. HP-67 target inventory:
  https://www.keesvandersanden.nl/calculators/hp67_inside.php.
  Existing hardware/provenance documents already record these anchors.

The 0231 assignment is a bounded inference, not a new part-labelled dump.
Replace its profile identity if a direct dump contradicts it; do not promote it
to exact physical proof. All logical coverage and bus reconstruction remain
independently testable. No new firmware/card bytes are added or altered.
The currently served ROMreader.zip was checked again; it does not supply the
historically described per-chip ROM dump files. No stronger provenance is claimed.

## Executable behavior

Five fixed endpoints each hold their own address/mask, select flag and latched
response. They sample the same resolved b16..b27 address. Selection completes only
after all twelve bits; zero/multiple matches fail before lookup. Exactly one
immutable firmware lookup is made for a successful word. A selected chip holds
its word through b46..b55; others release IS. ACT reconstructs the word from the
actual resolved net. No host-to-ACT direct response shortcut exists.

The atomic launch commits the five named contributions and releases the aggregate
IS slot. Non-ROM contributors survive. ROM0's display and fetch functions use
its identity on different nets; no display feedback selects ACT address/data.
The clocked DATA prefix explicitly receives the configured ROM set and releases
all of its IS outputs at b0/b1. Four full transport variants use the same mode.

## Remaining bridges and fidelity limits

The source provider still supplies the selected-bank image view. This includes
the established bank-0 fallback outside populated bank-1 PC 400–7ff. That behavior
is retained for compatibility, not established as autonomous ROM bank circuitry.
Per-chip recognition of bank instructions/SYNC and exact enable propagation
remain open. The direct map's bank-1 holes must be revisited with that migration.
RAM DATA remains at the aggregate peripheral driver and existing C/RAM authority
bridge; this milestone does not implement physical RAM storage/address mapping.
The ACT architectural oracle/restore path and M15H provisional PHI policy remain.

## Performance practice and validation

No per-word/cell heap allocation, dynamic device registry or copied firmware
image is introduced. Profiles/state use fixed arrays. Only twelve address samples
visit all chips; one lookup occurs per successful word. Named drivers retain
cached resolution/count semantics and joint commit, preserving HighZ/contention.

New tests cover all profiles across all four transports, independent address
receivers, selected/nonselected actual driver contributions, immutable response
latching, zero/multiple selects before lookup, both bank views over all 8192
requested addresses, physical-mode DATA split/whole parity, missing-ROM exact
trace prefix and real live boot/reset. Existing assertions are retained.
Current electrical benchmark rows use five ROMs; historical dual rows keep their
aggregate fixture. Actual live benchmarks also measure the new composition.

Rust is unavailable here. Code/diff and documentation coverage are reviewed;
Actions performs formatting only. Owner focused library/native tests, full
warnings-as-errors suite, release, diagnostic 12/12 and all benchmarks are pending.
