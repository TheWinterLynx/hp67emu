# M15M — contiguous ROM address and instruction reception

Base/main: owner-validated M15L `16e76444012abb27b8ea2f81de2d518202451568`.
Owner's nine-stage gate passed, including diagnostic 12/12 with unchanged cycles.

## Problem and implementation

The five-ROM receivers previously used masks to check address/return completeness.
A mask records which coordinates appeared, but can also accept reordered or
repeated callbacks. Such a sequence must not select a response or synthesize a
bank instruction in a deterministic serial transport.

The shared address receiver now requires b16..b27 once, in order, before chip
selection. Each chip still reconstructs its own twelve-bit address image. The
autonomous IS/SYNC observer requires b46..b55 once, in order, and only its tenth
valid sample can decode bank-set. The ordered return count replaces its mask.
Wrong order, duplicate/replayed cells, invalid levels and unstable SYNC return
errors before receiver state mutation. Valid samples can resume at the same
expected coordinate after a rejected test stimulus. Production still stops on
such errors. A valid completed address that then fails chip selection/lookup
retains the former terminal-error semantics, not transactional lookup rollback.

`received_word()` exposes the independent complete ROM observation. Incomplete
returns report the contiguous-prefix mask; provider-view fixtures, which do not
observe returns, remain incomplete. Cycle start clears receive state but keeps
bank latches. Power reset reconstructs the complete set as before.

## Evidence boundary

Existing HP-67 source windows remain unchanged: address LSB-first at b16..b27,
return at b46..b55. Sources and direct captures are pinned in M15J/M15K notes:
https://literature.hpcalc.org/community/classic-notes.pdf, printed pp67–74,
and HP-97 service manual figure 2-5 under owner-authorized equivalence.
This milestone enforces emulator callback order at those established coordinates.
It does not claim that silicon detects coordinate skips, rejects analog glitches
or has newly measured PHI latch/launch timing. M15H's provisional receive edge,
physical propagation, DATA output stage and RAM-chip mapping remain unresolved.

## Regression contract

Three library tests cover invalid coordinates after every valid prefix, address
reference/autonomous modes, complete-frame replay, invalid IS/SYNC at each
coordinate, identical full state on rejection, unchanged source-read count and
successful valid suffix completion. The M15K missing-b46 fixture is strengthened
from incomplete-mask/no-bank-set to immediate b47 order error plus unchanged
state/no-bank-set. No existing correctness condition is removed to hide a failure.

A native regression runs 1200 real firmware cycles in each traced/untraced mode,
comparing ROM-observed words against ACT, every chip's address, 224 transitions
per word, exact trace length, idle arrival and clearing on power reset. Existing
all-opcode SYNC tests, exhaustive ROM map, all four transports, clocked DATA tail,
RAM ownership tests and diagnostic 12/12 remain in the full gate.

## Performance and validation status

Two fixed counters guard shared observations; there are no extra clocks, net
commits, firmware copies, contributor scans or heap allocations. Per-chip source
selection and named IS ownership remain unchanged. Benchmarks measure the added
checks through the existing current and live paths; no speed threshold is proof
of correctness. Rust is absent here: static diff and companion-document checks
run in the editor, Actions performs formatting only, and the owner runs focused
and full warnings-as-errors tests, release, diagnostic and all three benchmarks.
M15M runtime gate remains pending.
