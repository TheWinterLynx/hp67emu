# `src/reference/mod.rs`

## Purpose

Defines the namespace for behavioural reference models used to validate the lower-level electrical emulator.

## Why it exists

The project needs an independent, easy-to-debug description of architectural results at microinstruction boundaries. This lets us reuse mature behavioural knowledge from projects such as Nonpareil without making an instruction-level model the final fidelity path.

## Relationships

It is exported by `src/lib.rs`. `reference::woodstock` provides the semantic CPU/RAM machine, `reference::crc` models the card-reader controller, `reference::hp67` composes those pieces into an HP-67-specific semantic machine, `reference::rom` supplies host-independent banked microcode storage, and `reference::snapshot` captures and compares instruction-boundary CPU state. These modules will be used together by differential checks against the timed implementation under `src/emulation` and `src/machines/hp67`.

## Responsibilities

Keep reference models headless, deterministic and UI-free. Expose semantic state, execution, peripheral behaviour, ROM fixtures and diagnostic snapshots needed for validation. Do not model presentation or claim electrical timing accuracy.

## Implementation

The module exports `reference::crc`, `reference::hp67`, `reference::rom`, `reference::snapshot` and `reference::woodstock`. Additional reference models should be added only when they provide a concrete validation benefit and should remain separate from the production electrical machine path.
