# `src/machines/hp67/data.rs`

## Purpose

Models the source-backed HP-67 DATA stream phase at logical-bit level before electrical polarity and PHI-relative launch/sample edges are known.

## Why it exists

Direct HP-67 captures establish that DATA is a continuous 56-bit LSB-first register stream with serial bit 0 at machine-word b2 and bits 54/55 wrapping into b0/b1 of the following word. RAM/ACT transfers therefore cannot be modeled as a self-contained b0..b55 payload without losing the real cross-word phase. M14B needs a reusable phase engine before physical 1818-* RAM devices are connected to the DATA net.

## Relationships

Uses `ActArchitecturalState`/`ActRegister` only as the current semantic source type and `data_serial_bit_for_word_bit()` from `timing.rs` as the hardware-evidence mapping. It does not read or write `Hp67Net::Data`; electrical DATA polarity, passive bias, ownership handoff and PHI-relative timing remain source-blocked.

## Responsibilities

Decode ACT RAM read/write-class instructions into transfer direction/address while refusing to classify a THEN-GOTO payload as an opcode, expose one register bit in the documented digit-LSB-first order, preserve bits 54/55 across the machine-word boundary, reconstruct back-to-back 56-bit frames, and reject missing/unexpected logical samples or out-of-order word-bit traversal.

## Implementation

`Hp67DataSerialSource` latches the current register for b2..b55 and carries only its bits 54/55 into the next word's b0/b1. `Hp67DataSerialSink` keeps an incomplete frame alive across that same boundary and returns a completed `ActRegister` only after the following b1. `Hp67DataSerialWordPath` composes those two primitives so a caller can visit DATA from an already-existing b0..b55 loop instead of running a second loop. Back-to-back transfers are supported: the previous frame completes at b1 and the next frame begins at b2. `Hp67DataTransferPlan` mirrors the existing ACT architectural address semantics for direct and register-select RAM read/write classes without making any electrical timing claim. It returns `None` while ACT is in `ThenGoto`, because that 10-bit ROM word is branch payload rather than an executable RAM opcode.

## Fidelity boundary

**PROVEN / EXACT:** 56 logical DATA bits, LSB-first within each 4-bit digit, b2 = serial bit 0, b55 = bit 53, next b0/b1 = bits 54/55.

**WORKING ARCHITECTURAL BRIDGE:** instruction classification and RAM address selection reuse the already validated ACT semantic model.

**SOURCE-BLOCKED:** DATA passive level, active drive polarity, exact driver-enable timing, PHI launch/sample edges, propagation delay and physical 1818-* chip/address mapping.


The fused word-path API exposes `frame_in_progress()` so the caller can select the DATA-aware transport only for a transfer word or the one following tail-completion word. This is specifically intended to preserve the no-DATA fast path.
