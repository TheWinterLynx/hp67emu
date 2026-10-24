# src/machines/hp67/clock_cache_tests.rs

## Purpose

Cross-check cached PHI ownership and isolate repeated ownership query cost.

## Why it exists

The owner requires performance preservation without weakening electrical fault
checks. A cache needs an independent source-of-truth comparison across every
committed mutation path.

## Relationships

A test child of electrical.rs uses public committed drives to reconstruct owner
masks independently of cached counts and the new summary. M16B's complete
transport/live fault campaigns remain enabled in both builds.

## Responsibilities

Cover source identity, missing/extra drivers, staged atomic visibility, failing
contention commits, release/reacquisition, clone/reset, non-clock independence
and deterministic mixed-net stress. Never use speed as a correctness assertion.

## Implementation

Three regressions enumerate all net/source/drive pairs, staged handoff orders
and 20,000 mixed updates against a full driver scan. An ignored release kernel
rotates cached and prior count-based queries in the same process. Both kernels
maintain the cache, resolve all 224 PHI transitions/word and perform eight probes
per bit. Inputs pass through black_box. This isolates query representation;
it is neither production throughput nor a whole-emulator M16B baseline.
