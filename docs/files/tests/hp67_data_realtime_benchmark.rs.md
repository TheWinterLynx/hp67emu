# `tests/hp67_data_realtime_benchmark.rs`

## Purpose

Measure M14B logical DATA performance without perturbing the long-lived electrical/structural benchmark binary.

## Why it exists

The first M14B implementation added DATA-only and firmware DATA comparison paths directly to `hp67_electrical_realtime_benchmark.rs`. After that benchmark binary grew, unrelated standalone rows such as raw PHI and structural fetch showed large code-generation-sensitive timing swings even though their implementation had not changed. Those swings made the established benchmark unsuitable as a stable regression reference.

Separating the M14B comparison into its own integration-test binary restores the original M14A benchmark source shape and isolates DATA-specific code generation.

## Relationships

Uses the same versioned `Hp67Firmware`, architectural oracle, structural display/fetch transport, `Hp67DataSerialSource`/`Sink`, and fused `Hp67DataSerialWordPath` used by the M14B implementation. The baseline, shadow and fused firmware paths are measured in rotating interleaved order.

## Responsibilities

Compare four things: real firmware without logical DATA, the deliberately pessimistic second-loop RAM DATA shadow, the fused RAM DATA phase that mirrors live integration, and the isolated logical DATA source/sink cost.

The three firmware paths must remain interleaved so sub-microsecond differences are not dominated by simple path-order drift. The isolated DATA row is intentionally separate because it answers a different question: the intrinsic cost of the logical cross-word phase engine.

## Implementation

Only transfers whose selected address is installed in `ActRamImage` participate in the RAM DATA experiment. CRC/card ports are excluded. The fused path advances DATA inside the same b0..b55 structural loop and completes bits 54/55 at the following word's b0/b1.

This is a performance benchmark, not an electrical-fidelity claim. DATA pin polarity, passive bias, device ownership, PHI-relative launch/sample edges, propagation and physical 1818-* RAM mapping remain source-blocked.
