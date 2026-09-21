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

## Run

Use release mode and keep the benchmark ignored during normal test gates:

```powershell
$env:RUSTFLAGS='-Dwarnings'; cargo test --release --locked --test hp67_electrical_realtime_benchmark hp67_electrical_realtime_benchmark -- --ignored --nocapture
```

Optional workload controls:

- `HP67_BENCH_WORDS` — words per round, default 50000;
- `HP67_BENCH_ROUNDS` — measured rounds, default 7;
- `HP67_BENCH_WARMUP_WORDS` — warm-up words per path, default 2500.
