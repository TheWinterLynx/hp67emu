# M16E performance follow-up — cached SYNC source ownership

Date: 2026-10-25
Branch: agent/m16e-sync-cache
Base: owner-validated/integrated 61d2633d4ac58d48eb2661744048a6aabcede450
Delivery: follow-up to 8/15; does not consume delivery 9
Status: owner M16E PERF FULL GATE GREEN; exact b86849d5d0f7819fc0b6f7c1c749d7f5ff8f5491 integrated. Both diagnostics 12/12 unchanged cycles. Kernel cached 0.254 versus counted 0.252 us/word, live 3.023 versus prior 2.942. No performance benefit demonstrated. M16F retires the cache while retaining its source-equivalence regressions.

## Trigger and attribution limits

Owner M16E FULL GATE GREEN, both diagnostics 12/12 with unchanged cycle counts.
Live median rose from M16D 2.745 to 2.942 us/word (+7.18%). The full no-DATA
structural row rose 1.291 to 1.326 (+2.71%), RAM 1.065 to 1.085 (+1.88%), while
IS-only fetch fell 0.962 to 0.953. These separate runs do not isolate the cause.
The new independent receiver checks repeatedly read the same committed SYNC
occupancy, so a representation change can reduce query work without relaxing
any validation. No recovered performance is claimed before owner measurement.

## Representation and invariants

One fabric boolean records whether SYNC has exactly one active source, ACT.
It starts false because pristine SYNC is undriven. Immediate/staged generic
updates refresh it only when a SYNC source enters or leaves High-Z. Low/High
polarity changes preserve the source set, including an invalid source set.

The specialized joint serial commit still installs IS/SYNC/optional DATA before
any externally visible snapshot. Unchanged SYNC skips cache work. A changed
valid ACT High/Low output keeps ownership valid without another ACT-slot load;
release recomputes it. Changed invalid images are recomputed, covering first
attachment and explicit raw-fixture repair. Every foreign contribution survives.
The cache is never inferred from resolved level alone.

The validator reads committed metadata and still checks the actual resolved
binary level. The exceptional fault contains the same ordinal coordinate, ACT
drive and complete contributor mask. Pristine tick-zero entry is unchanged;
actual receivers still demand ACT ownership. No receiver check is removed or
moved. All existing missing/replaced/agreeing/contention/phase/retry campaigns
remain mandatory.

The PHI owner mask is unchanged. Generic writes branch into the existing clock
updates or the new SYNC update only on a source occupancy change; ordinary PHI
polarity edges do not enter either path. No new net, driver, clock edge, passive
bias, drive event or heap allocation is introduced. All 224 transitions and
resolve/snapshot/evaluate/commit behavior remain. Physical SYNC edge/width/
propagation uncertainties are unchanged; this change claims no hardware timing.

## Verification and measurement

Five regressions cross-check cached SYNC and PHI against independent full-driver
scans for all net/source/drive pairs; staged handoffs in both orders, repeated
stages and contention-error commits; both aggregate and five-ROM joint serial
bundles with all foreign sources and both polarities; clone/reset; 20,000 mixed
immediate/staged/bundle updates; public validator fault details and pristine
entry policy. All nine original M16E source/receiver/live/capture tests remain.

An ignored rotating same-process release kernel compares cached source queries
against the prior count-plus-ACT-slot predicate with 33 equivalent probes per
word. Both paths perform joint IS/SYNC publication, 56 cells, all 224 PHI
transitions and new cache maintenance. It isolates query cost, NOT whole-emulator
old/new performance and NOT the cache maintenance delta versus the old binary.
The real live benchmark is still required; no synthetic speedup proves live gain.

Owner M16E baseline: PHI 0.353, staged commit 0.637, staged scheduler 0.676,
IS-only 0.953, full structural 1.326, autonomous RAM 1.085, live 2.942 us/word
(108.75x hardware); synthetic reference RAM 2.290 and CRC 2.181. The same full
gate, both feature configurations and diagnostics are required. Actions remains
fmt/check/save only; no runtime results or merge are claimed for this follow-up.
