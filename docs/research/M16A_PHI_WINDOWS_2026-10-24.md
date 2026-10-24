# M16A — PHI windows and diagnostic build separation, delivery 4/15

Date: 2026-10-24
Base: owner-validated M15Z `38a64171323a70586048dfc6e689e0e6567771ed`, integrated into main.
Branch: `agent/m16a-timing-observer`
Status: implemented; owner local gate pending in normal and diagnostic builds.

## Measured owner regression and scope

M15Y -> M15Z owner medians: PHI backplane 0.358 -> 0.623 us/word,
full no-DATA structural row 1.366 -> 1.760 us, live idle 2.955 -> 3.336 us.
These are sequential owner runs, not an isolated profiler attribution. The
M15Z code added recorder-option checks to every committed PHI/publication even
when no recording was requested. Removing diagnostic hooks from the normal
build is a concrete response; no speed recovery is claimed before the new gate.

## Completed block

pin-capture is an explicit diagnostic build feature. Default backplanes contain
only the electrical fabric; recorder fields, methods and per-publication calls
are compiled out. A default capture request fails explicitly, never silently
records or emits a bogus file. Diagnostic builds retain the full M15Z path and
all original tests. Offline import/audit/comparison remains available in both
builds. No bus, High-Z, contention check, device work or PHI edge is removed.

The analyzer now retains all four actual PHI observation rows and coordinates.
The new window auditor computes nearest observed setup/hold separations for each
selected pin at all four edges, preserves censored/unknown bounds and same-
publication ambiguity, reports PHI1-low interior and boundary changes, and
counts phase-value disagreements. --timing-pins closes the native workflow.
It uses no interpolation, automatic edge choice, physical-time conversion or
profile installation. See docs/PIN_CAPTURE_v1.1.md.

## Evidence review

Existing HP-67/HP-97 page-70 and service timing evidence remains authoritative.
The reviewed Laporte primary bench report at
https://archived.hpcalc.org/laporte/Woodstock/ws_HW_main.htm describes HP-21/25
Woodstock timing with approximately 0.7 us pulses and 180 kHz/310 us words.
These approximate family bench values are not an exact HP-67 four-interval
calibration. The owner accepts HP-97 equivalence, but it does not turn a raster
or family approximation into exact pulse widths or receiver edges. The large
HP-97 service PDF was not re-retrieved by the search fetcher; no new waveform
claim is made from that failed retrieval.

Synthetic test margins and all reported input timestamp separations are not
chip setup/hold requirements or causal propagation delays. Exact PHI/receiver,
DATA stage/chip mapping, display propagation and magnetic timing stay
SOURCE-BLOCKED until actual applicable evidence closes them.

## Acceptance

Four offline library regressions check explicit synthetic margins at every
edge, unknown/censored/coincident observations, PHI1 windows and ordinal delta
order, anchors and illegal topology. Conditional backplane tests prove the
normal build has no recorder storage/side effects and the diagnostic build keeps
all PHI/control observations. Native CLI tests run offline analysis in both
builds and require default capture refusal or diagnostic capture-to-audit.
All M15Z tests remain required under pin-capture. Full all-target checks, release
builds and ignored 12/12 diagnostics run in both modes; benchmarks measure the
normal build. Actions is formatting/check/save only. No runtime pass is claimed.


## Owner validation and integration

Owner reported M16A FULL GATE GREEN (delivery 4/15), exact
685407cf7739011478bd5eb5f643da27cd83bab9 integrated. Both diagnostic
builds pass 12/12 with unchanged cycle counts. Normal-build medians:
PHI backplane 0.362 us/word, full no-DATA structural 1.351 us/word,
autonomous RAM fixture 1.176 us/word, production live 2.845 us/word
(112.48x nominal hardware). Versus M15Z live 3.336 us this is 14.7%
less time; versus pre-capture M15Y 2.955 us it is 3.7% less time.
Host-run comparisons are engineering measurements, not calibrated timing evidence.
