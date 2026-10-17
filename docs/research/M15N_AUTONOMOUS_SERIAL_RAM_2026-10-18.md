# M15N — autonomous aggregate serial RAM device

Base/main: owner-validated M15M `1c5c8f987c9fde0bdd87f581db16a6ae99023ba8`.
The full owner gate passed: diagnostic 12/12 with unchanged cycles; live firmware
1.369 us/word, synthetic RAM 1.697 us, CRC 1.342 us. Exact head is integrated.

## Executable scope

This milestone adds an actual stateful RAM device to the reusable library. Its
64 logical registers are independent of ACT/oracle RAM. The only runtime command
input is resolved IS/SYNC; address and writes come from resolved DATA. A fetched
command executes next word. Reads use the device's own stored word, serialized
onto the named aggregate RAM DATA contribution. No ACT state, host opcode, host
address setter or read payload input exists in its public API.

This device is tested in a separate joint-launch electrical composition. It does
NOT yet replace the desktop live RAM bridge: that bridge does not currently
transmit C-to-address frames, and its synthetic instruction injection must be
made consistent for all bus observers before enabling this device there. Existing
live benchmarks continue to measure the old live composition. A separate ignored
release benchmark explicitly measures the new autonomous RAM fixture.

## Evidence and modeled boundaries

Direct HP-67 capture: Teenix, Notes on HP's Classic Calculators (2025), printed
pp66–69 and 74; https://literature.hpcalc.org/community/classic-notes.pdf.
SHA256 `bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
The captured STO/RCL route includes address opcode 270, write 2f0 and read 038.
Those exact words are also checked at corpus PCs cec, cd2 and ced respectively.
The direct captures establish next-word instruction execution and the continuous
56-bit DATA phase starting b2 and finishing next b1. HP-97 figure 2-5 remains
corroboration under the owner's equivalence instruction.

The existing logical contract supplies 64 installed words at 00..3f, selector
families 050/070 and 1260's aligned 16-word clear. This is aggregate logical
behavior, NOT a discovered per-IC RAM map. The current/next command pipeline,
byte selector and storage are device-owned. Required write/address samples need
one active ACT driver; read responses use the explicit logical High/Low adapter.
Peripheral addresses produce HighZ from this RAM; no CRC semantics are added.

b2 body binding, full-frame address completion and completed-b55 logical clear
are WORKING APPROXIMATIONS for composition, not source-backed physical mutation
edges. Cold logical reset uses zeros by the existing RAM convention; physical
power-up contents are not asserted. Exact DATA output stage/bias, RAM partitioning,
PHI timing and propagation remain source-blocked. No new firmware bytes are added.

## Scheduling and causality

begin_cycle at b0 binds only the prior resolved command. The previous write or
address frame completes at b1. bind_body at b2 then selects the new target and
freezes any read response from current storage. Immutable source proposal joins
IS/SYNC/ACT DATA in one commit. Receivers validate all IS/SYNC and required DATA
observations before changing any state. Partial faults cannot consume one input
while leaving another unconsumed. Read tails survive into next b0/b1; invalid
phase, coordinate, ownership, unresolved levels or changed SYNC are hard errors.
The functional fixture requires 56 cells/224 PHI transitions on every word.

## Coverage and performance practice

Functional tests cover captured firmware opcodes, all 64 addresses with direct
and low-selector transfers, cross-word write/read dependency, peripheral ranges,
implied-GOTO exclusion, logical clear and neighbour preservation, reset, lifecycle,
wrong sole DATA owner, extra/missing drivers, phase faults, IS contention and
invalid/changed SYNC. Faults preserve the entire RAM state. Changing the legitimate
ACT drive before receive must change stored/read data, proving no host-payload
shortcut. ACT collection in the fixture reads the resolved net with ownership,
bit-count and cross-word checks.

Production storage and frames use fixed arrays with no per-cell allocation or
full-device copy. The device is cloned only in fault tests. The release benchmark
uses 7 successive samples of 10000 words on one continuous device,
with allocation-free per-word fixture collection and no speed correctness limit.
It includes the source/ACT receive checks and 224 PHI transitions per word; it
excludes desktop oracle/authority costs and physical chip/output circuitry.

Rust is absent here. Static source/document checks and formatting-only Actions
are reviewed; focused/full tests, release, diagnostic and four benchmark suites
require the owner gate. No runtime pass or measured throughput is claimed yet.

## Next live integration

Transmit real C-to-address DATA through the live composition, forward actual
IS/SYNC command observations to RAM, use RAM read drive instead of oracle payload,
and compare autonomous writes/selectors/clear against the reference. Preserve
clocked tails, oracle restoration and diagnostic cycles. Synthetic tests must
inject a consistent command stream or use an explicitly separate reference
composition. Per-IC RAM mapping/output-stage replacement remains a later evidence
boundary; never label this aggregate as resolved physical chip fidelity.
