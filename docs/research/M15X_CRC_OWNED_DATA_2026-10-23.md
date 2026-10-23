# M15X — retained CRC DATA buffers and production transport ownership

Date: 2026-10-23
Base: owner-validated M15W `384c11197c8a30e0482f44aa5658a1bf4d2a058a`, integrated into main.
Branch: `agent/m15x-crc-owned-data`
Status: implemented; owner local gate pending.

## Completed lane

Production owns both pairs of CRC logical read/write buffers, FIFO heads/counts
and handshake state. ACT received instructions and retained ACT inputs select
0x99/0x9B through the established M14M classifier; staging reads owned storage
and the independent CRC write-mode flag. The source is no longer the host FIFO.
Only complete b55 images can commit; changed pre-state is rejected. ACT C uses
the same checked structural result with its existing 28-bit duplication rule.

Card transport ingress and draining now target retained buffers. A private narrow
port supports the real device owner and the existing architectural observer.
Independent IS/SYNC CRC latches supply mode/acknowledgement; owned buffer events
supply READY/F7. Active transport and media outputs are compared with an observer
before retained storage replaces its mirror. Inactive idle avoids media clones.
Pre-word FIFO corruption fails before clocks or pending RAM DATA tails. Reset
clears storage and explicit injected reference fixtures disable this lane.

## Evidence and limits

M14M logical port semantics and the existing HP card evidence remain unchanged:
two 28-bit buffers per direction, 34 record positions, nominal 28 ms cadence and
95-105 percent speed with fractional phase accumulation. HP-97 equivalence is
accepted by the owner. This adds ownership, not a new measured electrical fact.

b55 port publication and record-granularity transport remain WORKING
APPROXIMATION. CRC DATA polarity/driver/bit windows, intra-record magnetic order,
sense-amplifier timing, exact PHI/internal writes and propagation remain
SOURCE-BLOCKED. Installed-RAM DATA timing is not applied to CRC without evidence.
No PAC-specific semantics or new programs are introduced.

## Acceptance

Four library tests cover raw opcode/address/qualifier combinations against the
existing independent observer, early/stale commit rejection, empty/inactive/full
staging, repeated read/write FIFO wrap, mirror poisoning and all 34 records in
both directions at 95/100/105 percent speed. Three native tests compare traced,
untraced and reference firmware, reset and keyboard, FIFO corruption before a
pending DATA tail, and complete firmware write/eject/reinsert/read of 34 records.
All word-level transitions remain 56 cells and 224 PHI transitions.

Actions executes formatting/check/save only. Compilation, runtime, diagnostics,
release and benchmarks await the owner's local gate; no runtime pass is claimed.
