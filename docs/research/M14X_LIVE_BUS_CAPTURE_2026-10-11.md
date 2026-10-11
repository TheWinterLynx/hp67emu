# M14X bounded live bus capture

M14W owner FULL GATE GREEN; integrated at c0aeca92b4d7179e7320871c43f08e5c5358e250. Owner live benchmark: 0.873 us/word, 366.50x nominal hardware headroom. This is throughput evidence, not electrical timing accuracy.

## Capture contract

Native CLI: `hp67emu --trace-is <1..4096 words> <output.tsv>`. Starts a fresh live machine and executes exactly that many real firmware words with no key contact. The simulated reset hold is excluded, as explicitly stated in the header. File output occurs only after successful bounded capture. Malformed invocation, unsupported count, live failure or write failure exits with failure. This host adapter changes no calculator semantics.

The capture uses the existing live cycle, including immutable inputs, oracle-effect restore, real IS/display/fetch traversal, comparisons and authority commits. Trace hooks record only committed IS/PHI observations. The DATA-aware diagnostic twin preserves the same logical DATA participant and b0/b1 preconsumed tail, but does not pretend that logical RAM/CRC data is electrical DATA. No reset waveforms, SYNC/STR/RCD electrical edges or physical microseconds are exported.

## Fidelity classification

M14W sample schema and existing HARDWARE_SOURCES / M14A timing lock remain authoritative. Trace ticks are ordinal WORKING APPROXIMATION scaffolding; PHI widths/dead time, IS/DATA launch/sample edges, DATA electrical ownership/polarity/chip mapping and STR/RCD propagation remain SOURCE-BLOCKED. No new hardware timing claim is introduced.

## Isolation and validation

Normal live words use original production no-DATA/DATA transport loops, with one optional recorder selection at the word boundary. Dedicated diagnostic twins keep trace allocation and 224 row appends out of the normal bit loops. Allocation is capped at 4096 * 224 samples, plus bounded textual output.

Focused regressions compare 1200 actual firmware words against an unobserved live machine (ACT state, PC, pipeline, serial endpoint, display/optics and ticks), compare twelve alternating synthetic RAM-port words through the full live bridge including completion of the cross-word tail, and check limits/header/rows. Synthetic RAM is labelled test stimulus, not firmware card throughput. Owner gate includes native CLI export and existing diagnostics/benchmarks. Only rustfmt/check/save runs in Actions.

## Next work

Accept explicitly sourced physical recording input and report alignment / bit-order / level mismatches independently of still-unknown edge calibration. No new Pac acceptance tests are planned.
