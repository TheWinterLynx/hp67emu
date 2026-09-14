# `src/emulation/mod.rs`

## Purpose
Collects and re-exports generic simulation primitives.

## Why it exists
Calculator-specific chip models should share one deterministic implementation of time, electrical nets, device scheduling contracts and trace capture.

## Relationships
Used by machine modules such as `src/machines/hp67/`. It re-exports types from `clock`, `net`, `device` and `trace` so machine code has one clean import surface.

## Responsibilities
Define the generic emulation namespace and prevent calculator/model details from leaking into the kernel.

## Implementation
Declares four focused modules and publicly re-exports their core types. No GUI, HP part number or photograph dependency belongs here.
