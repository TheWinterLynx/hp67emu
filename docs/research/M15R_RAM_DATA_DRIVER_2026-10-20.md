# M15R — independent electrical DATA driver identity for autonomous RAM

Base/main: owner-validated M15Q 7356439539706b1f6bed51aa84f8712e737c0671,
integrated exactly after FULL GATE GREEN. Diagnostic 12/12 retained identical
cycles. Owner live idle 2.126 us/word (150.49x), standalone RAM 1.115 us/word.
Reference synthetic RAM/CRC rows are not production device throughput. These
are M15Q measurements, not M15R results.

## Executable change

M15Q device-owned read data still occupied StructuralRomResponder's contribution
on DATA. M15R gives autonomous RAM a separate RamAggregate driver slot. It is
appended to preserve every existing dense driver index. Counts, resolver arrays
and ownership masks extend through the existing COUNT/ALL vocabulary.

The RAM-capable body/prefix transports choose this named contribution in the
same atomic IS/SYNC/DATA launch. Reference transport callers retain the original
source. Foreign contributions are never cleared as a convenience: an active
StructuralRomResponder on DATA remains a competing source when RAM launches,
including opposing-level contention or agreeing-level multiple ownership.

The DATA receiver remembers which composition owns peripheral reads, including
the following-word b0/b1 tail. Its strict count, identity and resolved-level checks
consume the actual net. RAM read source, storage, command pipeline and oracle
comparisons stay M15Q. The standalone RAM fixture now publishes/collects using
the same RamAggregate identity. No per-cell allocation or new clock transition.

## Evidence boundary

This separates two software composition identities. It does not claim discovery
of a physical RAM IC, change the installed 64-register logical topology, infer
DATA transistor polarity/bias or promote provisional PHI sampling to measured
silicon timing. M15N/Q evidence and WORKING APPROXIMATION boundaries remain.
Per-IC chip mapping, physical output stage and exact PHI/propagation remain
SOURCE-BLOCKED.

## Validation

New tests replace every autonomous read body/tail sample with every other named
driver at both logic levels; matching levels with the wrong sole owner must fail
without DATA state mutation. Agreeing extra reference-ROM contributions also
fail. The fabric test preserves a foreign DATA contribution through joint launch,
observes contention and verifies idle release without advancing time. Native
real-firmware RCL0 runs check RAM identity, reference-ROM HighZ, sole ownership
and 224 transitions through both transports. Existing full differential/diagnostic
and standalone RAM benchmark remain required.

Static source/diff/document checks and formatting-only Actions are reviewed here.
Rust is absent; compilation, runtime tests, release, diagnostics and performance
remain the owner's local gate. M15R gate pending.
