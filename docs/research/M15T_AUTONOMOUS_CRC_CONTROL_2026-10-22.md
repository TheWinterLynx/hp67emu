# M15T — autonomous live CRC control

Date: 2026-10-22
Branch: `agent/m15t-autonomous-crc-control`
Status: owner M15T FULL GATE GREEN; exact f635ea8732baad5eb38fdc88dba59a76ecab403f integrated into main. Diagnostic 12/12, unchanged cycles; production live median 2.544 us/word (125.77x).

## Completed functional scope

One integrated change installs an independent raw IS/SYNC receiver, twelve CRC
internal control latches, frozen external contacts, previous-word command
pipeline, completed-word set/test-clear result, live motor/write-mode mirror and
ACT S3 result authority. There is no host opcode or ACT-state input to the
receiver. The existing strict RAM command receiver is reused as a raw ten-bit
IS/SYNC parser; its RAM semantic decoder is not used by CRC.

Production construction/reset retain the independent endpoint. Explicit synthetic
opcode fixtures retain the reference bridge, because their injected execution
stream differs from the actual concurrent ROM return. Live first verifies all
old CRC internal latches, freezes only external contacts, runs the shared word,
compares the endpoint result with the oracle and M14L image, and copies only that
validated result into the architectural mirror. False tests preserve S3;
external levels survive clearing internal latches. Non-control/ThenGoto frames
never execute CRC commands. Incoming frames and executing commands are distinct.

## Logical buffer boundary

M14M FIFO/data-port serialization remains a separate completed-word bridge.
Only its explicit data commit and a changed post-word card-transport handshake
may update READY/F7 in the independent endpoint. Other latches cannot enter
through this port. A mirror mismatch is detected before either handoff can mask
it. No full CRC FIFO clone or heap allocation occurs on a successful bit sample.
The small CRC image and RAM command image commit only after shared DATA sample
validation; a failed DATA sample cannot consume the CRC command window.

## Evidence and replacement criterion

Command meanings, internal OR external tests, clear-only-internal behavior and
sticky S3 follow the already established CRC behavioral contract described in
M14L_CRC_CONTROL_AUTHORITY_2026-10-04.md. FIFO ownership follows
M14M_CRC_DATA_AUTHORITY_2026-10-04.md. This change moves causal ownership and
connects independent devices; it introduces no new hardware timing claim.

Qualified serial reception and the b46..b55 window use the existing IS/SYNC
contract. M15H provisional sample phase and completed b55 latch/result publication
remain WORKING APPROXIMATION. Exact F2 wiring/pulses, CRC PHI latch/launch edges,
propagation and CRC DATA/magnetic serialization remain SOURCE-BLOCKED. Replace
those schedules only with source-backed hardware timing; HP-97 equivalence is
accepted by the owner, but a numerical edge is not inferred from semantics.

## Acceptance

New library regressions exercise all 1024 words with both qualifiers and all
internal/external boolean combinations, one-word latency, consecutive set/test/
clear, contact persistence, pin mutation, phase/order faults and unchanged
images on failure, including a joint DATA failure. Native regressions compare
real firmware against reference and traced production, card-present motor,
reset, poisoned mirror rejection before clock advancement, and actual logical
card READY/data-port consumption. Existing card write/read, diagnostics and live
benchmark are retained in the owner gate. GitHub runs formatting only; no build,
runtime, diagnostic or benchmark result is claimed before local acceptance.
