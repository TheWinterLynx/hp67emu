# HP-67 architectural closure v1.0

Decision date: 2026-11-11. Validated baseline: 896cd664bedfa629b83eaff257f6675b98c551ea,
owner CORE INTEGRITY AUDIT FULL GATE GREEN, integrated into main.
This is the active release closure plan. Older audit reports remain historical evidence.

## Deliverable and precision contract

Deliver a usable desktop emulator executing real microcode, with deterministic
structural transport, retained magnetic media, explicit faults and stable controls.
Nominal pacing is the reference operating mode. Optional unlimited CPU executes
complete additional words without accelerating reader time or hiding ROM timeouts.
It is not a physical overclock certification or an I/O compatibility guarantee.

Keep 56 cells and 224 PHI transitions per word, resolved ownership/High-Z/contention,
serial ROM/RAM/CRC paths and existing reference comparisons. Host UI must not decide
calculator results or firmware card flow. Diagnostic success establishes functional
regression coverage, not exhaustive correctness.

Exact PHI widths/dead time, physical launch/latch edges, DATA output stage/polarity
and RAM-chip mapping, propagation and magnetic sense/serialization remain
SOURCE-BLOCKED. No invented timing is a prerequisite for a functional release.

## Architecture to retain

| Boundary | Owner and rule |
|---|---|
| Generic digital primitives | src/emulation; no UI, artwork or calculator policy |
| HP-67 devices | src/machines/hp67; explicit pins, state and source-backed contracts |
| Live composition and pacing | Hp67LiveMachine in src/hp67.rs; production execution, elapsed-time adapter, fail-stop and retained-card recovery |
| Reference execution | Comparison/diagnostic oracle; never silently replace a failed production result |
| Host files | Import/export validation and metadata; filenames never define magnetic/electrical behavior |
| Desktop | Physical contacts, presentation, card artwork and documentation navigation |

The live composition currently resides in the binary adapter. Do not describe it
as a temporary mock or move it wholesale for cosmetic layering during stabilization.
A later library extraction needs API and headless parity tests; it is not itself
evidence of more faithful emulation.

## Three closure batches

| Batch | Work | Exit evidence |
|---|---|---|
| 1. Stable baseline and acceptance | Integrate validated core; correct stale architecture status; establish one logged candidate gate | Baseline integrated; new gate reviewed, owner gate green at 7544c45 |
| 2. One desktop product | Reconcile PAC/library/PDF/artwork features with the audited core; resolve B06 capability provenance and A06 error-contract exposure where reachable; finish conversion-tool documentation coverage (A08) | One main-derived branch; no reintroduction of PAC clock heuristic; focused tests and full gate green; every residual finding explicitly classified |
| 3. Release candidate | Run desktop acceptance, compare performance against baseline on same hardware, correct release blockers, package/document one build | Exact commit, automated log, manual checklist, performance comparison and documented limitations |

This is a finite scope, not a promise that three prompts suffice. Add work only for
a reproduced release-blocking defect or an unmet acceptance criterion. Do not reopen
new chip/timing redesign or remove reference checks solely to improve speed.

## Remaining integration work is real

The separate agent/hp67-pacs-imported branch is not part of the validated baseline.
Its embedded PDFs, per-program sections and richer card assets must be reconciled
selectively; blindly merging its old src/app.rs and src/hp67.rs would risk restoring
fixed audit defects and repeated-empty-CRC-poll pacing.

B06 and A06 are unresolved audit findings, not waived because the gate is green.
For each, record whether production can reach it, the invariant, regression and
fix, or the evidence-backed reason it is limited to an unsupported construction.
The ledger must not silently turn pending findings into completed work.

## Acceptance checklist

- [x] Core audit gate reported green by owner at the baseline above.
- [ ] Consolidated PAC/PDF desktop runs from the release executable.
- [ ] Power, keyboard, RUN/PRGM, program library and alphabetical menus behave correctly.
- [ ] Card insert/read/write/eject, both orientations and write protection work at 1x.
- [ ] Fault/power recovery preserves partially written media and allows saving it.
- [ ] Unlimited toggling preserves animation/reader time; its timeout limitation is visible.
- [ ] PDFs open at the intended program section; artwork remains legible at intended zoom.
- [ ] Final candidate passes tools/gate_release_candidate.ps1.
- [ ] Same-machine release timings compared with baseline; material regressions investigated.
- [ ] Final residual audit ledger, operating instructions and build are aligned.

No new broad PAC campaign is required; retain existing diagnostics and targeted
regressions for changed behavior. GUI acceptance cannot be replaced by a headless
test or inferred from compilation.

## Reproducible gate

Run tools/gate_release_candidate.ps1 from a committed candidate. It calls the
existing core audit gate then existing release benchmarks and saves a transcript
in target/release-gate with the commit ID. Tracked edits are rejected because a
commit ID alone cannot identify an uncommitted test binary. Untracked files are
not removed. Failures stop the run. Benchmark output is evidence, not a hardware
fidelity certificate.

Validation of this closure batch: documentation and script inspection only.
No PowerShell/Rust runtime is available in the agent workspace; the new wrapper
has not been executed here. No performance result is claimed.

## Desktop consolidation (2026-11-11)

Desktop consolidation branch now selectively imports PAC catalog/media, embedded manuals and high-resolution artwork from agent/hp67-pacs-imported. The production hp67.rs is byte-for-byte the validated baseline; bounded card_import and fail-stop recovery remain. PDF rendering raises the declared Rust minimum to 1.92 (existing PAC dependency contract). This integration is pending the owner gate and GUI acceptance. B06, A06 and A08 remain open; this delivery does not close them.

## Validated performance baseline

Owner gate 7544c45, 2026-11-11: production idle median 2.915 us/word
(343100 words/s, 109.79x nominal reference); synthetic RAM reference 2.276 us/word;
synthetic CRC reference 2.113 us/word. These are distinct workloads. Preserve the
owner transcript as the comparison baseline; no delta for the desktop integration
has yet been measured.

Integration checks performed here: embedded file references resolved against Git
trees, 232 full-resolution artwork mappings checked, PDF section metadata compared
with the reviewed profiles/manifest, diff whitespace checked. Asset Git objects
are reused unchanged from the PAC branch. Compilation, raster smoke, UI and
performance validation remain pending on the consolidated candidate.
