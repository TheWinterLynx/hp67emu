# `src/machines/hp67/act.rs`

## Purpose

Implements the independent instruction-boundary Woodstock ACT core for HP-67 bring-up and now owns the ACT-side word serializer that emits ROM0 display bits directly from A/B nibbles.

## Why it exists

The architectural core is required for complete firmware execution while the final pin/timing-accurate 1820-2530 is still being built. The display path must nevertheless respect the real ACT chip boundary: production transport must not precompose `(B << 4) | A` outside the ACT and hand a finished byte to the bus.

## Relationships

`fetch.rs` uses `ActDisplayWordSerializer` through a single `ActSerialEndpoint` that owns the currently modeled ACT roles on IS. `display.rs` supplies the source-backed fifteen-slot role order. `display_snapshot.rs` retains a whole-byte composition helper only as an instruction-boundary diagnostic/reference bridge. Production UI and power-on smoke no longer use that helper. The semantic reference model remains test-only.

## Responsibilities

Maintain all architecturally visible ACT state needed by HP-67 firmware and implement Woodstock instruction-boundary behavior. For display transport, map the current physical scan slot to its ACT A/B register digit and emit b0..b3 directly from A bit 0..3 and b4..b7 directly from B bit 0..3 using the wired-high/release IS convention. Reject invalid scan slots rather than inventing data.

## Implementation

`ActDisplayWordSerializer::from_state()` snapshots only the selected A and B nibbles at the structural word boundary. `drive_for_bit()` never constructs an eight-bit display code: it selects the corresponding source bit only when b0..b7 is visited. This removes the previous whole-byte production bridge while deliberately preserving a clear remaining boundary: A/B are still instruction-boundary arrays, not the final serial shift-register/ALU state evolving inside the 56-bit word. Exact PHI launch edges and true intra-word ACT mutation remain future work.
