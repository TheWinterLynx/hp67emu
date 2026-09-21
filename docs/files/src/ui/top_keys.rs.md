# `src/ui/top_keys.rs`

## Purpose
Applies a photo-specific correction to the exposed strip above the A–E key row during press animation.

## Why it exists
The upper key wells in the source photograph differ visually from the lower rows. Generic cropped-key travel exposed an olive strip that looked artificial, so this narrow correction restores real black well pixels without changing the otherwise good generic animation.

## Relationships
Called by `app.rs` after `panel.rs`. Uses the same source-photo dimensions and persistent animation timing as the panel key renderer, and receives the already-captured `panel.key_contact` so the A–E correction cannot visually release before the machine contact does.

## Responsibilities
Match the authoritative held A–E key identity supplied by `panel.rs`, match generic key travel timing, and repaint only the newly exposed strip from real photographed recess pixels.

## Implementation
Maintains five source hit/cap rectangles, derives press progress from the captured `KeyAction` contact supplied by the panel, computes the exposed strip and samples a nearby dark recess from `hp67.png`. Pointer hit testing and mouse-up release remain owned by `panel.rs`; this layer cannot independently "unpress" A–E. It does not own key semantics or machine timing.
