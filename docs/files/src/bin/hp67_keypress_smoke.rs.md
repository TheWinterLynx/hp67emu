# `src/bin/hp67_keypress_smoke.rs`

## Purpose

Provides a manual end-to-end smoke test for the first real HP-67 keyboard path after firmware has reached its normal no-key idle loop.

## Why it exists

The power-on smoke proves that real firmware, structural serial fetch, ACT/CRC instruction ownership and the display path reach the authentic `0.00` idle state. The next fidelity checkpoint is not a calculator-level `enter_digit` helper; it is a physical keyboard contact observed by ACT and interpreted by the original firmware. This binary isolates that path without depending on mouse timing in the graphical UI.

## Relationships

Uses `Hp67LiveMachine`, the same live machine driven by `app.rs`, plus the source-backed `Hp67Keyboard` mapping already connected through `KeyAction::Digit(1)`. Firmware remains external through the normal `.research/teenix-2026-hp67.tsv` corpus. The test does not edit ACT arithmetic registers, force a program counter or fabricate display segments.

## Responsibilities

Boot the live machine through the observed power-on delay, wait for its firmware-idle checkpoint, capture the hardware display frame, close the physical digit-1 contact, advance firmware word by word until the display changes, release the contact, allow firmware to settle, and fail explicitly if no persistent display response occurs.

## Implementation

The binary advances the machine with `HP67_OBSERVED_WORD_TIME_US`, using `Hp67LiveMachine::is_booting()` only as a boot-completion marker. Once idle, `set_key_contact(Some(KeyAction::Digit(1)))` presents the physical key through the keyboard model; the model supplies scan code octal `0142` to ACT at instruction boundaries. The binary observes only `HardwareDisplayFrame` output. A successful run therefore establishes a complete contact-to-firmware-to-display path without introducing calculator semantics in the host.
