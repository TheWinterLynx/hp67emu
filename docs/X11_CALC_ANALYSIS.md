# x11-calc HP-67 analysis — how it helps hp67emu

Research baseline: `mike632t/x11-calc` branch `stable`, commit `9599ba6b8dc9eb55a4501ec2171a43d7ab5f9983`, reviewed 2026-09-15.

## Executive conclusion

`x11-calc` is a valuable **independent HP-67 semantic implementation and ROM cross-check**, but it is **not cycle accurate at the electrical/56-bit level** we are targeting.

The decisive evidence is in `src/x11-calc-cpu.c`: `v_processor_tick()` is explicitly documented as "Decode and execute a single instruction". Register arithmetic operates over complete nibble ranges in host loops, and the main X11 loop calls `v_processor_tick()` once per emulator tick. The host loop then sleeps to control apparent execution speed. This is instruction-level timing, not PHI1/PHI2, ISA/DATA, SYNC or individual 56-bit serial bit-times.

Therefore:

- do **not** use x11-calc as the timing authority for the electrical ACT;
- do use it as a second semantic oracle beside Nonpareil;
- use its HP-67-specific discoveries and diagnostic material as regression targets;
- compare its embedded HP-67 ROM corpus against the Teenix physical dump and Nonpareil disassembly.

## Files reviewed

### `src/x11-calc-cpu.c`

This is the shared instruction-level CPU implementation. It is particularly useful because its development history records several HP-67-specific behaviours discovered while making the real firmware run:

- an HP-67-specific `P + 1 -> P` behaviour that differs from earlier Woodstock machines and resembles later Spice behaviour;
- implicit bank-0 selection when the HP-67 PC enters the low ROM region below octal `02000`;
- explicit HP-67 CRC/card-reader opcodes and flags;
- bank-switch, delayed-ROM, stack, RAM/register and key dispatch semantics.

These observations are valuable because some of them expose exactly the same boundary cases that instruction-level Nonpareil had to special-case. Agreement between two implementations is a strong regression clue, though not proof of electrical timing.

### `src/x11-calc-67.c`

Contains the model-specific HP-67 definition:

- 35 hardware key codes;
- OFF/ON and PRGM/RUN switches;
- a built-in HP-67 ROM image;
- HP-67 card UI and model configuration.

The embedded ROM is declared as `int i_rom[ROM_SIZE]` and `ROM_SIZE` is octal `020000`, i.e. 8192 host entries. This is large enough to represent the two-bank semantic address space used by the emulator.

The first two ROM words are octal `00000, 01743`. Nonpareil's HP-67 disassembly at address `@0000` starts with `nop` followed by `go to reset0`, so the two projects agree immediately at reset entry. A full word-for-word comparison is now a planned corpus check.

### `src/x11-calc-67.h`

Defines:

- 15 display positions;
- 35 keys and two switches;
- `ROM_SIZE 020000`;
- 64 semantic data-memory registers.

The 15-position display count agrees with our physical display model, but x11-calc's rendering path is presentation-level and is not evidence for scan timing.

### `src/x11-calc.c`

Confirms the timing model is host/instruction paced rather than electrical. The change history states that one instruction is executed per main-loop iteration, and display refresh was deliberately decimated to once every 100 emulator ticks to reduce flicker. The current loop calls `v_processor_tick(h_processor)` and sleeps according to a host interval.

This means x11-calc is useful for **functional speed and behavioural sequencing**, but not for establishing one ACT word time, PHI edge placement, bus drive windows or propagation delays.

### `prg/x11-calc-67-*`

The repository includes HP-67 card/program material, notably diagnostic program A and C card images plus other known programs. These are excellent future end-to-end acceptance fixtures for:

- keyboard/program entry semantics;
- CRC/card reader;
- RAM/program storage;
- branch/flag behaviour;
- long-running firmware validation.

The card files themselves are third-party/project data and must be reviewed for redistribution before copying them into hp67emu. Their program behaviours can nevertheless be used as external regression references.

## Card-reader observations

x11-calc contains HP-67-specific semantic operations for display/card-controller state, including motor control, card-present testing and read/write mode. Its model is host-file oriented and intentionally higher level than the electrical CRC we want.

Use it to answer questions such as:

- which firmware-visible condition changes after a semantic card operation;
- which code paths expect card present / write mode / buffer state;
- which diagnostic programs exercise the reader.

Do not use it for magnetic-head timing, motor inertia, sense-amplifier pulses or ISA/DATA ownership.

## P-pointer regression value

The x11-calc history independently records that the HP-67 `P` increment behaviour differs from earlier Woodstock machines. Nonpareil also has an HP-67/97-specific workaround around the label-search path.

This makes the P-wrap/label-search path one of our highest-value ACT regressions:

1. run the original microcode through our semantic reference;
2. capture the relevant instruction sequence from x11-calc and Nonpareil;
3. make the electrical ACT produce the same architectural result naturally;
4. prohibit PC-address-specific hacks in the electrical implementation.

## ROM provenance role

x11-calc embeds a complete HP-67 ROM corpus directly in `src/x11-calc-67.c`. That makes it a useful **third corpus** for comparison, but not our preferred provenance source because the repository does not establish physical-reader provenance for those words in the model file itself.

Our priority remains:

1. Teenix physical ROM-reader dump — canonical candidate;
2. Nonpareil symbolic disassembly/object mapping — symbolic cross-check;
3. x11-calc embedded ROM — independent implementation/corpus cross-check;
4. Panamatik/Sydney Smith — further behavioural/address-level checks.

A full comparison should report every differing address by bank/page rather than silently choosing one image.

## Licensing boundary

x11-calc source files are GPLv3-or-later. hp67emu must not line-for-line translate its implementation unless we deliberately make a compatible licensing decision.

We may safely use it as research evidence for behaviour, compare externally observable states, record independently corroborated hardware facts, and write our own Rust implementation from the hardware/microcode evidence.

## Concrete hp67emu actions

- Add x11-calc as a Tier-C behavioural/corpus source, not Tier-A/B timing evidence.
- Compare the entire x11-calc 8192-entry ROM image against Teenix/Nonpareil after the physical dump is unpacked.
- Add P-wrap/label-search differential tests informed by both Nonpareil and x11-calc.
- Cross-check the 35 HP-67 key codes against Nonpareil and the physical key matrix.
- Use x11-calc diagnostic-card programs as end-to-end functional acceptance cases once the CRC/card path works.
- Never derive PHI1/PHI2, 56-bit slot timing, ISA/DATA drive windows, RCD or STR timing from x11-calc.
