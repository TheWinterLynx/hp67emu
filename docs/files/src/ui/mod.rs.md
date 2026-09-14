# `src/ui/mod.rs`

## Purpose
Declares the small set of production UI helper modules used by the photographed front panel.

## Why it exists
Keeping UI helpers in one module boundary prevents presentation code from spreading into the reusable emulation library.

## Relationships
Exports `classic_display`, `sliders` and `top_keys` to `app.rs`/`panel.rs`.

## Responsibilities
Be the namespace boundary for front-panel rendering helpers and contain no machine-emulation state.

## Implementation
Contains only Rust module declarations. Removed vector-era modules are intentionally not reintroduced here.
