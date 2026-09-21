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


Right-edge regression guard: the current renderer repaints the entire slider clip from a clean recess sample, so the clip boundary is also the visible end of the synthesized black slot. User-visible inspection showed that the widened repaint regions extended a few source pixels too far right on both switches. The power clip is therefore restored to the earlier source x=330 recess boundary (3 px beyond its x=327 actuator edge), while the mode clip now stops exactly at the x=698 actuator/recess edge instead of extending to x=700 toward the RUN legend. `slider_cleanup_stops_at_the_photographed_right_edges` locks both boundaries.
