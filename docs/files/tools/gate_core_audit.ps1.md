# Core audit gate

## Purpose
Run owner-local checks for the core integrity corrections.

## Why it exists
Formatting success alone does not validate Rust or firmware behavior.

## Relationships
Runs Cargo against the repository containing the script.

## Responsibilities
Fail immediately on formatting, warnings, tests, release or diagnostics failure.

## Implementation
Checks formatting without modifying files, runs focused and full tests including pin capture, builds release binaries and runs the ignored firmware diagnostic suite. Restores working directory and RUSTFLAGS on success or failure.

The gate additionally runs core_audit_ library regressions in release with warnings denied, so release-only missing invariants cannot hide behind debug assertions.
