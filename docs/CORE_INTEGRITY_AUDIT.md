# Whole-emulator integrity audit — first correction set

Base: main af08aab9c1af578ff000e19f02b5676a9efefef2.
This audit extends beyond the PAC branch. It does not certify the emulator as complete.
The separate PAC/UI audit remains relevant and is not merged by this branch.

## Confirmed defects corrected

| Boundary | Previous behavior | Correction and regression |
|---|---|---|
| ROM provider to aggregate/five-chip fetch | An invalid 11-bit word was masked into a valid instruction; failure after the final address bit could leave partially changed receiver state | Reject oversized/missing words before final publication; retry remains possible; test all three receiver modes |
| ACT RAM topology configuration | Reinstalling overlapping addresses erased existing registers; ranges silently truncated | Validate whole range, return an error on overflow, preserve existing contents; test overlap and overflow atomicity |
| PHI trace audit | Partial traces could produce success despite missing phases | Incomplete trailing sequence or unobserved requested phase produces exit 3; complete constant-signal control remains accepted |

These are boundary/integrity defects. They do not demonstrate that canonical firmware supplied corrupt ROM words or that production repeatedly installed RAM. Their severity must not be inflated into observed calculator failures.

## Subsystem review and remaining boundaries

| Subsystem | Reviewed responsibility | Remaining work / evidence boundary |
|---|---|---|
| ACT execution and registers | Received instruction pipeline, serial execution, architectural comparisons | Instruction-boundary oracle and authority commits still exist; do not claim every register mutation has physically proven intra-word timing |
| ROM and banks | Five endpoints, IS/SYNC bank latches, source word validation | Physical launch/latch timing remains provisional |
| RAM and DATA | Autonomous command/address/storage/read drive, topology API | Physical RAM chip mapping and DATA output stage/polarity remain source-blocked |
| PHI and net resolution | Four transitions per cell, staged drives, ownership and trace analysis | Widths, dead time and propagation need hardware evidence; generic scheduler cross-tick identity ownership is corrected in the follow-up branch (owner gate pending) |
| Display | ROM0 serial decode, STR/RCD, cathode/anode/exposure | Unknown transient display modifiers have a documented conservative visual fallback; optical integration and exact STR/RCD timing are not certified |
| Keyboard and selector | Physical contact mapping and matrix model | Production F1/KC acquisition remains incomplete; direct ACT contact sampling is a working approximation |
| CRC and card contacts | Autonomous control, logical DATA buffers, resolved contacts | CRC buffers are not magnetic sense circuitry |
| Card transport | Record cadence, double buffering, physical card state | Head/startup handshake remains an integration policy; magnetic flux serialization is not the production read path |
| Power/reset | Live boot/reset and retained physical media boundaries | Exact electrical reset waveform needs evidence; host media preservation reviewed separately in PAC branch |
| Host/PAC/UI | Previous branch audit covers media gestures, filenames, pacing and fault recovery | Needs integration validation with these main-based changes; this branch does not contain those unmerged features |
| Diagnostics and benchmarks | Real firmware diagnostic suite and workload scope labels | Passing diagnostics and high speed are not pin-level hardware certification |

Do not replace these explicit unknowns with guessed electrical values. Follow-up work must close individual ownership/timing boundaries with evidence and causal regressions.

## Validation and performance

Source diff and whitespace reviewed locally. Rust compilation, tests and benchmarks have NOT run in this agent environment (no Rust toolchain).
GitHub automation is formatting-only. The owner gate performs focused regressions, all-target tests, release build and the ignored diagnostic suite.
Five-chip staging occurs only on the final address bit, not on all 56 cells. Performance has not been measured; no no-regression claim is made.


## Follow-up: persistent scheduler output ownership

The generic scheduler only detected duplicate output identities within one tick.
A second device could reuse the same (net, driver) on a later tick and overwrite
the original drive. External stimuli could similarly replace device outputs.

Correction: retain output ownership across ticks and High-Z releases, reject
external/device aliases in both directions, and retain same-owner updates.
Regressions cover alternating ticks, both device orders, released outputs,
unchanged net/tick/trace on rejection, and valid external release/reassertion.

This scheduler is reference infrastructure. The HP-67 production dense fabric
does not call this implementation; no map lookup is added to its hot path.
No physical timing assumptions are changed.
Previous correction set: owner reported CORE INTEGRITY AUDIT FULL GATE GREEN,
12/12 diagnostics, and validated head ce4b6565 is now integrated into main.
Follow-up: formatting automation only; Rust tests remain owner-local.


## Follow-up: firmware layout and trace coordinate validation

Scheduler ownership follow-up: owner reported the full gate green, including
12/12 firmware diagnostics. Validated head 5c6f6e0f is integrated into main.

New findings:
- Firmware initialization checked only aggregate length. A short block followed
  by a compensating long block could shift ROM addresses without rejecting the
  image. Initialization now requires exactly 512 words per block.
- ROM provider lookups masked oversized addresses into the 12-bit space. Both
  lookup methods now reject out-of-range addresses with None.
- In-memory pin analysis required monotonic pairs but accepted missing ordinal
  events or nonzero order at a new timestamp, unlike TSV import. Both paths now
  share the same coordinate validator.

Regressions exercise compensating block sizes, short/long final blocks, invalid
octal/oversized words, out-of-range addresses in both bank views, and malformed
in-memory event sequences alongside import rejection and a valid delta control.
Canonical firmware blocks were independently counted with Python: 10 x 512;
no firmware data changes. Existing exhaustive valid-address serial ROM tests
remain in the gate.

Performance: block validation runs once during firmware initialization; trace
checks are offline. Address bounds add a branch to provider lookup; performance
has not been measured and no speed claim is made.
Formatting is checked by the authorized formatting-only action. Rust compilation,
tests and diagnostics for this new set remain pending the owner-local gate.


## Follow-up: media and diagnostic output integrity

Firmware/trace validation head 2880b833 passed the owner full gate and is integrated
into main. This follow-up is based on that validated head.

Confirmed findings and corrections:
- Internal track writes used debug_assert for index and 28-bit data bounds.
  In release, an oversized word could enter media and later lose high bits when
  serialized; an out-of-range index on blank media materialized it before the
  bounds panic. Release-enabled preconditions now fail before any media change.
  Existing CRC producers already validate their words; this is defense of the
  internal media invariant, not evidence of an observed firmware corruption.
- Legacy IS diagnostics used canonical path comparisons followed by truncating
  writes. Different hard links to one input bypassed that protection. Capture,
  compare and decode now all create new outputs exclusively, as pin diagnostics
  already do. Existing reports must be explicitly removed or given a new name.
- Legacy IS reads checked file metadata then read without a stream bound.
  They now share the bounded reader used by pin diagnostics.

Reviewed without semantic changes: native card version/length/reserved/flag checks,
HPP checksum mirror and declared-length warning, CRC range/FIFO checks and physical
write-permission gate. HPP dummy nibbles and unrecorded native payloads are ignored
by documented format policy; they were not reclassified as corruption.
Partial physical writes remain physical writes: no host rollback of a real card
pass was added. Magnetic sense timing and record serialization remain source-blocked.
The separate PAC branch's title-based pairing fixes remain pending integration;
this branch does not duplicate them.

Validation: static diff review and whitespace check completed. Rust tests remain
owner-local; formatting-only GitHub action is authorized. The gate now also runs
focused library tests under release to exercise production assertions.
Performance impact is two invariant checks per written 28-bit card record, outside
the CPU per-cell path, plus diagnostic-only I/O changes. No benchmark claim.

## Whole-emulator resumption checkpoint (2026-11-06)

See [the versioned audit roadmap](WHOLE_EMULATOR_AUDIT_ROADMAP_v1.0.md) for exact
main/pending/PAC heads, confirmed host/API findings, evidence boundaries and
substantial follow-up packages. Historical pending statuses above describe each
entry when written; the new revision table is the current status reference.
The media/trace source batch still awaits the owner gate. This follow-up changes
documentation only and does not certify Rust execution or hardware equivalence.

## Implementation follow-up: duration, reset and topology boundaries

Based on roadmap checkpoint `6c1de947`. Added source corrections for A02 (normal
live pacing), A04 (public reset API) and A05 (generic topology installation):

- Accumulate submicrosecond host time instead of dropping it per call. Compare a
  whole duration with 500 ns and irregular partitions through real firmware.
- Reject reset before mutation while a physical card remains in the transport.
  Compare rejected-reset execution with an untouched control, then recover the
  card explicitly and reset. Existing selector/contact-only reset fixtures remain
  valid; contact assertion alone is not an owned physical card.
- Reject duplicate generic net IDs instead of clearing drives while retaining old
  ownership. Cover device/external sources, passive bias, ownership, tick and trace.

Four new regression tests; focused binary checks added in debug and release.
Static caller/type/whitespace review performed. Rust compilation and execution
remain pending the owner's gate; formatting-only automation is permitted.
These additions do not close A02 for the separate PAC unlimited implementation,
A03 for main UI fault recovery, A04's complete mechanical/electrical reset model,
or any source-blocked timing claim. The roadmap remains open.
Runtime cost: host remainder arithmetic per advance call, one reset precondition,
and duplicate detection only when configuring the generic scheduler. No new work
is inserted into the production per-bit electrical path. No speed measurement made.

## Read-only causal audit, 2026-11-06

The owner's current instruction is audit-only until they return to their PC.
[Findings v1.1](WHOLE_EMULATOR_AUDIT_FINDINGS_v1.1.md) follows startup/input,
frame pacing, unlimited policy, FIFO transport, retry budgets and ownership
provenance. It includes eight findings/limitations with exact inspected revisions,
source paths, counterexamples, existing-test scope and future reproduction plans.
No source/tests/scripts/workflows change in this documentation update. No Rust
execution or new hardware evidence is claimed. Existing fixes remain pending.
