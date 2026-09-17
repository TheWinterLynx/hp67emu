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

A small headless harness advances exactly the same structural word path used by the live machine. `press_and_settle()` closes a selected `Hp67Key`, samples S15 and the latched keycode into the ACT at instruction boundaries, waits for `keys -> a`, verifies the hardware code, waits for unshifted-table dispatch, opens the contact, and then runs real firmware until the main wait loop is reached with S15 low. The final assertion is `[00 00 00 4F 80 3F 3F 00 00 00 00 00 00 00 00]`: ROM0 segment masks for `3.00`, not a formatted host number.
