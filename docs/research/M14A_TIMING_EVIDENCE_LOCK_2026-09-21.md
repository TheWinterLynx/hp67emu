# M14A timing evidence lock — 2026-09-21

## Scope

This slice records only timing relationships that are directly visible in the reviewed HP-67 logic-analyser captures in **Notes on HP's Classic Calculators / Teenix 2025**.

Primary source:

- https://literature.hpcalc.org/community/classic-notes.pdf
- HP-67 section, especially pages 64-77.

No PHI launch/sample edge is invented here. The existing four-subphase PHI scheduler remains a scaffold until the waveform edge relationships are converted into an explicit reviewed contract.

## PROVEN / EXACT at machine-bit / pin-polarity level

### PHI pin polarity and phase order

The expanded direct HP-67 captures on page 70 show PHI1 and PHI2 resting high and producing alternating low-going pulses. The pulses do not overlap in the shown captures.

M14A therefore maps the generic scheduler's abstract active phase to an HP-67 **low** pin level:

```text
PHI1 active -> PHI1 low,  PHI2 high
interphase  -> PHI1 high, PHI2 high
PHI2 active -> PHI1 high, PHI2 low
interphase  -> PHI1 high, PHI2 high
```

This locks polarity and ordering only. It does not claim equal durations, absolute widths, dead-time length or voltage amplitude. The electrical backplane now tracks this current phase explicitly and returns the crossed physical edge on every scheduler step; the separate word-timing counter remains only a completed-subphase/bit coordinate.

### SYNC edge anchor

The first two expanded page-70 captures isolate the rising and falling transitions of SYNC against PHI1/PHI2. Both SYNC transitions align with the **rising edge of PHI2**. M14A records that observation as `HP67_SYNC_TRANSITION_EDGE = Phi2Rising`.

This does not yet assign launch/sample edges to IS or DATA and does not claim a quantified propagation delay.

### Canonical word and ROM fetch

Already locked before M14A:

- 56 serial bit times per machine word.
- ROM address is 12 bits LSB-first at b16..b27.
- ROM result is 10 bits LSB-first at b46..b55.
- SYNC distinguishes instruction data from the implied-GOTO word over b46..b55.
- The fetched word executes in the following 56-bit machine word.

Source: pages 64-66.

### DATA stream phase

Direct HP-67 captures now remove one important ambiguity that was previously listed as fully source-blocked.

Pages 67 and 69 show the 56-bit DATA/register stream. Page 69 states that the first DATA bit is at instruction-cycle bit 2. Page 74 states the same relationship for a RAM read: after the first two clock cycles, the 56 RAM bits are output LSB-first starting at clock cycle 2, and DATA bits 54 and 55 appear at clock cycles 0 and 1 of the following instruction cycle.

Therefore the source-backed circular mapping is:

| machine-word bit | DATA serial bit |
| ---: | ---: |
| b2 | 0 |
| b3 | 1 |
| ... | ... |
| b55 | 53 |
| next b0 | 54 |
| next b1 | 55 |

Equivalently:

`data_bit = (word_bit + 54) mod 56`

This mapping is encoded in `src/machines/hp67/timing.rs` as `data_serial_bit_for_word_bit()`.

What this does **not** establish by itself:

- passive DATA level;
- active drive polarity;
- which exact PHI edge launches or samples DATA;
- electrical ownership handoff delays;
- per-device RAM/CRC drive-enable propagation.

### Display timing

Pages 76-77 provide direct HP-67 display timing observations:

- 56-bit instruction cycle: about 320 us;
- 15 STR pulses per complete display refresh;
- complete refresh: about 4.8 ms;
- normal LED segment on-time: about 40 us;
- decimal-point LED on-time: about 30 us;
- about 5 us gap after DP before STR;
- STR pulse width: about 5 us;
- display data is on IS during b0..b7;
- STR occurs on display-data bit 7;
- the low-going STR edge starts the segment interval in the shown capture;
- the cathode driver appears to reset on the low-going RCD edge;
- RCD overlaps the final (15th) STR pulse;
- the 15th display word is discarded by the display scan and the following first slot restarts at exponent units.

The microsecond values are measured/approximate scope values, not oscillator design constants.

## PROVEN but not yet encoded as an edge scheduler

Page 70 contains expanded HP-67 captures specifically showing:

- PHI1/PHI2 vs rising SYNC;
- PHI1/PHI2 vs falling SYNC;
- PHI1/PHI2 vs IS and DATA transitions;
- PHI1/PHI2 vs instruction bit 0;
- PHI1/PHI2 vs instruction bit 9;
- PHI1/PHI2 vs address bit 0.

These plots are strong evidence that the next M14A step should be an explicit edge contract rather than another generic timing approximation.

The current code deliberately does **not** yet translate the plotted edge placement into named launch/sample phases, because that requires a reviewed convention for signal polarity, edge identity and propagation tolerance. The four-slot scheduler therefore remains temporary.

## WORKING APPROXIMATION retained

- `TwoPhaseClock` still uses four generic slots per serial bit.
- Those slots are coordinates only; equal duration is not a hardware claim.
- ROM/ACT structural endpoints currently drive/sample once per bit cell rather than on a source-backed PHI edge.
- STR/RCD are structural events, not yet timed electrical nets.
- RAM/CRC transfers do not yet use the DATA net.

## SOURCE-BLOCKED after this slice

The following remain blocked or insufficiently quantified by the reviewed HP-67 evidence:

- absolute PHI1 width;
- absolute PHI2 width;
- exact dead-time intervals;
- exact launch/sample edge convention for every IS/DATA participant;
- DATA passive level and active-drive polarity;
- per-device propagation delays;
- ACT internal register commit edge;
- full power-on reset qualification sequence;
- HP-67/97 magnetic 28-bit record serialization order;
- insertion/head geometry, acceleration and first-bit phase;
- sense-amplifier/CRC PHI-relative magnetic timing.

## Exit criterion for M14A timing lock

Before replacing the scaffold, add a versioned edge table that names, for each relevant net and transfer:

- driver;
- receiver;
- launch edge;
- sample edge;
- allowed propagation interval;
- source page/capture;
- confidence class.

No production scheduler timing should depend on an edge that is not present in that table.
