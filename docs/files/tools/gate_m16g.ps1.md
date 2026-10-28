# tools/gate_m16g.ps1

## Purpose
Run the complete owner-local M16G acceptance gate with fail-fast PowerShell argument forwarding.

## Why it exists
The M16G v1 pasted command accidentally substituted Clippy for the established compiler-warnings-denied all-target test gate. That stopped on inherited Clippy findings before full regression/release validation. Keeping the actual command in the repository makes its scope reviewable and avoids another long pasted command drifting from AGENTS.md and docs/CI.md.

## Relationships
Implements AGENTS.md and docs/CI.md for the resolved MOTOR branch. Exercises the normal and pin-capture builds, documentation contract, native firmware diagnostics, electrical/DATA/autonomous RAM benchmarks and actual live authority benchmark. Runtime execution remains owner-local; Actions only formats Rust.

## Responsibilities
Reject dirty source/test/tool/Cargo inputs, print the exact tested HEAD, stop at each native command failure, preserve Cargo's literal argument separator, deny compiler warnings, run the nine stages in order, and restore the caller's Rust flags and working directory on success or failure. Do not merge, fetch, select commits, write persistent logs or change emulator timing.

## Implementation
The caller first fetches and detaches the supplied exact SHA. The script runs relative to its own repository root. Invoke-HP67M16G accepts a named string-array CargoArgs parameter, preserving -- before libtest arguments. Formatting precedes all Rust execution. Existing RUSTFLAGS are retained with -Dwarnings appended and restored in finally. Both build variants receive all-target tests and release binary builds; both receive the ignored 12/12 diagnostic. Benchmarks run serially with one test thread. Clippy is not part of this established gate and no lint allow attributes are introduced. The green marker is printed only after every command succeeds; M16G remains unvalidated until the owner reports it.
