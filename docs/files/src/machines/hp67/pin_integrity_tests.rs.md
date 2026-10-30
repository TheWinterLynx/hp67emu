# src/machines/hp67/pin_integrity_tests.rs

## Purpose

Regress the offline production ownership auditor.

## Why it exists

Wrong named sources can drive valid logic and missing ownership must not produce a green certificate.

## Relationships

Uses manually observed resolved backplane snapshots, physical ROM source identities and strict TSV roundtrip.

## Responsibilities

Verify source substitution, agreeing sources, all IS roles, DATA source limits, absent observations, direct-image validation and bounded findings.

## Implementation

Six normal-build tests cover synthetic complete captures, all foreign drivers on eleven sole-owner nets, wrong-window/page/reference IS, passive release versus active Low, DATA release/sole-source/overlap, unknown contacts/owners/address and truncated captures. A synthetic nanosecond fixture has no invented internal owners and returns incomplete. Finding storage is capped at 64 while 112 violations remain counted; input snapshots are unchanged.


## M16L v2 display delta regressions

Three additional tests model the actual first b0 edge before and after same-tick display publication, keep the original Floating row unchanged and verify TSV roundtrip; reject missing/agreeing sources in the final publication row for STR and RCD at each b0/b7/b8; and prove that same-tick repair cannot hide SYNC faults or repair an uninitialized control using a later tick/non-publication b1. Existing six tests and native expectations are retained.
