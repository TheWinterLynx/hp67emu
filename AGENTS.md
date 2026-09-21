# AGENTS.md — hp67emu engineering rules

This file defines how coding agents must work in this repository. The project is not a generic calculator app: the target is a source-backed, cycle-accurate, pin-level digital-electrical emulation of the HP-67 executing real HP microcode, with the photographed front panel driven by the emulated machine.

If an explicit instruction from the repository owner for the current task conflicts with this file, follow the owner's instruction. Otherwise these rules are binding.

## Mission

The fidelity path is:

    physical control / card / power event
        -> hardware-facing or electrical state
        -> ACT / ROM / RAM / CRC / display / keyboard behavior
        -> real firmware execution
        -> physical result
        -> presentation layer

Do not reverse that dependency. UI code, host adapters, tests, or file formats must not perform calculator semantics that the original firmware/hardware should perform.

Core rule:

> If the real HP-67 firmware can decide it, host code must not decide it for the firmware.

Key meaning, prefixes, arithmetic, RUN/PRGM behavior, stored-program execution, Crd prompts, card motor control, read/write flow and display contents belong to the emulated machine.

## Priorities

In order:

1. Fidelity to HP-67 evidence.
2. Deterministic and testable timing/state.
3. Preservation of the real-firmware path.
4. Clear ownership and architecture boundaries.
5. Regression coverage and documentation in the same change.
6. Performance, convenience and abstraction only after the above are protected.

Never trade fidelity for shorter code, prettier APIs, fewer cycles, easier UI logic or speculative generalization.

## Evidence policy

Before changing hardware-visible semantics or timing, read:

- docs/HARDWARE_SOURCES.md
- docs/ARCHITECTURE.md
- the relevant docs/files companion
- relevant docs/research notes
- docs/ROADMAP.md and TODO.md for open evidence boundaries

Use the project source hierarchy:

- Tier A: direct HP-67 schematic, board/chip evidence, physical captures and HP-67 logic-analyser traces.
- Tier B: close HP-family/service evidence such as HP-97 and documented Woodstock measurements; corroboration only where equivalence is justified.
- Tier C: Nonpareil, x11-calc and other semantic/reverse-engineering references; useful for behavioral cross-checks and tests, not as sole proof of HP-67 pin timing.

Rules:

- A plausible value is not evidence.
- HP-97 behavior is not automatically HP-67 behavior.
- Nonpareil/x11-calc semantics are not automatically electrical timing.
- A software emulator is never the sole authority for pin timing.
- If sources disagree, record the disagreement and investigate it.
- If evidence is insufficient, leave an explicit TODO, unsupported state, source-blocked path, or carefully scoped presentation fallback. Do not invent a hardware rule.
- Do not copy GPL-covered implementations line-for-line. Reimplement independently from documented behavior/tests unless the owner explicitly changes licensing policy.

## Locked timing and bus invariants

Do not casually change source-backed facts:

- one HP-67 machine word is 56 serial bit coordinates, b0..b55;
- 14 digit times x 4 bits = 56 bits;
- display, fetch and execution traffic overlap structurally within the machine word;
- ACT sends the 12-bit ROM address LSB-first at b16..b27;
- the selected ROM returns its 10-bit word LSB-first at b46..b55;
- normal fetch uses the corresponding SYNC window; the documented post-IF implied-GOTO word has the required SYNC distinction;
- IS is weak/passively low and active participants pull high/release rather than actively drive zero;
- the word fetched in cycle N executes during cycle N+1;
- observed whole-machine timing currently includes about 320 us per word, about 4.8 ms per fifteen STR display slots, signal stabilization beginning around 330 us after power-on, and valid SYNC/fetch activity around 35 ms after power-on.

Those whole-interval measurements do not prove exact PHI1/PHI2 pulse width, dead time, launch/sample edges, RCD/STR overlap, DATA ownership or propagation delay. Keep that distinction explicit.

## Deterministic electrical scheduling

Preserve the scheduler model:

1. resolve nets from committed drives;
2. snapshot the same resolved inputs for all devices;
3. evaluate devices for the current tick;
4. collect proposed drives;
5. commit all drives together;
6. advance time.

No device may observe another merely because of Rust container iteration order.

Core behavior uses deterministic integer simulation time. Wall-clock time belongs only at explicit pacing/presentation boundaries. Machine correctness must not depend on host frame rate, host scheduling jitter or rendering cadence.

Shared electrical nets require explicit High, Low and High-Z behavior, evidenced passive bias, named drivers and contention detection. Never implement last-writer-wins buses. Floating or contended samples on a path that requires a valid logic level are hard failures unless documented hardware behavior says otherwise.

## ACT and structural execution

The architectural/reference ACT state is a verified instruction-boundary oracle, not permission to collapse production fidelity back to instruction-level execution.

When changing ACT execution:

- preserve the complete b0..b55 lifetime of the executing word;
- preserve the one-word fetch/execution pipeline;
- move effects from architectural fallback into intra-word behavior only when timing is source-backed;
- keep field selection, A/B/C routing, carry/borrow propagation, pointer/status behavior and writes explicit;
- never replace an incomplete executing word;
- fail unsupported architectural behavior with useful PC/word/tick/state context.

Instruction-level shortcuts may exist for reference/debug only. They are never the source of truth for fidelity mode.

## Display

The production display is a hardware result, not formatted text.

Required direction:

    ACT/ROM0 structural or electrical traffic
        -> ROM0 decode / STR / RCD / cathode-anode behavior
        -> raw physical segment state / integrated intensity
        -> UI optics

Do not format a host number/string into the fidelity display, synthesize 0.00 in the UI, or let downstream presentation decide calculator semantics.

Important M14 rule: a display-only uncertainty must not corrupt unrelated CPU/ROM transport. If an unknown transient display modifier affects only b0..b7 while the same 56-bit word still has valid fetch/execution traffic, the transport may continue with an explicitly documented conservative visual fallback. That fallback is uncertainty handling, not a claim about the real encoding.

Do not generalize this into ignoring unknown hardware. Unknown architectural or bus semantics capable of changing machine state remain hard failures.

## Keyboard, switches and power

UI controls represent physical controls.

- A pressed key closes one HP-67 key contact.
- The captured contact remains asserted until release; pointer drift must not silently switch contacts.
- Prefix/function meaning is firmware-owned.
- RUN/W-PRGM is a mechanical selector entering through its hardware-visible path, not a host program-mode semantic.
- OFF/ON controls machine power/reset, not a cosmetic flag.
- Power cycling resets electronic/transient machine state according to evidence while preserving physical media/mechanical state that the real calculator would preserve.

Prefer end-to-end regressions: physical contact -> firmware dispatch -> raw display/state.

## Magnetic cards

Do not collapse the card layers.

### Physical object

One Hp67MagneticCard is one physical card with two independent end-for-end logical tracks:

- Track 1 is selected by one insertion end.
- Track 2 is selected by the opposite end.
- Each track has independent recorded/unrecorded and write-protect state.
- The same physical card object must survive eject, 180-degree rotation and reinsertion.

Logical Track 1/Track 2 are not the two lower physical Zero/One flux tracks used to encode one logical stream.

### Firmware authority

Firmware owns:

- card-present reaction;
- motor-on;
- write/read mode;
- buffer-ready handshake;
- CRC read/write operations;
- Crd continuation requests;
- program/data header interpretation.

The host must not manufacture a second-pass request or bypass firmware because it knows a card has two tracks.

### Transport

Current source-backed transport rules include:

- exactly 34 CRC-visible 28-bit record positions per recorded logical track;
- nominal 28 ms record cadence;
- documented 95–105% reader-speed range;
- phase accumulation must avoid per-record rounding drift;
- CRC double buffering may retain two 28-bit records;
- head/startup readiness is distinct from motor-on;
- unrecorded media crosses the head without inventing zero data;
- writing an unrecorded track materializes recorded media only through the write path;
- write protection affects only the selected logical track;
- the same physical card returns after a pass.

Do not guess insertion acceleration, exact head geometry, intra-record 28-bit serialization order, absolute flux polarity or PHI-relative sense-amplifier timing unless new evidence closes those gaps.

### Host file formats

Host formats are adapters, not hardware:

- .hp67card is the native lossless physical-card container.
- .hp67raw is a logical two-recorded-track interchange format.
- .hpp is Teenix compatibility.
- artwork/title/filename metadata is presentation.

CRC/card electronics must not know or branch on these host formats. Packing used by a host file is not proof of flux order on a real magnetic card.

## UI and presentation boundaries

Keep the reusable emulation library GUI-free. src/emulation and machine/chip code must not depend on egui/eframe/image rendering.

The photographed UI may:

- animate mechanical keys/sliders/cards;
- map pointer interaction to physical controls;
- render raw physical display output;
- show program-card artwork;
- host independent desktop utility viewports;
- import/export host media.

It must not calculate calculator answers, inject display strings, choose firmware branches or special-case application programs.

Artwork and magnetic media are separate identities. A picture of a Standard/Games Pac card must never imply that magnetic bytes exist.

Presentation changes require direct visual comparison with the reference image/asset and measured geometry where available. Do not settle for a merely plausible look when a source image exists.

## Code structure and abstraction

Respect the current layering:

- src/emulation: generic deterministic electrical primitives/scheduling;
- src/machines/hp67: HP-67 composition and hardware-facing models;
- src/reference: semantic/reference models and differential oracle;
- src/hp67.rs: desktop-facing live machine adapter;
- src/app.rs, src/panel.rs, src/ui: presentation and host interaction;
- tools: research/conversion tooling;
- tests and src/hp67/*_tests.rs: regressions.

Rules:

- one layer owns each responsibility;
- do not leak UI or file-format concepts into electrical models;
- do not create a generic Woodstock abstraction until a second real calculator proves the common behavior;
- extract after two real users, not before;
- prefer explicit state and invariants over convenience magic;
- do not mix unrelated cleanup/refactors with fidelity fixes.

## Rust coding rules

- Keep warnings at zero under -Dwarnings.
- Keep code rustfmt-clean.
- Use explicit types where inference creates warning/future-compatibility ambiguity.
- Prefer deterministic integer arithmetic for simulation timing and phase accumulation.
- Avoid hidden global state.
- Avoid host-time dependencies in the core.
- Errors at fidelity boundaries must carry useful context.
- Comments explain evidence/invariants/why, not line-by-line what.
- Do not weaken strict parsers/resolvers merely to accept bad input unless the format specification/evidence justifies it.
- Test helpers must not become alternative semantic implementations of the calculator.

## Documentation is part of the implementation

Every Rust file under src/ and tests/ requires a companion Markdown file under docs/files/ using the complete repository-relative source path plus .md.

Each companion must contain these exact sections:

- ## Purpose
- ## Why it exists
- ## Relationships
- ## Responsibilities
- ## Implementation

If behavior or ownership changes materially, update source and companion documentation in the same branch/commit series.

If executable non-Rust source types are introduced, extend the documentation regression in the same change so the new source type is covered.

When adding/changing a hardware claim, update the appropriate evidence/research document. Documentation must not overstate certainty.

## Testing philosophy

Prefer tests that prove causality through the real path, for example:

- serial bus windows and bit order;
- instruction-boundary differential against the reference model;
- complete b0..b55 execution lifetime;
- contention/driver ownership;
- real-firmware power-on checkpoints;
- physical keyboard -> firmware -> raw display;
- PROGRAM entry/edit/run through firmware;
- physical card insertion -> firmware motor/head/CRC -> same-card return;
- two-pass Crd flow driven by firmware;
- write -> eject -> reinsert -> read;
- raw seven-segment results for diagnostic cards.

Do not weaken a failing regression simply to make a branch green. Decide whether implementation regressed, the test encoded an unsupported assumption, or new evidence changes the expectation. Document any justified expectation change.

The Custom Diagnostic Pac suite is intentionally ignored by ordinary cargo test because it is expensive. Run it explicitly for changes that can affect firmware execution, ACT semantics, keyboard/program execution, display transport, CRC/cards, timing/pacing or program media.

## Corpus and source discipline

For new ROM/card data:

- record origin and extraction/conversion method;
- record hashes where required;
- distinguish physical dumps from software-derived corpora;
- compare independent corpora when possible;
- do not silently redistribute copyrighted firmware/media without an explicit project policy decision.

The project uses independent Teenix, x11-calc and Nonpareil-derived comparisons. Preserve that approach: agreement is evidence, not a reason to merge their implementations.

Official HP application manuals and checked-in Standard Pac/Games Pac/diagnostic media are behavioral references for card loading, two-pass use, pauses/output and diagnostics. Reproduce those behaviors through the real calculator path; never special-case a named application to make an example pass.

## Development workflow

main is the stable integration branch.

For every task:

1. Start from current merged main.
2. Create one focused branch, normally agent/<topic> unless the owner specifies another.
3. Keep commits small, scoped and reviewable.
4. Do not stack a new long-lived feature branch on an old unmerged branch unless explicitly instructed.
5. Do not mix unrelated cleanup/reformatting/refactoring into a fidelity change.
6. Update source, tests and companion docs together.
7. Inspect the diff before asking for validation.

GitHub rules:

- Do not create or run GitHub Actions unless explicitly authorized.
- Do not add .github/workflows automation as a convenience.
- Do not open a PR unless explicitly requested.
- Normal flow is branch -> owner local validation -> explicit approval -> merge.
- Do not merge/promote to main before owner validation and approval.
- With repository write access, make requested changes directly in the branch rather than giving a manual patch unless requested.
- If write access is unavailable, say so; never pretend repository changes happened.

Failure rules:

- stop on the first hard gate failure;
- formatting failure blocks later tests/builds;
- compiler warning under -Dwarnings is a failure;
- failing regression is not close enough;
- fix branch-owned failures before asking the owner to retry;
- never hide a failure by deleting/ignoring/weakening a test without evidence and documentation.

## Local validation gate

Commands given to the repository owner must be PowerShell and one line.

Minimum gate:

    cargo fmt --all -- --check; if ($LASTEXITCODE -ne 0) { throw "FMT FAILED" }; $env:RUSTFLAGS='-Dwarnings'; cargo test --locked --all-targets; if ($LASTEXITCODE -ne 0) { throw "TESTS FAILED" }; cargo build --locked --release --bins; if ($LASTEXITCODE -ne 0) { throw "RELEASE BUILD FAILED" }

For changes that may affect end-to-end calculator behavior, also run:

    cargo test --release --locked --bin hp67emu live_custom_diagnostic_pac_suite_reports_ok_ko -- --ignored --nocapture

A clean diagnostic run ends with:

    DIAGNOSTIC SUITE OK: 12/12 passed

When giving a checkout/update command, keep fail-fast order: fetch/switch/update exact branch -> fmt -> focused tests -> full tests with -Dwarnings -> release build -> relevant ignored smoke/diagnostics.

Do not claim local tests passed unless they actually ran. If the agent cannot run the owner's Windows/Rust environment, say what was reviewed and give the exact one-line command.

## Commit and merge discipline

Before commit:

- inspect the diff;
- only intended files changed;
- source/docs/tests agree;
- no generated/temp/downloaded files leaked;
- no unrelated formatting churn;
- no invented hardware claim was introduced.

Commit messages describe the behavior or invariant, not merely "fix tests".

Before merge:

- branch has intended main lineage;
- local gates are green;
- relevant long-running diagnostics are green;
- owner explicitly approves.

After merge, the next task starts from the new main.

## Never do this

- synthesize calculator results in host/UI code;
- bypass firmware to make a sample program pass;
- replace cycle/word timing with one-instruction-per-frame behavior;
- use wall-clock timing in the deterministic electrical core;
- guess PHI edges, DATA ownership, RCD/STR ordering, card bit order, flux polarity or source-blocked details;
- silently coerce unknown machine states to zero/blank/default;
- confuse presentation fallback with hardware truth;
- let display-only uncertainty abort valid independent CPU/ROM execution when transport can continue;
- add GUI dependencies to the reusable core;
- make .hpp/.hp67raw/.hp67card/artwork/filenames part of CRC/electrical semantics;
- special-case Standard Pac/Games Pac program names in calculator core;
- generalize HP-67 behavior into a shared Woodstock layer before a second machine proves it;
- weaken regressions just to get green;
- run/create GitHub Actions without authorization;
- merge to main without owner validation and approval.

## Definition of done

A change is complete only when all applicable conditions are true:

- real hardware/firmware path preserved;
- every new hardware claim source-backed at the right evidence tier;
- uncertainty remains explicit;
- architecture boundaries intact;
- source and companion docs agree;
- focused regressions cover the new invariant;
- end-to-end real-firmware coverage exists when firmware-visible;
- cargo fmt --all -- --check is clean;
- RUSTFLAGS=-Dwarnings cargo test --locked --all-targets is clean;
- release build is clean;
- relevant ignored diagnostic/smoke suites are clean;
- diff is scoped/reviewable;
- no unauthorized GitHub Actions used;
- owner validated branch before merge.

## First files to read

For any non-trivial task, read in this order:

1. AGENTS.md
2. README.md
3. docs/ARCHITECTURE.md
4. docs/HARDWARE_SOURCES.md
5. docs/DOCUMENTATION_POLICY.md
6. docs/ROADMAP.md
7. TODO.md
8. relevant docs/files/<source>.md
9. relevant docs/research/*.md
10. relevant regressions

For cards/media also read docs/HP67_CARD_FORMATS.md.
For ROM/corpus work also read docs/MICROCODE_PROVENANCE.md and docs/ROM_CORPUS_WORKFLOW.md.

> Preserve what is proven, expose what is unknown, and make every new fidelity claim testable.
