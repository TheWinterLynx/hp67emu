# M16I — resolved HDS/WPS and retained CRC write gate, delivery 12/15

Date: 2026-10-29
Base: owner-validated/integrated M16H v2 b68f97facc5b3c900eef5a5e03269112065d5d91
Branch: agent/m16i-reader-head-wps
Status: owner FULL GATE GREEN at 774f0eb408e98da9469204ec7f350f99fa4bd6e6; exact tested head integrated into main (delivery 12/15).

## Evidence inspected

HP-97 Service Manual https://www.teenix.org/97ServMan_2.pdf was downloaded,
text-searched, rendered and visually inspected. SHA256:
7a102cd7051ed63b23b62b18f6768083f7e4abf4e607d9aa978541dfb25c5048.
The same HP-67 schematic/input identity reference recorded in M16G/M16H names
HDS/WPS at the shared reader electronics. HP-97 equivalence is owner-authorized.

- Section 2-37, printed 2-8/PDF page 22: head arrival closes the head switch,
  grounding HDS. This is distinct from MTRS insertion and MOTOR command.
- Section 2-39, printed 2-9/PDF page 23: CRC interrogates the write-protect
  switch when the head switch closes; a clipped corner inhibits writing and
  firmware is informed through its error path.
- Service Note 97-101 page 6/7, PDF page 148: grounding the write-protect
  connection (explicitly HP-67 pin 15 / HP-97 pin 13) permits recording protected
  cards. The ground must be opened/reclosed for each pass; leaving it grounded
  causes an error. The note also explicitly warns that head-switch adjustment
  changes start-bit position, so no fixed arrival geometry is inferred here.

These sources establish active-low head closure, grounded WPS as write permission
and head-time interrogation/rearming semantics. They do not establish idle bias,
voltage thresholds, switch bounce, relative WPS/HDS physical edge order or PHI
sampling/propagation delays. High/open source normalization, one joint software
publication, initial CRC permission arming and the existing logical head-arrival
checkpoint remain WORKING APPROXIMATIONS. Exact mechanics and timing are
SOURCE-BLOCKED. No simultaneous physical contact transition is claimed.

## Ownership and complete path

Physical/approximated head-arrival event -> selected card corner contact state ->
checked External HDS/WPS source bundle -> independent resolved CRC-side receiver ->
retained head/write permission -> owned CRC record transport -> READY/F7 -> F2 ->
ACT/real firmware. Media metadata drives physical corner stimulus only; it is not
an input to the production CRC receiver or the production write-inhibit decision.

The source checks both old conductor images before replacing either. Each pin
requires one External contribution and a binary level; agreeing extra drives,
opposition, missing/replaced source and changed held slots are rejected. Both
proposals install before the backplane publishes one optional capture row; no
receiver sees a half-updated pair and no PHI tick is consumed.

CRC samples WPS when logical HDS closes, retains that decision while HDS stays
closed, and drops permission when HDS opens. An accepted grounded WPS consumes its
arming; an observed open is required before another head closure can grant write
permission. A held ground cannot silently authorize every subsequent pass. Reset
starts the bridge armed, explicitly not a claim about the silicon reset waveform.

The new advance_us_owned_with_reader entry point checks receiver/transport head
agreement before any CRC buffer synchronization. Its head gate and write decision
consume the independent CRC receiver. Existing advance_us/advance_us_owned remain
explicit metadata-driven reference fixtures. Write-inhibited passes raise the same
F7/READY path at the existing logical boundary without draining FIFO or modifying
media; grounded WPS can permit the documented service bypass despite clipped-card
metadata. No host program/file name is involved.

## Live integration and preserved uncertainty

Production attaches/resets both sources and receiver; the reference composition
releases/disables them. Word-entry preflight precedes inputs, oracle execution,
DATA tails and media changes. Insertion, withdrawal and power-off also preflight
contacts before mutation. The existing head-arrival policy still waits for two
startup buffer clears after firmware MOTOR. At that same logical event, source
publication uses the selected physical track's corner state, CRC consumes the
resolved pair, and transport's head Boolean becomes the compared receiver image.

Startup acknowledgement, first READY, nominal 28 ms/record phase accumulation,
34-record lifecycle, CRC FIFO rules, read handling and magnetic data serialization
policy are unchanged. Completed exit and power-off open/rearm both contacts.
Physical head geometry, WPS/HDS edge separation, contact bounce, insertion speed,
magnetic sense and exact DATA/PHI/internal CRC mutation timing remain unresolved.

## Regressions

Seven ordinary library tests cover WPS before/at/after HDS closure, rearming after
open/reclose and rejection of held-ground reuse; every foreign source and polarity
on both pins; missing/replaced/tampered source and atomic receiver failure; all
four metadata/write-permission combinations including source-authoritative inhibit
and grounded service bypass; head mismatch before buffer/media mutation; and legacy
sixteen-pin captures retaining missing HDS/WPS observations as unknown. Write-mode
selection and READY acknowledgement use actual delayed IS/SYNC CRC words.

A feature-only capture test proves complete pair publication, no intermediate pair,
no extra rows on unchanged proposals, zero added PHI ticks and exact TSV roundtrip.
Its delta-order coordinates are software events, not physical simultaneity evidence.

Five native tests cover pending RAM-tail preservation/retry; every foreign source
before word/card mutation; selected-end corner state, fault-safe power-off/withdrawal,
rearming and reset/reference release; rejection of head-observer substitution; and
real program-mode firmware reaching F7 on either protected end without any CRC DATA
write or changed card, compared word-for-word to the independent reference.
The existing full 34-record real-firmware write/read regression additionally checks
resolved HDS and retained permission after every word, including actual card exit.

## Capture and performance discipline

HDS/WPS append at indices 16/17, preserving all previous indices and driver masks.
Exports contain eighteen optional named pins. Legacy omitted values remain None
for level and ownership; re-export stays unknown. See PIN_CAPTURE_v1.4.md.
Default builds still compile recorder storage/hooks out.

No per-bit participant or transition is removed/added: each word still has 56 cells
and 224 PHI transitions. Sources publish only at changed physical events; preflight
runs at word boundaries. Fixed staging storage grows to 18 x 13 = 234 entries,
within u8 marker capacity; a compile-time bound now prevents future silent overflow.
No persistent acceleration cache or zero-cost/speedup claim is introduced.

M16H baseline: live 2.777 us/word (115.25x), best 2.748/worst 2.799; full no-DATA
structural 1.338, IS fetch 0.933, PHI 0.355, autonomous RAM 1.088. The live idle
benchmark includes reader preflight/retained receiver but does not measure active
card throughput. Synthetic reference workloads do not claim production head/WPS
throughput. Measurement order, fixtures and workload sizes remain unchanged.

Owner-only tools/gate_m16i.ps1 retains the validated nine-stage gate: fmt, focused
library/native both builds, compiler-warnings-denied all-target tests, release
binary builds, both ignored 12/12 diagnostics, serial electrical/DATA/RAM/live
benchmarks. Actions performs fmt/check/save only. No local runtime result is claimed.

## Owner acceptance — 2026-10-30

Exact 774f0eb408e98da9469204ec7f350f99fa4bd6e6 is owner FULL GATE GREEN. Both diagnostic builds 12/12 with unchanged cycles. Live median 2.703 us/word (118.37x), best 2.674/worst 2.728; full structural 1.313, IS fetch 0.923, PHI 0.350, stage 0.660, staged scheduler 0.673, autonomous RAM 1.094. This run has about 2.7% lower live time than M16H 2.777, without isolating implementation cost from host/run variation. Exact integration and continuation are authorized by the green marker.
