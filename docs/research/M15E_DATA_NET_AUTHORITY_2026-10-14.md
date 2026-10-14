# M15E DATA net authority — 2026-10-14

## Scope and evidence

Owner instruction: always deliver executable fidelity progress, and assume HP-97
equivalence. Start from validated M15D aa7241c. HP-97 service manual sections
2-14 and figure 2-5 describe ACT/ROM bidirectional DATA transfer; direct HP-67
Classic Notes pp67/69/74 establish 56 bits LSB first starting at b2, with bits
54/55 at next-word b0/b1. Sources:
https://www.teenix.org/97ServMan_2.pdf and
https://literature.hpcalc.org/community/classic-notes.pdf.

## Executable advance

Installed-RAM live transfers now bind source direction and drive explicit named
DATA contributions. The receiver samples resolved DATA, not a direct source bool,
to construct the eventual ACT/RAM payload. ACT is the write source; the existing
aggregate structural ROM/RAM responder is the read source. Direction for an old
frame's tail survives a new frame of the opposite direction. The two DATA loops
share this phase-engine visitor; the dedicated no-DATA loops remain unchanged.

Two source contributions commit as one observable image. No evaluator observes
an intermediate source handoff, and commit does not advance time. Idle releases
both sources in HighZ against Floating bias. Extra committed contributions fail
even if they agree electrically. Required samples must be resolved Low/High;
conflicting or floating active samples fail before sink mutation. Destination
commit retains the existing frozen-payload comparison; an early-tail fault
preserves pending transfer and destination, preventing the next execution.

## Explicit approximation

Low/High are **logical-level adapter values**, not a researched claim that the
real DATA output transistors push/pull both states or use a specific voltage
polarity. No passive DATA bias is invented. HP-97 equivalence does not prove
unknown electrical output circuitry. The adapter is named `electrical_level_bridge`
and defaults off in logical reference fixtures. Live opt-in makes resolved-net
causality testable while physical output stage and chip mapping stay unresolved.
The existing early b0/b1 tail is still sampled before advancing the clock; later
structural traversal replays pins and avoids double sink commit. That scheduling
split remains approximate, not a physical PHI sampling claim. All ordinary
transport words still traverse 56 cells and 224 PHI transitions. CRC payloads,
magnetic serialization, display nets and exact edges remain outside this slice.

Replacement criterion: source-backed output stage/bias and named PHI launch/
sample edges, then remove early-tail scheduling while preserving net authority,
ownership failures and end-to-end firmware results. Never infer those facts from
this bridge's passing diagnostic.

## Tests and performance

Library tests compare net versus logical frames through read/write handoff,
check early-tail replay without duplicate completion, reject missing owner,
reject same-level overlap and opposing-drive contention before sink mutation,
and verify one-image source handoff without tick advancement. A native test
injects a DATA tail fault and requires unchanged RAM/pending transfer. Existing
real firmware boot and traced/RAM authority regressions remain in the full gate.

M14B benchmark fixture is unchanged; its scope now says it excludes M15E. The
actual live benchmark measures this new path, including synthetic RAM stress.
Formatting only runs in Actions; compilation, tests, release, diagnostic and
benchmarks require the owner local gate. No unrun result is claimed.
