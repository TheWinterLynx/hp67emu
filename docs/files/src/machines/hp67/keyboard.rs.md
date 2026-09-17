# `src/machines/hp67/keyboard.rs`

## Purpose

Models the HP-67 physical keyboard state presented to the ACT at instruction boundaries without performing calculator-level key semantics.

## Why it exists

The HP-67 firmware does not treat a key code alone as a key press. It polls status bit S15 to detect a closed key contact and separately executes `keys -> a` or related keyboard instructions to consume the hardware key code. A held contact must therefore be able to reassert S15 after firmware clears it, while key release must not itself clear firmware-owned status state.

## Relationships

Uses `ActArchitecturalState` from `act.rs` as the current instruction-boundary ACT bring-up interface. The 35 hardware key codes follow the preserved HP-67 keyboard definition and firmware listings. The module does not call calculator operations, edit display state, or dispatch functions.

## Responsibilities

Represent one physical key contact, preserve the most recently driven hardware key code independently of contact release, assert ACT status bit S15 while a contact remains closed, and leave S15 untouched when the contact is open so firmware release-detection loops remain authoritative.

## Implementation

`Hp67Key::scan_code()` maps all 35 physical keys to their historical octal hardware codes, including digit 1 as `0142`. `Hp67Keyboard::press()` closes a contact and updates the hardware code; `release()` only opens the contact. `sample_into_act()` presents the retained code through `key_buffer` and asserts S15 only while a contact is closed. Unit tests verify code uniqueness and the critical S15 lifecycle: firmware can clear S15, a held contact reasserts it, release does not synthesize a clear, and after release a subsequent firmware clear remains low.
