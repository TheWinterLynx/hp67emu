# `src/hp67/m12_tests.rs`

## Purpose

Own the M12 integration regressions for RUN/PRGM hardware input, real PROGRAM-mode entry and stored-program execution without bloating the live machine implementation module.

## Why it exists

M12 needs a relatively large source-backed harness that observes firmware dispatch, user-program RAM, the firmware PROGRAM latch, the running flag and the physical LED frame. Keeping that harness inline in `src/hp67.rs` made the production-facing live-machine file harder to review and encouraged temporary diagnostics to accumulate there.

## Relationships

This is a child module of `src/hp67.rs`, so it can inspect the private live-machine state needed for white-box integration assertions without widening the production API. It uses `Hp67Key`, ACT operation metadata and the architectural RAM image from the reusable HP-67 core. The behavioral evidence is documented in `docs/research/M12_PROGRAM_EXECUTION_AUDIT_2026-09-18.md` and the cleanup decisions in `docs/research/M12_CODE_AUDIT_2026-09-18.md`.

## Responsibilities

Drive the mechanical RUN/PRGM flag, hold real key contacts through firmware `keys -> A` and `A -> ROM address` dispatch, verify exact program bytes and user-PC encodings, validate PROGRAM-mode `SST`, `h BST` and `h DEL`, validate RUN-mode single-step execution and non-executing back-step behavior, start stored programs through physical `R/S`, observe S1/S2 state transitions, and require the expected physical LED frames. Unknown or mistimed behavior remains a hard test failure.

## Implementation

The harness uses one common physical-dispatch helper for PROGRAM and RUN paths. Per PROGRAM key, it verifies real firmware dispatch and the exact user-PC step advance; exact storage is checked independently from PC motion. After a key path reaches the no-key firmware wait, display-sensitive observations allow one full 15-slot physical refresh so the frame oracle cannot mistake a partially refreshed LED frame for the settled result. The edit regression navigates with firmware `SST/BST`, deletes step 003 through real `h DEL`, verifies the shifted RAM bytes, reinserts Digit3 and then executes the edited `11 1B 13 37 00` program to physical `4.00`. The RUN stepping regression observes firmware S1, requires entry into the real user-instruction executor at `06021` for `SST`, and requires that `h BST` never enters that executor while restoring the original X display. Named cycle limits bound mode transitions, key dispatch, firmware settle and program execution. The tests do not implement calculator semantics or normalize machine state in host code.
