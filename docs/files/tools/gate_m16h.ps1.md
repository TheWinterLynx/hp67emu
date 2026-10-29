# tools/gate_m16h.ps1

## Purpose
Owner-local full M16H gate in nine ordered stages.

## Why it exists
Keep the owner command reviewable and reproducible after M16G exposed drift in a pasted command.

## Relationships
Follows AGENTS.md and docs/CI.md; extends gate_m16g.ps1 for resolved MTRS and documentation-contract coverage.

## Responsibilities
Stop on every failed command, reject dirty source/tests/tools/Cargo inputs, print exact HEAD, deny compiler warnings, run both build variants and restore caller flags/directory.

## Implementation
Same named CargoArgs string-array forwarding, try/finally, fmt-first ordering, both focused suites, both all-target suites, both release builds, both ignored diagnostics and serial benchmarks as the validated M16G v2 runner. Only milestone/filter/marker change. No Clippy substitution, persistent logs, deployment or merge.
