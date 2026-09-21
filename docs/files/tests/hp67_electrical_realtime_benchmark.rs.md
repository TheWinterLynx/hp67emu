# `tests/hp67_electrical_realtime_benchmark.rs`

## Purpose
Measures the wall-clock cost of the current HP-67 electrical/structural fidelity path against the observed physical HP-67 machine-word time.

## Why it exists
M14 adds more explicit electrical work per machine word. This ignored release benchmark provides a stable performance baseline so fidelity can increase without losing sight of realtime headroom. It is deliberately separate from correctness tests and does not run in the normal test suite.

## Relationships
Uses the production HP-67 backplane, PHI edge advancement, resolved IS transport, ROM fetch endpoints, ROM0 display endpoint and serial ACT execution path. The physical comparison reference is `HP67_OBSERVED_WORD_TIME_US` from `src/machines/hp67/timing.rs`.

## Responsibilities
Benchmark three continuous paths: PHI/backplane only, structural ACT↔ROM fetch over IS, and the heaviest currently wired structural path with ROM0 display plus serial ACT execution. Report median/best/worst microseconds per word, words per second and realtime multiple versus the physical HP-67.

## Interpretation
The benchmark measures computational headroom, not completeness of electrical fidelity. DATA/RAM electrical transfers, final PHI-relative IS/DATA launch/sample edges, electrically scheduled STR/RCD, exact PHI widths/dead time and propagation delays are not yet part of the measured full path and are printed explicitly as exclusions.

## Execution
Run explicitly in release mode with `--ignored --nocapture`. Workload size can be changed with `HP67_BENCH_WORDS`, `HP67_BENCH_ROUNDS` and `HP67_BENCH_WARMUP_WORDS`.
