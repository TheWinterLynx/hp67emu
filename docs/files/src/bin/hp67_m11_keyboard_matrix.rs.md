# `src/bin/hp67_m11_keyboard_matrix.rs`

## Purpose

Exercise the HP-67 physical keyboard and function-dispatch surface through the real versioned firmware as the M11 validation frontier.

## Why it exists

M10 proves one arithmetic path, but it does not establish that all 35 physical contacts, decimal entry, clearing, the four arithmetic operators, or the shifted `f/g/h` paths can traverse the same hardware/firmware route. M11 needs a repeatable headless matrix before individual function results are promoted to exact regressions.

## Relationships

Uses `Hp67Firmware`, `Hp67Keyboard`, `Hp67ArchitecturalMachine`, the structural display/fetch cycle, ROM0 display decoding and the 1820-1749 cathode scan model. It deliberately duplicates the proven M10 harness shape rather than introducing a shared semantic calculator helper.

## Responsibilities

Boot real firmware to the established no-key idle; verify all 35 hardware scan codes are captured by `keys -> a` and reach the firmware key table through `a -> rom address`; verify exact physical display patterns for digits 0 through 9, decimal entry, CLX and the four basic arithmetic operations; report CHS and EEX display frontiers; exercise every `f/g/h + physical key` pair for architectural/serial execution failures; and fail loudly on any transport, decode, keycode, dispatch or execution error.

## Implementation

Each direct-key matrix case starts from a fresh power-on state, closes one `Hp67Keyboard` contact, waits for the real firmware's `keys -> a` and `a -> rom address` instructions, opens the contact, and verifies firmware can clear S15. Exact function sequences use the same `press_and_settle` path as M10 and compare raw 15-slot ROM0 segment masks. Shifted-function coverage starts from a fresh boot with X=1, applies a real `f`, `g` or `h` prefix, dispatches the second physical key, releases it, then continues executing real firmware for a bounded observation window. No host-side calculator result is computed or injected.
