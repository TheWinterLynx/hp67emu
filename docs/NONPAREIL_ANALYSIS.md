# Nonpareil analysis — what hp67emu should reuse

Research baseline: `brouhaha/nonpareil` `main`, reviewed 2026-09-15.

## Executive conclusion

We should **not re-discover the Woodstock instruction set from scratch**. Nonpareil already contains a mature Woodstock semantic model, a complete HP-67 calculator definition, HP-67/97 ROM disassemblies, keyboard hardware codes, ROM/RAM mapping, display character mapping, bank behaviour and a CRC/card-reader model.

However, Nonpareil is not an electrical or 56-bit bit-cycle model. Its Woodstock core fetches a complete 10-bit microinstruction, executes the instruction as one semantic operation over 14-nibble register arrays, advances the display once per microinstruction, and exposes keys through semantic key codes/flags. That is an excellent **reference/oracle layer** for hp67emu, but it cannot be the final fidelity path.

Our design therefore uses two implementations with different jobs:

1. `reference::woodstock` — a small, deterministic Rust semantic model used for instruction decoding, differential tests and bring-up.
2. `emulation` + `machines::hp67` — the production electrical model, where ACT/ROM/RAM/display/CRC communicate through timed signals and the real microcode causes visible behaviour.

At microinstruction boundaries both models should agree on architectural state. Inside a microinstruction only the electrical model is authoritative.

## Licensing boundary

Nonpareil's simulator source explicitly uses **GNU GPL version 2 only**. The repository `COPYING` file also says the `ncd` tree and derived files have separate licensing, while individual newer files may carry their own notices.

A line-by-line C-to-Rust translation of `proc_woodstock.c`, `crc.c`, etc. would therefore be a derivative of GPL-covered source and would force a project-level licensing decision for distributed combined binaries/source.

For now hp67emu will use Nonpareil as a behavioural specification and independent cross-check, and will implement the Rust semantic model in a different structure from facts that can be corroborated by HP/Teenix documentation. No Nonpareil source text is copied into production files. If the project later deliberately adopts GPL-compatible licensing, this boundary can be revisited.

## Files reviewed

### `src/proc_woodstock.h`

This exposes the architectural state Nonpareil considers necessary for Woodstock:

- eight 14-nibble registers: A, B, C, Y, Z, T, M1, M2;
- 4-bit `F` register;
- pointer `P` in the 0..13 range plus undocumented wrap states used by later ACTs;
- decimal/binary arithmetic mode;
- carry and previous carry;
- 16 status bits plus external flags;
- 12-bit program counter;
- delayed ROM-selection state;
- a two-entry return stack;
- keyboard buffer/flag and scanner inputs;
- display state;
- up to four 1K-word logical page groups and two banks;
- RAM address/data state and peripheral hooks.

The architecture-variant comments are especially useful: Nonpareil records the unusual P-wrap behaviour as applying to later ACT variants including HP parts `1820-1596` and `1820-2530`. This is a regression target for our hardware model, not an invitation to reproduce Nonpareil's compatibility hack.

### `src/proc_woodstock.c`

This is the central semantic implementation.

The 10-bit opcode space is dispatched through a 1024-entry table. At the broadest level the low two opcode bits classify instructions as:

- `..00` — special/miscellaneous instruction families;
- `..01` — JSB;
- `..10` — arithmetic/register instruction;
- `..11` — GOTO.

Arithmetic instructions decode:

- a 5-bit operation number;
- a 3-bit field selector.

The eight fields are `p`, `wp`, `xs`, `x`, `s`, `m`, `w`, and `ms`. The semantic operation then acts on the selected slice of one or more 14-nibble registers. This is the right instruction-level behaviour for our Rust oracle.

The special decoder covers status bits, P manipulation, constants, ROM/page selection, delayed ROM selection, register/RAM access, decimal/binary mode, display control, key dispatch, return, bank switch and related instructions.

### Important fidelity gap in `woodstock_execute_cycle`

Nonpareil's `execute_cycle` means approximately **one microinstruction cycle**, not one electrical bit time. It:

1. reads the complete opcode from ROM;
2. increments PC;
3. invokes the selected semantic handler;
4. applies delayed ROM selection;
5. increments the cycle counter;
6. updates key flags;
7. performs one display-scan step.

Register arithmetic and transfers update complete field ranges in one host call. ISA/DATA bit traffic, PHI1/PHI2, SYNC, bus release and individual 56-bit serial positions are not represented.

This distinction is central: Nonpareil gives us the expected **instruction-boundary state**, while our ACT model must explain how that state is produced electrically.

### `src/digit_ops.c`

Contains reusable semantic ideas for BCD/hex digit arithmetic, carry/borrow, shifts, copies, exchanges and comparisons. We implement equivalent Rust helpers for the reference model, then separately implement bit-serial ALU behaviour for the electrical ACT.

The production ACT must not call a whole-field `add()` and pretend that operation is cycle accurate.

### `src/crc.c`

Nonpareil has a useful functional model of the HP-67/97 CRC/card-reader chip. It identifies twelve logical flags, including:

- buffer ready;
- program-mode switch;
- merge;
- pause;
- motor enable;
- card inserted;
- write mode.

It maps CRC-specific opcode slots into the Woodstock opcode table and models read/write access through special RAM addresses. This is excellent semantic behaviour for card-reader regression tests and is now represented independently in `reference::crc`.

It is not a model of magnetic-head timing, sense-amplifier pulses, motor transport or serial electrical bus timing. Those remain separate hp67emu devices.

### `ncd/67-97/67.ncd.tmpl`

This is unusually valuable HP-67-specific metadata. It declares:

- architecture: Woodstock;
- platform: Hawkeye;
- model: 67;
- nominal instruction clock: 185000 Hz;
- cathode driver `1820-1749`;
- ROM0/anode driver `1818-0268`;
- ROM/RAM devices `1818-0550`, `1818-0551`, `1818-0232`, `1818-0231`;
- CRC `1820-1751`;
- instruction-memory bank/page layout;
- data-memory ranges;
- the 8-segment character generator;
- RUN/PRGM switch through CRC flag 1;
- every HP-67 hardware key code.

The file names `1820-1596/MK6216N` as the ACT. Our HP-67 schematic and an inspected physical HP-67 instead identify `1820-2530`. This apparent conflict is now reconciled rather than left as an unknown: the HP-97 service manual's logic-PCA replacement table explicitly allows `1820-2530` when `1820-1596` is unavailable, and Nonpareil's own architecture-variant table assigns the same P-wrap behaviour to both parts. We therefore keep `1820-1596` as a valid semantic compatibility reference while targeting the physically documented `1820-2530` revision in hp67emu. See `docs/HARDWARE_SOURCES.md`.

### `ncd/67-97/*.asm`

Nonpareil includes disassembled HP-67 material rather than treating the firmware as a black box:

- `67.asm` — HP-67-specific bank-0 quad starting at address 0000;
- `6797.asm` — common HP-67/97 code;
- `67b1.asm` — HP-67 bank-1 code;
- `6797cr.asm` — common HP-67/97 card-reader code;
- corresponding HP-97 files for comparison.

The startup disassembly begins at ROM/anode driver `1818-0268` and immediately exposes real labels, CRC flags, run/stop paths and cross-ROM entry points. This will be extremely useful for naming traces and diagnosing boot mismatches.

For canonical firmware bytes we still prefer independently hashed physical ROM dumps (Teenix). The Nonpareil disassembly is a cross-check and symbolic map.

## HP-67 configuration adopted as reference/test data

### ROM/RAM topology

Use the Nonpareil HP-67 definition as one source for the mapping, then verify it against the HP-67 schematic and Teenix raw dumps. The machine has banked instruction storage with 1K-word logical groups and 16-register RAM blocks in the ROM/RAM chips. Teenix's later notes express the same physical ROM organization as sixteen 256-word pages; the two descriptions differ in grouping, not in the 4K PC address space.

### Keyboard codes

Nonpareil gives all 35 user-key to hardware-key-code mappings. We can import the mapping as test data after checking it against the schematic/key scan. The final UI still closes physical contacts; the semantic hardware key code is an observation produced by the scanner, not the input API.

### RUN / W-PRGM

Nonpareil models RUN/PRGM through CRC flag 1. This directly supports the direction already chosen for hp67emu: the slider must affect a hardware-visible CRC input rather than assign a UI `RunMode` enum.

### Display

Nonpareil's HP-67 definition gives the segment character generator and its Woodstock scan code documents special 14-digit/sign handling. We should reuse those facts for regression expectations, but drive our renderer from actual anode/cathode timing and integrated segment energy.

## Rust implementation status from this analysis

### Phase R1 — semantic reference core

Implemented in `reference::woodstock` and `reference::rom`:

- 14-nibble registers and architectural control state;
- complete four-way 10-bit decoder;
- all 32 arithmetic/register operations and eight fields;
- required HP-67 CPU special operations;
- RAM/register access;
- JSB/GOTO/THEN-GOTO/return/bank behaviour;
- host-independent banked ROM storage and fetch.

### Phase R2 — differential harness

Implemented in `reference::snapshot` and `reference::differential`:

- instruction-boundary CPU snapshots;
- readable state differences;
- timed-target trait with monotonic completed-instruction count;
- bounded tick advance to the next target instruction boundary;
- skipped-boundary/stall/divergence diagnostics.

The harness is ready; the electrical ACT still needs to expose its reconstructed architectural snapshot and instruction counter before it can be connected.

### Phase R3 — run original HP-67 ROM in both paths

Pending verified external ROM corpus. First target remains reset/startup until the firmware reaches a stable key-wait/display loop, then longer traces and real key sequences.

### Phase R4 — peripherals

CRC/card behaviour is now implemented semantically in `reference::crc` and composed into `reference::hp67`. Electrical CRC/card transport/display behavior remains pending.

## What we should not port

Do not bring across:

- GUI/toolkit code;
- Nonpareil simulator/event framework;
- SCons/build machinery;
- image/display rendering;
- direct semantic key injection as the final input path;
- display scanning once per host-level microinstruction;
- whole-field ALU helpers into the production electrical ACT;
- address-specific compatibility hacks as hardware implementation.

Those pieces would either duplicate our architecture or reduce fidelity.

## Important Nonpareil compatibility clue: P wrap

Nonpareil contains an explicit address-specific workaround for an undocumented P-pointer behaviour used by HP-67/97 label search. This is valuable evidence because it tells us exactly where an instruction-level model leaks hardware detail.

Our semantic reference expresses the condition generically from recent P movement, and the historical HP-67 address around octal `06132` is preserved as a regression test. The eventual electrical ACT must produce the same effect naturally from pointer timing without consulting the firmware PC.

## Implementation rule

When Nonpareil and primary electrical evidence disagree, primary HP-67 evidence wins. When our electrical model and Nonpareil disagree at an instruction boundary, treat Nonpareil as a strong bug signal, then resolve the difference against the ROM disassembly, Teenix traces/dumps and HP hardware documentation.
