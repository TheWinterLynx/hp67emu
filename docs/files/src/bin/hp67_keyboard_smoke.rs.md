# `src/bin/hp67_keyboard_smoke.rs`

## Purpose

Proves M8 at the real-firmware boundary: a physical HP-67 Digit1 contact reaches ACT keyboard inputs, is detected through S15, is consumed by the preserved firmware with `keys -> a`, is remapped by that firmware, and dispatches with `a -> rom address` to the source-backed unshifted key-72 entry.

## Why it exists

A direct unit test of ACT keyboard opcodes can pass while bypassing the firmware's actual key-detection loop. HP-67 firmware first polls S15, then executes `keys -> a`, transforms the hardware code, and only then dispatches through a key table. This smoke prevents a keyboard implementation from claiming success merely because a key code can be injected into one isolated ACT instruction.

## Relationships

Starts from the same external 5120-word ROM corpus and structural ACT/ROM/display fetch components used by `hp67_poweron_smoke.rs`. It uses `Hp67Keyboard` only as a physical input source; calculator semantics remain entirely in the real HP-67 firmware. The source-backed Digit1 hardware code is octal `0142`, and the preserved unshifted key table identifies octal `1440` as `unshifted key 72: 1`.

## Responsibilities

Reach the established no-key boot idle checkpoint, revalidate the source-backed `0.00` display through the shared structural word path, close the Digit1 contact, present keyboard state before each ACT instruction boundary, prove the firmware executes `keys -> a` with A[2:1] equal to hexadecimal `62`, and prove the subsequent `a -> rom address` lands at octal `1440`. It must fail rather than substituting calculator-level digit entry if the firmware does not traverse that path.

## Implementation

The smoke keeps the normal one-word fetch pipeline and b0..b55 serial execution checks intact. After idle has completed its current structural word, it closes `Hp67Key::Digit1`. On every following instruction boundary `Hp67Keyboard::sample_into_act()` reasserts S15 while the contact remains held and presents the retained hardware code. The smoke recognizes `keys -> a` only when opcode `0120` executes in normal ACT instruction state, checks A[2:1], then waits for normal opcode `0220` and requires the resulting PC to be octal `1440`. M8 stops there deliberately; proving that the dispatched digit-entry routine eventually changes the physical display is the next milestone, not part of this smoke.
