# `src/machines/hp67/fetch.rs`

## Purpose

Provides the first structural ACT-to-ROM-to-ACT serial instruction-fetch path for the HP-67 over the shared IS/ISA electrical net.

## Why it exists

The instruction-boundary reference model can fetch a 10-bit word from a host array instantly, but the real HP-67 does not. Hardware evidence shows the ACT sends a 12-bit address LSB-first during b16..b27 and the selected ROM returns a 10-bit word LSB-first during b46..b55. The emulator needs a transport layer that reconstructs both values from individual resolved bus levels before physical ACT and 1818-* chip models are complete.

## Relationships

Uses the evidence-backed windows and serializers in `timing.rs` and `isa.rs`, and the generic `LogicLevel`/`Drive` electrical primitives. `Hp67ElectricalBackplane` supplies the pull-down-biased IS/ISA net used by the integration tests. Future 1818-* ROM devices will implement `Hp67RomWordSource`; the current semantic `reference::rom` layer is deliberately not imported here.

## Responsibilities

Represent ACT-side address transmission and ROM-word reception, represent ROM-side address reception and returned-word transmission, reject floating/contentious samples and missing ROM locations, preserve LSB-first 12-bit/10-bit widths, and model the one-word fetch/execution pipeline boundary without embedding any HP firmware payload.

## Implementation

`ActFetchEndpoint` owns the 12-bit address being sent and reconstructs a returned 10-bit word only from samples taken in b46..b55. `RomFetchEndpoint` reconstructs the address only from resolved IS/ISA samples in b16..b27; after all twelve bits are present it performs one lookup through the `Hp67RomWordSource` trait and serializes that latched word during b46..b55. Both sides use the wired-high convention from `isa.rs`: one actively drives High and zero releases the bus to its passive low bias.

`FetchPipelineLatch` keeps the just-fetched word separate from the word eligible to execute during the following 56-bit cycle, matching the published HP-67 pipeline description. Tests pass the measured `0x07b -> 0x04c` example and physical-startup `0x001 -> 0x3e3` word through an actual `Hp67ElectricalBackplane` one bit at a time. The module intentionally stops at the bit-cell boundary: the exact PHI launch/sample edge and propagation delay remain unimplemented until the waveform evidence is transcribed unambiguously.
