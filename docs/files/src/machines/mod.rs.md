# `src/machines/mod.rs`

## Purpose
Names calculator-specific machine compositions.

## Why it exists
Generic electrical simulation and specific calculator wiring are different concerns. This namespace keeps model-specific decisions out of the kernel.

## Relationships
Currently exports only `hp67`; future calculators get sibling modules rather than conditionals inside HP-67 code.

## Responsibilities
Be the model-composition namespace and enforce the direction `machine -> generic kernel`, never the reverse.

## Implementation
Contains only module declarations. Shared chip families should be extracted later only after multiple machine implementations prove real reuse.
