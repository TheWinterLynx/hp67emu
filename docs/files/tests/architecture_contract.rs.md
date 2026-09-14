# `tests/architecture_contract.rs`

## Purpose
Protects the reusable emulator and behavioural reference layers from accidental GUI and image-rendering dependencies.

## Why it exists
The project must support headless testing and future calculators. If `src/emulation`, `src/machines` or `src/reference` starts importing egui/eframe/current front-panel code, the modular boundary is already broken.

## Relationships
Checks the reusable source trees exported by `src/lib.rs` and complements the documentation contract. The `reference` subtree is intentionally covered because the semantic oracle must remain usable by headless differential tests.

## Responsibilities
Fail local regressions when reusable source contains forbidden GUI/image/current-binary dependency tokens.

## Implementation
Recursively reads `.rs` files in `src/emulation`, `src/machines` and `src/reference` and asserts that banned dependency-shaped strings are absent. The test is intentionally simple and conservative; if the architecture evolves, update the policy and regression together.
