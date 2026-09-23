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

Secondary eframe viewports are a desktop feature. Browser builds therefore use in-canvas Program Library UI. The presentation is responsive: wide browser viewports retain a movable two-column window, while compact viewports (less than 700 points wide or 560 points high) switch to a full-screen single-column selection flow. The compact flow uses 44-point minimum interaction height, explicit Close and Back controls, a scrollable program/detail body, and a full-width persistent Load card action. This avoids squeezing a desktop two-pane dialog into iPhone portrait/landscape sizes. Loading a card closes the library automatically. Save Card also uses a closable embedded window on web and currently reports that browser export is not implemented. Native child viewports remain unchanged.

GitHub Pages deployment is handled by the checked-in Pages workflow on `main`; feature branches do not deploy unless the workflow trigger is explicitly changed.

## Trunk release / wasm-opt compatibility

Current rustc can emit standard WebAssembly bulk-memory instructions such as `memory.copy`. Trunk's cached Binaryen `wasm-opt` may validate release output with those features disabled unless the Rust asset passes the matching feature flags. `index.html` therefore supplies `--enable-bulk-memory` and `--enable-nontrapping-float-to-int` through Trunk's documented `data-wasm-opt-params` attribute. A flood of validator messages about `memory.copy operations require bulk memory operations` is one root cause repeated across many generated functions, not hundreds of independent emulator failures.

The wasm target is built with `-Dwarnings` during validation. Native-only save helpers are compiled out on wasm rather than suppressed with dead-code allowances, and the web Program Library consumes the same source-PDF metadata as the native library.

The browser Program Library keeps `Load card` visible regardless of listing scroll position: in wide mode it is in the top toolbar, while compact mode pins a full-width action below the detail scroller. This is a wasm-only presentation adaptation; the native library layout is unchanged.


## Compact mobile UX rationale

The compact Program Library follows platform-neutral responsive principles and Apple iPhone guidance rather than imitating a desktop window at a smaller size. Apple recommends using full-screen modal presentation for focused or multistep tasks when appropriate, keeping dismissal obvious, and targeting roughly 44 by 44 points for touch controls on iPhone/iPad. The library is a browse-select-review-load task with dozens of entries, so it uses a full-screen drill-in flow on constrained viewports instead of an action sheet or a permanently expanded menu. The breakpoint is a presentation heuristic based only on available egui points; it does not alter calculator semantics or infer device identity.

## Mobile shell and orientation

The web shell now consumes CSS safe-area insets because `viewport-fit=cover` is enabled. The canvas and loading surface are inset by `safe-area-inset-top/right/bottom/left`, and dynamic viewport height (`100dvh`, with `100vh` fallback) follows mobile browser chrome without placing calculator controls under a notch or home indicator.

Compact browser mode also applies a 44-point minimum interaction height to the Cards bar/menu and a 44-point minimum invisible hit target to calculator keys, slide switches and magnetic-card interactions. Visual geometry remains the original photographed geometry; only hit regions grow.

Portrait continues to contain the complete HP-67 in the available safe canvas. Compact landscape uses a different presentation strategy because fitting the entire tall calculator into a short landscape viewport would make the physical controls too small: the calculator is rendered at up to 528 points wide, its source aspect ratio is preserved, and the central area scrolls vertically. No keys are rearranged and no calculator semantics change when orientation changes.

