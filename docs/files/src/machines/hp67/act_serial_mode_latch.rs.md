# `src/machines/hp67/act_serial_mode_latch.rs`

## Purpose

Builds the completed-word structural result image for the focused ACT mode/latch family introduced in M14H.

## Why it exists

After M14E, M14F and M14G, arithmetic, P/status condition state and selected non-arithmetic register transfers no longer take their final causal value directly from the instruction-boundary executor. Decimal/hex mode, display control, the 14-digit display latch and the ACT RAM-address latch still did. M14H moves those selected final latch values behind the same structural `b0..b55` execution lifetime without inventing unsupported internal timing.

## Relationships

Uses `ActSerialExecution` only to identify the active special/peripheral word and preserve its structural lifetime. It reads decimal/display/RAM-address state and the pre-instruction C digits from `ActSerialStateSnapshot`. `ActSerialEndpoint` owns one optional `ActSerialModeLatchResultImage` and marks it complete only after the bound execution reaches b55. The live bridge compares its result against the architectural oracle before committing it.

## Responsibilities

Decode display toggle/off, decimal mode, hexadecimal mode, 14-digit display enable and RAM-address selection from C; preserve their pre-instruction values; derive the RAM address from captured C[1:0]; expose completed final latch values; and reject words outside the M14H family.

## Implementation

The image starts from the immutable pre-instruction snapshot. `complete_word()` applies one decoded mode/latch transition after the structural execution reaches b55. `SelectRamAddressFromC` computes `(C[1] << 4) | C[0]` from the captured pre-instruction C image, so an architectural oracle mutation cannot leak into the structural result.

The live machine restores decimal/display/RAM-address fields immediately after the oracle runs, keeps the same-word display source bound to the pre-instruction snapshot, and then commits the M14H image only after exact oracle agreement. Display-side host bookkeeping may consult the oracle-expected post-word display-enable value while the live ACT latch remains structurally deferred.

The b55 handoff is a **WORKING APPROXIMATION** for final-state authority. M14H does not claim the real 1820-2530 changes any of these latches at b55 or on a particular PHI edge. Bank selection, delayed-ROM state and PC/return-stack control remain outside this slice because they participate directly in concurrent fetch/control-flow behavior.
