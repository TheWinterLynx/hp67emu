# `src/app.rs`

## Purpose
Owns the desktop application's top-level egui state, embedded HP-67 photograph texture and live structural HP-67 machine used as the display source.

## Why it exists
The emulator needs a presentation adapter that loads assets and connects headless emulation state to the photographed frontend without putting GUI concerns into the reusable emulation library. The display is no longer allowed to come from a formatted placeholder string.

## Relationships
Uses `hp67::Hp67LiveMachine` to boot the external firmware and obtain `HardwareDisplayFrame`, `hp67::Hp67State` only for remaining mechanical UI controls, `panel::Hp67Panel` for the photographed body/input regions, and `ui::sliders` / `ui::top_keys` for visual corrections.

## Responsibilities
Embed/decode `assets/hp67.png`, configure egui visuals, boot the live machine when the app starts, pass raw display segment masks to the panel, dispatch UI events, and leave the LEDs blank rather than inventing a fallback display when the external corpus is unavailable.

## Implementation
`include_bytes!` compiles only the PNG into the executable; firmware remains external. `Hp67LiveMachine::boot_default()` is attempted once during application construction. A boot failure is reported to stderr and represented by `None`, which maps to `HardwareDisplayFrame::BLANK`. Each frame passes a copy of the current raw segment frame into the panel before drawing key/switch overlays.
