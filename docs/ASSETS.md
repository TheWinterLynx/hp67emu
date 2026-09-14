# Assets

## `assets/hp67.png`

This is the production photographed HP-67 body/front-panel asset.

It exists so the application does not reconstruct the calculator chassis, legends, glass or key materials procedurally. The UI overlays only state that must move or emit light: pressed keycaps, slide-switch actuators and LED emission.

`src/app.rs` embeds the PNG at compile time with:

```rust
include_bytes!("../assets/hp67.png")
```

Consequences:

- the final executable does not require an external PNG beside it;
- changing the image requires recompiling the program;
- source-photo pixel coordinates used by `panel.rs`, `sliders.rs` and `top_keys.rs` must be recalibrated if the asset dimensions/content change;
- the image is presentation only and must never become a source of calculator electrical state.

The current coordinate registration in the UI assumes a 928 × 1695 source image.
