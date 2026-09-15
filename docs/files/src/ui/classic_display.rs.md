# `src/ui/classic_display.rs`

## Purpose
Draws photorealistic HP Classic-series LED emission over the photographed display glass directly from physical A..G/DP segment masks.

## Why it exists
The source photograph supplies the bezel/filter/reflections but cannot show dynamic LEDs. The electrical/structural display path can now provide raw segment state, so the production renderer must draw those masks without first formatting characters or a calculator value.

## Relationships
Called by `panel.rs`. Its production entry point `paint_segments()` consumes the fifteen-position `HardwareDisplayFrame` generated from `machines::hp67` ROM0/cathode state. The former text-to-cell renderer has been removed from this module, leaving raw segment masks as the only display input path.

## Responsibilities
Map the 15 physical positions, calibrated LED geometry, A..G masks, decimal-point mask, optical color and glow into egui drawing primitives while preserving the photographed filter and glass underneath.

## Implementation
`paint_segments()` positions all fifteen physical character cells and passes each nonzero byte directly to `draw_segment_mask()`. Bits 0..6 drive the seven segment emitters and bit 7 drives the centered decimal emitter. No numeric parsing, text-to-cell conversion or inferred calculator value occurs in this path. Existing physical dimensions and three-bar segment artwork are retained.
