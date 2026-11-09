# Whole-emulator audit — causal findings v1.1

2026-11-06. **Audit only. No source, tests, scripts, workflow or hardware behavior
changes are included in this report.** The owner's latest instruction supersedes
implementation sequencing: investigate now; corrections wait for the owner at the PC.

## Revision and confidence ledger

- Core/pending source inspected: `16d979c802a088851d27331876a74824728adef4`.
- Separate PAC source inspected: `79c9ce8997c9d1c6abb42044d8a098caae9be7ed`.
- Stable main remains the separately validated `2880b833` baseline. The changes
  after it have not acquired owner gate approval through this audit.
- Prior inventory: [roadmap v1.0](WHOLE_EMULATOR_AUDIT_ROADMAP_v1.0.md).
- Evidence below is source control-flow/data-flow analysis and inspection of
  existing regressions. **None of the proposed reproducers has been executed in
  Rust here.** “Confirmed” means the described source behavior is directly
  established, not that a Windows/GUI failure was observed.
- No new physical HP-67 measurement or timing interpretation is claimed.

This pass follows complete causal routes: host event -> pacing -> firmware input;
startup qualification -> timeout; write FIFO -> transport phase -> media; failed
preflight -> caller budget; output token -> fabric slot; owned CRC -> observer.
It does not claim exhaustive coverage of every instruction or every file.

## B01 — Early key input permanently prevents boot qualification

**Priority: high. Confirmed source-level defect. Both inspected branches.**

Evidence in core revision:
- `src/hp67.rs:71`: `BOOT_CYCLE_LIMIT = 2_000`.
- `Hp67LiveMachine::step_firmware_cycle_with_trace`: before execution, a non-Idle
  machine at that limit returns a hard boot error.
- The `idle_after_word` predicate requires `machine.act.state.key_buffer.is_none()`.
- `src/machines/hp67/keyboard.rs::press` stores a scan code. `sample_into_act`
  writes that code to `act.key_buffer` at each executing instruction boundary.
- `release` opens the contact but intentionally retains the code. Searches of the
  production ACT/serial code found initialization/reset and copying, not a normal
  instruction path clearing this retained field. Owned register reception carries
  the field through the external-input snapshot.

Causal chain: press before the first Idle checkpoint -> retained code becomes Some
-> releasing the contact does not remove the code -> the no-key predicate remains
false -> the 2,000-word supervisor aborts valid ongoing firmware execution. At the
nominal pacing reference this bound represents 640 ms of words, plus reset hold;
that arithmetic is not a measured GUI failure time. Pressing after Idle is different:
Idle remains latched, so this guard no longer qualifies the machine as booting.

Reproducer to implement later: create the production machine; advance through reset
hold and two words; press Digit1; advance enough for one executing word; release;
continue beyond the 2,000-word boundary. Control: identical elapsed time without a
key. Add a held-key case and a key after Idle. Record the exact error, phase,
key_buffer, S15, PC and cycle. Do not “fix” this by deleting the retained keyboard
code: that is intentional key-latch behavior. Separate startup health monitoring
from a no-input diagnostic checkpoint and justify the new completion criterion.

Coverage gap: the existing key dispatch/program-entry tests predominantly boot to
Idle before pressing. The pending duration partition regression also uses frozen
no-key inputs. Those passing would not exclude this defect.

## B02 — Newly sampled UI input is applied to the preceding frame interval

**Priority: high for temporal fidelity. Confirmed ordering; visible impact pending.**

Core `src/app.rs` calls `show_calculator_surface` before calculating `elapsed`
and calling `machine.advance(elapsed)` (call sites around 1514/1527, elapsed at
1530, advance at 1541). The surface handler applies selector/power/card actions and
`machine.set_key_contact(contact)` (1342). PAC update has the same surface-before-
advance ordering; its pacing loop is later in the frame.

For a late event in a long frame interval, the new input is therefore already
present while the machine simulates the earlier part of that interval. Example:
if 100 ms accumulated and a key closes near its end, hundreds of simulated words
can observe the new contact. Card insertion can similarly be observed during time
that accrued before insertion. This is distinct from fractional duration loss.

Reproducer specification: replay identical timestamped press/release/card events
under fine frames and a frame spanning an event. Compare the first observed contact
word, first motor transition, card record position and firmware dispatch. Use an
initially booted control so B01 does not hide B02. Exercise 16 ms, 100 ms and a
suspend/resume-sized interval. Assert input-event causality, not equal wall-clock
rendering. Timestamp and order events where available; define an explicit boundary
for untimestamped events. Merely capping dt silently loses simulated time and is not
an adequate correction.

The headless tests apply contacts between explicitly stepped words; they establish
that route, not the GUI event/pacing order. No GUI replay was run in this audit.

## B03 — Unlimited mode uses a firmware-observing pacing heuristic

**Priority: high architectural review. Confirmed working approximation, not a newly
observed calculator-result failure. PAC branch only.**

`advance_processing_batch` and the end of `step_firmware_cycle_with_reader_time`
in PAC `src/hp67.rs` maintain `last_empty_reader_poll: Option<(bank, PC)>` and
`reader_poll_wait`. Two empty buffer-ready tests at the same execution site cause
the host to require a full 320 us of reader budget before executing another word.
The gate clears when head/motor is inactive or the buffer becomes ready.

The implementation is not a hard-coded numeric-PC exception, but its scheduling
policy still observes instruction semantics and repeated ROM execution sites.
Once latched, the wait condition gates subsequent words, not only the flag-test
instruction. The justification is preservation of an instruction-counted timeout
while keeping mechanical time real. That is a compatibility policy, not an
independent physical CPU/reader clock model. A real full-speed CPU with an unchanged
mechanical reader can have different timeout behavior; this tradeoff must be named.

Existing test `unlimited_processing_advances_cpu_while_card_waits_for_physical_time`
is useful: it requires a complete 34-record read, no manufactured reader time,
processing bursts and comparison with the real-time program result. It does not
cover all write/empty/protected/error paths or interleaved work at a repeated test
site. The same-site key can also conflate temporally separated visits until the
explicit clear conditions run.

Closure tests: real firmware write as well as read, empty media, protection, buffer
error, motor stop/restart, timeout and switching speed while waiting. Audit the
interval between the second failed test and the next status change to determine
which useful instructions are rate-limited. Choose an explicit supported unlimited
policy and document its semantic consequences. Do not assert universal CPU-only
acceleration based on the current one-read-path regression.

## B04 — Write transport depends on how elapsed time is partitioned at FIFO exhaustion

**Priority: medium; confirmed algorithmic counterexample. Direct transport API.**

`src/machines/hp67/card_transport.rs::advance_write_us` resets `record_phase_units`
and returns when the write FIFO is empty at function entry. If it empties inside
the loop, the loop breaks and retains unused accumulated phase.

After normal startup acknowledgement, at 100% speed, writing permitted and motor
on, start two identical transports with records A and B queued:

| Time/input | One-call path | Split-call path |
|---|---|---|
| Advance 84,000 us total | One 84,000 us call writes A/B, retains 28,000 us phase | Three 28,000 us calls write A/B; the empty third call clears phase |
| Queue record C at 84,000 us | Same input time and payload | Same input time and payload |
| Advance 0 us | Retained phase permits C to be written immediately | C remains queued |

The computation uses the exact 28,000 us threshold and branch ordering in source;
it is not a Rust execution result. A future regression should construct two real
`Hp67CardTransport`/CRC pairs using the existing startup helper, queue the same words
at the same simulated timestamps and assert media, FIFO, position and phase parity.

Production currently advances normal transport in 320 us words. Therefore this
large-call counterexample is not evidence that ordinary card reads already fail.
It demonstrates a contract problem in a public elapsed-time adapter and a gap in
partition testing. The broader policy also stalls write position while the FIFO is
empty even if motor/head remain active. Decide and evidence underrun behavior
before claiming mechanical fidelity; do not hide it with a phase-reset patch.
Existing tests cover boundary writes and 105% cadence, not this exhaustion schedule.

## B05 — Preflight retry guarantees do not include the public time budget

**Priority: medium. Confirmed caller/callee contract mismatch.**

Current `Hp67LiveMachine::advance` subtracts a word from `pending_us` before calling
`step_firmware_cycle`. The word's PHI/SYNC/F2 preflight can then reject without
advancing electrical time. The already-subtracted pacing budget is not restored.

Existing `m16e_live_sync_fault_preserves_pending_ram_tail_and_retry_matches_unfaulted_firmware`
and the clock analogue call `step_firmware_cycle_with_execution` directly. They
correctly prove those low-level preflight retry cases. They do not prove retry via
`advance(Duration)` or preservation of its time debt.

Reproducer: boot two controls; inject a foreign SYNC source into one; call advance
with exactly one word; require preflight error and unchanged tick; release the
source; retry with zero elapsed time. The unaffected control executes the same
one-word elapsed input. Currently the rejected call has consumed the time budget.
Specify whether the public adapter is terminal after any error, or whether known
preflight failures retain the unexecuted budget. Do not indiscriminately refund
post-publication failures: those may already have mutated the machine.

## B06 — Dense ownership tokens are not bound to their originating fabric

**Priority: low/medium API integrity. Confirmed capability scope gap; no production
misuse identified.**

`Hp67DriverOwner` in `electrical.rs` contains only a driver enum. Claiming is
one-time per fabric, but staging accepts a token by reference and uses only that
enum. A token claimed from fabric A can be used against B; B can also issue its own
token for the same enum. The two tokens then address the same pending slot, where
the last proposal wins. Non-Clone/Non-Copy prevents duplicating a token value; it
does not establish provenance across fabrics.

Reproducer specification: two fabrics, claim ACT in both, stage contradictory
ACT/DATA values into B through the two tokens, commit; record that same-slot
replacement occurs rather than ownership rejection. Existing one-fabric ownership
regressions do not exercise this. Treat as a composition contract gap, not hostile
memory access and not proof of ordinary production contention. Consider lifetime/
composition-bound APIs before adding runtime maps or global identity state to
per-cell code. Public immediate-drive APIs are also intentionally available for
bridges/fault injection; they must not be mistaken for token-enforced capabilities.

## B07 — CRC transport differential shares the transport algorithm

**Priority: validation limitation. Confirmed call graph; not itself a code defect.**

`Hp67LiveMachine::advance_crc_transport` clones transport/reference CRC while active
and compares owned execution with `transport.advance_us`. Both routes converge on
`Hp67CardTransport::advance_crc_port`, then the same read/write advancement code.
They differ in CRC port/storage implementation and reader qualification, which is
valuable coverage. They cannot independently disprove a common transport-phase
error such as B04. Equality of the observer is not independent timing evidence.

Keep these checks. Add independent conservation/partition properties and external
expected timelines, with source provenance for physical claims. Label each oracle
by what it independently computes. The hardware gaps in PHI/sense remain separate.

## B08 — The new reset guard does not finish end-to-end media retention

**Priority: high integration gap; revisits A03/A04 with the new guard present.**

The pending core guard rejects an occupied transport before resetting. Main-based
`app.rs` still handles a power-on reset error by assigning `live_machine = None`.
Power-off recovery failure logs an error and still resets presentation; a later
power-on then reaches the guard. Dropping the machine on that error can still drop
its card. The API-level protection and its new unit test must not be described as
closing the GUI loss path. PAC `stop_on_fault` retains media, but that implementation
is separate and has not been integrated into this main-based branch.

Future fault test: inserted, partly written media -> injected reader contact fault
-> power off recovery fails -> power on -> reset rejects -> exact same media must
remain recoverable with its dirty data and orientation. Verify the UI state and
failure message as well as the API return. Preserve the physical object rather than
replacing it with a fresh blank. No code was changed in response during this audit.

## Review controls and decisions

- Not all masks/defaults/panics are defects. CRC 28-bit checks, inactive read/write
  branches, read-overflow error state and final-bit staging have explicit contracts.
- A display-only conservative blank fallback is not equivalent to synthesizing a
  calculator answer. Its scope must remain isolated from CPU/bus faults.
- The topology/reset/pacing fixes already published are still pending owner tests;
  no new source was added, and this report does not retroactively validate them.
- No time or speed percentage is inferred from old benchmark logs. Active transport
  observer cloning is a profiling candidate; removing it for speed without an
  equivalent validation strategy is not an audit conclusion.

## Concrete next validation queue when the owner returns

1. B01 early key startup, with before/after-Idle controls.
2. B08 GUI error/power/media retention, including partially written media.
3. B02 timestamped UI event replay under varied frame partitions.
4. B04 write-FIFO exhaustion with identical event timestamps and varied dt chunks.
5. B05 public pacing retry versus direct-word retry.
6. B03 read/write/error/speed-switch unlimited policy coverage.
7. B06 cross-fabric token provenance and B07 independent transport properties.

Run existing gates on the exact pending source head first; retain every failure.
This queue is a specification for later tests, not permission to add code now.
A complete audit still needs the remaining ACT field/authority ledger, instruction
edge-case campaign, corpus/tooling provenance checks, rendering/PDF checks and
measured performance review. This report advances the audit with explicit causal
findings; it does not close those unreviewed areas.

## Implementation follow-up — 2026-11-09
The owner has now explicitly requested implementation, superseding the audit-only
instruction above. See [fix status](AUDIT_FIXES_2026-11-09.md) for implemented
changes, remaining limitations and the exact pending validation boundary. The
historical findings above remain the pre-fix record.
