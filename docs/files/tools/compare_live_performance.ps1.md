# `tools/compare_live_performance.ps1`

## Purpose

Compare release live-authority benchmark timings for two exact commits on one Windows machine.

## Why it exists

Independent release transcripts combine code differences with host and toolchain variation. This script records both revisions and measures them in one session with the same explicitly selected Rust toolchain.

## Relationships

Uses Git worktrees, rustup and the ignored `m14o_live_authority_realtime_benchmark` binary test. Complements the release candidate gate; it does not replace correctness validation or desktop acceptance.

## Responsibilities

Resolve commit IDs before building, use each revision's lockfile, prebuild in separate detached worktrees, and capture all three benchmark rows. Keep the current checkout intact and log the raw outputs and comparisons under `target/performance-comparison`. Requires a Rust toolchain supported by both revisions and enough temporary disk space for two release builds.

## Implementation

Run A–B–B–A sequentially after both builds to reduce order bias. Print both per-run medians and the percentage difference between the averages of those medians, in microseconds per word. Positive means the candidate took longer. The production firmware idle row and two synthetic reference workloads are reported separately. Missing or ambiguous rows fail the script. No performance threshold is enforced, and two runs per revision do not establish causality. Clean up the temporary worktrees after measurement without forcing removal of changed worktrees.
