# M16B — PHI ownership, failure atomicity and recovery

Date: 2026-10-24
Base: owner-validated M16A 685407cf7739011478bd5eb5f643da27cd83bab9
Branch: agent/m16b-clock-integrity
Delivery: 5/15
Status: implemented; formatting and owner runtime gate pending.

## Problem and resulting behavior

A foreign driver holding PHI High can agree with ACT at the existing launch and
sample checkpoints. The old predicate accepted both High levels; a subsequent
falling pulse could become contended. A replacement source could likewise
masquerade as ACT if ACT were released. Debug assertions are insufficient
release-mode fault handling.

Both phase predicates now require one active ACT contribution on each PHI.
Word/prefix entry validates ownership and current phase before endpoint reset
or DATA binding. Live validates before contacts, retained ACT inputs, oracle
execution and pending tails. Rejected requests preserve their pre-call image.
A public checked edge API rejects before advancing the ordinal coordinate and
constructs a detailed driver mask only on failure. Raw scaffold stepping remains
for controlled fixtures; monolithic transports validate entry and no external
mutator runs during their edge traversal. Shared receivers independently check
clock ownership at their existing checkpoint.

## Evidence and limits

Existing wiring.rs/machine.rs and the recorded page-70 PHI evidence establish
ACT output identity and alternating active-low clock topology. This change
enforces that existing model; it introduces no pulse width, dead time, physical
latch edge, signal propagation or analog glitch assumption. An injected extra
driver is a simulator fault, not a new claim about a real board failure mode.
HP-97 equivalence remains owner-authorized; unknown physical timing stays blocked.

## Performance discipline

The normal path uses existing dense active-contributor counts and direct ACT
slots. It adds no storage, heap allocation, driver enumeration, trace hook,
clock edge or new device pass. All diagnostic capture remains compiled out by
default. The owner gate benchmarks the same workloads as M16A (live 2.845 us,
full no-DATA 1.351 us, PHI 0.362 us). No speedup or zero-cost claim is made:
the required additional integrity checks must be measured locally.

## Acceptance

Six always-enabled library tests cover all four phases/two clocks/all twelve foreign sources
with both Low and High drives, replaced/missing/wrong-phase ACT outputs, staged
visibility, six transport entries, receiver non-consumption, and fifteen-word
checked/raw waveform equivalence. Two native tests cover every foreign source
and real firmware pending-tail recovery against an unfaulted machine.
An additional feature-only test verifies that a rejected checked edge publishes
no phantom trace row. Both feature configurations, full suites, release, ignored diagnostic and the
same electrical/DATA/RAM/live benches are required. GitHub Actions formats only.
