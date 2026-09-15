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
  -> independent minimal ACT execution core
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

The evidence-driven probe has now advanced through both ACT memory registers and reached delayed ROM selection:

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

Octal `0264` is the source-backed Woodstock `delayed select rom 2` special operation. The low-level startup core now stores a pending ROM nibble rather than changing PC immediately. At `0x0fd`, executing `0264` advances normally to `0x0fe`, so the next word is fetched from `0x0fe`; only after that following word executes is ROM 2 applied to PC bits 11..8. This ordering is regression-tested independently and is required for authentic delayed-ROM control flow.

## What this proves

The directly observed physical startup anchors `0x000=0x000`, `0x001=0x3e3` and `0x0f8=0x11a` can be fetched from the reconciled firmware corpus through the current structural HP-67 serial bus model and executed in the correct one-word pipeline order. The conditional branch reaches `0x0f8`, `0 -> c[w]` executes, and subsequent real firmware words through `0x0fd` are reached through the same serial path.

This test does **not** pass a 10-bit opcode directly from the host corpus to the execution core. The ACT-side endpoint emits the 12-bit address one serial bit at a time, the ROM-side endpoint reconstructs it from resolved IS levels, the response is emitted one serial bit at a time, and the ACT-side endpoint reconstructs the 10-bit word.

## What this does not prove

The current transport operates at evidenced `b0..b55` bit-cell coordinates but still uses the temporary four-subphase PHI scaffold. This result therefore does not establish final PHI1/PHI2 launch/sample edges, physical pulse widths, propagation delays, full 1820-2530 reset state, RAM/peripheral behavior, or display operation.

The minimal ACT core is intentionally incomplete. Unknown opcodes remain hard failures rather than being delegated to the semantic reference machine.

## Immediate continuation

Run the probe again after the delayed-ROM implementation. It should execute `0264` at `0x0fd`, fetch the following word from `0x0fe`, execute that word, then apply ROM 2 to the resulting PC before the next serial fetch. The next `PROBE STOP` identifies the next real startup operation to implement.
