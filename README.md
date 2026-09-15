# hp67emu

A Rust HP-67 emulator project whose target is **cycle-accurate, pin-level electrical emulation executing the calculator's real microcode**.

The production front panel is photorealistic. `assets/hp67.png` is compiled into the executable with `include_bytes!`, so the built program has no runtime dependency on an external image file.

## Project direction

The project is being split into two deliberate layers:

- **Reusable emulation library** (`src/lib.rs`, `src/emulation/`, `src/machines/`) — deterministic timing, electrical nets, chip models and calculator compositions. It must stay independent of egui and image rendering.
- **HP-67 desktop front end** (`src/main.rs`, `src/app.rs`, `src/panel.rs`, `src/ui/`) — photographed body, input hit regions, key/switch animation and LED optics.

The current `src/hp67.rs` remains a temporary UI smoke-test state. It is explicitly not the future calculator core and will be removed once the electrical machine can boot real microcode and drive the panel.

## Architecture and plan

- [Architecture](docs/ARCHITECTURE.md)
- [Roadmap](docs/ROADMAP.md)
- [Current TODO](TODO.md)
- [Hardware evidence and source policy](docs/HARDWARE_SOURCES.md)
- [Documentation contract](docs/DOCUMENTATION_POLICY.md)
- [Embedded assets](docs/ASSETS.md)
- [Per-file documentation index](docs/FILES.md)

Every Rust source file must have a companion Markdown document under `docs/files/`. `cargo test` contains a regression that fails when a source file is added without its documentation or when required documentation sections are missing.

## Build and regression suite

Formatting is a hard regression gate. Do not continue to tests or release builds while `cargo fmt --all -- --check` reports any difference.

```powershell
cargo fmt --all -- --check; if ($LASTEXITCODE -eq 0) { $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets }; if ($LASTEXITCODE -eq 0) { cargo build --locked --release --bins }
```

The regression suite currently checks, among other things:

- repository-wide `rustfmt` cleanliness before any later gate;
- compiler warnings denied across all test targets;
- the non-overlapping two-phase timing scaffold;
- electrical net floating/pull/drive/contention resolution;
- the HP-67 hardware inventory and core named nets;
- separation between the reusable emulation core and GUI/image dependencies;
- the per-file Markdown documentation contract.

## Branch discipline

`main` is the stable integration branch. Emulator work is developed in focused branches created from the current `main`; after a milestone is validated and merged, the next branch starts from the new `main`. This keeps timing, ACT, ROM/RAM, display, keyboard and card-reader work reviewable in isolation.

## Current front panel

- `assets/hp67.png` is embedded into the EXE.
- Keys use photographed keycaps with animated travel.
- OFF/ON and W/PRGM/RUN use animated photographed sliders.
- The LED renderer uses measured HP Classic-style geometry over the photographed glass.
- The visible `0.00` at startup is still a temporary UI placeholder. In the final emulator the display will be an electrical consequence of ACT/ROM/display-driver activity, not formatted UI text.
