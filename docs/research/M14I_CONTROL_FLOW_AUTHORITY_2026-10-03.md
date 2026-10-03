# M14I — structural control-flow authority

Date: 2026-10-03
Branch: `agent/m14i-control-flow-authority`
Status: implemented; owner local validation pending.

## Scope

Migrate PC, bank, delayed-ROM and two-level return-stack final authority. Cover sequential increment, JSB/GOTO, implied-GOTO, return, key/A dispatch, immediate/delayed ROM selection and bank switching. Implied-GOTO payload always bypasses normal ACT/CRC opcode interpretation.

## Causal path

Pre-instruction ACT state -> independent flow image -> concurrent successor address/bank -> resolved IS fetch -> completed b0..b55 flow -> oracle comparison -> live commit.

The oracle runs for expected flow and still-unmigrated effects. Its flow changes are restored before transport. Fetch no longer consumes the oracle-mutated PC/bank. The preview is calculated before transport because successor address is needed at b16..b27; withholding all flow computation until b55 would fetch the wrong word. This scheduling bridge is explicitly not evidence of a physical internal write edge. A pending delayed-ROM page overrides the operation's resulting page, after a JSB has saved its incremented return address.

## Evidence and remaining boundary

Semantic behavior follows the reviewed ACT contract and composed-machine regressions, including physical delayed-ROM startup landmark 0x0068 -> 0x0fc6. Real transport retains the proven fetch windows and one-word pipeline. Exact PC, return-stack, bank and delayed-ROM internal write timing remains SOURCE-BLOCKED. Completed-word authority is a WORKING APPROXIMATION. Other ACT prelude effects, peripheral electronics and true intra-word mutation remain outside M14I.

## Validation

All 1024 words are checked against the composed oracle over representative page/wrap PCs, both carry states, normal/implied-GOTO state, absent/low/high delayed ROM and both stack slots. Rejected oracle operations are not assigned semantics. Endpoint regression rejects completion before b55 and checks a fresh next-word image. Live boot regression checks committed successors and bank normalization. Existing card, arithmetic and 12-program release diagnostics remain required.

GitHub Actions is authorized only for cargo fmt. No compilation, tests, release build or deployment is executed there. Local formatting, warnings-denied all-target tests, native release build and the ignored release diagnostic must pass before owner approval and merge.
