# `src/bin/hp67_arithmetic_smoke.rs`

## Purpose

Prove the first complete HP-67 arithmetic sequence through physical key contacts and the real `hp67firmware` image versioned with the emulator: `1 ENTER 2 +`, ending in the physical `3.00` LED pattern.

## Why it exists

M8 proves hardware keyboard dispatch and M9 proves one entered digit reaches the real display path. M10 additionally exercises repeated press/release handshakes, ENTER stack behavior, a second numeric entry, arithmetic execution, RAM/stack state and the firmware-driven display update without falling back to host calculator semantics.

## Relationships

Uses `Hp67Firmware`, `Hp67Keyboard`, `Hp67ArchitecturalMachine`, the structural ACT/ROM shared-word transport, ROM0 display decoder and 1820-1749 cathode model. `Hp67Firmware` reads only the repository-versioned `hp67firmware.*` blocks embedded by `include_str!`; this smoke accepts no ROM filename and performs no runtime firmware I/O.

## Responsibilities

Re-prove firmware-driven M7 boot to `0.00`; re-prove M9 physical `1.` entry; present ENTER, 2 and + as separate physical contacts; require every key to pass through real `keys -> a` and `a -> rom address` firmware dispatch; release each contact and allow firmware to return to the no-key wait loop; and assert the final fifteen-slot physical segment pattern for `3.00`.

## Implementation

The harness boots the versioned firmware to the proven idle checkpoint, presents each key as a held `Hp67Keyboard` contact, verifies real `keys -> a` / `a -> rom address` dispatch, releases the contact and waits for firmware-owned S15 release handling. It asserts the raw ROM0 segment patterns for `1.` and final `3.00`. Display observation is deliberately non-invasive: `capture_display(&self)` creates local firmware, backplane, fetch, ROM0, ACT-serial and cathode objects, so reading the display cannot advance or mutate the harness under test. No host-side calculator arithmetic or formatted display value is injected.
