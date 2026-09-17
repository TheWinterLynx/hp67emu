# `src/bin/hp67_digit_display_probe.rs`

## Purpose

Continue from the proven physical Digit1 keyboard dispatch through real HP-67 firmware until the machine returns to its no-key wait loop, then capture the physical ROM0/1820-1749 display state without injecting calculator semantics.

## Why it exists

M8 proves that a real keyboard contact asserts S15, supplies hardware code `0142`, reaches `keys -> a`, and dispatches through firmware to the source-backed unshifted key-72 table entry. M9 must not jump directly from that dispatch to an assumed digit result. This probe exposes the next real hardware/firmware frontier first.

## Relationships

Uses the same structural ACT/ROM shared-word transport, architectural ACT+CRC machine, keyboard model, ROM0 display endpoint, cathode driver and external Teenix-derived ROM corpus as the M7/M8 smokes. The source-backed firmware landmark for Digit1 is octal `1440`, the unshifted key-72 table entry in the reviewed HP-67 disassembly.

## Responsibilities

Re-prove M7 boot idle and the source-backed `0.00` display; reproduce the M8 Digit1 path through S15, `keys -> a`, remapping and `a -> rom address`; release the physical key while preserving the latched keycode and firmware-owned S15 clearing; continue real firmware through post-key display initialization and back to the no-key main wait loop; capture the resulting fifteen physical display segment slots and ACT A/B/C state.

## Implementation

The probe advances the same `Hp67ArchitecturalMachine` and structural shared-word transport used by the earlier firmware smokes. Before each instruction boundary it samples the external `Hp67Keyboard` into ACT state, executes the currently latched firmware word, then performs the full structural fetch/display cycle and verifies that the serial execution reached the end of the 56-bit word. After the source-backed Digit1 dispatch reaches octal `1440`, the contact is released without clearing the latched code or S15 directly. Firmware is then allowed to execute until it returns through display initialization to the no-key wait loop, where a fresh fifteen-slot ROM0/cathode scan is captured and printed together with ACT A/B/C state. No calculator-level digit or display state is written by the host.

## Non-goals

This probe does not yet declare M9 passed and does not hard-code what the post-Digit1 display must look like. The observed source-derived segment pattern is intentionally printed first so the exact M9 assertion can be established from the real firmware path rather than guessed.
