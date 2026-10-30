# tools/gate_m16k.ps1

## Purpose

Run the owner-only M16K local validation gate.

## Why it exists

Formatting, both warning-denied feature variants, real-firmware diagnostics and performance must be accepted before integration.

## Relationships

Uses cargo and the exact checked-out Git HEAD; follows the nine-stage M16J gate.

## Responsibilities

Fail first, restore RUSTFLAGS/directory, retain -- argument separation, print tested SHA and progress; never run Clippy or persist logs.

## Implementation

Typed CargoArgs invokes fmt, focused library/native tests in both variants, all-target tests in both variants, both release builds and both ignored 12/12 diagnostics. Electrical/DATA/autonomous RAM/live workloads are unchanged; stage 8 additionally runs the rotating CRC transaction kernel. The success marker identifies delivery 14/15, which is project sequencing rather than full physical fidelity completion.
