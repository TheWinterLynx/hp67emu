# M15O — live C-to-address DATA

Base: owner-validated M15N ad4e1053622329d1650a505c4d68839b32b291c3,
integrated into main. Diagnostic 12/12 kept identical cycles. Independent RAM
fixture 1.003 us/word; live idle 1.344 us, RAM load 1.747 us, CRC 1.330 us.
Those are the owner's measurements on M15N, not measurements of this change.

## Executable change

The live ACT now sends its immutable C snapshot onto resolved DATA when its
structural execution action is SelectRamAddressFromC. The action decoder already
excludes implied-GOTO words. Both transports use the existing joint source
publication, owner checks, body and clocked next-word tail. Address reception
compares the entire reconstructed frame against frozen C, then clears its own
pending authority. It cannot call RAM write or replace C. An address frame and a
RAM data frame cannot share one source word. No per-bit allocation is added.

The ACT address scalar remains under M14H completed-word authority. Peripheral
full-frame completion is a distinct checkpoint, not a newly asserted silicon
latch edge. M15N autonomous RAM is NOT yet connected to this live composition.
Next: consistent resolved command forwarding, device-owned read drive/storage,
and differential comparisons against the oracle including selectors/clear.

## Evidence and limits

Uses the same direct HP-67 source read for M15N: Teenix, Notes on HP's Classic
Calculators (2025), printed pp66–69 and 74, especially captured C-to-address 270.
https://literature.hpcalc.org/community/classic-notes.pdf
SHA256 bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da.
The supported phase is 56 LSB-first bits from b2 through next b1. HP-97 fig2-5
corroborates under the owner's equivalence assumption. This adds no claim about
physical DATA polarity/output stage/bias, RAM chip mapping, propagation or exact
PHI-relative launch/latch edges. M15H's receiver checkpoint remains provisional.

## Verification

New native tests cover frozen C through the following write; write/read result;
installed and peripheral addresses; expected ACT driver and idle release;
ThenGoto exclusion; reset; contention preserving state/pending authority before
next execution; and 1200 real-firmware words in both transports with trace parity
and 224 transitions per word. Existing diagnostic remains required.

Rust is absent in this editing environment. Static diff/source/doc checks and
formatting-only Actions are reviewed here. All runtime tests, compilation,
release and benchmarks remain the owner's local gate; M15O gate pending.
