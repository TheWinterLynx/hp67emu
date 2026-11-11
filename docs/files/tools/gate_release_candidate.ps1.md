# tools/gate_release_candidate.ps1

## Purpose
Run the automated release-candidate checks and retain their exact revision and output.

## Why it exists
Functional acceptance and performance evidence must refer to the same source revision.
A benchmark success alone must not certify physical hardware equivalence.

## Relationships
Calls tools/gate_core_audit.ps1, then the existing release electrical, DATA and live
benchmarks. Closure criteria are in docs/ARCHITECTURE_CLOSURE_v1.0.md.

## Responsibilities
Reject tracked uncommitted changes, stop on failures, preserve RUSTFLAGS and working
directory, and save a transcript under target/release-gate. It does not merge,
publish, install dependencies or alter calculator semantics.

## Implementation
PowerShell invokes the existing functional gate first, including formatting,
debug/release focused tests, all targets, pin-capture, release binaries and ignored
diagnostics. Then it explicitly runs three ignored release benchmarks with warnings
denied. It records timings without inventing a speed threshold. Existing benchmark
scope labels distinguish production idle from synthetic reference workloads.
A green automated gate still requires the documented manual desktop acceptance.

## Desktop consolidation (2026-11-11)

After the functional gate builds release, the wrapper copies only hp67emu.exe into a unique temporary directory and runs --verify-embedded-pdfs there. Cleanup occurs in finally. GUI acceptance remains manual.
