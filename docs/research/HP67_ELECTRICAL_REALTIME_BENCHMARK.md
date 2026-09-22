# HP-67 electrical realtime benchmark

This benchmark measures whether the current HP-67 electrical/structural path can be computed faster than the observed physical calculator while preserving continuous state across thousands of 56-bit machine words.

## Physical reference

The current hardware evidence lock records an observed physical HP-67 machine-word time of approximately **320 us** for one 56-bit word.

That corresponds to:

- 3,125 machine words/s;
- 175,000 serial bit-cells/s;
- 700,000 named PHI transitions/s in the current four-transition topology.

The last number is a scheduler workload comparison only. M14A does not claim four equal-duration physical subphases.

## Measured paths

`tests/hp67_electrical_realtime_benchmark.rs` reports eight continuous paths:

1. `PHI backplane / resolved clock nets`
   - advances all four named PHI transitions for every bit-cell;
   - drives/resolves the HP-67 PHI nets continuously.

2. `dense stage+commit / PHI`
   - advances the same PHI topology through dense pending-output publication plus atomic commit;
   - deliberately omits evaluation snapshot reads so pending/commit cost can be measured independently.

3. `dense staged scheduler / PHI`
   - adds the zero-copy borrowed resolved snapshot and PHI input reads to the same dense stage/commit path;
   - measures the complete scheduler representation intended for M14B devices, independently of IS/ROM work.

4. `IS ACT<->ROM structural fetch`
   - includes the immediate PHI path above;
   - resolves shared IS ownership;
   - serializes ACT->ROM 12-bit addresses at b16..b27;
   - reconstructs the ROM address from resolved IS;
   - serializes ROM->ACT 10-bit instruction words at b46..b55;
   - reconstructs the returned word at the ACT endpoint.

5. `IS + ROM0 display + serial ACT execution`
   - includes the previous path;
   - adds ROM0 display traffic;
   - advances the 15-word display phase continuously;
   - advances one serial ACT instruction through b0..b55 for every measured word.

6. `architectural execution only`
   - isolates the current instruction-boundary `Hp67ArchitecturalMachine::execute_word()` path;
   - exposes host-side semantic/transactional cost that is not part of the structural row.

7. `production dual architectural + structural`
   - mirrors the current live-machine ordering: bind serial pre-state, execute the architectural fallback, then traverse the structural display/fetch word;
   - is the relevant current throughput baseline until the serial/electrical path becomes authoritative and the architectural bridge can be removed.

8. `real firmware architectural + structural`
   - runs the versioned `Hp67Firmware` image instead of a constant ROM fixture;
   - follows the fetched HP-67 control flow while keeping the same dual architectural+structural execution arrangement;
   - is the most representative CPU/ROM throughput row in this benchmark, although UI, card transport and future DATA/RAM electrical devices remain outside its scope.

Every benchmark round uses one uninterrupted backplane/ACT/ROM state stream. The default 50,000 words therefore represent approximately 16 seconds of physical HP-67 machine time per round. At the current >100x host speed this also keeps each measured host interval long enough that ordinary Windows scheduling and turbo changes are less likely to masquerade as a few-percent emulator regression. The three PHI comparison rows use identical pre-transition observation: PHI1 and PHI2 are both read and passed through `black_box` before every one of the 224 transitions per word. Their rounds are measured interleaved with a rotating execution order, so the immediate/stage+commit/staged comparison is not biased by one path always running earlier or later in the benchmark.

## Interpretation

The `vs hardware` column is:

`physical HP-67 word time / host median time per emulated word`

Therefore:

- `1.00x` means the current model computes at real HP-67 speed;
- `10.00x` means ten times realtime computational headroom;
- `<1.00x` means the current model cannot yet sustain realtime on that host.

This is a **performance/headroom benchmark**, not a claim that M14 electrical fidelity is complete. The matched PHI observation barriers are intentionally artificial benchmark instrumentation; only differences between the three identically observed PHI rows should be attributed to scheduler representation.

The benchmark deliberately prints the fidelity gaps still outside the measured path: DATA/RAM electrical transfers, PHI-relative IS/DATA launch/sample edges, electrically scheduled STR/RCD, exact PHI widths/dead time and propagation delays. As those become real devices/events, they should be added to the full row rather than creating a faster shortcut path.

## Validated direct-pending result (2026-09-22)

On the validated Windows release run with matched PHI observation and interleaved PHI rounds:

| Path | Median us/word | Realtime multiple |
| --- | ---: | ---: |
| PHI backplane / resolved clock nets | 0.718 | 445.68x |
| dense stage+commit / PHI | 1.243 | 257.41x |
| dense staged scheduler / PHI | 1.221 | 262.05x |
| IS ACT<->ROM structural fetch | 0.889 | 359.83x |
| IS + ROM0 display + serial ACT execution | 0.970 | 329.92x |
| architectural execution only | 0.057 | 5581.14x |
| production dual architectural + structural | 1.020 | 313.60x |
| real firmware architectural + structural | 0.956 | 334.72x |

The direct typed pending-entry experiment reduced the matched-observation stage+commit median from 2.903 to 1.243 us/word and the full staged scheduler median from 2.954 to 1.221 us/word. The difference between stage+commit and full staged is now below measurement noise in this run, so `begin_evaluation()` and the borrowed snapshot are not the performance bottleneck.

The current staged scheduler therefore exceeds the project engineering headroom target of 150x on this host while preserving the resolve/snapshot/evaluate/stage/atomic-commit contract. This is a performance result only, not a fidelity promotion.

## Run

Use release mode and keep the benchmark ignored during normal test gates:

```powershell
$env:RUSTFLAGS='-Dwarnings'; cargo test --release --locked --test hp67_electrical_realtime_benchmark hp67_electrical_realtime_benchmark -- --ignored --nocapture
```

Optional workload controls:

- `HP67_BENCH_WORDS` — words per round, default 50000;
- `HP67_BENCH_ROUNDS` — measured rounds, default 7;
- `HP67_BENCH_WARMUP_WORDS` — warm-up words per path, default 2500.


## Validated one-time driver ownership result (2026-09-22)

After moving duplicate-driver validation out of the per-edge hot path and into one-time machine composition, the matched-observation benchmark measured:

| Path | Median us/word | Realtime multiple |
| --- | ---: | ---: |
| PHI backplane / resolved clock nets | 0.677 | 472.71x |
| dense stage+commit / PHI | 1.548 | 206.73x |
| dense staged scheduler / PHI | 1.255 | 255.04x |
| real firmware architectural + structural | 0.966 | 331.37x |

The full staged scheduler therefore recovered from the per-edge ownership experiment's 1.894 us/word (168.93x) to 1.255 us/word (255.04x), within roughly 3% of the 1.221 us/word direct-pending baseline. Ownership is retained as a composition-time correctness invariant rather than a per-transition check. The stage+commit helper remains slower than the full staged path on this host and is treated as a diagnostic micro-path, not the intended M14B device evaluation API.

## Validated commit/timing separation result (2026-09-22)

After separating atomic electrical commit from explicit timing advancement, the matched-observation release benchmark measured:

| Path | Median us/word | Realtime multiple |
| --- | ---: | ---: |
| PHI backplane / resolved clock nets | 0.691 | 463.15x |
| dense stage+commit / PHI | 1.142 | 280.14x |
| dense staged scheduler / PHI | 1.171 | 273.27x |
| IS ACT<->ROM structural fetch | 0.869 | 368.25x |
| IS + ROM0 display + serial ACT execution | 0.946 | 338.35x |
| architectural execution only | 0.052 | 6171.17x |
| production dual architectural + structural | 1.000 | 320.14x |
| real firmware architectural + structural | 0.955 | 335.24x |

All correctness gates, the custom diagnostic PAC suite and the matched PHI trace passed. The staged scheduler therefore improved from the one-time-ownership measurement of 1.255 us/word (255.04x) to 1.171 us/word (273.27x) on the same validation host. The architectural separation adds no performance penalty and removes the false invariant that every electrical visibility boundary must also advance the PHI/serial timing coordinate.

## Commit/timing separation

The dense scheduler no longer advances `Tick` inside `commit_staged()`. PHI benchmark paths explicitly call `advance_tick()` after each successful staged commit, so the measured topology remains four transitions per bit-cell and 224 transitions per word. This is an architectural separation only: it does not add a new physical subphase or claim a propagation delay. Future evidence-backed settling events can commit electrical state without falsely incrementing the PHI coordinate.
