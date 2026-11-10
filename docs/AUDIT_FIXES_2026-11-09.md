# Audit corrections — implementation ledger

Baseline: pending `agent/media-trace-write-integrity` at `43edd37f14327f407a9391212bdefc205612fe7b`.
Owner request: implement the audit fixes. No integration into stable main yet.

| Finding | Implemented change | Validation boundary |
|---|---|---|
| A01 | Complete temporary sibling write + sync + rename; no truncate/delete of destination | Added injected write/replace error tests; Windows gate pending |
| A03/B08 | Consuming fault recovery preserves card, dirty records and insertion end; UI reports error | Added API partial-write/contact-fault and UI recovery tests |
| B01 | No-input checkpoint timeout restricted to diagnostic fixtures | Held/released early-key real-firmware regression beyond 2,000 words |
| B02 | Advance old state before current frame inputs; power anchor at event boundary | Added card boundary regression; no sub-frame OS replay claim |
| B04/B07 | Empty FIFO phase policy consistent at entry and exhaustion | Independent expected write timeline and partition tests at 95/100/105 percent |
| B05 | Public pacing is terminal on error until explicit reset | Fault, no-retry/no-new-debt, reset-to-fresh regression |
| UI replacement | Failed new-blank insertion preserves prepared card | Included in UI fault recovery regression |

Existing fractional pacing, occupied-reset and duplicate-net corrections remain
in this branch and still require the owner's gate. The write-underrun rule is an
existing logical approximation, now internally consistent; actual mechanical
underrun remains source-blocked. The shared CRC observer is retained and not
misrepresented as independent transport timing evidence.

## Remaining work

- A07: explicit HPP pairing/import identity and bounded host file input.
- B03/A02: reconcile the separate PAC unlimited-clock branch, its heuristic and
  fractional budget. This batch does not merge that branch or certify its policy.
- B06: composition-bound dense driver capabilities; no per-edge maps/global ID
  introduced to disguise the ownership boundary.
- A06: document/cover other scheduler and device post-publication error contracts.
- A08: extend documentation coverage to conversion tools.
- Broader ACT authority/edge-case campaign, physical trace validation, corpus,
  PDF/artwork and performance audit remain open. No percentage completion claim.

## Validation

No Rust compiler/cargo is installed in this workspace; no compile, tests, release
build, GUI run or benchmark result is claimed. The existing GitHub workflow is
used only for rustfmt. The existing `tools/gate_core_audit.ps1` selects new
`core_audit_` regressions in debug/release, all targets, pin-capture, release binaries
and the ignored real-firmware diagnostic PAC. Owner must run it on the published
head before merge. No measured performance delta is available; no per-bit tracing,
map, allocation or new oracle was added by these fixes.

## Follow-up 2026-11-10 — A07
Prior fixes through c8f0a20 received owner CORE INTEGRITY AUDIT FULL GATE GREEN
and were integrated into main. New branch agent/card-import-integrity starts there.
A07 implementation now removes title-based physical pairing, supports explicit
same-drop HPP pairing and bounds host reads. Error and length-warning messages
are visible. Added regression coverage is pending the owner gate; no local Rust
execution is claimed. PAC unlimited policy and dense capability scope remain open.

## Follow-up 2026-11-10 — independent processor clock
A07 received the owner's green gate and is integrated through e37b3ba.
This main-derived branch implements CPU acceleration without the separate PAC
branch's repeated-empty-CRC-poll/PC heuristic. Extra words receive zero reader
elapsed time; normal pacing alone owns elapsed time and fractional remainders.
All 56 cells, 224 PHI transitions and reference comparisons remain.

This optional host mode changes the ratio of CPU execution to transport time.
Instruction-counted firmware timeouts may therefore expire before a physical
card operation completes. No firmware patch or hidden 1x switch masks that fact.
Use nominal speed for original I/O timing. The separate PAC branch is not merged;
its old unlimited implementation must be replaced, not layered onto this API.

Added focused core/UI regressions are pending the owner gate. Rust is unavailable
locally. No runtime, benchmark or GUI validation is claimed. Existing electrical
source-blocked boundaries and the other remaining audit items stay open.
