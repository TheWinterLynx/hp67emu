# `tests/keyboard_contact_path.rs`

## Purpose

Locks the first end-to-end architectural keyboard checkpoint: a physical HP-67 key contact reaches the ACT firmware key-dispatch primitive as the historical scan code, without any semantic shortcut.

## Why it exists

Adding a keyboard table is not sufficient evidence if UI or host code can still bypass firmware and directly perform calculator operations. This test protects the intended architecture by proving that digit `1` is presented as the hardware code `0o142` and that the existing Woodstock ACT dispatch instruction consumes that code to form the firmware target address.

## Relationships

Uses `Hp67Keyboard` and `Hp67Key` from `src/machines/hp67/keyboard.rs` together with `Hp67ArchitecturalMachine`. It deliberately tests the instruction-boundary architectural bridge rather than the photo UI or final electrical keyboard matrix timing. Later real-firmware display tests build on this checkpoint.

## Responsibilities

Verify that pressing physical digit `1` exposes `0o142` in the ACT key buffer, that executing Woodstock opcode `0o0020` dispatches the architectural PC to `0o142`, and that releasing the key clears the ACT key input. Fail if host-side digit semantics are substituted for the firmware path.

## Implementation

The test creates a default keyboard and architectural HP-67 machine, closes the `Digit1` contact, samples it into ACT and asserts `key_buffer == Some(0o142)`. It then executes ACT special opcode `0o0020`, whose documented firmware role is to replace the low ROM address with the key code, and requires `pc() == 0o142`. Finally it releases the contact, samples again and requires the key buffer to return to `None`.
