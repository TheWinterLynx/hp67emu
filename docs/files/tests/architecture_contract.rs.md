# `tests/architecture_contract.rs`

## Purpose
Protects the reusable emulator core from accidental GUI and image-rendering dependencies.

## Why it exists
The project must support headless testing and future calculators. If `src/emulation` or `src/machines` starts importing egui/eframe/current front-panel code, the modular boundary is already broken.

## Relationships
Checks the source trees exported by `src/lib.rs` and complements the documentation contract.

## Responsibilities
Fail CI/tests when core source contains forbidden GUI/image/current-binary dependency tokens.

## Implementation
Recursively reads `.rs` files in `src/emulation` and `src/machines` and asserts that banned dependency strings are absent. The test is intentionally simple and conservative; if the architecture evolves, update the policy and regression together.
