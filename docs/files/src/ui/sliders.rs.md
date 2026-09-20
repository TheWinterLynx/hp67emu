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


RUN legend regression guard: the current renderer repaints the entire mode-slider clip, so its effective right paint boundary must stop at source x=700. The older `f195891d` implementation used an outer clip ending at x=702 but only erased through x=700; carrying 702 into the later full-slot renderer reintroduced the cut `R`. The actuator itself ends at x=698, leaving a 2-source-pixel cleanup margin while keeping the anti-aliased `RUN` legend untouched. `mode_slider_cleanup_never_reaches_run_legend` locks this effective boundary.
