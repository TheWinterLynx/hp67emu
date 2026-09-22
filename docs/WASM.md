# WebAssembly build

## Scope

The browser build reuses the same `Hp67App`, `Hp67LiveMachine`, real firmware, embedded front-panel artwork and built-in program-card media as the native executable. No calculator semantics are reimplemented in JavaScript.

The first WASM slice targets `wasm32-unknown-unknown` with eframe 0.27.2 and the existing WGPU renderer. eframe 0.27 can use WebGPU on browsers that provide it and fall back through its web rendering support; the project does not introduce a second renderer-specific calculator path.

## Prerequisites

Install the Rust target:

```powershell
rustup target add wasm32-unknown-unknown
```

For a browser-hosted build, install Trunk:

```powershell
cargo install --locked trunk
```

## Compile-only gate

This checks that the Rust application and all embedded assets compile for the browser target without involving a web server:

```powershell
cargo build --locked --release --target wasm32-unknown-unknown --bin hp67emu
```

## Run locally

From the repository root:

```powershell
trunk serve --release --open
```

`index.html` contains the Trunk Rust asset declaration and the full-window canvas expected by the eframe 0.27 `WebRunner`.

## Distribution build

```powershell
trunk build --release
```

The generated site is written under `dist/`, which is intentionally ignored by Git.

## Fidelity boundary

WASM is only a host/presentation target. Deterministic HP-67 word/bit timing, firmware execution, DATA phase handling, magnetic-card semantics and the electrical scheduler remain in Rust and are identical to the native build.

The front end uses `web_time::Instant` at the host pacing boundary so elapsed browser time can advance the same live machine API. This does not change the integer simulation timing inside the emulator.

## First-slice limitations

Browser filesystem import/export is not yet promoted as working functionality. The native code still owns path-based `.hpp`, `.hp67raw` and `.hp67card` import/save. Embedded program-library media and artwork do not need browser filesystem access and are expected to remain available.

Secondary eframe viewports are a desktop feature. On web, egui may embed unsupported child viewports into the root UI. The calculator itself is the acceptance target for the first browser slice; Program Library and Save Card presentation will be adapted after the first real browser run if required.

No GitHub Actions or automatic deployment are introduced by this slice.

## Trunk release / wasm-opt compatibility

Current rustc can emit standard WebAssembly bulk-memory instructions such as `memory.copy`. Trunk's cached Binaryen `wasm-opt` may validate release output with those features disabled unless the Rust asset passes the matching feature flags. `index.html` therefore supplies `--enable-bulk-memory` and `--enable-nontrapping-float-to-int` through Trunk's documented `data-wasm-opt-params` attribute. A flood of validator messages about `memory.copy operations require bulk memory operations` is one root cause repeated across many generated functions, not hundreds of independent emulator failures.
