# `src/reference/mod.rs`

## Purpose

Defines the namespace for behavioural reference models used to validate the lower-level electrical emulator.

## Why it exists

The project needs an independent, easy-to-debug description of architectural results at microinstruction boundaries. This lets us reuse mature knowledge from projects such as Nonpareil without making an instruction-level model the final fidelity path.

## Relationships

It is exported by `src/lib.rs`. Its `woodstock` child is compared conceptually and, later, by differential tests with the timed ACT implementation under `src/emulation` and `src/machines/hp67`.

## Responsibilities

Keep reference models headless, deterministic and UI-free. Expose only semantic state/decoding needed for validation. Do not model presentation or claim electrical timing accuracy.

## Implementation

The module currently exports `reference::woodstock`. Additional reference models should be added only when they provide a concrete validation benefit and should remain separate from the production machine path.
