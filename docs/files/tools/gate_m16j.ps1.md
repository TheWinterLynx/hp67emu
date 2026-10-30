# tools/gate_m16j.ps1

## Purpose
Owner-local nine-stage M16J full validation gate.

## Why it exists
Keep the exact tested revision, full warning-denied suites and performance acceptance reproducible.

## Relationships
Extends the validated M16I gate for PGMSW; follows AGENTS.md and docs/CI.md and the documentation contract.

## Responsibilities
Reject dirty source/tests/tools/Cargo inputs, print HEAD, fail immediately on each native command, preserve argument separators and restore caller flags/directory.

## Implementation
Only milestone/filter/marker differ from M16I. Runs fmt, both focused library/native variants, both all-target suites with compiler warnings denied, both release binary builds, both ignored 12/12 diagnostics and serial electrical/DATA/RAM/live benchmarks. No Clippy substitution, logs, merge or deployment.
