# HP-67 IS/ISA fetch timing evidence

Research snapshot: 2026-09-15.

## Scope

This note records only timing/ownership facts that are directly supported by the HP-67 section of Tony Nixon's *Notes on HP's Classic Calculators* and the existing physical HP-67 bus work. It intentionally separates those facts from still-open PHI edge and absolute-time questions.

Primary source used here:

- https://literature.hpcalc.org/community/classic-notes.pdf
- section headed **“Woodstock – HP-67”**, pages 64-76 in the current 83-page PDF.

## Canonical 56-bit word coordinate

The HP-67 trace is described in terms of one 56-bit instruction/machine cycle. hp67emu names those serial positions `b0..b55`.

The current scheduler still uses an abstract four-subphase PHI scaffold. The `b0..b55` positions are now evidence-backed coordinates, but the scaffold's subphase durations are not physical calibration.

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

The same page states that after the address transfer there are 19 intervening bit times before the selected ROM begins returning data at bit times **46 through 55**, again **LSB first**.

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

This means the ROM-word electrical window is always b46..b55, while SYNC tells the ACT how to interpret the returned ten bits.

## IS/ISA passive and active drive behavior

The HP-67 bus investigation on page 74 describes IS and DATA as shared serial buses with multiple ICs attached. Only one device is intended to control a bus at a time.

For IS specifically, resistor tests indicate a weak/passive low path and an active device driving the line high; the source explicitly reasons that devices do not actively pull the line low because that would create a short-circuit risk when another participant drives high.

hp67emu therefore models the currently evidenced IS behavior as:

```text
logical 0 -> release / High-Z -> passive low bias
logical 1 -> actively drive High
```

The generic net model still retains explicit Low/High/High-Z/contention capability because other HP-67 nets may use different electrical behavior.

## Ownership during instruction fetch

For the fetch path now encoded in hp67emu:

```text
b16..b27 : ACT supplies 12-bit ROM address, LSB first
b28..b45 : no fetch payload is assigned here; ROM access/turnaround interval
b46..b55 : selected ROM supplies 10-bit word, LSB first
```

This ownership description applies only to instruction fetch. IS has other functions outside those windows. In particular, the HP-67 ROM0/display section states that ROM0 outputs display-anode data on IS during bit times 0..7. That behavior belongs in the future 1818-0268/ROM0 device rather than the generic fetch serializer.

## Pipelining consequence

The source explicitly states that the instruction fetched during one 56-bit machine cycle is executed during the following 56-bit cycle. hp67emu must therefore keep fetch and execution as adjacent pipeline stages rather than treating a ROM lookup as an instantaneous read at the moment an instruction executes.

This is the key architectural bridge from the current instruction-boundary reference model to the future electrical ACT/ROM implementation.

## What is still open

The following are deliberately **not** fixed by the code added with this note:

- absolute PHI1/PHI2 frequency for the HP-67 target;
- exact PHI1/PHI2 high widths and dead time;
- the precise PHI edge on which the ACT launches each address bit;
- the precise PHI edge on which a ROM launches and the ACT samples each returned bit;
- propagation delay between PHI transitions and IS transitions;
- DATA bus passive/active drive behavior beyond the currently recorded source observations;
- exact reset-to-first-valid-SYNC timing in production code.

The current PDF includes expanded PHI/SYNC/IS waveforms on page 70, so the next evidence task is to convert those waveform edges into an explicit launch/sample convention without relying on visual guesswork.

## Implementation mapping

- `src/machines/hp67/timing.rs`
  - canonical `b0..b55` coordinates;
  - address window b16..b27;
  - ROM-word/SYNC-decision window b46..b55.
- `src/machines/hp67/isa.rs`
  - LSB-first address/word serialization;
  - active-high / release-for-zero IS drive behavior.
- `src/machines/hp67/machine.rs`
  - passive pull-down bias for the shared IS/ISA net.

No ROM bytes are embedded by these modules.
