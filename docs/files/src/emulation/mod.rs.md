# `src/emulation/mod.rs`

## Purpose
Collects and re-exports generic simulation primitives.

## Why it exists
Calculator-specific chip models should share one deterministic implementation of time, electrical nets, device evaluation, scheduling and trace capture.

## Relationships
Used by machine modules such as `src/machines/hp67/`. It re-exports types from `clock`, `net`, `device`, `scheduler` and `trace` so machine code has one clean import surface.

## Responsibilities
Define the generic emulation namespace and prevent calculator/model details from leaking into the kernel.

## Implementation
Declares five focused modules and publicly re-exports their core types. The scheduler provides the order-independent resolve/snapshot/evaluate/commit loop. No GUI, HP part number or photograph dependency belongs here.
