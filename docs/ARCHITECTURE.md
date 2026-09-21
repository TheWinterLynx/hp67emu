# Architecture

## Goal

The end state is not an instruction-level calculator reimplementation. The target is a **cycle-accurate, pin-level digital-electrical simulation** of the HP-67 that executes the original machine microcode and makes the front panel a consequence of the emulated electronics.

“Electrical” here means that behaviorally relevant IC pins and shared wires are explicit nets with drivers, high-impedance states, passive bias and detectable contention. It does **not** initially mean transistor-level SPICE simulation of every analog component. Analog details are added only where they affect externally observable digital timing, card sensing, display brightness or power/reset behavior.

## Layering

```text
┌──────────────────────────────────────────────┐
│ Desktop UI / photographed HP-67             │
│ src/main.rs, app.rs, panel.rs, ui/*          │
└──────────────────────┬───────────────────────┘
                       │ physical controls / emitted light
┌──────────────────────▼───────────────────────┐
│ Machine adapter                              │
│ maps key contacts, switches and LED energy   │
└──────────────────────┬───────────────────────┘
                       │ pins / nets
┌──────────────────────▼───────────────────────┐
│ HP-67 machine composition                    │
│ src/machines/hp67/*                          │
│ ACT + ROM/RAM + display + CRC + card reader  │
└──────────────────────┬───────────────────────┘
                       │ generic electrical primitives
┌──────────────────────▼───────────────────────┐
│ Reusable emulation kernel                    │
│ src/emulation/*                              │
│ time, nets, scheduling, traces, device API   │
└──────────────────────────────────────────────┘
```

The reusable library is intentionally GUI-free. `tests/architecture_contract.rs` fails if the core starts depending on `eframe`, `egui`, image decoding or current front-panel modules.

## Timing model

The HP calculator family uses serial data paths and explicit clocking. The generic kernel therefore treats time as a monotonic integer `Tick`, not wall-clock time. The current `TwoPhaseClock` is a four-slot scaffold:

1. PHI1 asserted;
2. non-overlap dead time;
3. PHI2 asserted;
4. non-overlap dead time.

This scaffold establishes deterministic ordering but deliberately does **not** claim final HP-67 pulse widths. Measured/documented timing will calibrate the mapping from scheduler ticks to real bit times before ACT behavior relies on it.

The HP-67-related documentation shows 56-bit word timing, two-phase clocks, serial ISA/IS and DATA activity, SYNC and display-control signals. The final scheduler must be able to represent every behaviorally relevant transition inside those word times rather than executing one whole microinstruction atomically.

## Electrical net model

`src/emulation/net.rs` models a shared wire with:

- `Drive::Low`, `Drive::High`, `Drive::HighZ`;
- passive `Floating`, `PullUp` and `PullDown` bias;
- resolved `Low`, `High`, `Floating` or `Contention` state;
- named drivers so bus ownership can be audited.

This matters because ISA and DATA are shared serial buses. A functional emulator can silently overwrite a value; an electrical emulator must expose illegal simultaneous drive as a regression failure or trace anomaly.

The current resolver is a generic digital primitive. Exact HP bus polarity, passive bias and “only actively drive one state” behavior must be established from source evidence and scope traces before being encoded as HP-specific rules.

## Deterministic device scheduling

Devices obey one scheduling contract:

1. **resolve** all nets from already-committed drives;
2. **snapshot/sample** the same resolved inputs for every device;
3. **evaluate** each device for the current tick;
4. **collect/stage** proposed output drives;
5. **commit** all drives together;
6. advance time.

No chip may observe another chip merely because it happened to be earlier in an implementation container. This rule is central to cycle accuracy and repeatable traces.

The generic `ElectricalScheduler` remains the model-independent reference implementation of that contract. It intentionally uses ordered maps and boxed devices for clarity and arbitrary topologies. The HP-67 production machine does **not** use that representation in its edge hot path: `src/machines/hp67/electrical.rs` implements the same contract with fixed `Hp67Net` and `Hp67Driver` indices, fixed pending-drive buffers, bitmask dirty tracking, cached resolved levels and direct driver counters. Normal HP-67 ticks therefore require no map construction, no driver-name lookup, no whole-net-set re-resolution and no heap allocation. This specialization is an implementation detail only; it must never alter source-backed electrical ordering, passive bias, contention visibility or trace semantics.

## HP-67 machine composition

`src/machines/hp67/wiring.rs` contains only hardware identities and signal names for which we have source evidence. The current confirmed inventory includes:

- 1820-2530 ACT;
- 1818-0231 and 1818-0232 ROM/RAM;
- 1818-0268 ROM/display anode driver;
- 1818-0550 and 1818-0551 ROM/RAM;
- 1820-1749 display cathode driver;
- 1820-1751 card reader controller (CRC);
- 1826-0322 card-reader sense amplifier;
- 1858-0050 / CA3082 transistor array.

The first named nets include PHI1, PHI2, ISA, DATA, SYNC, RCD, STR, F1, F2 and keyboard-column lines KC1–KC5. More pins are added only when wiring and semantics are verified.

## What “cycle accurate” means for this project

The target requires all of the following:

- the original ROM words are fetched through emulated bus timing;
- ACT registers and ALU operations occur at the correct serial bit/word phases;
- branches, return stack, status and pointer state follow the hardware microarchitecture;
- RAM/ROM chips respond on the same shared buses and at the correct phases;
- display strobe/cathode/anode activity is generated by the machine, not by formatting a string;
- key presses close electrical contacts and are discovered by the emulated scan path;
- RUN/W/PRGM and power state enter through hardware-facing signals, not direct semantic state changes;
- CRC/card-reader behavior is driven by its actual interface and timing;
- logic-analyzer traces can be compared against documented or captured real-machine traces.

Instruction-level shortcuts may exist only as optional debug accelerators and must never be the source of truth for the fidelity mode.

## Display integration target

Today `classic_display.rs` converts temporary text into LED artwork. That is transitional.

The final path is:

```text
ACT/ROM0 -> ISA/STR/RCD/anode/cathode electrical activity
        -> per-segment on-time integration
        -> optical persistence / brightness
        -> photographed display renderer
```

The renderer then knows only which physical LEDs emitted how much energy during the last visual integration window. It does not know the number `0.00` as text.

## Keyboard integration target

Today the UI emits semantic `KeyAction` values. That is also transitional.

The final input path is:

```text
mouse down on photographed key
    -> physical key contact closure
    -> keyboard/cathode scan electrical state
    -> ACT/PIK/CRC-visible key information
    -> microcode decides what the key means
```

This keeps prefix handling, key timing and unusual hardware behavior inside the emulated calculator instead of the GUI.

## Multi-calculator modularity

The generic emulation layer must not know any HP-67 part number or key. Calculator-specific composition lives under `src/machines/<model>/`.

Shared chips can later move to families such as `src/chips/woodstock/` once at least two machines prove the abstraction. We intentionally avoid premature “universal calculator” interfaces before the HP-67 behavior is understood well enough to know what is truly shared.

## Current architectural debt

`src/hp67.rs` still contains the temporary live-machine bridge that runs instruction-boundary architectural execution alongside the structural serial path. That bridge remains a correctness oracle until timed ACT/RAM device state becomes authoritative; it must not survive as the source of truth in the final fidelity mode. The measured architectural fallback cost is small compared with the electrical path, so removal is driven by correctness/ownership rather than micro-optimization.

The generic map-backed `ElectricalScheduler` is retained as reusable reference infrastructure, not as the HP-67 production scheduler. The HP-67-specific dense fabric now owns the fixed topology needed for M14B and later device scheduling.
