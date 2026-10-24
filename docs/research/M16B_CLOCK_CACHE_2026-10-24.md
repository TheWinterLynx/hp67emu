# M16B — preserve PHI validity with cached owner occupancy

Date: 2026-10-24
Branch: agent/m16b-clock-cache
Base: validated/integrated 692ce6c5d1409e1acd167813630a1c3d5b9ce334
Delivery: follow-up to 5/15; does not consume delivery 6
Status: owner M16B PERF FULL GATE GREEN; exact 976e8e4fadf269d0fba22f307ba11e3d4084a613 integrated.

## Trigger and attribution limits

Owner M16B is FULL GATE GREEN with unchanged 12/12 diagnostic cycle counts.
Live median rose from M16A 2.845 to 3.690 us/word (+29.7%). Even rows that do
not invoke the new predicate slowed: raw PHI 0.362 to 0.418, stage/commit
0.534 to 0.642. This shows run variation but does not explain the full live
change. No performance recovery is claimed before measurement.

## Representation change

The exact predicate remains: both PHI nets have one active ACT source. Store
a two-bit invalid-owner summary in the fabric. ACT clocks are seeded active
and valid. Every generic immediate/staged clock write refreshes the affected
summary bit when a source enters/leaves HighZ; changing Low to High or High
to Low preserves occupancy even if the existing owner set is invalid. Queries
become a mask comparison. Both resolved phase levels and the exceptional
full-driver fault diagnostic remain independent and unchanged. No electrical
operation or edge is removed; default capture remains compiled out.

The specialized source bundles write only IS/SYNC/DATA/STR/RCD, never PHI.
The fixed-topology inventory locks PHI indices 0 and 1. This is cached
bookkeeping, not a physical timing/model change or a new evidence claim.

## Verification and performance

Independent all-driver scans contrast every committed cache result for all
net/source/drive pairs, both staged handoff orders, commits reporting contention,
clone/reset and 20,000 deterministic mixed updates. All nine original M16B
regressions remain; default backplane storage still equals its fabric storage.

An ignored release kernel rotates cached and original count/ACT-slot queries
with the same eight probes per bit, all resolved 224 transitions and cache
maintenance enabled for both. It isolates query cost, not full-emulator old/new
performance. The existing eight electrical benchmark rows now rotate together
in each round; their workload code is unchanged. Live already rotates.
Compilation, both build suites, release, both diagnostics and all existing
benchmarks are owner-only gates. Actions runs formatting/check/save only.


## Owner measurements

Both diagnostics 12/12 retain all cycle counts. Cached ownership kernel 0.549
versus count-based 1.225 us/word in the same rotating run. Production live
3.348 us/word, full no-DATA structural 1.573, raw PHI 0.422, stage/commit
0.800, staged scheduler 0.815, autonomous RAM 1.294. Live improves 9.3%
versus prior 3.690 but remains 17.7% above M16A 2.845. Other controls varied
and electrical measurement order changed; do not assign all differences to
this cache or claim full recovery. Existing fidelity checks remain enabled.
