# `src/app.rs`

## Purpose
Owns the desktop application's top-level egui state and embedded HP-67 photograph texture.

## Why it exists
The emulator needs a presentation adapter that loads assets, creates the current temporary UI state, delegates panel drawing, and schedules repaints without putting GUI concerns into the reusable emulation library.

## Relationships
Uses `hp67::Hp67State` as a temporary smoke-test model, `panel::Hp67Panel` for the photographed body/input regions, and `ui::sliders` / `ui::top_keys` for visual corrections. It will later adapt `hp67emu::machines::hp67` electrical state to the renderer.

## Responsibilities
Embed/decode `assets/hp67.png`, configure egui visuals, own the texture handle and current prototype state, dispatch UI events, and request responsive repaint while input is held.

## Implementation
`include_bytes!` compiles the PNG into the executable. The bytes are decoded once into an egui texture. Each frame calls the panel renderer, applies temporary UI events to the prototype state, then draws key/switch overlays.
