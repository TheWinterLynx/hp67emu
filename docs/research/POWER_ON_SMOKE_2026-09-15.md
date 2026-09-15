# HP-67 structural real-microcode power-on smoke — 2026-09-15

## Result

A local run against the normalized 5120-word Teenix HP-67 corpus completed the first structural power-on checkpoint successfully.

Command path:

```text
external normalized ROM corpus
  -> ACT serial address b16..b27
  -> resolved pull-down-biased IS/ISA net
  -> ROM serial response b46..b55
  -> ACT reconstruction
  -> one-word pipeline
  -> independent HP-67 architectural composition
```

The directly observed physical startup anchors remain:

```text
0x000 -> 0x000
0x001 -> 0x3e3
0x0f8 -> 0x11a
```

and the structural run reaches `0x0f8`, executes `0x11a` (`0 -> c[w]`) and advances to `0x0f9` with each word reconstructed from the serial IS/ISA path rather than handed directly to the executor.

## Probe continuation

The earlier evidence-driven probe advanced through both ACT memory registers and delayed ROM selection:

```text
0x0f9 -> 0x108  m1 exchange c
0x0fa -> 0x11a  0 -> c[w]
0x0fb -> 0x188  m2 exchange c
0x0fc -> 0x11a  0 -> c[w]
0x0fd -> 0x0b4  delayed select rom 2
0x0fe -> 0x003  goto low-page target 0
next fetch -> 0x200
```

The delayed-ROM implementation proves the correct one-word delay: `0264` at `0x0fd` leaves the following word at `0x0fe` in the current ROM, then applies ROM 2 to the resulting low eight PC bits.

## Acceleration pivot

The opcode-by-opcode method served its purpose: it demonstrated that the first power-on path was really executing firmware reconstructed from the serial bus, not delegating to the semantic reference model. Continuing that way would be unnecessarily slow.

The HP-67 low-level path now uses `ActArchitecturalCore`, an independent complete Woodstock instruction-boundary implementation covering all four opcode classes, all 32 arithmetic/register operations and eight fields, JSB/GOTO/THEN-GOTO, return stack, status/P/constants, ROM selection, bank switching, M1/M2, architectural RAM access, display control and key dispatch. `reference::woodstock` is not imported by production machine code; it is used only by differential regression.

`tests/act_architectural_differential.rs` compares every 10-bit word across several seeded architectural states and separately compares all 1024 THEN-GOTO payload words for both carry outcomes. Successful cases compare every ACT-visible field and the installed HP-67 architectural RAM range.

## First long-probe hardware boundary

The first accelerated local probe ran through 20 executed words before stopping at cycle 21:

```text
cycle 20: EXEC pc=0x20a word=0x213 ThenGoto { taken: false, target: 531 } -> pc=0x20b
cycle 20: FETCH bank=0 pc=0x20b -> word=0x200
PROBE STOP: unknown ACT special at cycle 21: pc=0x20b word=0x200 (octal 1000)
```

That stop exposed an ownership issue rather than a missing ACT instruction. Octal `1000` is a valid HP-67 CRC control opcode: CRC `set flag 4` (the firmware's `default_fn` flag), not an ACT special. The bring-up machine now composes the independent ACT core with an independent CRC control core. Recognized CRC words apply the ACT's universal instruction-boundary work as a NOP-equivalent cycle, then apply the CRC flag side effect. CRC test-and-clear words that evaluate true latch the controller's output into ACT status bit S3. THEN-GOTO payload words bypass CRC decode entirely.

The CRC DATA ports remain intentionally unimplemented in this architectural bridge. A read/write targeting addresses `0x9b`/`0x99` now stops explicitly as `CrcDataPortNotModeled`, so the next probe boundary should represent genuinely missing card-reader/data-path behavior rather than a valid CRC flag command.

## What this proves

The physical startup anchors can be fetched from the reconciled firmware corpus through the structural HP-67 serial bus and executed in the correct one-word pipeline order. Subsequent firmware continues through the same serial path, with ACT-versus-CRC instruction ownership resolved independently of the reference implementation.

## What this does not prove

The current transport operates at evidenced `b0..b55` bit-cell coordinates but still uses the temporary four-subphase PHI scaffold. It therefore does not establish final PHI1/PHI2 launch/sample edges, physical pulse widths, propagation delays, bit-serial ACT ALU timing, DATA-bus timing, physical 1818-* RAM behavior, CRC card transport, full peripheral timing or display operation.

`ActRamImage` and the CRC flag core are explicitly architectural bring-up scaffolds. They are not substitutes for the final physical RAM chips or 1820-1751 card-reader controller model.

## Immediate continuation

Run the long probe again. Valid CRC set/test-control words should no longer stop execution. The next stop should identify a genuinely unsupported hardware boundary such as CRC DATA transfer, ROM self-test, missing physical ROM population or another peripheral/electrical behavior.
