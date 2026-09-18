# `src/bin/hp67_m11_keyboard_matrix.rs`

## Purpose

Exercise the HP-67 physical keyboard and function-dispatch surface through the real versioned firmware as the M11 validation and coverage harness.

## Why it exists

M10 proves one arithmetic path, but it does not establish that all 35 physical contacts, decimal entry, clearing, sign/exponent entry, the four arithmetic operators, or shifted `f/g/h` paths can traverse the same hardware/firmware route. The M11 harness separates exact regressions from coverage probes so a path is never called semantically correct merely because it executes without an error.

## Relationships

Uses `Hp67Firmware`, `Hp67Keyboard`, `Hp67ArchitecturalMachine`, the structural display/fetch cycle, ROM0 display decoding and the 1820-1749 cathode scan model. The literal 35-key expected-code oracle was independently cross-checked against the complete HP-67 key definitions in the pinned semantic reference recorded in `docs/HARDWARE_SOURCES.md`; that provenance stays in Markdown rather than product source.

## Responsibilities

Boot real firmware to the established no-key idle; verify all 35 `Hp67Key::scan_code()` values against an independent literal oracle before injection; verify the firmware captures each hardware code with `keys -> a` and reaches the expected unshifted dispatch page through `a -> rom address`; verify exact raw display patterns for digits 0 through 9, decimal entry, CLX, CHS, EEX and the four basic arithmetic operations; exact-lock independently specified shifted functions `√x`, `1/x`, `ABS`, `INT` and `FRAC`; exercise every `f/g/h + physical key` pair for architectural/serial execution failures; and describe shifted coverage explicitly without certifying unverified shifted-function results.

## Implementation

Each direct-key case starts from a fresh power-on state. The independently recorded expected code is compared to the production `scan_code()`, then one `Hp67Keyboard` contact is closed, real firmware must execute `keys -> a` and `a -> rom address`, and the contact is opened so firmware can clear S15. Exact sequences use the same physical path and compare raw 15-slot ROM0 segment masks. Display observation is non-invasive: `capture_display(&self)` uses fresh local firmware/transport/backplane/ROM0/cathode objects and therefore cannot advance the harness backplane, fetch endpoint, display phase, pipeline, keyboard or architectural machine. `shifted_function_exact_matrix()` uses only independently specified identities: `√9 = 3`, `1/4 = 0.25`, `|-5| = 5`, `INT(1.2) = 1`, and `FRAC(1.2) = 0.2`; their expected raw segment patterns are fixed before running the emulator. Shifted coverage starts from a fresh boot with X=1, applies a real `f`, `g` or `h` prefix, records the firmware-selected shifted target, releases the second contact, and executes a bounded observation window. These shifted probes establish reachability and absence of modeled execution errors only; semantic results remain open until independently specified regressions are added.
