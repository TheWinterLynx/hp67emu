# M15F clocked DATA tail — 2026-10-14

## Problem and source

M15E resolved-net RAM transport still sampled the previous frame's b0/b1 tail
before any clock progression, then replayed the DATA pins when normal transport
visited those cells. This preserved logical phase and final results but did not
consume that tail during its physical-word coordinates.

Direct HP-67 Classic Notes pp67/69/74 fix serial bit 0 at b2 and bits 54/55 at
next b0/b1; HP-97 service manual 2-14/figure 2-5 corroborates the bidirectional
DATA lifetime under the owner's model-equivalence instruction. Sources:
https://literature.hpcalc.org/community/classic-notes.pdf and
https://www.teenix.org/97ServMan_2.pdf. No new exact PHI edge is claimed.

## Executable change

Live pending RAM DATA now completes in a shared b0/b1 prefix: ACT display source
on IS, ROM0 receive samples, resolved DATA sink, and eight real backplane PHI
transitions. Only after the prefix returns does live commit the completed payload
to C/RAM and bind the new instruction's input snapshot. The paired body continues
from b2 through b55, with the first two ROM0 samples preserved and no duplicate
pin/sink visits. Direction of a prior frame still survives a new read/write body.

ACT holds a one-use prefix display byte. Source A/B are frozen before the tail;
C/RAM completion cannot change them. The body uses that same byte rather than
mixing an old execution snapshot into b0/b1. IS ownership remains continuous.
Each full live word still has 56 electrical bit cells and 224 PHI transitions;
a traced word has exactly 224 continuous samples. Partial/manual prefix calls
leave the word at b2 and must be paired with the body, not treated as complete.
Invalid/mid-cell starts fail with tick context; the prefix requires the resolved
DATA level bridge. Existing default logical fixtures keep their old API.

## Remaining approximation

The newly bound serial execution accounts for coordinates b0/b1 as bookkeeping
before b2 body traversal. Its first digit checkpoint remains b3; final control
images remain completed-word authority. This does not settle internal ACT latch,
ALU or register-mutation edges. DATA Low/High logical encoding and Floating idle
remain the M15E abstract output-stage adapter. Exact within-cell launch/sample
edges, propagation and physical chip mapping remain unknown. This slice removes
the production unclocked DATA-tail scheduling/replay, not all timing approximations.

## Regression and gate

A library regression compares three split versus whole resolved-net words across
opposite DATA directions and an idle tail word. It requires equal words, ACT/data
state, all final nets, tick totals and every IS/PHI trace sample. Prefix must advance
exactly eight ticks and leave an incomplete ROM0 byte before the body finishes it.
Invalid/mid-cell starts are rejected. A native regression runs write/read/NOP through
the live authority bridge, proving the dependent read sees the completed write and
that the read destination commits only in its following prefix. Each step requires
224 transitions; the complete 672-sample trace requires monotonic ticks and bit
coordinates. Existing 12-program diagnostic and benchmark gates remain required.

The synthetic capture fixture now finishes a partially flushed word before its
capture starts; it still requires twelve complete words and traced/untraced parity.
No assertion is weakened to hide missing rows. Standalone authority tests may stop
at b2 after checking a completed tail; this is now actual partial-word progression.
Rust is not installed in the agent environment. Actions performs formatting only;
owner focused/full tests, release, diagnostic and benchmarks are pending.
