# tools/gate_m16l.ps1

## Purpose

Run the owner-only delivery 15 M16L gate.

## Why it exists

The new offline audit must be checked in both feature configurations without changing the existing runtime/performance acceptance workloads.

## Relationships

Follows the nine-stage M16K v2 local gate with m16l_ focused tests.

## Responsibilities

Fail first, restore flags/directory, keep compiler warnings denied and exact tested SHA, both diagnostic variants and no persistent logs.

## Implementation

Runs formatting, focused library/native tests, all-target tests, release bins and ignored diagnostic 12/12 in both variants. Electrical/DATA/RAM/live benchmarks and the M16K CRC matched kernel are retained unchanged. Actions is formatting only. Delivery 15 indicates the agreed implementation sequence, not full measured physical completion.


## M16L v2

Marker and progress now identify v2. All cargo calls, focused filters, both variants, nine stages and benchmark workloads are unchanged. The correction concerns offline STR/RCD post-publication observations only. Owner runtime gate remains pending.
