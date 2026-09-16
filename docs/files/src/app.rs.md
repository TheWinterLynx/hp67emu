# `src/app.rs`

## Purpose
Owns the desktop egui state, embedded HP-67 photograph and wall-clock scheduling adapter for the live structural HP-67 display machine.

## Why it exists
The emulator must expose startup over time rather than constructing the machine at its final idle state. GUI frame timing belongs here, outside the reusable electrical and architectural modules, and must not fabricate display contents.

## Relationships
Uses `Hp67LiveMachine::power_on_default()` and `advance()` for the external firmware, `Hp67State` for mechanical controls, `Hp67Panel` for input/body rendering and the photo overlay modules for visual corrections. `HardwareDisplayFrame` remains the only display payload passed to the panel.

## Responsibilities
Decode the embedded photograph, load the external firmware source, measure elapsed host time between egui updates, advance HP-67 startup only while power is on, replay machine reset when the switch goes OFF->ON, and request repaints while boot is progressing. If the external corpus or runtime fails, leave LED emission blank rather than synthesize a calculator value.

## Implementation
The first UI frame advances by zero elapsed time and is blank because the pre-SYNC ROM0 sampling state is not yet established by reviewed evidence. Subsequent frames pass monotonic `Instant` deltas into `Hp67LiveMachine`. Once firmware transport starts, visible slots are filled by the structural machine itself. While `is_booting()` is true, egui requests another repaint after approximately one measured HP-67 display refresh (4.8 ms). Power-off stops advancement and suppresses LED painting in `panel.rs`; power-on resets the live machine and its UI clock so the natural transient repeats.
