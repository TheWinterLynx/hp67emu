# M15Q — autonomous aggregate RAM in the actual live composition

Base/main: owner-validated M15P 627abbc3cc9aa03c46b3c05001f932f646fed3fd,
integrated exactly after FULL GATE GREEN. Owner diagnostic 12/12, unchanged
cycles; live idle 1.852 us/word (172.83x), RAM synthetic 1.969 us, CRC 1.850 us.
Separate RAM fixture 1.006 us. Those are M15P results, not M15Q measurements.

## Executable ownership change

Default desktop live owns an independent 64-register Hp67SerialRam. It receives
only resolved IS/SYNC commands and DATA address/write frames. The shared transport
begins its command cycle at b0 and binds the body at b2 after the preceding
frame completes. RAM's immutable read proposal joins the existing atomic serial
launch. Oracle read bits no longer drive DATA: the phase bridge receives only
a zero-valued frame mask to retain receive lifetime/ownership checks. Its sink
still reads actual pins. ACT C authority completes after the next clocked b1.

Address reception is checked against the frozen C frame. Writes first commit
independent device RAM through observed DATA; the instruction-boundary reference
RAM mirror is then updated from checked device storage. Read C, selector and
16-word clear retain independent oracle comparisons. Clear checks touch only the
relevant block; production never clones or scans all 64 registers every word.

The shared receive helper prepares RAM inputs without mutation, lets DATA validate
and consume its bit, then commits the small validated RAM image. Failure in either
receiver cannot consume just the other. No RAM array clone or per-bit allocation.
Existing standalone RAM sample API uses the same prepare/commit methods.

## Reference fixtures and benchmark comparability

Old transport API signatures wrap the same loops with None(RAM). New with_ram
variants supply the device. No duplicate bit-cell loops or transitions are added.
The no-RAM live constructor is cfg(test) only, named power_on_reference_fixture.
Existing opcode-injection, direct ACT/RAM mutation and manually constructed DATA
fixtures use it explicitly: they intentionally execute a different command from
the actual concurrent ROM bus. They cannot correctly operate an autonomous RAM
without a consistent ROM stream. No production auto-fallback exists. Real boot,
keyboard, program/card roundtrips and the diagnostic suite keep the default
autonomous composition. The new library fixture sends a consistent actual ROM
command stream through the shared transport instead of injecting RAM opcodes.

The live benchmark's real-firmware idle row now includes autonomous RAM. Its two
host-injected RAM/CRC stress rows explicitly use and label the reference bridge;
they are no longer labelled full production live. Their clocks, oracle checks,
DATA levels and M15P command observer remain active. The separate M15N fixture
continues to measure autonomous RAM load. No correctness speed threshold.

## Evidence and scope

Retains M15N's direct HP-67 Teenix capture evidence: Notes on HP's Classic
Calculators (2025), printed pp66–69 and74. Captured 270/2f0/038 commands, next-word
execution and DATA b2 through following b1 are preserved.
https://literature.hpcalc.org/community/classic-notes.pdf
SHA256 bdc7a403e22b03f4305d2d7b638ff4358e49e2d5ae5328f590f6946f243c66da.
HP-97 corroboration follows the owner's equivalence instruction.

This is an aggregate logical 64-register device on resolved buses, not a proven
physical RAM-chip partition/output circuit. High/Low DATA remains a logical-level
adapter; passive bias/polarity, per-IC mapping, precise PHI widths/edges/delays
and analog propagation remain SOURCE-BLOCKED. Address full-frame completion,
b2 read binding and b55 clear remain WORKING APPROXIMATIONS. ACT's M14H address
scalar still commits at completed-word authority; peripheral completion is next
b1. CRC data/sense circuitry and absent-read compatibility are unchanged.

## Verification

New shared-transport tests send address/write/read/clear via actual IS, require
next-b1 dependencies and 224 transitions in traced/untraced words, and use wrong
zero/all-one reference read payloads while requiring the actual stored response.
Shared receive fault tests require RAM and DATA state unchanged on either error.
Native tests compare 2000 real-firmware words against the reference in each
transport, including all RAM slots, full ACT/CRC state, trace counts and reset.
An oracle-memory corruption test must receive the original independent RAM value
and reject authority at the next b1 without overwriting C. Existing M15N, full
all-targets, release, diagnostic and benchmarks remain required.

Rust is absent here. Static source/diff/document checks and formatting-only
Actions are reviewed; compilation, all runtime tests, release, diagnostic and
measurements require the owner's local gate. M15Q gate pending.
