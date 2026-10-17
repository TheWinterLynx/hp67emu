# `src/machines/hp67/data.rs`

## Purpose

Models the source-backed HP-67 DATA stream phase at logical-bit level before electrical polarity and PHI-relative launch/sample edges are known.

## Why it exists

Direct HP-67 captures establish that DATA is a continuous 56-bit LSB-first register stream with serial bit 0 at machine-word b2 and bits 54/55 wrapping into b0/b1 of the following word. RAM/ACT transfers therefore cannot be modeled as a self-contained b0..b55 payload without losing the real cross-word phase. M14B needs a reusable phase engine before physical 1818-* RAM devices are connected to the DATA net.

## Relationships

Uses `ActArchitecturalState`/`ActRegister` only as the current semantic source type and `data_serial_bit_for_word_bit()` from `timing.rs` as the hardware-evidence mapping. Default reference fixtures use logical samples. The live logical-level bridge
publishes and samples resolved `Hp67Net::Data` with explicit driver roles;
physical DATA polarity, passive bias, chip mapping and exact PHI timing remain
source-blocked.

## Responsibilities

Decode ACT RAM read/write-class instructions into transfer direction/address while refusing to classify a THEN-GOTO payload as an opcode, expose one register bit in the documented digit-LSB-first order, preserve bits 54/55 across the machine-word boundary, reconstruct back-to-back 56-bit frames, and reject missing/unexpected logical samples or out-of-order word-bit traversal.

## Implementation

`Hp67DataSerialSource` latches the current register for b2..b55 and carries only its bits 54/55 into the next word's b0/b1. `Hp67DataSerialSink` keeps an incomplete frame alive across that same boundary and returns a completed `ActRegister` only after the following b1. `Hp67DataSerialWordPath` composes those two primitives so a caller can visit DATA from an already-existing b0..b55 loop instead of running a second loop. Back-to-back transfers are supported: the previous frame completes at b1 and the next frame begins at b2. `Hp67DataTransferPlan` mirrors the existing ACT architectural address semantics for direct and register-select RAM read/write classes without making any electrical timing claim. It returns `None` while ACT is in `ThenGoto`, because that 10-bit ROM word is branch payload rather than an executable RAM opcode.

## Fidelity boundary

**PROVEN / EXACT:** 56 logical DATA bits, LSB-first within each 4-bit digit, b2 = serial bit 0, b55 = bit 53, next b0/b1 = bits 54/55.

**WORKING ARCHITECTURAL BRIDGE:** instruction classification and RAM address selection reuse the already validated ACT semantic model.

**SOURCE-BLOCKED:** DATA passive level, active drive polarity, exact driver-enable timing, PHI launch/sample edges, propagation delay and physical 1818-* chip/address mapping.


The fused word-path API exposes `frame_in_progress()` so the caller can select the DATA-aware transport only for a transfer word or the one following tail-completion word. This is specifically intended to preserve the no-DATA fast path.


## M14C authority split

`Hp67DataSerialWordPath::preconsume_previous_tail()` now reconstructs the preceding frame's serial bits 54/55 from the documented following-word b0/b1 positions before the new instruction-boundary bridge runs. `visit_transport_word_bit()` prevents those two logical DATA samples from being consumed twice when the shared structural IS/display/fetch loop subsequently traverses the same machine-word coordinates. A back-to-back frame may then start normally at b2.

This is intentionally a logical scheduling split, not an electrical timing claim. No PHI edge, DATA polarity, passive level or propagation delay is assigned. Its purpose is causal: an installed-RAM read/write result can be committed from the reconstructed serial frame before the following architectural fallback instruction is allowed to observe C or RAM.

## M15E resolved-net logical-level bridge

The default DATA path remains logical-only for reference fixtures. Live RAM
opts in per word with `enable_electrical_level_bridge(direction)`. Its sources
encode bit values as explicit High/Low logical levels and release HighZ when no
frame is active. This is a WORKING APPROXIMATION, not a claim about the physical
DATA transistor stage, passive bias, voltage polarity or PHI timing. DATA's
passive bias remains Floating. ACT writes use the ACT driver; reads use the
existing aggregate structural ROM/RAM slot. Physical chip mapping is unknown.

The source publishes both ownership contributions in one observable commit;
the sink reconstructs from `Hp67Net::Data` resolved levels instead of receiving
the source's bool directly. Counts reject extra contributors even when their
levels agree, including idle ghost drivers. Floating or contention while a bit
is required fails before sink mutation. Directions are retained independently
for the prior b0/b1 tail and current b2..b55 body, preserving opposite-direction
back-to-back frames. The existing early-tail scheduling bridge now uses the net;
the later structural traversal replays those pins without consuming the sink
twice. Early-tail publication remains unclocked and explicitly approximate.

Replacing the logical-level adapter requires evidence for DATA output stage and
launch/sample/handoff edges; it must preserve net reconstruction and diagnostics.

## M15F production tail completion

Live transport no longer calls the unclocked early-tail method. The shared b0/b1
prefix visits the same resolved-net sink while advancing the actual backplane.
`take_clocked_prefix_result()` extracts that completion and marks the already
consumed prefix for body continuation. The source tail/direction are retained
until the following body binds its new payload. Legacy preconsume APIs remain
for phase/reference fixtures; they no longer schedule production RAM authority.

## M15H provisional PHI receiver scheduling

Production DATA now separates `launch_transport_word_bit()` from
`sample_transport_word_bit_at_edge()`. Publication validates bit order and the
after-PHI2 interphase boundary; it does not mutate the sink. Sampling requires
PHI1 rising's committed phase, correct bit coordinate and both resolved clocks
High. It reads existing DATA contributions without re-driving them, validates
ownership/level and only then consumes the bit. Reference combined visitors
retain their original unclocked API for isolated fixtures.

New tests reject early/late samples without sink mutation, traverse both DATA
directions plus following-word tails, and introduce agreeing/opposing extra
drivers between launch and receive. A changed committed source bit is consumed
as observed rather than overwritten by the expected source value.

This is a WORKING APPROXIMATION with a source-backed stable interval, not
a measured latch-edge/propagation claim. Full words remain 56 cells and 224
transitions. See `docs/research/M15H_PHI_RECEIVER_SCHEDULING_2026-10-16.md`.

## M15I atomic serial publication

Production `propose_transport_word_bit()` computes optional ACT/RAM DATA drives
from immutable state, validates bit order/launch phase and returns errors before
any serial pin changes. It is composed with IS/SYNC at the shared launch commit.
No electrical DATA role returns `None` (preserve the net); an opted-in idle cell
returns two HighZ contributions (release the modeled sources). Reference combined
visitors retain their original API. The old standalone M15H launch helper is now
test-only for isolated receiver fixtures; it is not a production publication path.

The launch boundary and PHI1-rising sample checkpoint retain M15H
WORKING APPROXIMATION status. No propagation delay, physical output stage or
unknown latch edge is promoted to proven hardware behavior.

## M15L DATA receiver owner identity

A sole active driver is necessary but insufficient: a different device could
replace the legitimate source, preserve the same Low/High level and previously
be accepted. Sampling now requires the expected named driver to be non-HighZ,
in addition to the existing active-count and resolved-level checks. ACT writes
expect `Act1820_2530`; peripheral reads expect the existing aggregate
`StructuralRomResponder`. This validates the M15E bridge contract, not physical
RAM-chip mapping or transistor polarity.

`direction_for_word_bit()` supplies the preceding frame's direction at b0/b1
and the current frame's direction at b2..b55 to both proposal and receive paths.
Wrong sole ownership reports `InvalidElectricalOwner` with bit, expected driver
and resolved level before changing sink/frame/completion state. Extra/missing
contributors, idle ghost drivers and invalid levels retain their existing error
checks. Sampling never republishes a source or compares its value to the
expected payload. No clocks, allocations or bus commits are added.

Tests replace every active cell's owner with every other named driver at both
logic levels, including the opposite-direction handoff and both following-word
tails. All twelve idle ghost contributions remain rejected. The full DATA state
must remain unchanged on failure, while a correct owner changing zero to one
at the receive edge must reconstruct an all-ones register in both directions.
Each three-word fixture retains exactly 672 PHI transitions. See
`docs/research/M15L_DATA_OWNER_IDENTITY_2026-10-17.md`.
