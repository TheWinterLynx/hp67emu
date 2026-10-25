# M16D — physical keyboard switch network, delivery 7/15

Date: 2026-10-25
Base: owner-validated M16C v2 8ea080b80242122062c986811e8818df63131633, integrated main.
Branch: agent/m16d-keyboard-contact-network
Status: implementation prepared; owner local runtime gate pending.

## Direct evidence

HP-67 schematic https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf
SHA256 c567d6fdf2a924c998a395fe05d593d6466ef9bbf488ff0e9845ecb5f8b09e73.
Downloaded and rendered PDF page 8, keyboard/display PCB sheet 3; inspected upright.
This is a reverse-engineered HP-67 wiring drawing, not an original HP design
specification. It directly records the target's 35 switch connections and shows
no series diode in this drawing. Further board/capture corroboration is welcome.

| Cathode wire | KC1 | KC2 | KC3 | KC4 | KC5 |
|---|---|---|---|---|---|
| 8 | A | B | C | D | E |
| 6 | Sigma+ | GTO | DSP | indirect | SST |
| 13 | f | g | STO | RCL | h |
| 5 | absent | ENTER | CHS | EEX | CLX |
| 11 | absent | minus | 7 | 8 | 9 |
| 3 | absent | plus | 4 | 5 | 6 |
| 9 | absent | multiply | 1 | 2 | 3 |
| 7 | absent | divide | 0 | decimal | R/S |

These are physical wire labels. They are deliberately NOT mapped to existing
fifteen-word display scan indices or ACT key-code row values.

## Implemented lane

Physical press/release -> retained switch bank -> explicit cathode/KC network.
The production Hp67Keyboard now stores contact closure in that bank rather than
an unrelated optional key. Its existing press API replaces a single contact;
release opens it, and the last compatibility key code remains latched. A separate
matrix API supports arbitrary simultaneous closures without selecting a winner.

The ideal digital switch solver connects wires bidirectionally. Explicit source
arrays feed all eight cathodes and five columns. High-Z means no source; no
sources leave a component Floating, opposite sources resolve Contention, and
agreeing sources resolve their level. The returned cathodes also show backfeed.
Three closed corners can connect a fourth sense path without creating a fourth
physical key. Reopening contacts splits the graph; no retained voltage is invented.

Components rebuild only when contacts change; resolution uses thirteen bitmask
intersections and fixed arrays, without allocation or scanning the 35 switches.
No new per-bit hook, backplane driver/net slot or PHI work enters production.
Existing 56 cells and 224 transitions remain mandatory. No speed claim is made
until the owner's gate measures the production path.

## Explicit limits

Wiring identity is source-backed. Zero-resistance instantaneous digital contact
connection is a WORKING APPROXIMATION. There are no analog thresholds, contact
resistance, capacitance or bounce assumptions. Supplied Drive values are fixture
stimuli, not claims about active cathode polarity or a physical output stage.

Production ACT still uses the existing retained key-code/S15 compatibility lane.
This delivery does not claim that KC pins drive ACT reception, that display slot
numbers equal physical cathode labels, or that timing is electrically complete.
Column bias/output stage, scan-to-code encoding and ACT acquisition/latch timing
remain SOURCE-BLOCKED. They must be established before connecting this solver
into the PHI loop and replacing the compatibility receiver. No fake host scan
or immediate firmware key recognition is introduced as a new electrical claim.

## Regressions and acceptance

Seven focused tests: all 35 independently transcribed wire coordinates; both
directions at High/Low/High-Z for every key; three-corner backfeed and release;
connected conflict/agreement/isolation; every two-key chord with all 169 source
node pairs compared against an independent graph walk; insertion order,
idempotence and complete removal; production single-contact and retained-code
lifecycle for every key. Existing keyboard/firmware and both diagnostic builds
remain required. No runtime or compilation is performed by the agent.

M16C owner baseline: live 2.910 us/word (109.96x); PHI 0.352; full structural
1.300; autonomous RAM 1.086. These are separate host runs, not controlled A/B.
Actions is authorized for fmt/check/save only. New delivery remains unmerged
until the owner reports its full local gate green.
