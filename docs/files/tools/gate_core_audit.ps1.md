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

Focused `core_audit_` binary tests now run in both debug and release before the full
suite, covering live pacing and reset contracts as well as diagnostic I/O. These
checks complement the focused library tests; they do not replace full pin-capture
coverage or the ignored firmware diagnostic suite.

The dense-fabric failure contract also runs explicitly in release; full all-targets covers it in debug.

The release gate also verifies three Standard Pac polynomial samples through the isolated plot sampler and original firmware.

The Biorhythms regression reads both program tracks, enters the manual's example dates through the firmware and checks the first two PRINT/PAUSE triplets.
