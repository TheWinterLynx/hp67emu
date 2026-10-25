# src/machines/hp67/sync_cache_tests.rs

## Purpose

Verify SYNC source ownership metadata against independent full scans and compare query cost in one process.

## Why it exists

M16E introduced repeated receiver source checks; caching their exact committed predicate must preserve every source fault and commit path.

## Relationships

Tests electrical fabric immediate/staged/joint serial paths, backplane forwarding and sync.rs public validation. Retains all original M16E source/live/capture regressions.

## Responsibilities

Lock cache equivalence, first attachment, foreign agreeing/replaced sources, invalid polarity changes, stage visibility, contention commits, clone/reset and PHI independence. Keep benchmark scope explicit.

## Implementation

Five regressions independently scan source arrays across exhaustive pairs, 20,000 mixed writes and all serial bundle/foreign-source combinations. An ignored rotating release kernel retains 56 cells/224 PHI transitions and cache maintenance in both paths while comparing 33 equivalent cached/count-based queries. It isolates query cost, not whole-emulator old/new speed. No timing threshold asserts correctness.
