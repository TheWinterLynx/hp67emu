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
| PHI and net resolution | Four transitions per cell, staged drives, ownership and trace analysis | Widths, dead time and propagation need hardware evidence; generic scheduler cross-tick identity ownership still needs a dedicated regression review |
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
