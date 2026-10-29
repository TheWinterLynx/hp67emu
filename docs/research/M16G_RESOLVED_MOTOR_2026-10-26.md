# M16G — resolved CRC MOTOR control, delivery 10/15

Date: 2026-10-26
Base: owner-validated/integrated M16F 2186b6bc1e524fe784e51ef2051224109b80b4a2
Branch: agent/m16g-resolved-motor
Status: owner FULL GATE GREEN v2 on b2fc0de22c464b3a3eba56f1a3d41f8556b4e5e6; exact validated head integrated into main (delivery 10/15).

## Evidence inspected

HP-67 logic PCB schematic, PDF page 1 (logic sheet 2), identifies CRC 1820-1751
and its barred MOT output leading to the barred Motor card-reader connection.
Downloaded, rendered, rotated and visually inspected:
https://www.hpcc.org/cdroms/schematics5.1/handhelds/classic/hp67.pdf
SHA256 c567d6fdf2a924c998a395fe05d593d6466ef9bbf488ff0e9845ecb5f8b09e73.
This reverse-engineered drawing establishes target wiring identity, not transistor
construction, thresholds or measured delay.

HP-97 Service Manual, section 2-36, printed page 2-8 (PDF page 22), describes
CRC pulling MOTOR to ground to activate the sense-amplifier power stage and
card motor. Same page distinguishes MTRS insertion from HDS head arrival.
Retrieved, searched, rendered and visually inspected:
https://www.teenix.org/97ServMan_2.pdf
SHA256 7a102cd7051ed63b23b62b18f6768083f7e4abf4e607d9aa978541dfb25c5048.
Owner-authorized HP-97 equivalence corroborates active-low CRC MOTOR behavior;
no PIK/printer motor circuitry is imported into HP-67.

## Production path

Received IS/SYNC -> independently decoded retained CRC motor latch -> CRC-owned
MOTOR conductor -> strict resolved digital receiver -> card-transport motor gate.
The old latch boolean is comparison-only on production; explicit synthetic
reference composition retains its old logical path and releases MOTOR.

Motor appends to the fixed net inventory at index 14, preserving all earlier
pin/driver indices. No new driver identity is needed: CRC owns the output.
A strict sample requires one CRC contribution and binary resolution; Low means
On. A retained source records its last output so a tampered CRC slot cannot
be overwritten by a later publication. Agreeing extras, contention, missing and
replacement sources fail before new inputs, oracle execution or DATA tails.

The pre-word resolved sample gates the elapsed word interval, preserving the
existing transport convention. Independent completed CRC latches publish the
next held output at word end. No commanded motor transition moves to an earlier
interval, and no record cadence, startup handshake or head-arrival rule changes.
Unchanged levels are not published. The source is held through idle words;
MOTOR is not a test pulse like F2. Reset reconstructs normalized inactive High;
reference reset retains Floating/no source. UI logical motor inspection remains
an observer of the compared CRC mirror; it is not the transport input.

## Explicit approximation boundary

Normalized High/Low output drive is a WORKING APPROXIMATION. Barred wiring and
ground-to-activate evidence support active Low, but do not establish push-pull
silicon, open-drain release/pull bias, voltage thresholds or propagation. No
passive bias is invented. Source publication at completed b55/b0 and prior-word
interval sampling are existing logical scheduling policy, not exact physical
MOTOR/PHI alignment. No motor voltage, inertia, speed-control waveform, sense
amplification or magnetic serialization is simulated by this lane. MTRS/HDS
contacts and the existing startup/head policy are separate work. Physical motor
electronics and exact timing remain SOURCE-BLOCKED.

## Capture and performance discipline

Fifteen named pins are now exported; MOTOR includes resolved level and source
masks. Existing named sparse-column import preserves an absent MOTOR column as
None, never as Off/High/Floating. Re-export writes ? for unknown observations.
No TSV version bump is needed for the already extensible named-column contract.
See PIN_CAPTURE_v1.2.md. Default builds still omit recorder storage/hooks.

The fixed fabric/stager grows by one net; 15 x 13 = 195 pending-drive slots stay
within the existing u8 marker capacity. No new per-bit participant or PHI edge
is added. Normal work is word-boundary source/receiver checking and changed-only
publication. Existing PHI resolution and all 56 cells/224 transitions remain.
No zero-cost or speedup claim is made. M16F baseline: live 2.885 us/word, full
structural 1.314, PHI 0.347, autonomous RAM 1.098. Live benchmarks include MOTOR;
no-DATA structural fixture leaves the new net idle and does not instantiate it.

## Regressions and acceptance

Five ordinary library tests cover all 1024 opcodes, both qualifiers and initial
motor states over actual delayed IS/SYNC words; every foreign source and both
polarities; missing/replaced/tampered source and wrong phase; hold/clear behavior;
and legacy fourteen-pin import retaining unknown MOTOR. A feature-only capture
test proves On/Off changes at ordinal ticks 448/896, unchanged publication with
no extra row, CRC ownership and full export/import parity. These coordinates are
logical fixture events, not measured motor timing.

Three native tests cover preflight before input/oracle/time mutation, pending
RAM-tail preservation and retry against unfaulted firmware, actual card-present
motor start with reference parity, reset and explicit reference release. The
existing full real-firmware 34-record write/read roundtrip additionally checks
resolved MOTOR against independent CRC control after every word, exercising the
new production gate through actual card transport without duplicating that harness.

Full suites/release/both diagnostic builds and electrical/DATA/RAM/live benches
remain owner-only gates. Actions performs fmt/check/save only. No runtime result
or integration is claimed until the owner full gate is green.

## Owner gate feedback — 2026-10-28

The owner log for formatted head 70645a294f14cddc53a91dbdc12470ef89037c34 ends the focused native stage with 3/3 passed, then stops in stage 4 at Clippy (14 library findings, 49 including library tests). The pasted v1 gate had mistakenly substituted Clippy for the established warnings-denied all-target regression gate. No full-suite/release/benchmark result is available from that run.

The v2 correction is checked in as tools/gate_m16g.ps1 and follows AGENTS.md/docs/CI.md: compiler warnings remain denied; both build variants run all-target tests, release builds and the ignored diagnostic. The nine-stage order, literal -- argument forwarding and serial benchmarks are explicit. The documentation contract now includes tools PowerShell companions. Emulator source, pin scheduling, source checks and benchmark workloads are unchanged. No Clippy allowances or test exclusions were added. Owner full gate remains pending; no M16G integration is authorized by this failed run.

## Owner acceptance — 2026-10-29

Owner full gate v2 green at exact b2fc0de22c464b3a3eba56f1a3d41f8556b4e5e6. Both diagnostic builds pass 12/12 with unchanged cycle counts. Live median 2.971 us/word (107.72x), best 2.954/worst 2.987; full structural 1.312, IS fetch 0.970, PHI 0.358, stage 0.644, staged scheduler 0.679, autonomous RAM 1.077. Relative to M16F live 2.885, elapsed word time is approximately 3.0% higher; host/run noise and the new lane are not isolated by that comparison. No zero-cost or speedup claim. The green marker authorizes exact integration and continuation; M16H starts there.
