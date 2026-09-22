# `tests/hp67_electrical_realtime_benchmark.rs`

## Purpose
Measures the wall-clock cost of the current HP-67 electrical/structural fidelity path against the observed physical HP-67 machine-word time.

## Why it exists
M14 adds more explicit electrical work per machine word. This ignored release benchmark provides a stable performance baseline so fidelity can increase without losing sight of realtime headroom. It is deliberately separate from correctness tests and does not run in the normal test suite.

## Relationships
Uses the production HP-67 backplane, PHI edge advancement, resolved IS transport, ROM fetch endpoints, ROM0 display endpoint, serial ACT execution path and `Hp67ArchitecturalMachine`. The physical comparison reference is `HP67_OBSERVED_WORD_TIME_US` from `src/machines/hp67/timing.rs`.

## Responsibilities
Benchmark ten continuous paths: PHI/backplane only, dense stage+commit PHI without snapshot reads, the dense zero-copy snapshot/stage/commit PHI scheduler path, structural ACT↔ROM fetch over IS, the heaviest currently wired structural path with ROM0 display plus serial ACT execution, architectural execution in isolation, a production-equivalent dual path that runs architectural execution alongside the structural word, a real-firmware dual path that follows the versioned HP-67 ROM/control flow, and an isolated M14B DATA logical-phase source/sink stream. Report median/best/worst microseconds per word, words per second and realtime multiple versus the physical HP-67.

## Interpretation
The benchmark measures computational headroom, not completeness of electrical fidelity. DATA/RAM electrical transfers, DATA polarity/drive ownership, final PHI-relative IS/DATA launch/sample edges, electrically scheduled STR/RCD, exact PHI widths/dead time and propagation delays are not yet part of the measured full path and are printed explicitly as exclusions.

## Execution
Run explicitly in release mode with `--ignored --nocapture`. Workload size can be changed with `HP67_BENCH_WORDS`, `HP67_BENCH_ROUNDS` and `HP67_BENCH_WARMUP_WORDS`.

## Implementation
The ignored test uses `std::time::Instant` around repeated continuous workloads after a warm-up phase. Defaults are 50,000 words per measured round and 2,500 warm-up words so each host-side sample is long enough to reduce scheduler/turbo noise when comparing a few-percent change. The three PHI microbenchmark rows now perform the same two pre-transition PHI level reads through `std::hint::black_box`, so observer/compiler visibility is matched before comparing immediate, stage+commit and full staged paths. Their measured rounds are interleaved with rotating order to reduce path-order drift. The stage+commit row isolates pending-output publication plus atomic commit followed by the same explicit timing advance without `begin_evaluation()`. The ACT owner token is claimed once before the measured loop and reused on every edge, matching the intended machine-composition ownership model instead of measuring ownership validation repeatedly. The current experiment keeps the same scheduler semantics while storing each staged `(net, driver, drive)` directly in the fixed pending list and using a dense slot-marker matrix only for deduplication/final-write-wins. The dense staged PHI row then adds the zero-copy evaluation snapshot that M14B devices will use. Additional rows expose architectural execution by itself, the same dual architectural+structural arrangement used by the live machine, and a versioned-ROM firmware stream, so structural headroom cannot hide host-side work or an unrealistically trivial ROM/control-flow fixture. It keeps one backplane/device state alive for every measured round, summarizes median/min/max durations, converts them to microseconds per 56-bit word and words per second, and compares the median with the observed `HP67_OBSERVED_WORD_TIME_US` physical reference. A non-ignored regression compares the complete PHI1/PHI2/tick trace for the immediate, stage+commit and full staged paths across three machine words. `std::hint::black_box` intentionally adds the same two pre-transition observation barriers to all three PHI timing rows; those barriers are benchmark instrumentation and must not be interpreted as production scheduler cost.


The staged PHI benchmark paths call `commit_staged()` and then `advance_tick()` explicitly on every PHI transition. This preserves the same 224-transition trace while verifying that electrical commit and timing progression are separate API operations.


## M14B DATA phase row

The DATA row is intentionally measured after all established M14A/production rows so adding the new microbenchmark cannot change their execution order. It exercises continuous back-to-back logical 56-bit DATA frames with the source-backed b2 phase and b0/b1 tail carry, including reconstruction in the sink. It does not touch `Hp67Net::Data`, does not choose electrical polarity and is not included in the production-dual or real-firmware rows yet. Its purpose is to measure the cost of the phase engine before electrical integration so a later production regression can be attributed rather than hidden.


## Conditional real-firmware DATA shadow

A second M14B experiment duplicates the real-firmware benchmark path but only invokes the logical DATA phase engine when the currently executing word is an architecturally recognized transfer backed by the temporary RAM image. The transfer plan is computed from the pre-instruction ACT state, writes source C, reads source the selected architectural RAM word, and completed frames are compared against their pending expected payload. Idle firmware words do not run a second 56-bit DATA loop. This row remains separate from production and is printed after every established baseline row; it exists solely to quantify realistic incremental overhead before DATA is allowed into the live structural loop.
