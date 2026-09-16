# `src/panel.rs`

## Purpose
Renders the embedded HP-67 photograph, maps photographed control coordinates to the current window, handles key/switch hit regions and registers physical LED emission to the photographed calculator.

## Why it exists
The production UI is photo-based. Dynamic LEDs must align to the physical front plane of that photograph at every window size without inheriting distortion from the display-glass crop.

## Relationships
Reads mechanical `Hp67State`, receives `HardwareDisplayFrame` from `app.rs`, emits `UiEvent` values, and calls `ui::classic_display::paint_segments()` with the raw A..G/DP masks plus physical photo registration.

## Responsibilities
Fit the 928x1695 source photograph uniformly, map controls, animate key travel, provide the display clip aperture, derive the LED optical centre and pixels-per-millimetre scale from the projected HP-67 case, and forward raw hardware segment masks unchanged.

## Implementation
The source-photo display registration uses a projected case centre at x=453 px, an LED optical centre at y=156 px and a local case width of 792 px. With the documented 81.0 mm calculator width this yields 9.777778 source pixels/mm. `DISPLAY_GLASS` remains only the clipping aperture. Because the photo itself is uniformly scaled into the window, multiplying this source calibration by the photo scale preserves one isotropic physical transform at every UI size.
