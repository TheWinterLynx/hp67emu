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


## M14X bounded live IS/PHI capture

The native diagnostic CLI `hp67emu --trace-is <1..4096 words> <output.tsv>` exports actual live power-on firmware words with frozen no-key inputs. Reset-hold time has no modeled bus transport and is explicitly excluded. Ordinal ticks remain uncalibrated.

The live cycle routes an optional recorder only at the word boundary; ordinary calls pass None. Traced and ordinary words share the existing restore/oracle/authority-commit bridge. A dedicated traced DATA twin advances the same logical DATA visitor, preserving next-word b0/b1 tail authority without recording electrical DATA. Original production transport loops remain unchanged. Capture storage is local, bounded and discarded after export; no UI state or persistent trace session is introduced. Invalid counts or capture errors fail before file writing.

Focused tests compare real firmware boot with and without recording, exercise alternating synthetic RAM ports through the same live bridge and validate export bounds/row count/workload labels. See research/M14X_LIVE_BUS_CAPTURE_2026-10-11.md.


## M14Y offline comparison

Adds GUI-free bus_trace module and native `--compare-is expected.tsv actual.tsv report.txt [expected_start actual_start]` adapter. Strict import preserves provenance/unknown levels, bounded inputs and optional driver coverage. Comparison uses explicit row indices and relative ordinal ticks, counts mismatches and unequal lengths, and writes a report. Native exit codes: 0 match, 2 mismatch, 1 input/output error. Input paths cannot be overwritten by report output. Windows GUI-subsystem processes must be invoked with Start-Process -Wait -PassThru in PowerShell gates. No transport loop or firmware execution changes. See research/M14Y_TRACE_COMPARISON_2026-10-12.md.


## M14Z structural word decoding

Native `--decode-is input.tsv report.txt [start_row words]` decodes an explicit range of complete 224-observation words. Default requires the entire trace to contain whole words. Every bit must carry four consecutive ordinal ticks, the native PHI scaffold shape, matching b0..b55 coordinate and stable binary IS. Malformed/partial/unresolved/shifted observations fail rather than fabricate fields. The decoder uses existing source-locked timing windows to recover display byte, 12-bit ROM address and 10-bit fetched instruction LSB-first. Reports retain provenance, tick/row origins and explicitly excluded samples; ROM bank and concurrently executing instruction are not inferred. No production execution changes. See research/M14Z_IS_WORD_DECODE_2026-10-12.md.
