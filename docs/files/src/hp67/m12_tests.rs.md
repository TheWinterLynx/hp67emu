# `src/hp67/m12_tests.rs`

## Purpose

Own the M12 integration regressions for RUN/PRGM hardware input, real PROGRAM-mode entry and stored-program execution without bloating the live machine implementation module.

## Why it exists

M12 needs a relatively large source-backed harness that observes firmware dispatch, user-program RAM, the firmware PROGRAM latch, the running flag and the physical LED frame. Keeping that harness inline in `src/hp67.rs` made the production-facing live-machine file harder to review and encouraged temporary diagnostics to accumulate there.

## Relationships

This is a child module of `src/hp67.rs`, so it can inspect the private live-machine state needed for white-box integration assertions without widening the production API. It uses `Hp67Key`, ACT operation metadata and the architectural RAM image from the reusable HP-67 core. The behavioral evidence is documented in `docs/research/M12_PROGRAM_EXECUTION_AUDIT_2026-09-18.md` and the cleanup decisions in `docs/research/M12_CODE_AUDIT_2026-09-18.md`.

## Responsibilities

Drive the mechanical RUN/PRGM flag, hold real key contacts through firmware `keys -> A` and `A -> ROM address` dispatch, verify exact program bytes and user-PC encodings, start the stored program through physical `R/S`, observe S2 running and halting, and require the physical `3.00` LED frame. Unknown or mistimed behavior remains a hard test failure.

## Implementation

The harness uses one common physical-dispatch helper for PROGRAM and RUN paths. PROGRAM RAM snapshots are fixed-size arrays captured once before and after an insertion rather than repeatedly allocated inside settle loops. Named cycle limits bound mode transitions, key dispatch, firmware settle and program execution. The tests do not implement calculator semantics or normalize machine state in host code.
