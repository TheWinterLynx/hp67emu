# Whole-emulator audit and resumption roadmap — v1.0

Date: 2026-11-06. Scope: core, composition, host adapters, diagnostics, tooling,
media persistence and the separate PAC branch. This is a static audit checkpoint,
not a certificate of complete hardware equivalence or a completed dynamic audit.

## Exact repository state

| Line | Inspected revision | Status |
|---|---|---|
| Stable main | `2880b833acdd8cf70eaf1f8e815e5c4cb976678d` | Owner reported full core gate green; merged |
| Media/trace follow-up | `6e56d6828e80fc762ddaf039ddbe5509cf501b3b` | Formatting follow-up to `30ac6ef2`; full Rust gate pending |
| Separate PAC/UI branch | `79c9ce8997c9d1c6abb42044d8a098caae9be7ed` | Selected host/pacing paths inspected; not merged into main |

This document is appended to `agent/media-trace-write-integrity` while the owner
is away from the Windows PC. No further source changes, merges, test/build Actions
or claims of local Rust execution are made by this documentation checkpoint.
The branch is intentionally pending. `agent/hp67-pacs-imported` is a separate
integration dependency, not an alternative definition of main.

## Audit method and limitations

Read the engineering rules, architecture/evidence policy, existing integrity
report, format contract and relevant source/companion paths. Followed the live
reset, pacing, execution/authority, keyboard, media import/save/fault and scheduler
paths. Compared selected PAC host and pacing implementations with the core branch.
Checked the scope text of pin integrity and the actual contents of the core gate.
Searched core/reference code for host clocks, filesystem/UI dependencies and
unfinished/panic/default patterns; the returned host-clock uses were benchmark
code, not evidence of host time driving production electrical state.

The findings below distinguish directly visible code behavior from a predicted
failure requiring a reproducer. Static inspection cannot establish execution
coverage, Windows file-system behavior, rendering quality, physical timing or
performance. No Rust toolchain is available in this session. No hardware captures
were acquired or newly interpreted. This is broad subsystem review, not a claim
that every source line, asset, opcode combination or fault permutation was audited.

## Open findings with concrete closure tests

### A01 — Existing card files can be truncated on save failure (high, confirmed path)

Evidence: `src/app.rs::save_card_file` serializes then calls `fs::write(path, bytes)`
in both inspected branches. Marking the in-memory card clean only after success is
correct, but does not preserve the previous on-disk file if opening/truncating
succeeds and writing later fails. No actual user data loss was reproduced here.

Implement host-only transactional persistence: a uniquely created sibling temporary
file, complete write and flush, then a platform-appropriate replace operation.
Do not delete the destination before renaming. Specify Windows replacement, existing
links, permissions, cleanup and durability guarantees; rename alone is not a proof
of crash durability. Keep the original file and dirty in-memory card on failure.
Tests: injected short write, flush/replace failure, existing target, absent target,
read-only target, stale temporary file, successful reimport byte equality. This is
host file atomicity, not rollback of a partially written physical magnetic card.

### A02 — Duration partitioning loses fractional time (medium, confirmed arithmetic)

Evidence: `src/hp67.rs::advance` converts each supplied duration with `as_micros()`
before accumulating. Two calls of 500 ns contribute zero; one call of 1000 ns
contributes one microsecond. The PAC `advance_processing_batch` does the same at
its public input. Its UI reader helper preserves fractions for its own caller,
but that does not make the live API partition-independent or repair normal pacing.

Retain a bounded submicrosecond remainder at the explicit pacing boundary. Keep
integer simulation time; no wall clock in chips. Specify reset and speed-switch
ownership so remainder time is neither lost nor charged twice. Tests: the same
aggregate duration partitioned into submicrosecond, irregular and ordinary frame
chunks must give identical completed words and remaining time, including reset
hold, reader cadence and normal/unlimited transitions. This does not authorize
inventing finer physical PHI timing.

### A03 — Main discards media on a live fault (high, confirmed ownership path)

Evidence: main `src/app.rs` sets `live_machine = None` after execution and other
adapter errors. A card owned by that machine is dropped with it. The PAC branch
has `stop_on_fault`, which consumes the stopped machine, recovers its retained
card and stores the error. It does not preserve the whole machine for replay.

Reconcile the PAC correction into the main-based integration, including mode,
power recovery and execution failures. Do not reimplement an incompatible second
fault path. Tests: injected failure with blank, protected, recorded and partially
written cards; preserve exact tracks/dirty flags/orientation and make the error
visible; no silent restart or resumption of partially committed chip state.

### A04 — Public reset can discard inserted media (medium, confirmed API behavior)

Evidence: `Hp67LiveMachine::reset_power_on` replaces `card_transport` with default.
The ordinary UI attempts to extract the card before resetting, so this is not a
claim that every normal power cycle loses it. Direct callers and failed extraction
need an explicit contract. The method also resets keyboard contacts, requiring a
clear caller responsibility for reasserting held mechanical inputs.

Choose and document either rejection before mutation when media remains inserted,
or explicit preservation/return of physical state. Do not silently eject or move a
card to make reset easy. Tests: reset at startup, idle, inserted before motor,
mid-read, mid-write and after completion; selector retained; failed precondition
leaves electronic and physical state unchanged. Keep electrical reset waveform
qualification separate from these software ownership guarantees.

### A05 — Reinstalling a generic net silently clears drives (medium, configuration)

Evidence: `src/emulation/scheduler.rs::install_net` unconditionally inserts a new
`Net`; the persistent ownership map remains. Reusing an ID therefore resets drives
and bias without resetting ownership. Observed callers install topology initially;
no production HP-67 occurrence was found. Production uses the dense backplane.

Reject duplicate topology installation, or introduce an explicitly named reset
operation with a coherent ownership contract. Test before/after first tick,
external/device drives and High-Z ownership; failure preserves level, bias, owner,
tick and trace. Avoid adding dynamic ownership maps to the production hot path.

### A06 — Failed ticks are not generally retryable (contract risk, not blanket bug)

Evidence: generic scheduler evaluates mutable devices before detecting output
collisions. It checks resulting contention after committing outputs and advancing
tick. Dense `electrical.rs::commit_staged` also reports contention after applying
all pending drives. Atomic publication does not imply full rollback on every error.

Retain fail-stop semantics unless a caller explicitly requires transactions.
Audit callers for accidental retries; document which errors precede mutation and
which stop an already-mutated machine. Add fault injection before final receive,
at final receive and after publication. Assert the promised boundary, not universal
rollback. Avoid cloning an entire machine on every cell to manufacture atomicity.

### A07 — HPP identity and unbounded host import (mixed)

Main `load_card_file` combines tracks by equal card title. The PAC branch changes
this import path; reconcile that work rather than treating filenames/titles as
physical identity. Main also calls unbounded `fs::read` before dispatching the
extension. A large file may allocate far more than native media requires.

Bound native reads to their exact format sizes plus one byte. Establish a generous,
documented compatibility bound for HPP from the supported corpus; do not reject
valid files based on guessed formatting. Reject unsupported extensions before
reading. Tests: same title/different cards, explicit pair, reversed order, duplicate
track, malformed second file, oversized input, no partial replacement of old media.
Preserve documented HPP length warnings, dummy nibble handling and checksums.

### A08 — Documentation gate omits executable Python tools (confirmed coverage gap)

`tests/documentation_contract.rs` enumerates Rust and PowerShell only, while tools
include `convert_hp67_card.py`, `generate_hp67_diagnostic_pacs.py` and
`extract_hp67_card_artwork.py`. This is a gap between policy and enforcement; it
is not proof that each omitted tool lacks documentation or is defective.

Inventory first-party executable tools, add companions where needed and enforce
Python coverage with intentional exclusions for third-party/generated directories.
Audit conversion separately: deterministic output, provenance, checked sizes,
checksum/round-trip equality, no source overwrite and artwork/media separation.

### A09 — Old milestone status is misleading (documentation defect corrected here)

The old roadmap describes physical matrix and autonomous devices as absent despite
later implementations. TODO still lists rustfmt gating as pending. Entry-point
notes now direct readers to this dated checkpoint and mark the old status snapshot
historical; rustfmt gating is marked complete. Historical evidence is retained.

### A10 — Full core gate does not measure performance (confirmed gate scope)

`tools/gate_core_audit.ps1` checks formatting, focused debug/release invariants,
all-target default and pin-capture tests, release binaries and ignored diagnostics.
It does not run the ignored wall-clock benchmarks. A green gate is not evidence
that performance is unchanged. Add explicit measured comparison to relevant work;
do not use correctness assertions with fragile speed thresholds.

## Fidelity and subsystem coverage map

| Area / source | Existing implementation reviewed or located | Remaining boundary and acceptance evidence |
|---|---|---|
| ACT: `hp67.rs`, `act_serial_*`, owned registers/flow | Received instruction pipeline, serial results and reference comparisons/restoration | Inventory every mutation's authoritative owner; trace execution through all 56 cells; source-backed intra-word observability remains open |
| ROM: `rom_devices.rs`, `fetch.rs`, `hp67firmware.rs` | Five endpoints, autonomous bank latches, validated provider words/addresses | Exhaustive bank/page transitions and invalid-provider final-bit tests; physical launch/latch edges still provisional |
| RAM: `serial_ram.rs`, `data.rs` | Autonomous command/address/storage/read drive | Prove wrapped b54/b55 tail, simultaneous fetch, clear/read/write and foreign-source faults through production; physical chip map and output stage need sources |
| CRC: `serial_crc_control.rs`, `crc_owned_data.rs`, `crc_flag.rs` | Autonomous command/control and logical buffers; staged receiver paths | Final-bit rejection, FIFO ordering, empty/full, external/internal flags and one-word delay in production; logical buffers do not prove sense circuitry |
| Electrical: `electrical.rs`, generic scheduler | Dense fixed topology, named drivers, staging, cached resolution | Preserve agreeing-foreign-owner detection as well as contention; order-independent receiver observations; define fail-stop boundary |
| PHI/SYNC: `timing.rs`, `sync.rs`, `pin_timing.rs` | 56 cells/224 transitions and trace coverage checks | Measured pulse widths/dead time, launch/latch offsets and propagation require provenance-bearing captures; do not assign guessed ns |
| Keyboard: `keyboard.rs`, `keyboard_matrix.rs` | Physical matrix representation, production `sample_into_act` sets key buffer/S15 at instruction boundary | F1/KC electrical acquisition remains unassessed; held/released contact, key rollover and bounce policy need explicit supported scope |
| Display: `display_circuit.rs`, `display_exposure.rs` | Pin-derived cathode/anode exposure and complete-scan publication | Exact STR/RCD edges, scan reset overlap and calibrated intensity; transient blank fallback must remain presentation-only and observable |
| Transport: `card_transport.rs`, `hp67.rs` | Logical 34-record cadence, motor/head/contact gate, same physical card | Head activation after second buffer-ready clear is integration policy; replace only with evidence-backed mechanics/handshake; physical serialization still separate |
| Magnetic: `card_flux.rs`, `magnetic_card.rs` | Physical card container and separate flux model | Prove flux/sense/CRC link with source-backed order, polarity and timing before claiming production magnetic fidelity |
| Power: `reset_power_on`, app events | Reset hold and firmware boot, selector retention | Physical reset waveform and contact preservation; do not confuse host extraction with mechanics |
| Diagnostics: `pin_integrity.rs`, CLI | Scoped ownership/coverage and safe report output | Exit 0 does not certify ROM bank, DATA payload/direction, CRC serialization or F1/KC; extend checks only with real observations |
| Host/PAC/UI | Separate PAC branch: manuals/artwork/import and speed control | Integrate exact heads, test error paths and pacing; visually inspect card graphics/PDF sections separately from magnetic data |
| Browser | Responsive UI and embedded media paths | Native persistence does not imply browser import/export; target build/manual checks remain separate |

## Roadmap: substantial, independently reviewable deliveries

These are work packages with exit criteria, not a promise of 15 prompts or a
percentage of completion. Source-blocked work can finish with an evidence report
and explicit remaining gap; it cannot be marked hardware-complete without data.

| Order | Package | Required exit criterion |
|---|---|---|
| 1 | Validate pending media/trace integrity | Exact branch head passes core gate, default/pin-capture, release invariants and 12/12 diagnostics; then owner-authorized merge |
| 2 | Preserve host media across fault/reset/import | Reconcile PAC changes; cover A03/A04/A07 identity with fault injection and same-card tests |
| 3 | Transactional card persistence | A01 failure matrix on Windows; old file survives failed save, successful saved bytes reimport identically |
| 4 | Partition-independent pacing and unlimited mode | A02 properties plus normal/unlimited/OFF transitions; CPU acceleration does not create mechanical time or charge elapsed time twice |
| 5 | Topology and failure contracts | A05/A06 covered; no accidental retry after partial commit; no production per-cell allocation added |
| 6 | ACT authority ledger and differential campaign | Every mutable field classified by owner and commit coordinate; supported opcode/field/flow boundaries covered without PC-specific result patches |
| 7 | Production RAM/CRC causal workloads | Real ROM command stream exercises read/write/clear/FIFO/tail paths; synthetic reference rows remain labeled separately |
| 8 | Physical input acquisition | F1/KC integration backed by evidence, or precise missing evidence report; contact-to-firmware regressions and honest analyzer scope |
| 9 | Clock/edge evidence campaign | Capture plan, provenance, units and comparison artifacts; close only measured PHI/SYNC/IS/DATA relations |
| 10 | Display timing and optics | STR/RCD overlap/reset and complete-scan causality; source-supported exposure parameters and visual checks |
| 11 | Magnetic sense and transport | Evidence-backed physical stream to CRC path, write/read round trip and protection; no host-created continuation or guessed flux polarity |
| 12 | PAC/tooling/corpus integration | A08; complete names/order/PDF mapping, all card payloads round-trip, contact sheets for artwork and explicit pairing |
| 13 | Trace/replay and observability | Reproducible input-event workload, scoped comparisons, preserved fault context, missing observations distinguished from violations |
| 14 | Performance campaign | Comparable real-production workloads, profile actual costs, optimize only with state/trace equivalence and before/after measurements |
| 15 | Release closure audit | All applicable gates on exact candidate; unresolved approximation/source-blocked list published; no unsupported complete-electrical claim |

Dependencies: 1 precedes integration. 2 and 4 must reconcile PAC instead of silently
replacing its app/hp67 changes. 7 provides meaningful workload coverage for 14.
9 supplies timing evidence for 8/10/11 where needed. 12 must be reconciled before
15. Evidence gathering need not wait for unrelated host fixes.

## Performance acceptance plan

Use the same machine, Rust toolchain, release flags, power profile and workload
for parent/candidate; rotate/interleave runs where practical and retain medians,
spread, word counts and exact commits. Compare default and pin-capture separately.
Include firmware idle, key/program execution, RAM-heavy real firmware, real card
read/write and unlimited mode. Keep existing synthetic reference bridge numbers
as reference workloads, not production throughput. Profile before selecting an
optimization. Reuse dense buffers, immutable snapshots and fixed ownership tokens;
avoid per-cell heap allocation, format strings, maps or full-state cloning.

Do not explain speed changes from different historical gates as measured code
regressions without a controlled comparison. No new performance numbers were
measured in this checkpoint. Documentation-only changes cannot establish the
performance of the pending source batch.

## Resume at home

Do not alter the original PAC worktree or discard local changes. Use the existing
core-audit worktree. First fetch/check status and run the pending branch's gate:

```powershell
Set-Location 'D:\ProyectosProgramacion\hp67emu-core-audit'; git fetch origin; if ($LASTEXITCODE -ne 0) { throw 'FETCH KO' }; if (git status --porcelain) { throw 'WORKTREE HAS LOCAL CHANGES: preserve them before switching' }; git switch --detach origin/agent/media-trace-write-integrity; if ($LASTEXITCODE -ne 0) { throw 'UPDATE KO' }; git rev-parse HEAD; & .\tools\gate_core_audit.ps1
```

Record the printed head with the gate result. A green previous head does not
validate later source changes. Stop at the first failure; preserve full output,
including the assertion and preceding context, not only the failure summary.
After the owner reports this exact gate green, integrate the validated head into
main under the standing workflow. Start the next focused package from updated
main; reconcile PAC changes explicitly without overwriting newer core fixes.

## Completion criteria for this checkpoint

- [x] Exact inspected revisions recorded, including separate PAC branch.
- [x] Broad source review and actionable findings recorded with closure tests.
- [x] Roadmap/TODO entry points reconciled with the current audit.
- [x] Pending source batch remains unmerged; hardware unknowns remain explicit.
- [ ] Pending branch Rust compilation and full owner gate.
- [ ] Windows persistence/fault reproductions and controlled performance campaign.
- [ ] Remaining implementation packages and physical evidence closure.

## Implementation update after checkpoint 6c1de947

Source changes are now prepared on the same pending branch for A02 normal live
pacing (fractional remainder), A04 reset API (reject occupied transport before
mutation), and A05 topology (reject duplicate nets). Four new regressions and
focused binary debug/release gate steps accompany them. See the latest section of
`CORE_INTEGRITY_AUDIT.md`. Their status is **implemented, owner gate pending**, not
validated/closed. PAC unlimited pacing and UI fault retention still require
explicit integration; the broad audit and the remaining packages remain open.
