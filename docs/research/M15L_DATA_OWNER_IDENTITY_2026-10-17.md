# M15L — DATA receive ownership by driver identity

Base/main: owner-validated M15K `8937b8e99408f1df89e511554c8b183d0d21158e`.
Owner's full v3 gate passed, including diagnostic 12/12 with unchanged cycles.
Only the DATA receive contract changes; no ROM bytes, chip profiles or physical
timing values change.

## Failure and executable change

The resolved DATA receiver required one active driver, but did not verify its
identity. Releasing the proper source and enabling a different sole contributor
with the same bit level therefore passed. This violates the bridge's explicit
ACT-write/peripheral-read ownership contract without producing contention.

`sample_level_bridge()` now retains the count check and additionally requires
the expected driver to be active. The direction lookup is shared with source
proposal: previous frame at b0/b1, current frame at b2..b55. Wrong sole ownership
fails with bit, expected driver and resolved-level context before sink mutation.
This shared receiver covers normal, traced and clocked-prefix paths. It never
re-drives the source or substitutes expected payload bits for sampled values.

## Evidence boundary

Direct physical captures in Teenix, Notes on HP's Classic Calculators (2025),
printed pp67–69 and 74, document ACT C writes and ROM/RAM read responses and the
cross-word 56-bit DATA phase. Source:
https://literature.hpcalc.org/community/classic-notes.pdf,
SHA256 `bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
HP-97 service manual 2-14/figure 2-5 corroborates the bidirectional role under
the owner's equivalence instruction:
https://www.teenix.org/97ServMan_2.pdf.

The named aggregate peripheral is still a WORKING APPROXIMATION. This check
establishes correct logical-bridge ownership only; it does not establish which
physical RAM IC owns an address, DATA transistor polarity/bias, exact enable or
PHI sample edges, or propagation delays. M15H's provisional receiver checkpoint
remains unchanged. Replace the aggregate owner with an evidenced chip selection
when source-backed mapping is available; preserve wrong-owner rejection.

## Validation and performance practice

Library regressions inject every incorrect driver at every active body/tail cell
for both transfer directions at Low and High, while preserving the original
resolved bit and active count. Wrong ownership must fail and preserve the entire
DATA state. Idle cells reject every single ghost driver. A separate regression
changes the legitimate driver's committed zero bits to one between launch and
receive: both 56-bit frames must reconstruct all ones. This prevents ownership
validation from accidentally becoming a source-value shortcut.

Native regressions exercise actual live RAM write/read/NOP through traced and
untraced paths at both bit levels, checking selected driver, HighZ release,
C/RAM causality, complete frame lifecycle, 224 transitions per word and 672 trace
samples. Existing M15E/M15F/M15H fault and dependency assertions remain intact.

Production adds one fixed-index driver lookup for required DATA samples, with
no per-cell allocations, contributor scans, clocks or extra commits. M15K's
synthetic PC fixture remains unchanged. Rust is absent here; static diff and
documentation checks run in the editing environment, Actions formats only, and
focused/full tests, release, diagnostics and benchmarks require the owner local
gate. No unrun runtime result is claimed. M15L owner gate is pending.
