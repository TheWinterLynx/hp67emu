# Local regression checks

Owner workflow: GitHub Actions executes exclusively cargo fmt/check and saves formatting on the focused branch. Compilation, tests, release builds, diagnostics and benchmarks run locally. A reported milestone FULL GATE GREEN authorizes integration of the exact validated head and immediate continuation.

Formatting is the first hard gate: `cargo fmt --all -- --check`. Then run warnings-denied locked all-target tests, locked release binary build and the explicitly ignored Custom Diagnostic Pac 12/12 test. The provided PowerShell milestone command checks clean worktree, fetches the branch, updates by fast-forward, verifies the exact formatted head, prints progress and stops on any failure.

## Performance gates

M14O additionally runs the ignored release electrical benchmark, DATA benchmark and actual live authority benchmark serially with one test thread. Use identical HP67_BENCH_WORDS/ROUNDS/WARMUP_WORDS values for comparison. Default measurement is 50,000 words per round, seven rounds and 2,500 warm-up words. Keep host load and power mode comparable; record commit, host and median/min/max.

Historical dual fixtures and current live authority rows are different workloads. Compare each established row to its own same-host baseline. RAM/CRC live fixture numbers include synthetic host operations and do not claim physical card throughput. No clock transitions, contention checks, resolved nets or authority checks may be removed to improve a number.

Clippy and any additional workflow require the current task's authorization; they are not silently substituted for this owner's gate.

## M16G reproducible local gate

`tools/gate_m16g.ps1` is the owner-only nine-stage gate. Fetch and detach the exact supplied review SHA before invoking it. It runs fmt, focused library/native regressions, `RUSTFLAGS=-Dwarnings` all-target tests, release binary builds, both ignored diagnostic builds, and serial electrical/DATA/RAM/live benchmarks. It preserves the caller's flags and directory and prints the tested SHA. No persistent log is created.

The v1 pasted M16G command incorrectly introduced Clippy; the owner report on 2026-10-28 shows the feature-native focused tests passed (3/3), then Clippy stopped on inherited findings. Those findings are not a runtime MOTOR regression or a full-gate pass. Version 2 restores the gate already required here and in AGENTS.md without changing source semantics, suppressing lints or claiming the remaining stages passed. Clippy cleanup remains a separate, explicitly scoped task.
