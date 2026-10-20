# M15P — shared resolved RAM control reception in live

Base/main: owner-validated M15O f55482968a065ee9d4ea838c9671a00c7a473af7.
Integrated exactly after FULL GATE GREEN. Owner diagnostic 12/12 retained all
cycles. Live idle 1.332 us/word; synthetic RAM 1.712 us; CRC 1.340 us; separate
M15N RAM fixture 0.974 us. These are M15O owner results, not M15P measurements.

## Executable scope

A reusable fixed-size receiver observes IS/SYNC directly on b46..b55. It has no
ACT state, source word, host address or opcode input. Frame publication requires
all ten ordered bits and one stable qualifier. Clear/read/write/address command
classification is extracted unchanged from M15N; implied-GOTO frames decode Idle.
The independent RAM now consumes this receiver into its existing next-word latch.
It still validates all command and DATA inputs before committing either image.

The actual live composition attaches the same receiver to the DATA-capable
shared transport, even when DATA is idle. After each word, its raw word/qualifier
must match ACT's independently received fetch. Both traced and untraced loops
already call the same DATA sampling method. No loop, PHI transition, resolution,
HighZ or contention check is removed. Pending DATA prefixes remain clocked.

This connects the RAM command receive lane; autonomous live storage/read driving
is NOT claimed. The reference RAM bridge still supplies live reads/writes.
Synthetic execution fixtures may inject an executing opcode while concurrently
fetching another real ROM word. The observer intentionally sees that actual
fetch, never the injected opcode. This distinction is now executable coverage
and printed in the live benchmark scope.

## Evidence boundaries

No new physical timing claim. The original M15N direct HP-67 Teenix captures
(Notes on HP's Classic Calculators 2025, printed pp66–69/74) establish next-word
execution, captured port words and DATA phase; existing IS b46..b55 and qualifier
contracts are preserved. https://literature.hpcalc.org/community/classic-notes.pdf
SHA256 bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da.
HP-97 service evidence remains corroboration under the owner's equivalence
assumption. M15H PHI1-rise sampling is still a WORKING APPROXIMATION. Exact
PHI edges, propagation, DATA output stage/bias and per-IC RAM map remain blocked.

## Verification and performance practice

New tests receive every 10-bit word with both qualifiers through real nets,
exercise captured opcodes and low selectors, reject invalid phase/order, IS/SYNC
contention and changed qualifier without receiver mutation, and prove a changed
valid source bit changes the received word. DATA integration faults cannot consume
a DATA bit while rejecting a command bit. Native tests compare 1200 real-firmware
cycles across both transports, require normal/implied qualifiers, preserve full
state/trace/tick counts and reset. Existing independent RAM tests and benchmarks
remain required after extracting its receiver. No existing test is disabled.

The receiver is fixed-size, Copy and allocation-free. Transactional receive
validation copies only this small image, never the 64-word RAM. Live idle words
now pay the DATA-capable loop/observer cost; benchmark that production path in the
owner gate without reducing fidelity or adding a speed correctness threshold.

Rust is absent here. Static diff/document checks and formatting-only Actions are
reviewed here; all compilation/runtime/full/release/diagnostic/benchmark gates
remain local to the owner. M15P gate pending.

## Next implementation

Connect full autonomous RAM to the same command/DATA lane, use its stored-data
read proposal in the joint source commit, compare its writes/selector/clear
against the oracle, and retain explicit reference compositions for host-injected
fixtures. Do not feed its command state from those host-injected opcodes.
