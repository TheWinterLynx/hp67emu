# HP-67 structural real-microcode power-on smoke — 2026-09-15

## Result

A local run against the normalized 5120-word Teenix HP-67 corpus completed the structural power-on checkpoint successfully.

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

The CRC DATA ports remain intentionally unimplemented in this architectural bridge. A read/write targeting addresses `0x9b`/`0x99` stops explicitly as `CrcDataPortNotModeled`, so a later probe boundary will represent genuinely missing card-reader/data-path behavior rather than a valid CRC flag command.

## 10,000-cycle continuation

After adding CRC control ownership, the local real-firmware probe completed all 10,000 requested additional machine cycles without hitting an unsupported architectural or hardware boundary:

```text
PROBE LIMIT: completed 10000 additional machine cycle(s) without an unsupported architectural/hardware boundary;
executed_words=10003; pc=0x087; bank=0.
```

The detailed prefix also independently traversed the previously observed delayed-ROM/JSB landmark:

```text
cycle 85: EXEC pc=0x067 word=0x3f4 ... -> pc=0x068
cycle 86: EXEC pc=0x068 word=0x319 ... -> pc=0xfc6
```

and executed CRC control traffic such as `set flag 4`, test/clear flag 5 and test/clear flag 6 while continuing through the same serial fetch path.

## Deterministic boot-to-idle checkpoint

A subsequent local run enabled `--stop-at-idle` and reached the complete source-backed idle criterion after only 281 executed words:

```text
BOOT IDLE PASS: real firmware completed initialization and cycled through the documented no-key wait loop with display_enable=true.
BOOT IDLE: cycle=281; executed_words=281; pc=0x078; bank=0
BOOT SUMMARY: display_init_seen=true; main_wait_visits=2; card_poll_visits=1; display_enable=true; no_key=true; physical_0x067_to_0xfc6=true; idle_cycle=281
```

This closes the architectural power-on checkpoint. The criterion is deliberately stronger than reaching one convenient PC: firmware must execute the display-initialization landmark, visit the main wait loop twice, pass through the card-present poll at least once, have the display enabled, have no buffered key, and reproduce the physical `0x067 -> 0x0fc6` delayed-ROM/JSB path. The second wait-loop visit proves a complete no-input loop pass rather than a one-time initialization branch.

The ending PC `0x078` is the instruction immediately after the documented main wait entry `0x077`, consistent with stopping just after the second observed visit.

## Display fidelity handoff

With architectural reset/idle established, fidelity work moves to the actual display path rather than extending the instruction probe. Direct HP-67 logic-analyser evidence fixes ROM0 display data at IS `b0..b7` LSB-first, identifies the ROM0/anode decoder, the RCD/STR-controlled cathode scan and the fifteen observed STR slots. Those source-backed facts are now represented by the structural display module; exact PHI-relative edges, pulse widths, electrical LED current and integration with the ACT serial display source remain separate milestones.

## What this proves

The physical startup anchors can be fetched from the reconciled firmware corpus through the structural HP-67 serial bus and executed in the correct one-word pipeline order. Subsequent firmware continues through the same serial path, with ACT-versus-CRC instruction ownership resolved independently of the reference implementation. The real microcode completes initialization and enters a stable documented no-key idle loop after 281 executed words under the current architectural bridge.

## What this does not prove

The current transport operates at evidenced `b0..b55` bit-cell coordinates but still uses the temporary four-subphase PHI scaffold. It therefore does not establish final PHI1/PHI2 launch/sample edges, physical pulse widths, propagation delays, bit-serial ACT ALU timing, DATA-bus timing, physical 1818-* RAM behavior, CRC card transport or final display electrical behavior.

`ActRamImage` and the CRC flag core are explicitly architectural bring-up scaffolds. They are not substitutes for the final physical RAM chips or 1820-1751 card-reader controller model. Likewise, `display_enable=true` at architectural idle is not yet equivalent to emitted LED segment energy.

## Immediate continuation

Use the deterministic boot-idle checkpoint as the fixed starting point for display bring-up. The next steps are to observe the ACT display-register state at idle, connect its source-backed eight-bit display code to IS `b0..b7`, feed the ROM0 decoder, apply RCD/STR scan ordering through the cathode model, then integrate segment on-time so the first visible `0.00` emerges from microcode and modeled electronics rather than UI formatting.
