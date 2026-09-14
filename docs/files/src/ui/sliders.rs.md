# `src/ui/sliders.rs`

## Purpose
Animates the photographed OFF/ON and W/PRGM/RUN slider actuators.

## Why it exists
`hp67.png` contains the switches in their photographed right-hand endpoints. Moving only the ribbed actuator while reconstructing the empty track preserves the real materials without a vector replacement.

## Relationships
Called by `app.rs` after the base photograph is drawn. Reads the temporary `Hp67State` power/mode values and uses the same source-photo coordinate system as `panel.rs`.

## Responsibilities
Interpolate visual actuator position, restore the exposed track cleanly, avoid covering printed legends, and keep animation purely presentational.

## Implementation
Defines source-photo crop rectangles for each actuator/track, uses egui's timed boolean animation, repaints the slot from a clean track sample, then draws only the ribbed actuator at the translated position. Future electrical switch state will replace the temporary semantic state input.
