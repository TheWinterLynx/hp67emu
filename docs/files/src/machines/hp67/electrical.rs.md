# src/machines/hp67/electrical.rs

## Purpose
Provides the fixed-topology electrical fabric and staged scheduling primitive used by the HP-67 production machine.

## Why it exists
The generic ElectricalScheduler deliberately favors model-independent clarity and uses ordered maps plus boxed devices. That is useful as a reference contract, but it is the wrong production representation for an HP-67 whose net and device inventory is known at compile time and whose PHI topology creates hundreds of thousands of host-side transitions per second. This file removes per-tick map construction, lookup and heap allocation without weakening electrical semantics.

## Relationships
Uses the generic Bias, Drive, LogicLevel and Tick primitives from src/emulation, and the HP-67-specific dense Hp67Net and Hp67Driver indices from wiring.rs. machine.rs owns this fabric as the calculator backplane. The generic scheduler remains available for model-independent tests and as a semantic reference.

## Responsibilities
Maintain committed drive state for every known HP-67 net/driver pair, resolve passive bias and contention, expose immutable resolved snapshots, stage output changes without making them visible during evaluation, and commit all staged changes atomically at the next deterministic tick.

## Implementation
Each net stores a fixed drive array plus cached low/high driver counts. The fabric also caches the resolved `LogicLevel` for every HP-67 net and updates only the nets whose committed drive changed, so ordinary reads and immutable per-tick snapshots never re-resolve all fourteen nets. Pending device outputs use a fixed two-dimensional option array and a fixed dirty-key list; normal ticks perform no heap allocation and commit work is proportional to changed outputs. The current M14A bridge can still publish already-committed PHI/IS events immediately, while M14B devices use snapshot/stage/commit ordering.

## Invariants
A staged drive never changes the currently resolved level. `resolved_levels` is updated only during immediate bridge commits or atomic staged commit and must remain exactly equivalent to resolving the underlying dense driver counters. Every device participating in one future scheduler tick can therefore read the same immutable snapshot. Opposite committed drives resolve to Contention; contention diagnostics may allocate only on the exceptional error path. No timing edge, passive bias or physical driver ownership is inferred by this representation itself.
\n## Regression strategy\nUnit tests lock evidenced idle levels, snapshot invisibility before commit, final-write-wins staging, contention reporting, and direct-index drive updates. A cross-check exhaustively compares every two-driver HighZ/Low/High combination on floating DATA and pull-down ISA against the generic Net resolver so the dense production representation cannot silently diverge from reference electrical semantics.\n