# tests/electrical_failure_contract.rs

## Purpose
Lock observable dense electrical fabric state after errors.

## Why it exists
Callers must distinguish failed preflight from a committed image that reports contention.
An Err is not a general promise of rollback.

## Relationships
Exercises public Hp67ElectricalFabric APIs. Hp67LiveMachine separately latches
pacing failures until reset; these lower-level recovery tests do not permit live retry.

## Responsibilities
Prove all staged outputs publish before contention is reported, time does not
advance, pending entries are drained, and rejected evaluation leaves pending work intact.

## Implementation
Two regressions stage real driver contributions, cause contention and observe both
the faulting net and an independent net. Explicit external-drive release tests that
old pending writes are not replayed. No mock timing or per-edge production checks
are added. This covers the dense-fabric facet of A06, not all device error paths.
