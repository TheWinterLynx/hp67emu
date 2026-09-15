# `src/machines/hp67/fetch.rs`

## Purpose

Provides the structural HP-67 shared-word transport for ACT display traffic and ACT-to-ROM-to-ACT instruction fetch over the resolved IS/ISA electrical net.

## Why it exists

The instruction-boundary reference model can fetch a 10-bit word from a host array instantly, but the real HP-67 does not. Hardware evidence shows three separate serial windows within the same 56-bit machine word: the ACT emits the eight-bit ROM0 display code LSB-first at b0..b7, sends a 12-bit ROM address LSB-first during b16..b27, and the selected ROM returns a 10-bit word LSB-first during b46..b55. The emulator needs a transport layer that lets those evidenced roles coexist on one resolved bus and one word coordinate before physical ACT, ROM0 and generic 1818-* chip models are complete.

## Relationships

Uses the evidence-backed windows and serializers in `timing.rs` and `isa.rs`, the ROM0 receiver in `display.rs`, and the generic `LogicLevel`/`Drive` electrical primitives. `Hp67ElectricalBackplane` supplies the pull-down-biased IS/ISA net. Future 1818-* ROM devices will implement `Hp67RomWordSource`; the current semantic `reference::rom` layer is deliberately not imported here. Bank selection is intentionally not encoded into the 12-bit wire protocol because it is separate physical state, not an additional address bit on IS. `hp67_poweron_smoke.rs` continues to use the fetch-only runner for long real-microcode execution while the combined runner is the structural bridge toward a single electrical word cycle.

## Responsibilities

Represent ACT-side display-byte and address transmission, ROM0-side display reception, ROM-side address reception and returned-word transmission, reject floating/contentious samples and missing ROM locations, preserve LSB-first 8-bit/12-bit/10-bit widths, model the one-word fetch/execution pipeline boundary without embedding any HP firmware payload, and prove that display and fetch can occupy their evidenced non-overlapping windows on one deterministic structural machine word.

## Implementation

`ActFetchEndpoint` owns the 12-bit address being sent and reconstructs a returned 10-bit word only from samples taken in b46..b55. `RomFetchEndpoint` reconstructs the address only from resolved IS/ISA samples in b16..b27; after all twelve bits are present it performs one lookup through the `Hp67RomWordSource` trait and serializes that latched word during b46..b55. `Rom0DisplayEndpoint` receives b0..b7 from the same resolved IS net. All contributors use the wired-high convention from `isa.rs`: one actively drives High and zero releases the bus to its passive low bias.

`FetchPipelineLatch` has an explicit cycle boundary: `complete_cycle()` stores the word fetched during the current machine word, while `begin_cycle()` promotes the previously prefetched word to the executing slot. This makes a word fetched during cycle N eligible to execute during cycle N+1 rather than introducing an accidental two-cycle delay. `run_structural_fetch_cycle()` preserves the original fetch-only harness. `run_structural_display_fetch_cycle()` drives display b0..b7, ACT address b16..b27 and ROM response b46..b55 through the same `Hp67ElectricalBackplane` word and returns both the reconstructed display byte and fetched microinstruction. Tests pass the measured `0x07b -> 0x04c` fetch while simultaneously transporting display byte `0x30`, proving the two paths share one resolved 56-bit word without contention.

The module intentionally stops at the bit-cell boundary: exact PHI launch/sample edges, ROM0 sampling edges, STR/RCD overlap ordering and propagation delay remain unimplemented until the waveform evidence is transcribed unambiguously.
