# `src/panel.rs`

## Purpose
Renders the embedded HP-67 photograph, maps photographed control coordinates to the current window, handles the main key/switch hit regions and places raw emulated LED emission into the photographed display glass.

## Why it exists
The production UI is image-based rather than a recreated vector chassis. This module centralizes source-photo registration so controls and the dynamic physical display remain aligned at any window size.

## Relationships
Reads mechanical `Hp67State`, receives `HardwareDisplayFrame` from `app.rs`, emits `UiEvent` values, calls `ui::classic_display::paint_segments()` for raw A..G/DP masks, and shares the same 928x1695 coordinate system with `sliders.rs` and `top_keys.rs`.

## Responsibilities
Fit the source image without distortion, paint it, create hit boxes for 35 keys and two switch regions, animate generic key travel, locate the photographed display glass and forward raw hardware segment masks without converting them to characters or numbers.

## Implementation
All geometry is expressed in source-image pixels and transformed uniformly into the fitted photo rectangle. Pressed keys reuse cropped pixels from the source photograph plus small travel/shadow corrections. When the power switch is on, the display layer calls `paint_segments()` with the fifteen-position hardware frame; when power is off, no LED emission is painted. Key contacts are still semantic UI events pending electrical keyboard-matrix work.
