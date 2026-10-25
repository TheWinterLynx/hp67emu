# src/machines/hp67/sync_tests.rs

## Purpose

Exercise SYNC source identity, stage visibility, all return bits, receiver non-mutation, transport entry and diagnostic capture.

## Why it exists

A valid binary SYNC level did not prove that the ACT alone supplied the instruction/data qualifier. Agreeing or substituted foreign sources could escape contention-only checks.

## Relationships

Uses the named backplane, dense contributor counts and direct ACT drive slot. ACT reception and the shared RAM/CRC command receiver enforce the same policy. See docs/research/M16E_SYNC_SOURCE_INTEGRITY_2026-10-25.md.

## Responsibilities

Reject missing, replaced, contended and agreeing foreign sources before state mutation; preserve retry and all 224 PHI transitions. Keep physical SYNC timing uncertainty explicit.

## Implementation

The normal path checks existing fixed-size counts/slots and resolved binary level. Only the cold error path enumerates a driver mask. Transport entry alone permits an undriven pristine tick zero; actual receivers never accept that exception. No new storage, heap allocation, per-bit hook or drive is added. Library/native tests cover sources, both qualifiers, all ten receive positions, six transports, pending tails and optional capture.
