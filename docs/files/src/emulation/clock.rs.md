# `src/emulation/clock.rs`

## Purpose
Provides deterministic simulation time and the initial non-overlapping two-phase clock scaffold.

## Why it exists
The HP-67 cannot be cycle-accurate if microinstructions are executed as atomic host-language calls. Clock phases and later individual bit times need an explicit timeline.

## Relationships
Used by `Hp67ElectricalBackplane` today and eventually by the global scheduler/ACT timing model.

## Responsibilities
Represent monotonic ticks, expose PHI1/PHI2 line levels, guarantee the two phases never overlap, and avoid baking in an unverified physical frequency.

## Implementation
A four-slot state machine produces PHI1-high, dead-time, PHI2-high, dead-time repeatedly. Unit tests lock the sequence and non-overlap invariant. Absolute HP-67 timing remains a roadmap item pending source-backed calibration.
