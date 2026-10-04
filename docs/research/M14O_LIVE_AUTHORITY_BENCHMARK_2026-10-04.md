# M14O — measure actual live authority throughput

Date: 2026-10-04
Branch: `agent/m14o-live-authority-benchmark`
Status: implemented; owner local validation and timings pending.

## Finding and change

The historical production-dual and real-firmware benchmark functions execute the oracle then structural transport but do not run the live adapter's authority restoration, guard, comparisons and commits. Their order/workload is retained and labels now explicitly identify historical fixtures. A new test-only benchmark calls the actual live adapter cycle. No production semantics, electrical scheduling or checks are changed.

## Workloads

| Row | Source/work | Included | Limits |
|---|---|---|---|
| Live firmware idle | Real ROM and firmware control flow | Full live authority path, display/fetch, input sampling, all guards, transport advance | Idle throughput, not all application behavior |
| Synthetic RAM ports | Alternating injected fixed RAM write/read at installed 0x20 | Same live cycle plus continuous logical DATA and next-word tail | Synthetic stress, not real firmware application |
| Synthetic CRC ports | Alternating injected fixed CRC write/read, logical buffer refill/drain | Same live cycle plus M14M FIFO/payload commits; fixture overhead timed | No physical card/head/flux throughput claim |

## Measurement contract

Construction, boot and warm-up occur before timing. Each machine survives all rounds; rotating path order reduces thermal/order bias. Seven 50,000-word rounds and 2,500 warm-up words match the historical workload defaults. Print median/best/worst us/word, words/s and headroom versus ~320 us physical word. Each stream asserts exact live word count, architectural execution count, complete execution and 56 times four clock transitions per word. CRC queues are drained/consumed; RAM maintains its pending cross-word frame. A non-ignored regression locks odd/even repeated batches and final RAM reconstruction.

## Validation and interpretation

Owner full warnings-denied test/release gate and diagnostic 12/12 remain required. Run all three benchmark targets serially with one test thread and record timings on the same host. Do not compare a historical fixture directly as if it were the live workload. A material regression needs investigation and reproducible same-workload evidence; no fidelity checks/transitions may be removed. This branch introduces no guessed speed threshold and makes no unmeasured throughput claim.

## Fidelity boundaries and next work

Wall-clock timing belongs only to instrumentation. Observed throughput is computational headroom, not electrical correctness. DATA passive level/drive edges, physical RAM partitioning, exact PHI widths, ACT intra-word write edges, STR/RCD propagation, keyboard pin scanning and magnetic sense/serialization remain SOURCE-BLOCKED or pending. After measurements, prioritize source-backed device/input/transport work and broader official Pac firmware acceptance.
