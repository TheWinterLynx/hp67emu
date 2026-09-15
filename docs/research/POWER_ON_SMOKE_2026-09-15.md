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

## What this proves

The directly observed physical startup anchors `0x000=0x000`, `0x001=0x3e3` and `0x0f8=0x11a` can be fetched from the reconciled firmware corpus through the current structural HP-67 serial bus model and executed in the correct one-word pipeline order. The conditional branch reaches `0x0f8`, `0 -> c[w]` executes, and the next real firmware word at `0x0f9` is fetched as `0x108`.

This test does **not** pass a 10-bit opcode directly from the host corpus to the execution core. The ACT-side endpoint emits the 12-bit address one serial bit at a time, the ROM-side endpoint reconstructs it from resolved IS levels, the response is emitted one serial bit at a time, and the ACT-side endpoint reconstructs the 10-bit word.

## What this does not prove

The current transport operates at evidenced `b0..b55` bit-cell coordinates but still uses the temporary four-subphase PHI scaffold. This result therefore does not establish final PHI1/PHI2 launch/sample edges, physical pulse widths, propagation delays, full 1820-2530 reset state, RAM/peripheral behavior, or display operation.

The minimal ACT core is intentionally incomplete. Unknown opcodes remain hard failures rather than being delegated to the semantic reference machine.

## Immediate continuation

The next reconciled firmware word reached by the successful run is `0x108` (octal `0410`), `c exchange m1`. The low-level startup core now implements that operation and the smoke runner supports `--probe-cycles N`, allowing the local real firmware path to continue until the next unsupported ACT word is encountered. This creates an evidence-driven loop for expanding the ACT without inventing instructions or control flow.
