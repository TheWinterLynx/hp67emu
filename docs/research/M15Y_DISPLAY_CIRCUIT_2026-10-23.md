# M15Y — pin-derived display circuit, delivery 2/15

Date: 2026-10-23
Base: owner-validated M15X `29b99098e517963d43f2bc08374cfc4505c88e65`, integrated into main.
Branch: `agent/m15y-display-owned-scan`
Status: owner FULL GATE GREEN; exact `ae0865b0e33a1eead2ed0798ae178ae0deb06c33` integrated into main. Delivery 2/15 complete.
Diagnostic 12/12 unchanged cycles; live median 2.955 us/word (108.27x hardware).

## Completed scope

ROM0 proposes only STR after reconstructing b0..b7 IS. ACT proposes RCD from
its own scan phase. A single committed bundle publishes both named outputs;
no receiver sees an intermediate handoff. The ordered receiver stages data and
controls together, validates resolved PHI/checkpoint and every coordinate,
rejects extra or wrong sole owners, and records a raw byte/RCD/STR-tick event.
The event contains no ACT scan index and cannot be manufactured through public
field construction. All four transports and the clocked DATA prefix use it.

The library display circuit owns cathode phase, retained anodes and nominal
complete-scan exposure. It selects the scan role from its own counter and checks
RCD overlap independently. Replayed or skipped events, invalid decode, reset
or exposure faults cannot partially mutate it. ACT slot/transport byte remain
comparison observers. A pre-word phase guard rejects poisoning before clocks
or a pending DATA tail. The desktop adapter consumes emitted segments/dwell;
its old decoder/cathode/exposure ownership is removed. Reset and startup/inhibit
and optical scan-floor policies retain their established behavior.

## Evidence boundary

Existing Teenix HP-67 captures (classic-notes.pdf printed pp. 76-77), M15G/M15H
and display source notes establish fifteen roles, low-going STR at b7, RCD
final-slot overlap and observed nominal 40/30 us dwell. HP-97 equivalence remains
accepted. This change adds source/receiver ownership and fault/order checks;
it introduces no new physical timing value.

The one-cell pulse bridge, provisional PHI1-rising checkpoint, completed-word
cathode application and nominal exposure remain WORKING APPROXIMATION. Exact
STR/RCD widths, launch/sample edges, propagation, internal edge priority and
current/brightness remain SOURCE-BLOCKED. Ordinal event cadence is not elapsed
microseconds. Unknown display codes remain failures; the existing serializer's
unsupported-modifier blank fallback is unchanged.

## Acceptance

Nine new regressions cover 45 scan words and masks/dwell, replay/missing/skipped
strobes, early/missing RCD and atomic decode errors, both control nets with
agreeing/opposing extra owners at every bit, wrong phase/replay/floating/wrong
sole owners, all four transports with intentionally poisoned slot annotations,
and real firmware/tracing/keys/DATA tails/reset/optical publication. Native
phase corruption is rejected before clocks. Host frame edits cannot feed optics.

Actions is fmt/check/save only. Compilation, runtime, release, diagnostics and
benchmarks await the owner local gate; no runtime pass is claimed here.
