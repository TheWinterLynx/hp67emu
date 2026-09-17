# `src/app.rs`

## Purpose

Run the desktop HP-67 application loop around the photorealistic panel and live structural machine.

## Why it exists

Input sampling, elapsed-time advancement and painting need one deterministic ordering. Physical key contact must reach the machine before the elapsed firmware work for the frame is executed.

## Relationships

Owns `Hp67State`, `Hp67LiveMachine`, the embedded front-panel texture, `Hp67Panel`, and the small visual correction layers for top keys and sliders.

## Responsibilities

Create the versioned-firmware live machine; process power/mode switch events; reset on OFF-to-ON; translate the panel's held `KeyAction` into a physical `Hp67Key`; release the machine keyboard whenever no key is held or power is off; advance firmware from real elapsed time; and repaint continuously while powered so firmware, display scanning and contact/release polling continue after boot idle.

## Implementation

The frame first paints the currently available hardware display and collects panel interaction. Mechanical events are applied, the current key contact is passed to `Hp67LiveMachine::set_key_contact()`, then elapsed time is calculated and the live machine advances. Resetting `last_live_tick` on power transitions preserves the original zero-elapsed first powered frame. No UI key callback performs calculator-level work.
