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
  -> independent ACT execution core
```

Observed local output:

```text
HP-67 real-microcode structural power-on smoke
corpus: .research\teenix-2026-hp67.tsv (5120 populated words)
fetch path: ACT b16..b27 -> resolved IS -> ROM b46..b55 -> ACT
cycle 0: EXEC <pipeline fill>
cycle 0: FETCH pc=0x000 -> word=0x000 (word_index=1)
cycle 1: EXEC pc=0x000 word=0x000 Nop -> pc=0x001
cycle 1: FETCH pc=0x001 -> word=0x3e3 (word_index=2)
cycle 2: EXEC pc=0x001 word=0x3e3 ConditionalGoto { taken: true, target: 248 } -> pc=0x0f8
cycle 2: FETCH pc=0x0f8 -> word=0x11a (word_index=3)
cycle 3: EXEC pc=0x0f8 word=0x11a ZeroCWhole -> pc=0x0f9
cycle 3: FETCH pc=0x0f9 -> word=0x108 (word_index=4)
PASS: serial microcode power-on reached 0x0f8, executed 0x11a (0 -> c[w]), and advanced to 0x0f9.
```

## Probe continuation

The evidence-driven probe advanced through both ACT memory registers and reached delayed ROM selection:

```text
PROBE: continuing for up to 128 additional machine cycle(s), stopping at the first unsupported ACT word.
cycle 4: EXEC pc=0x0f9 word=0x108 ExchangeCAndM1 -> pc=0x0fa
cycle 4: FETCH pc=0x0fa -> word=0x11a (word_index=5)
cycle 5: EXEC pc=0x0fa word=0x11a ZeroCWhole -> pc=0x0fb
cycle 5: FETCH pc=0x0fb -> word=0x188 (word_index=6)
cycle 6: EXEC pc=0x0fb word=0x188 ExchangeCAndM2 -> pc=0x0fc
cycle 6: FETCH pc=0x0fc -> word=0x11a (word_index=7)
cycle 7: EXEC pc=0x0fc word=0x11a ZeroCWhole -> pc=0x0fd
cycle 7: FETCH pc=0x0fd -> word=0x0b4 (word_index=8)
PROBE STOP: unsupported ACT word at cycle 8: pc=0x0fd word=0x0b4 (octal 0264)
```

Octal `0264` is `delayed select rom 2`. The implementation proved the correct one-word delay: executing it at `0x0fd` leaves the immediate next PC at `0x0fe`, and only after that following word executes is ROM 2 applied to PC bits 11..8.

## Acceleration pivot

The opcode-by-opcode method served its purpose: it demonstrated that the first power-on path was really executing firmware reconstructed from the serial bus, not delegating to the semantic reference model. Continuing that way would be unnecessarily slow.

The HP-67 low-level path now uses `ActArchitecturalCore`, an independent complete Woodstock instruction-boundary implementation covering all four opcode classes, all 32 arithmetic/register operations and eight fields, JSB/GOTO/THEN-GOTO, return stack, status/P/constants, ROM selection, bank switching, M1/M2, architectural RAM access, display control and key dispatch. `reference::woodstock` is not imported by production machine code; it is used only by an exhaustive differential regression.

`tests/act_architectural_differential.rs` compares every 10-bit word across several seeded architectural states and separately compares all 1024 THEN-GOTO payload words for both carry outcomes. Successful cases compare every ACT-visible field and the installed HP-67 architectural RAM range. This shifts slow development effort away from already-understood instruction semantics and toward the electrical behavior that actually requires hardware evidence.

The smoke runner can now continue for many thousands of real firmware cycles with `--probe-cycles`, while `--trace-limit` limits console volume. Bank selection is tracked architecturally, page population is precomputed from the normalized corpus, and the HP-67 first-page bank-zero rule is applied before fetch.

## What this proves

The directly observed physical startup anchors `0x000=0x000`, `0x001=0x3e3` and `0x0f8=0x11a` can be fetched from the reconciled firmware corpus through the current structural HP-67 serial bus model and executed in the correct one-word pipeline order. Subsequent firmware continues through the same serial path; no 10-bit opcode is passed directly from the host corpus to the execution core.

## What this does not prove

The current transport operates at evidenced `b0..b55` bit-cell coordinates but still uses the temporary four-subphase PHI scaffold. It therefore does not establish final PHI1/PHI2 launch/sample edges, physical pulse widths, propagation delays, bit-serial ACT ALU timing, DATA-bus timing, physical 1818-* RAM behavior, full peripheral timing or display operation.

`ActRamImage` is explicitly an architectural bring-up scaffold outside the ACT state. It will be replaced in the fidelity path by physical RAM devices; its presence is not a claim that RAM is part of the ACT chip.

## Immediate continuation

Run a long probe with a short trace window. The next stop should now identify a genuinely unsupported hardware/architectural case such as ROM self-test, missing physical ROM population or a later peripheral/electrical boundary rather than simply the next ordinary Woodstock opcode.
