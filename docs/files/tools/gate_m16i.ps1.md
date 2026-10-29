# tools/gate_m16i.ps1

## Purpose
Owner-local nine-stage M16I validation gate.

## Why it exists
Keep exact-head validation and argument forwarding reproducible in the repository.

## Relationships
Extends the owner-validated M16H v2 gate for the HDS/WPS integration. Follows AGENTS.md and docs/CI.md.

## Responsibilities
Reject dirty source/test/tool/Cargo inputs, print tested HEAD, stop on each failure, deny compiler warnings and restore caller flags/directory.

## Implementation
Only milestone names, focused m16i_ filters and marker differ from M16H. Runs fmt, both library/native focused variants, both all-target suites, both release binary builds, both ignored 12/12 diagnostics and serial electrical/DATA/RAM/live benchmarks. No Clippy substitution, persistent logs, merge or deployment.
