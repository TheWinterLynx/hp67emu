# `tests/hp67_electrical_realtime_benchmark.rs`

## Purpose
Measures the wall-clock cost of the current HP-67 electrical/structural fidelity path against the observed physical HP-67 machine-word time.

## Why it exists
M14 adds more explicit electrical work per machine word. This ignored release benchmark provides a stable performance baseline so fidelity can increase without losing sight of realtime headroom. It is deliberately separate from correctness tests and does not run in the normal test suite.

## Relationships
Uses the production HP-67 backplane, PHI edge advancement, resolved IS transport, ROM fetch endpoints, ROM0 display endpoint, serial ACT execution path and `Hp67ArchitecturalMachine`. The physical comparison reference is `HP67_OBSERVED_WORD_TIME_US` from `src/machines/hp67/timing.rs`.

## Responsibilities
Benchmark eight continuous paths: PHI/backplane only, dense stage+commit PHI without snapshot reads, the dense zero-copy snapshot/stage/commit PHI scheduler path, structural ACT↔ROM fetch over IS, the heaviest currently wired structural path with ROM0 display plus serial ACT execution, architectural execution in isolation, a historical dual path that runs architectural execution alongside the structural word, and a real-firmware dual path that follows the versioned HP-67 ROM/control flow. Report median/best/worst microseconds per word, words per second and realtime multiple versus the physical HP-67.

## Interpretation
The benchmark measures computational headroom, not completeness of electrical fidelity. DATA/RAM electrical transfers, final PHI-relative IS/DATA launch/sample edges, electrically scheduled STR/RCD, exact PHI widths/dead time and propagation delays are not yet part of the measured full path and are printed explicitly as exclusions.

## Execution
Run explicitly in release mode with `--ignored --nocapture`. Workload size can be changed with `HP67_BENCH_WORDS`, `HP67_BENCH_ROUNDS` and `HP67_BENCH_WARMUP_WORDS`.

## Implementation
The ignored test uses `std::time::Instant` around repeated continuous workloads after a warm-up phase. Defaults are 50,000 words per measured round and 2,500 warm-up words so each host-side sample is long enough to reduce scheduler/turbo noise when comparing a few-percent change. The three PHI microbenchmark rows now perform the same two pre-transition PHI level reads through `std::hint::black_box`, so observer/compiler visibility is matched before comparing immediate, stage+commit and full staged paths. Their measured rounds are interleaved with rotating order to reduce path-order drift. The stage+commit row isolates pending-output publication plus atomic commit followed by the same explicit timing advance without `begin_evaluation()`. The ACT owner token is claimed once before the measured loop and reused on every edge, matching the intended machine-composition ownership model instead of measuring ownership validation repeatedly. The current experiment keeps the same scheduler semantics while storing each staged `(net, driver, drive)` directly in the fixed pending list and using a dense slot-marker matrix only for deduplication/final-write-wins. The dense staged PHI row then adds the zero-copy evaluation snapshot that M14B devices will use. Additional rows expose architectural execution by itself, the historical dual architectural+structural fixture, and a versioned-ROM firmware stream, so structural headroom cannot hide host-side work or an unrealistically trivial ROM/control-flow fixture. It keeps one backplane/device state alive for every measured round, summarizes median/min/max durations, converts them to microseconds per 56-bit word and words per second, and compares the median with the observed `HP67_OBSERVED_WORD_TIME_US` physical reference. A non-ignored regression compares the complete PHI1/PHI2/tick trace for the immediate, stage+commit and full staged paths across three machine words. `std::hint::black_box` intentionally adds the same two pre-transition observation barriers to all three PHI timing rows; those barriers are benchmark instrumentation and must not be interpreted as production scheduler cost.


The staged PHI benchmark paths call `commit_staged()` and then `advance_tick()` explicitly on every PHI transition. This preserves the same 224-transition trace while verifying that electrical commit and timing progression are separate API operations.

## M14O scope correction

The dual architectural/structural and firmware rows now explicitly say historical. Their implementation and order remain unchanged for comparison, but they do not exercise live restoration, the ACT pre-state guard or completed authority commits. Use `m14o_live_authority_realtime_benchmark` in the hp67emu binary test target for actual live throughput. Do not equate the different workloads when comparing timings.

## M15E workload distinction

The structural full-row label explicitly names its no-DATA workload and directs
resolved RAM DATA measurement to the separate actual-live benchmark. No timing
fixture or fidelity operation is removed.

## M15G scope

Display-capable rows now include the coarse resolved STR/RCD bridge. Printed
scope distinguishes that logical pin path from unproven physical pulse widths,
PHI edges and propagation. Historical rows retain their fixtures but also use
the current shared display transport where previously wired.

## M15H scope

Printed scope now names the provisional PHI1-rising receiver checkpoint. The
benchmark continues to distinguish computational headroom from proof of physical
launch/latch edges, pulse widths and propagation.

## M15I scope

The printed full-row scope includes atomic IS/SYNC source publication. This
no-DATA row does not acquire RAM transfers; optional DATA joins the same commit
only in DATA-aware production paths, measured by the live authority benchmark.
No timing calibration or performance threshold is introduced.

## M15J five-ROM composition

Current structural-fetch and full-current rows now instantiate five ROM
endpoints, matching the new live ROM-side composition. Historical dual rows
retain aggregate fixtures for comparison. Scope text names distinct chip IS
drivers. Logical M14B DATA fixtures remain separate; actual production including
physical-ROM fetch plus resolved RAM DATA is measured by the live benchmark.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.

## M15K autonomous ROM bank control

The current structural fetch/full paths construct autonomous ROMs; historical
paths remain unchanged. Output labels include autonomous IS/SYNC bank latches.
This adds bank-decoder work to the measured current path without changing its
fixed workload, physical timing claims or diagnostic correctness contract.

See `docs/research/M15K_AUTONOMOUS_ROM_BANKS_2026-10-16.md`.


## M16B rotating measurement order

All eight paths now rotate together per round instead of measuring only the
three PHI paths interleaved and the remaining paths sequentially. Workload
functions, word/round counts, scope, checks and output rows remain unchanged.
This reduces fixed-order bias; it does not calibrate prior host runs or create
a whole-emulator old/new comparison. The live authority benchmark already
uses rotating order and is unchanged.

M16E: measured production paths include sole ACT SYNC source validation at entry and instruction receivers. Workloads and all 224 PHI transitions remain unchanged.

M16F: the measured paths include IS identity checks at the existing sample checkpoints; 56 cells, 224 transitions, workload and measurement order remain. No new performance claim precedes owner measurement.

M16G: live includes the CRC MOTOR source/receiver gate. The no-DATA structural fixture does not instantiate this output. Added Motor fabric inventory preserves old indices and all 224 transitions; workloads and rotation remain unchanged.


M16H scope notes that the no-DATA fixture does not instantiate live MTRS; the live authority benchmark measures that production path. Measurement loops and structural behavior are unchanged.

M16I scope distinguishes the no-DATA structural fixture from live HDS/WPS receiver checks. Measured loops and physical transition counts are unchanged.
