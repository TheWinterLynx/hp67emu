# HP-67 ACT serial execution evidence

Research snapshot: 2026-09-16.

## Purpose

This note defines the evidence boundary for moving the production HP-67 ACT model from instruction-boundary semantics toward true intra-word serial execution. The rule is deliberately strict: no register, ALU or carry transition is assigned to a specific HP-67 bit or PHI edge unless the available hardware evidence supports that assignment.

## Physical target

The production target remains the HP-67 ACT part identified by the HP-67 schematic and inspected hardware: **1820-2530**. HP-97 service documentation explicitly permits 1820-2530 as a replacement for 1820-1596 in the closely related programmable/card-reader logic assembly, so 1820-1596 family documentation is useful corroboration but not a license to copy unverified timing into the HP-67 model.

Primary project references:

- `docs/HARDWARE_SOURCES.md`
- `docs/research/HP67_ISA_TIMING.md`
- Tony Nixon, *Notes on HP's Classic Calculators*: https://literature.hpcalc.org/community/classic-notes.pdf
- HP-97 Service Manual: https://literature.hpcalc.org/community/hp97-sm-en.pdf

## Facts safe to encode now

### One machine word is 56 serial bit times

Direct HP-67 evidence uses a 56-bit machine word. The project coordinate is `b0..b55`, grouped as fourteen four-bit digit times:

```text
b0..b3    digit coordinate 0
b4..b7    digit coordinate 1
...
b52..b55  digit coordinate 13
```

This coordinate is already represented by `Hp67WordTiming::digit_index()` and `bit_in_digit()`.

### Fetch and execution are pipelined by one word

The HP-67 traces show that the word fetched in machine cycle N executes during machine cycle N+1. Therefore the final electrical ACT must execute the current instruction while the next address/ROM word exchange is occurring on the shared buses; execution cannot remain an instantaneous operation outside the 56-bit cycle.

### The ACT family performs serial arithmetic

The HP-97 service manual describes the compatible ACT-generation arithmetic unit as a 56-bit serial binary adder/subtracter operating on the A/B/C register set. This is strong HP-origin family evidence that arithmetic state must evolve serially rather than being replaced atomically at the instruction boundary.

This establishes the architecture direction, but not the exact HP-67 PHI edge on which each internal bit is written.

### Earlier HP serial-BCD evidence constrains hypotheses but does not prove ACT timing

An earlier Hewlett-Packard calculator patent describes the predecessor serial BCD datapath in substantially more detail. It states that the arithmetic/register circuit uses fourteen-digit 56-bit registers and a serial BCD adder/subtracter. For decimal correction, the design cannot know whether correction is required until the first three bits of a digit's sum have been generated, so it uses a four-bit holding register before inserting the corrected result.

This is valuable family-level evidence for **bit-serial activity within each four-bit digit**, and it argues strongly against a model that treats a whole BCD digit as an indivisible arithmetic event. However, it is not HP-67/1820-2530-specific evidence. hp67emu must not copy the predecessor's holding-register placement, correction edge, address timing or internal register topology into the ACT unless an ACT-generation source corroborates it.

Reference: Hewlett-Packard patent material describing the 56-bit serial BCD arithmetic/register design, including the four-bit decimal-correction holding register. The project treats this as architecture-family evidence only.

### Existing HP-67 external windows stay fixed

Moving arithmetic inside the word must preserve the already locked direct HP-67 bus facts:

```text
b0..b7    display data observed by ROM0
b16..b27  ACT -> ROM 12-bit address, LSB first
b46..b55  selected ROM -> ACT 10-bit word, LSB first
```

The current live-bit display transport is intentionally useful here: `ActSerialEndpoint` now reads A/B at the actual b0..b7 bit-cell visit instead of snapshotting their nibbles at word start. Once A/B truly evolve intra-word, ROM0 will therefore see that evolution without another display bridge.

## Facts that are not yet safe to encode

The current source set does **not** yet prove any of the following for the HP-67/1820-2530:

- which PHI edge launches an internal register bit into the ALU;
- which PHI edge captures the ALU result back into A, B or C;
- whether the project's architectural digit index 0 corresponds directly to the first four internal ALU bit times of the 56-bit word for every operation;
- the precise temporal relationship between carry generation and the next serial bit;
- whether a clear/copy/exchange result becomes externally observable before or after the corresponding internal shift/capture edge;
- exact DATA-line launch/sample timing for RAM transfers;
- exact interaction between internal arithmetic updates and b0..b7 display serialization when they refer to the same register storage;
- whether an operation-specific qualifier delays a write within a digit time;
- whether the predecessor serial-BCD holding-register/correction arrangement is physically unchanged in 1820-2530.

Therefore code must not currently implement rules such as "commit every nibble on b3", "write A on PHI2", or "carry becomes visible on the next bN" merely because they are plausible.

## Consequence for M3 implementation

The next production ACT slice should separate **coordinate/scheduling** from **state mutation**.

Safe first layer:

1. latch the executing 10-bit word from the existing one-word pipeline;
2. decode the instruction once for the current execution word;
3. step an ACT execution object through all `b0..b55` coordinates alongside the existing structural transport;
4. expose the current digit index, bit-in-digit, selected field and operation to trace diagnostics;
5. perform no new register/carry mutation at an invented edge;
6. require an explicit evidence-backed mutation rule before an operation migrates from the architectural fallback into the serial executor.

This lets the emulator gain the correct execution *lifetime* immediately while retaining instruction-boundary semantics as a temporary fallback for operations whose internal timing is still unknown.

## Migration rule for individual operations

An arithmetic opcode can move from the architectural fallback to serial execution only when all of the following are known or bounded sufficiently for a deterministic model:

- source register(s);
- destination register(s);
- field selection;
- serial bit/digit ordering relevant to that operation;
- carry/borrow direction where applicable;
- the bit-cell or PHI-relative commit rule required for externally observable behavior.

The completed serial result must be differential-tested against the existing architectural core at the instruction boundary. The architectural core remains the oracle for semantics, not for electrical timing.

## M14D boundary-authority bridge

M14D deliberately separates **final-state causal authority** from **physical intra-word mutation timing**. The source set still does not justify writing A/B/C or carry on a particular HP-67 bit or PHI edge, so the production model does not do that.

For ADD/SUB-family arithmetic, the structural endpoint now initializes a private result image from the pre-instruction snapshot and accumulates selected-digit results from the same b0..b55 traversal that already owns instruction lifetime and carry/borrow chaining. The instruction-boundary architectural executor is still run once, but its A/B/C/carry mutation is immediately restored and retained only as the expected final-state oracle. After the structural word completes, the accumulated image must match that oracle exactly before the live A/B/C/carry state is updated.

This is classified as a **WORKING APPROXIMATION** at the physical-timing level. It removes the architectural executor from the causal final ADD/SUB result path and makes the real structural traversal the producer of that result, but the final handoff at word completion is not evidence that the 1820-2530 physically waits until b55 to update storage. The existing source-blocked questions about bit-level write timing, carry visibility, decimal-correction holding behavior and PHI-relative capture remain unchanged.

True migration of internal register mutation onto individual bit/PHI events still requires the evidence listed in the migration rule below. M14D must therefore not be cited as proof of exact internal ACT timing.

## Validation target for the first serial opcode

For the first migrated operation, tests must record every `b0..b55` step and prove both:

1. the intermediate register/carry evolution follows the reviewed timing evidence; and
2. after b55, A/B/C/P/carry and other affected architectural state exactly match the existing instruction-boundary implementation.

A passing final-state comparison alone is not sufficient to claim cycle accuracy.

## Research still required

The highest-value evidence search is now specifically internal ACT timing rather than more ROM-fetch work. Candidate sources are:

- enlarged Woodstock/HP-67 PHI traces in Tony Nixon's notes where internal/external timing relationships can be bounded;
- HP-97 service-manual theory sections for ACT DATA/ALU timing that can be shown to apply to the compatible ACT revision;
- original Woodstock-era HP Journal diagrams and patents describing ACT/arithmetic timing;
- Tom Napier's HP-67 anatomy articles if scans expose the 56-bit machine-time register ordering;
- direct logic-analyser capture from a physical HP-67 if an externally visible operation can constrain internal update timing.

Until one of those sources resolves the missing ordering/edge facts, hp67emu must keep the unknown explicit rather than promoting a plausible bit-serial algorithm to hardware truth.
