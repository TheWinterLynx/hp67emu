# `src/machines/hp67/rom_devices.rs`

## Purpose

Models five ROM-side serial receivers with independently held address, selection,
response word and named IS contributions for the live HP-67 transport.

## Why it exists

An aggregate ROM response cannot expose which IC received an address or drives
IS. Each chip now receives all twelve resolved address bits, decodes its 1K
coverage, and holds a response only when selected. ACT still reconstructs the
returned word from the actual resolved electrical bus.

## Relationships

`RomFetchEndpoint` owns an optional `Hp67RomChipSet`. Live construction/reset
opts in; default reference fixtures retain their aggregate endpoint. Immutable
firmware words remain behind `Hp67RomWordSource`; `selected_rom_bank()` supplies
the existing image/bank view. `electrical.rs` publishes the five chip contributions
atomically with ACT/SYNC/optional DATA. ROM0's IS fetch role and its display
receiver share a physical identity but retain separate functional endpoints.

## Responsibilities

Reconstruct b16..b27 independently for every chip; require one selected ROM;
reject incomplete/unresolved address samples; latch one word after complete
selection; release nonselected chips; serialize b46..b55 through named drivers;
and expose read-only chip state for validation. No per-cell heap allocation.

## Implementation

A fixed five-element array holds chip state. `HP67_ROM_PROFILES` maps ROM0 through
ROM4 to 1K bank/address windows. ROM0=1818-0268 and ROM3=1818-0232 use HP-67
inventory/capture evidence; ROM1=1818-0550 and ROM2=1818-0551 use the owner's
HP-97 equivalence instruction and service table 4-3. ROM4=1818-0231 is explicitly
a working inference by elimination from the five-part inventory, pending a
part-labelled dump. This uncertainty does not change address/data bit order.

All five address masks update before selection. Zero/multiple selected chips
fail before lookup; unselected responses stay absent. Only one lookup is made
for a successful word. A later source-bank change cannot rewrite the latched
response. Production publishes `RomIsDrives::Chips`; the legacy trace's single
ROM contribution remains an aggregate annotation, not per-chip observation.

Bank control remains the existing provider bridge, including its bank-0 fallback
outside populated bank-1 PC 0x400..0x7ff. Autonomous ROM bank-instruction/SYNC
latches are not implemented or inferred. RAM DATA ports remain aggregate and
physical RAM address mapping/output circuitry remain separate work. M15H's
provisional PHI checkpoints remain unchanged. See the M15J research note.
