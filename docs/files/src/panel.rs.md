# `src/panel.rs`

## Purpose
Renders the embedded HP-67 photograph, maps photographed control coordinates to the current window and handles the main key/switch hit regions.

## Why it exists
The production UI is image-based rather than a recreated vector chassis. This module centralizes the source-photo registration so controls remain aligned at any window size.

## Relationships
Reads `Hp67State` from the temporary UI model, emits `UiEvent` values, calls `ui::classic_display` for current LED artwork, and shares the same 928×1695 coordinate system with `sliders.rs` and `top_keys.rs`.

## Responsibilities
Fit the source image without distortion, paint it, create hit boxes for 35 keys and two switch regions, animate generic key travel, and locate the photographed display glass.

## Implementation
All geometry is expressed in source-image pixels and transformed uniformly into the fitted photo rectangle. Pressed keys reuse cropped pixels from the source photograph plus small travel/shadow corrections. This is presentation logic only; the future cycle-accurate path will replace semantic key events with electrical contact closures.
