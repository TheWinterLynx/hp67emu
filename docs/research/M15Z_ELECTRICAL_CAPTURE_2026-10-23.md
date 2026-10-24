# M15Z — full electrical capture and physical-time audit, delivery 3/15

Date: 2026-10-23
Base: owner-validated M15Y `ae0865b0e33a1eead2ed0798ae178ae0deb06c33`, integrated into main.
Branch: `agent/m15z-electrical-capture`
Status: owner FULL GATE GREEN; exact `38a64171323a70586048dfc6e689e0e6567771ed` integrated into main. Delivery 3/15 complete.
Diagnostic 12/12 unchanged cycles; live median 3.336 us/word (95.92x hardware).
The observed disabled-recorder regression is addressed in M16A; recovery awaits its local gate.

## Completed lane

A bounded opt-in backplane recorder observes all fourteen named nets and high/
low driver masks after each committed publication and PHI edge. Same-tick delta
order preserves controls published after a PHI checkpoint. Joint source bundles
have one snapshot, never a half-updated bus image. Disabled recording enumerates
no pins and allocates nothing. Failed words retain their electrical prefix;
overflow is explicit and prevents truncated success. Legacy IS-only traces and
all 56-cell/224-transition behavior remain unchanged.

The strict v1 importer separates ordinal emulator coordinates from caller-
declared measured/synthetic integer nanoseconds, requires provenance and a bit
anchor, forbids physical driver claims and preserves unobserved/floating/
contention. The analyzer validates low-going PHI sequence/non-overlap, reports
all four interval distributions and low-pulse durations in input units, decodes
IS/SYNC and wrapped DATA with masks, and reports control cell positions.

Native --trace-pins / --audit-pins / --compare-pins close the full workflow.
Comparison uses explicit complete-word ranges and pin lists, distinct mismatch/
incomplete statuses, bounded examples and owner coverage; output paths are
create-new. No interpolation, automatic alignment, time conversion or profile
installation is hidden in this lane. See docs/PIN_CAPTURE_v1.0.md.

## Evidence boundary

Existing M14A/M15A/M15H timing evidence applies unchanged: active-low
nonoverlapping PHI, 56 cells, established IS and DATA bit coordinates, provisional
PHI1-rising sample policy. HP-97 equivalence remains accepted. M15Z adds a way
to inspect recordings, not a new hardware waveform. Input source labels are
caller claims and acquisition error/threshold/resolution must be documented.
Synthetic 2/3/5/7 ns tests are explicitly not measured HP-67 timings.

Exact PHI widths/dead times, receiver edges, propagation, DATA electrical stage/
chip mapping and magnetic timing remain SOURCE-BLOCKED until actual evidence
is reviewed. Sampling equality and 12/12 functional agreement do not close them.

## Acceptance

Seven library tests cover full pin/mask roundtrip and atomic bundles, synthetic
nonuniform timing, illegal clocks, malformed inputs, comparison coverage and
ownership, DATA wrap/qualifiers/display cells and capture bounds. Four native
tests cover CLI export/audit/compare/no-overwrite/status codes and a 512-word
recording-on/off differential over ACT/RAM/CRC/display, real RAM owners and all
PHI transitions, plus failed-word prefix/reset. Actions is fmt/check/save only.
Compilation, runtime, release, diagnostics and benchmarks await owner local gate.
