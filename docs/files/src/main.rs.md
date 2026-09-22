# `src/main.rs`

## Purpose
Defines the platform entry point for both native desktop and WebAssembly builds.

## Why it exists
The reusable emulator core is a library, but the project needs platform launchers that create the same egui application on Windows/Linux/macOS and in a browser without duplicating calculator semantics.

## Relationships
Owns module declarations for the current front end (`app`, `hp67`, `panel`, `ui`) and launches `app::Hp67App`. It intentionally does not import `src/emulation` or machine internals directly. Native uses `eframe::run_native`; wasm32 uses the version-matched `eframe::WebRunner` API.

## Responsibilities
Create `eframe::NativeOptions`, configure resize/MSAA/WGPU behavior, start the native app, start the browser app on the `the_canvas_id` canvas for wasm32, and regression-test the native window policy.

## Implementation
Native builds construct `eframe::NativeOptions` with a resizable viewport and 4× MSAA, then call `eframe::run_native`. wasm32 builds spawn the async eframe 0.27 `WebRunner::start("the_canvas_id", ...)` future; `index.html` owns the full-browser canvas. Release desktop builds use the Windows GUI subsystem, but `cfg(test)` deliberately keeps the test executable attached to the console even in `--release`; otherwise Cargo can execute long ignored diagnostic tests while their `println!` OK/KO report is invisible on Windows. The native-only test checks that no maximum window size is imposed.
