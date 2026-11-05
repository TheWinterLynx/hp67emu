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
