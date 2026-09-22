# src/machines/hp67/electrical.rs

## Purpose
Provides the fixed-topology electrical fabric and staged scheduling primitive used by the HP-67 production machine.

## Why it exists
The generic ElectricalScheduler deliberately favors model-independent clarity and uses ordered maps plus boxed devices. That is useful as a reference contract, but it is the wrong production representation for an HP-67 whose net and device inventory is known at compile time and whose PHI topology creates hundreds of thousands of host-side transitions per second. This file removes per-tick map construction, lookup and heap allocation without weakening electrical semantics.

## Relationships
Uses the generic Bias, Drive, LogicLevel and Tick primitives from src/emulation, and the HP-67-specific dense Hp67Net and Hp67Driver indices from wiring.rs. machine.rs owns this fabric as the calculator backplane. The generic scheduler remains available for model-independent tests and as a semantic reference.

## Responsibilities
Maintain committed drive state for every known HP-67 net/driver pair, resolve passive bias and contention, expose immutable resolved snapshots, stage output changes without making them visible during evaluation, validate each physical driver owner once during machine composition, and commit all staged changes atomically without implicitly advancing the timing coordinate.

## Implementation
Each net stores a fixed drive array plus cached low/high driver counts. The fabric also caches the resolved `LogicLevel` for every HP-67 net and updates only the nets whose committed drive changed. `begin_evaluation()` splits the fabric into a zero-copy immutable snapshot borrowing that resolved array and an independent mutable stager borrowing only pending-output storage, so all devices can read one stable electrical image without copying fourteen levels on every PHI transition. Pending device outputs use a fixed slot-marker matrix plus a fixed array of typed pending-drive entries. A zero marker means the `(net, driver)` slot has not yet been staged in the current evaluation; otherwise it points directly to the pending entry, so repeated staging updates that entry in place and preserves final-write-wins. Commit walks only the staged entries, clears their markers, and applies the typed `(net, driver, drive)` payload without a second pending-matrix lookup or `Option::take()` round trip. `claim_driver_owner()` issues one non-Clone/non-Copy `Hp67DriverOwner` token per physical driver identity. Devices retain that token and reuse it across later evaluations, so duplicate ownership is rejected at composition time rather than rechecked on every electrical edge. The current M14A bridge can still publish already-committed PHI/IS events immediately, while M14B devices use snapshot/stage/commit ordering. `commit_staged()` now changes electrical state only; timing advancement is a separate explicit `advance_tick()` operation. This prevents future settling or propagation commits from masquerading as extra PHI transitions.

## Invariants
A staged drive never changes the currently resolved level. The zero-copy snapshot and stager borrow disjoint fabric fields, so this isolation is enforced by Rust as well as by convention. `resolved_levels` is updated only during immediate bridge commits or atomic staged commit and must remain exactly equivalent to resolving the underlying dense driver counters. Every device participating in one future scheduler tick can therefore read the same immutable snapshot. Opposite committed drives resolve to Contention; contention diagnostics may allocate only on the exceptional error path. No timing edge, passive bias or physical driver ownership is inferred by this representation itself.
\n## Regression strategy\nUnit tests lock evidenced idle levels, snapshot invisibility before commit, one-time driver ownership, final-write-wins staging, contention reporting, and direct-index drive updates. A cross-check exhaustively compares every two-driver HighZ/Low/High combination on floating DATA and pull-down ISA against the generic Net resolver so the dense production representation cannot silently diverge from reference electrical semantics.\n

## Validated performance note

The direct typed pending-entry representation was validated on 2026-09-22 with the matched-observation benchmark. It reduced the dense staged scheduler from 2.954 us/word to 1.221 us/word on the measured host (108.33x to 262.05x versus the ~320 us physical word reference) while all correctness gates passed. The measured difference between stage+commit and the full zero-copy evaluation path was smaller than run noise, so snapshot borrowing is not currently a performance concern. These values are host-specific engineering measurements, not hardware-fidelity claims.


## Timing/commit separation

A scheduler commit is an electrical visibility boundary, not evidence that physical time or PHI advanced. Current PHI fixtures therefore perform `commit_staged()` followed by one explicit `advance_tick()`. Future same-coordinate settling events may commit without advancing PHI. Exact propagation timing remains source-blocked until hardware evidence supports it.
