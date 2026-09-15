# HP-67 IS/ISA fetch and display timing evidence

Research snapshot: 2026-09-15.

## Scope

This note records timing/ownership facts directly supported by the HP-67 section of Tony Nixon's *Notes on HP's Classic Calculators* and the existing physical HP-67 bus work. It intentionally separates those facts from still-open PHI edge and absolute-time questions.

Primary source used here:

- https://literature.hpcalc.org/community/classic-notes.pdf
- section headed **“Woodstock – HP-67”**, especially pages 64-77 in the current 83-page PDF.

## Canonical 56-bit word coordinate

The HP-67 trace is described in terms of one 56-bit instruction/machine cycle. hp67emu names those serial positions `b0..b55`.

The current scheduler still uses an abstract four-subphase PHI scaffold. The `b0..b55` positions are evidence-backed coordinates, but the scaffold's subphase durations are not physical calibration.

## Exact ROM0 display-data window

The HP-67 ROM0 investigation on pages 75-77 states that the data **for ROM0** is present on IS during bit times **0 through 7**. The captured byte is LSB first. A measured example is `00000100` on the wire, which reconstructs to `0x20` and is decoded as blank.

Therefore hp67emu fixes:

```text
b0 -> ROM0 display-code bit 0
b1 -> ROM0 display-code bit 1
...
b7 -> ROM0 display-code bit 7
```

The same direct trace states that STR occurs during display bit 7. We record `b7` as the coarse STR coordinate but do not yet choose the final PHI-relative transition or pulse width.

The direct HP-67 decode table is:

```text
00..09 -> digits 0..9
0A     -> o
0B     -> C
0C     -> r
0D     -> d
0E     -> E
0F     -> blank
20     -> blank (measured example)
30     -> decimal point
4x     -> blank
```

The shared sign position is scan slot 3. Only display-code bits 0 and 1 are relevant there: bit 0 low indicates a negative mantissa and energizes ROM0 segment E; bit 1 high indicates a negative exponent and energizes segment G. The board's transistor routing turns those anode selections into the two visible sign bars.

A reviewed Nonpareil semantic character table reverses the `o`/`r` code numbers (`0A`/`0C`). The production structural decoder follows the direct HP-67 logic-analyser table above and keeps that discrepancy explicit.

## Cathode scan evidence

ROM0 (`1818-0268`) provides STR to the `1820-1749` cathode driver; the ACT provides RCD. The source says RCD resets the cathode driver to digit/slot 1 and the captured waveform indicates reset on the low-going RCD edge. STR advances the display one position at a time and segment A begins on the low-going STR edge.

Fifteen STR slots are observed per refresh even though the display/cathode topology has fourteen driver positions because the two signs share a cathode position. The captured scan-data order is:

```text
1  exponent units
2  exponent tens
3  mantissa/exponent shared signs
4  mantissa digit 11
5  mantissa digit 10
...
14 mantissa digit 1
15 exponent-units duplicate
```

The source notes that slot 15 carries the same display data as slot 1 and appears to be discarded; RCD overlaps the final STR. hp67emu therefore models a fifteen-slot structural scan sequence but does not invent a fifteenth physical cathode output or commit yet to overlap ordering inside a PHI subphase.

## Exact ROM-address window

The HP-67 pipeline trace on page 66 states that the ROM instruction address is **12 bits wide, LSB first**, and is sent to the ROM during bit times **16 through 27**.

Therefore hp67emu fixes:

```text
b16 -> ROM address bit 0
b17 -> ROM address bit 1
...
b27 -> ROM address bit 11
```

This supersedes earlier Classic-family hypotheses such as an eight-bit address window. The HP-67 uses the Woodstock 12-bit address path.

The same trace shows address `0x07B` as an example and the subsequent fetched word `0x04C`.

## Exact ROM-word window

The same page describes a following **19-cycle ROM access interval** and states that the selected ROM starts outputting data at bit times **46 through 55**, again **LSB first**. We preserve the source's timing statement without reinterpreting that 19-cycle phrase as a count of only the strictly-between numbered slots.

Therefore hp67emu fixes:

```text
b46 -> ROM word bit 0
b47 -> ROM word bit 1
...
b55 -> ROM word bit 9
```

Each normal Woodstock microinstruction is 10 bits, so this window transports one complete fetched word.

## SYNC meaning and width

The HP-67 key-wait trace on pages 64-65 shows that SYNC occupies the same final ten bit times, b46..b55, when a normal instruction is being returned.

The important exception is the word fetched immediately after an `IF`/test instruction. In that case SYNC remains low during b46..b55 and the returned ten bits are not decoded as an opcode; they become the full within-1K destination for the implied `THEN GOTO`.

The measured HP-67 examples include ordinary instructions with `Sync = 1` and following THEN-GOTO words with `Sync = 0`.

This means the ROM-word electrical window is always b46..b55, while SYNC distinguishes an ordinary instruction-fetch cycle from the following implied-GOTO target cycle.

## IS/ISA passive and active drive behavior

The HP-67 bus investigation on page 74 describes IS and DATA as shared serial buses with multiple ICs attached. Only one device is intended to control a bus at a time.

For IS specifically, resistor tests indicate a weak/passive low path and an active device driving the line high; the source explicitly reasons that devices do not actively pull the line low because that would create a short-circuit risk when another participant drives high.

hp67emu therefore models the currently evidenced IS behavior as:

```text
logical 0 -> release / High-Z -> passive low bias
logical 1 -> actively drive High
```

The generic net model still retains explicit Low/High/High-Z/contention capability because other HP-67 nets may use different electrical behavior.

## Ownership by window

For the currently encoded roles on shared IS:

```text
b0..b7   : display code presented to ROM0, LSB first
b16..b27 : ACT supplies 12-bit ROM address, LSB first
b28..b45 : no instruction-fetch payload is assigned here; ROM access/turnaround interval
b46..b55 : selected ROM supplies 10-bit word, LSB first
```

These windows describe only the roles already supported by direct HP-67 evidence. They do not imply that IS has no other behavior at other times, and they do not yet identify every physical driver or PHI edge for the display transfer.

## Pipelining consequence

The source explicitly states that the instruction fetched during one 56-bit machine cycle is executed during the following 56-bit cycle. hp67emu must therefore keep fetch and execution as adjacent pipeline stages rather than treating a ROM lookup as an instantaneous read at the moment an instruction executes.

`src/machines/hp67/fetch.rs` represents that boundary explicitly: the ACT endpoint sends a 12-bit address through the resolved IS net one bit at a time; the ROM endpoint reconstructs it from those electrical levels, latches a caller-supplied 10-bit ROM word, and returns that word through the same resolved net one bit at a time. A pipeline latch promotes a word fetched in cycle N to the executing slot when cycle N+1 begins.

The integration regression uses the measured HP-67 `0x07b -> 0x04c` pair and the physical startup `0x001 -> 0x3e3` pair. Neither 10-bit value is passed directly from the ROM endpoint to the ACT endpoint; the receiver rebuilds it from ten resolved b46..b55 samples.

## What is still open

The following are deliberately **not** fixed by the current structural code:

- absolute PHI1/PHI2 frequency for the HP-67 target;
- exact PHI1/PHI2 high widths and dead time;
- the precise PHI edge on which the ACT launches each ROM-address bit;
- the precise physical driver/PHI edge that launches the b0..b7 display-code bits;
- the precise PHI edge on which a ROM launches and the ACT samples each returned instruction bit;
- propagation delay between PHI transitions and IS transitions;
- exact STR pulse width and its final PHI-relative rising/falling edges;
- exact RCD pulse placement and RCD/last-STR overlap ordering in the scheduler;
- DATA bus passive/active drive behavior beyond the currently recorded source observations;
- exact reset-to-first-valid-SYNC timing in production code;
- physical bank-selection storage/propagation inside the individual 1818-* devices;
- analog LED/inductor current and the exact brightness-transfer function.

The PDF includes expanded PHI/SYNC/IS waveforms on page 70 and display waveforms on pages 76-77, so later timing work can convert those edge relationships into explicit scheduler conventions without visual guesswork.

## Implementation mapping

- `src/machines/hp67/timing.rs`
  - canonical `b0..b55` coordinates;
  - ROM0 display window b0..b7 and coarse STR bit b7;
  - address window b16..b27;
  - ROM-word/SYNC-decision window b46..b55.
- `src/machines/hp67/display.rs`
  - ROM0 display-byte reconstruction from resolved IS levels;
  - direct HP-67 display-code decoder and shared-sign behavior;
  - source-backed fifteen-slot scan order;
  - structural RCD-falling reset and STR-falling scan events.
- `src/machines/hp67/isa.rs`
  - LSB-first address/word serialization;
  - active-high / release-for-zero IS drive behavior.
- `src/machines/hp67/fetch.rs`
  - ACT address shift-out and ROM-word shift-in;
  - ROM address reconstruction and response serialization;
  - one-cycle fetch/execution pipeline latch;
  - hard failure on floating/contentious samples or absent fixture words.
- `src/machines/hp67/machine.rs`
  - passive pull-down bias for the shared IS/ISA net.

No ROM bytes are embedded by these modules.
