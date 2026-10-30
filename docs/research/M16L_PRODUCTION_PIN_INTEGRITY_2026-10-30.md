# M16L — offline production pin integrity, delivery 15/15

Date: 2026-10-30
Base: owner-validated/integrated M16K v2 3920d01adee5683a0038e17948cabc2c433149da
Branch: agent/m16l-pin-integrity
Status: implementation prepared; owner full gate pending.

## Implemented gap

The capture parser validates masks against levels, but a valid High can still
carry an agreeing foreign contributor or the wrong sole named source. Existing
topology/timing analysis does not diagnose the source role of CRC/contact/IS/DATA
pins. M16L adds a library audit and native CLI that report those violations and
incomplete observations without a complete-hardware certificate.

It validates every ownership image for direct API callers, uses the existing
strict PHI topology analyzer, checks PHI owners at every row, then checks eleven
other production pins at the model's provisional PHI1-rising checkpoint. The
thirteen source contracts follow current production code, not new physical
measurements. IS display/address/idle/return roles retain passive Low release;
ROM return candidates use only the observed address page and existing profiles.
Bank state is not guessed from the responding owner and is not certified. DATA
checks release or sole ACT/aggregate RAM ownership, not semantic direction or bits.

All nineteen pins appear. F1/KC acquisition is unassessed. A scoped exit 0 requires
all thirteen contracts with complete known levels/owners, complete addresses and
whole-word capture; 2 reports violations and 3 incomplete coverage. Physical
nanosecond traces correctly lack internal masks and cannot pass ownership from
level observations alone. Missing contacts remain unknown. Violation counters
are unbounded by the 64 stored details; input is immutable and output create_new.

## Evidence classification

No new hardware-visible timing/semantic claim is introduced. Source roles inherit
M16B/C/F/G/H/I/J, HARDWARE_SOURCES and current executable guards. IS window order
and passive Low have direct evidence; ROM page/part assumptions retain their
explicit HP-97 equivalence/inference classification. Binary CRC/contact levels,
completed-word scheduling and PHI1-rising checkpoints retain working-approximation
status. Offline checking does not promote them to exact physical edges.

SOURCE-BLOCKED remains: exact PHI pulse widths/dead time, physical IS/DATA/F2 and
STR/RCD launch/latch offsets/propagation, DATA analog output/polarity and physical
RAM map, magnetic sense/serialization and ACT scan acquisition. Internal ACT
completed-word mutation policies and existing visual integration approximations
are not calibrated by this audit. Delivery 15 closes the agreed delivery sequence
only after owner acceptance, not the full hardware-fidelity objective.

## Regression coverage

Six library tests run in both feature variants: complete synthetic source images
and TSV roundtrip; all agreeing foreign/replacement drivers on eleven sole-owner
nets; IS roles, passive release versus active zero, wrong page/reference source;
DATA foreign/agreeing overlap and allowed sole contributors; missing contact/
ownership/address and partial/physical coverage; malformed direct images and
bounded detail with complete 112-fault counts. Inputs are compared unchanged.

Native CLI tests check routing, argument errors, output preservation/no overwrite
and incomplete sparse nanosecond input in both variants. A feature test audits
sixteen actual firmware words, injects agreeing PGMSW ownership and removes MOTOR
for the distinct 0/2/3 statuses. Another actual live feature test captures 256 words
after each PROGRAM/RUN change and after physical card insertion, requiring all
14336 checkpoints and scoped pass for each capture.

## Performance and validation

All new work is offline; no production emulation loop, serial receiver, fabric,
clock, capture hook or semantic oracle changes. Normal capture remains compiled
out. Existing electrical/DATA/RAM/live and M16K kernel workloads are retained.
Owner M16K baseline: live 3.029 us/word, 105.65x hardware; same-process CRC prepared
0.485 vs 0.575 us/word. No further runtime improvement is claimed by this delivery.

Actions is exclusively cargo fmt/check/save. Tests, release builds, both 12/12
firmware diagnostics and all benchmarks remain owner-only gate_m16l.ps1. No runtime
validation has been executed by the agent; no new measured physical fixture exists.


## Owner v1 failure / v2 correction — 2026-10-30

Owner focused feature-native gate: 2/3 passed; the sixteen-word startup CLI audit failed with one STR and one RCD violation at tick 2, order 0, word 0/b0. Captured masks and Floating levels are valid observations. The existing sample_display_transport in fetch.rs first samples display data, then commits the joint control pair at b0/b7/b8, then consumes controls. The backplane recorder observes PHI1 rising before this zero-time publication and records the committed pair as a later delta at the same tick. The auditor incorrectly used the pre-publication row as the display receiver input.

v2 changes only offline observation: STR/RCD at those three publication bits use the last row at exactly the same tick, matching the current receiver boundary. All other pins and display bits use the original PHI1 edge row. Mask validation and PHI ownership still cover every row. No initial-row exemption, unknown-to-High substitution, later-tick lookahead, clock change, runtime mutation or new physical timing assertion is added. The report states this provisional display boundary explicitly.

Three additional normal-build regressions preserve the original Floating startup image, verify post-publication fault rejection for both controls/all three commit bits, and prevent hiding other receiver faults or borrowing later ticks. Native CLI expected scoped success remains unchanged. Full v2 runtime/performance gate is pending; no tests/builds/benchmarks run by the agent, and M16L remains unmerged.
