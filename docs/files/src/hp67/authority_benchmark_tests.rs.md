# `src/hp67/authority_benchmark_tests.rs`

## Purpose

Measures the actual desktop live authority bridge after M14N, separately from historical structural fixtures.

## Why it exists

The historical dual benchmark never called the live restore guard and result commits. Treating that row as current production throughput concealed the cost of the new authority path.

## Relationships

A test-only child of `src/hp67.rs` uses its real cycle, machine, backplane and port lifecycle. It retains the same HP-67 physical 320 us/word reference and workload environment variables as the historical integration benchmarks.

## Responsibilities

Measure actual firmware idle, synthetic alternating installed-RAM ports and synthetic CRC ports; boot/warm outside timed samples; keep machines continuous between rotating-order rounds; report median/min/max us per word, words/s and hardware multiple. Require exact word, oracle execution and 224-transition counts. Label host fixture overhead and source-blocked electrical scope.

## Implementation

The ignored release-only benchmark directly calls `step_firmware_cycle_with_execution`, including snapshot/binding, architectural oracle, restoration guard, structural transport, exact result comparisons, authority commits and card-transport advance. RAM fixtures retain the continuous logical DATA tail. CRC fixtures refill/drain logical buffers within timed samples, with no inserted physical card; their numbers are stress costs, not card throughput. A normal regression runs odd/even word batches to lock continuous fixture state, tick counts, execution counts, port ownership and reconstructed RAM payload. Invalid workload environment values fail instead of silently using a default. Wall-clock Instant appears only in benchmark instrumentation. No fixed host speed threshold is a correctness claim.

## M15E measured scope

The actual live path now includes the opted-in resolved DATA logical-level bridge.
The output labels distinguish that executable path from unresolved physical DATA
output circuitry/chip mapping. The RAM stress row includes the bridge; the CRC
row retains its separate peripheral path. Workloads and timing boundaries are
unchanged.

## M15J five-ROM composition

Printed live workload scope now names the five-ROM serial fetch. All three
workloads use the actual live endpoint composition; RAM/CRC stress remains
synthetic and the existing timing/authority assertions remain unchanged.

Exact launch/latch delays remain unproven. The part/profile evidence and
remaining bank/RAM bridges are recorded in
`docs/research/M15J_FIVE_ROM_ENDPOINTS_2026-10-16.md`.

## M15K autonomous ROM bank control

The actual desktop benchmark now includes autonomous ROM bank decoding, and
its scope label says so. Workloads, timing methodology and assertions remain
unchanged. Synthetic port loads still do not prove physical card throughput.

See `docs/research/M15K_AUTONOMOUS_ROM_BANKS_2026-10-16.md`.

## M15K synthetic fetch coherence correction

The owner's v2 gate passed stages 1–8 and failed the live benchmark at cycle
2198: ROM bank 1 versus ACT bank 0 at PC 7f5. Synthetic RAM/CRC injection
replaced every next executing word while its PC kept walking the real ROM.
At 7f4 the real resolved return was bank-set 230, observed by all ROMs. The
next injected port replaced that opcode before ACT executed it, correctly
triggering the production bank-consistency failure. This is not evidence for
changing ROM latch or ACT behavior.

Only synthetic workloads now clear the inherited delayed-ROM selector during
fixture setup and pin the pre-execution fixture PC to 000, so the concurrent
real ROM fetch is always 001/3e3, including a one-word warmup. The assignment is included in timed
work. Actual firmware idle has no PC override. No source-bank setter, ROM-latch
reset, alternate executor or disabled guard is introduced. Bus resolution,
56 cells, 224 transitions, oracle comparisons and DATA/CRC lifecycles remain.
The printed scope explicitly identifies the pinned PC. Old synthetic figures
used a different address workload and are not directly equivalent measurements.

A negative regression reproduces the former 7f4/7f5 mismatch and requires the
unchanged guard to reject it. A positive regression starts both synthetic loads
at that boundary and runs 1 + 2046 + 2050 words, asserting complete execution/tick
counts, address 001, returned word 3e3, zeroed autonomous bank latches and RAM/CRC
invariants across batches. Rust runtime validation remains the owner's gate.


M15P: the production live path now includes an independent resolved IS/SYNC RAM
command observer on every word. The benchmark scope reports that cost and notes
that synthetic injected execution does not feed this observer: it receives the
concurrent real ROM fetch. Autonomous RAM storage/read response is still measured
only by the separate M15N fixture. No workload, timing loop or threshold changed.


## M15Q autonomous live RAM

Production construction now installs independent Hp67SerialRam storage. The
shared DATA transports and clocked prefix begin its cycle at b0, bind its body
at b2, publish its own read proposal jointly with IS/SYNC/ACT DATA, and receive
actual pins at the existing provisional checkpoint. The read payload passed to
the phase path is a zero frame mask; oracle RAM bits are never published.

Shared receive first prepares the RAM input image, validates/consumes DATA, then
commits the prepared RAM sample. No full RAM clone or allocation occurs per bit.
The old public transport APIs wrap the same loops with no RAM for reference
fixtures. Production live uses Some(RAM), reset preserves that composition, and
the only no-RAM live constructor is test-only. Host-injected opcode/RAM fixtures
explicitly use that constructor because their execution differs from real IS.
Actual firmware boot/key/program/card/diagnostic paths retain autonomous RAM.

Live checks command reception, full address completion, net write against device
storage, selector and block clear against the oracle. Reference RAM remains an
instruction-boundary comparison mirror; ACT read commits still come from resolved
DATA after next b1. Physical per-IC mapping/output stage, precise internal latch
edges and PHI propagation remain unproven. See
`docs/research/M15Q_LIVE_AUTONOMOUS_RAM_2026-10-20.md`.


### M15T autonomous CRC control

The shared resolved sampling path now carries an independent CRC IS/SYNC
receiver and twelve control latches. Production constructors install it; the
explicit injected-opcode reference fixture disables it. Each command executes
one word after qualified reception and publishes a completed b55 result. The
live adapter checks the old mirror before freezing external contacts, compares
against both the oracle and M14L image, and commits the endpoint's result/S3.
Only explicit logical FIFO/transport handshake changes enter READY/F7; motor,
write mode and all other latches retain the endpoint's own state. Reset clears
the endpoint and preserves the selected production/reference composition.
Per-sample CRC/RAM command images commit only after DATA validation succeeds.
No CRC DATA serialization or physical F2/PHI latch timing is asserted. The live
benchmark includes the endpoint only in its production firmware row; injected
RAM/CRC workloads retain the explicit reference bridge.


### M15U ACT received instruction authority

Production ACT endpoints retain the complete qualified IS/SYNC frame at b55 and
consume it at the following execution boundary. The live pipeline stores only a
comparison mirror; it cannot supply a production opcode or override the received
qualifier. Mirror checks precede DATA-prefix clock advancement. Reset clears the
pending frame and preserves the production/reference choice. Synthetic injected
fixtures explicitly disable received execution, autonomous RAM and CRC control.

All four shared transports use the same atomic, ordered ACT return sampler.
Wrong phase/order, unresolved IS/SYNC, changed qualifier and unconsumed-frame
overwrite fail before either receive latch changes. No full ACT copy is made
per bit. Standalone IS-only reception remains an ordered reference adapter.
The command frame reuses the existing raw word/qualifier carrier; its RAM decoder
is not part of ACT execution. This preserves the existing 56-cell/224-transition
schedule and provisional PHI checkpoint, without claiming internal physical
ACT write edges or propagation. See the M15U research note.


### M15V retained ACT flow latches

Production ACT endpoints now retain PC, bank, delayed-ROM selection, the two
return addresses and stack pointer independently of the architectural mirror.
The flow image receives those latches explicitly; carry, dispatch A/key inputs
and instruction classification keep their existing frozen ACT input contract.
The own successor preview feeds the ROM address serializer, including startup.
Owned latches advance only when the current execution completes b55, with the
existing low-page bank-zero normalization. Live compares every flow field before
clocks or DATA tails and after the checked result reaches the mirror. Reset
clears latches and preserves the selected production/reference composition.
No full ACT copy or heap allocation is added per bit. This retains M14I's logical
preview/completion scheduling; exact PC/stack write edges remain SOURCE-BLOCKED.
See `docs/research/M15V_ACT_OWNED_FLOW_2026-10-22.md`.
