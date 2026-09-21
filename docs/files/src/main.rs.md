# `src/main.rs`

## Purpose
Defines the desktop executable entry point and native window configuration.

## Why it exists
The reusable emulator core is a library, but the project still needs a concrete Windows/Linux/macOS process that creates the egui application and chooses renderer/window defaults.

## Relationships
Owns module declarations for the current desktop UI (`app`, `hp67`, `panel`, `ui`) and launches `app::Hp67App`. It intentionally does not import `src/emulation` or machine internals directly.

## Responsibilities
Create `eframe::NativeOptions`, configure resize/MSAA/WGPU behavior, start the native app, and regression-test the window policy.

## Implementation
Builds an `eframe::NativeOptions` with a resizable viewport and 4× MSAA, then calls `eframe::run_native`. Release desktop builds use the Windows GUI subsystem, but `cfg(test)` deliberately keeps the test executable attached to the console even in `--release`; otherwise Cargo can execute long ignored diagnostic tests while their `println!` OK/KO report is invisible on Windows. The test checks that no maximum window size is imposed.
