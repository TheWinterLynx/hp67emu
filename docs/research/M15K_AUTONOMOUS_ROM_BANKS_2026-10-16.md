# M15K — autonomous ROM bank latches

Base/main: owner-validated M15J `fa0c06364ffba7db7f8193b1301d970c6c9326a5`.
No firmware bytes or part profiles change.

## Evidence and scope

Teenix, Notes on HP's Classic Calculators (2025), printed p71–72 (PDF p72–73):
https://literature.hpcalc.org/community/classic-notes.pdf,
SHA256 `bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da`.
The author's physical bus captures describe all ROMs observing bank-set 0x230,
ROM4 responding in bank 1, and low addresses restoring bank 0. Repeating bank-set
does not restore bank 0. The captured route is 7f4/230 (bank 0), 7f5/11a and
495/210 (bank 1), then 0d5/0b4 (bank 0). This is word-level evidence, not a
measurement of the precise silicon decode/latch propagation inside b55.

SYNC distinction uses the existing HP-97 service IF/THEN evidence under the
owner's equivalence instruction (see M15D). The ROM decoder must not interpret
an implied-GOTO payload as a bank opcode. M15J's part-profile confidence levels
remain unchanged, including inferred ROM4 part identity.

## Implementation and causality

Production `autonomous_roms()` holds five persistent bank latches. Address
receivers reconstruct the resolved twelve-bit address, then reset the latches
for 000..3ff before selecting a chip. Other addresses select against the chip's
own latch. Only the selected chip asks for immutable explicit-bank ROM data.
No selection consults provider `selected_rom_bank()`, and no live host code
calls `select_bank()`. The independent structural ACT bank remains a comparison
that reports cycle/address on mismatch; it cannot control these latches.

All four transports pass their resolved ten-bit return and SYNC to the observer.
Stable High SYNC with complete 0x230 sets all latches after the final bit; Low
SYNC never does. The response already being returned is not rewritten. Cycle
initialization clears transient receivers/response but preserves bank latches.
Power/reset reconstructs zeroed latches. Invalid IS/SYNC and incomplete returns
cannot commit a bank-set. Bank-1 800..fff has no device and fails before lookup;
000..3ff restores bank 0 rather than addressing nonexistent bank-1 ROM0.

M15J provider-view mode and aggregate fixtures remain explicit references to
preserve their previous exhaustive 8192-view comparison. This legacy view has
fallback semantics; autonomous production does not. The ACT reference's bank
bookkeeping is not proof of ROM silicon behavior for arbitrary synthetic code.
Source-backed ROM bank-set is set-only; the real corpus route is regression-locked.

## Timing and performance practice

No extra time steps, bus publishes, heap allocations, copied firmware images or
host timing are added. One shared resolved observation is validated before the
five identical latch updates. Fixed arrays and one selected-ROM lookup remain.
56 bit cells / 224 PHI transitions stay mandatory, including clocked DATA tails.
The complete-word b55 decoder uses the existing provisional PHI1-rise checkpoint.
Exact bank decode propagation/latch edge and address reset edge are WORKING
APPROXIMATIONS within their word windows; replace when physical captures resolve
those points. Physical PHI width/dead time, DATA output circuitry/chip mapping,
STR/RCD exact edges, internal ACT mutation and magnetic serialization remain open.

## Validation

New regressions cover all 1024 words under both SYNC qualifiers; repeated set;
incomplete/invalid/changed qualifiers; all 8192 initial-bank/address combinations,
with provider bridge methods that panic if consulted; captured real-firmware
bank roundtrip on all four transports with poisoned provider selection; and live
boot/reset with poisoned provider bank. Existing M15J tests are preserved.

Rust is absent in the editing environment. Static diff/documentation checks are
run here; GitHub Actions formats Rust only. Focused tests, full -Dwarnings suite,
release, diagnostic 12/12 and three benchmark suites are the owner's local gate.
Owner M15K runtime gate is pending; no measured result is claimed in advance.

## Owner v2 gate feedback — 2026-10-17

Stages 1–8 completed; stage 9 failed at cycle 2198 with ROM bank 1 versus ACT
bank 0 at 7f5. The synthetic port fixture walked real ROM addresses but replaced
every fetched opcode before execution. It therefore let ROMs observe 230 at
7f4 while preventing ACT from executing the bank instruction. The bank guard
correctly exposed the inconsistent fixture.

The correction is confined to the benchmark/test fixture: pin synthetic PC 000
before each injected port word, preserving real concurrent fetch 001/3e3 and all
live authority/electrical work. FirmwareIdle is unchanged. Timed samples include
PC fixture setup; output states the changed workload. A negative reproduction
keeps the guard strict and a long positive regression covers both port loads.
No production bank semantics, firmware bytes, PHI transitions or resolver logic
change. M15K remains unmerged until the corrected full local gate passes.
