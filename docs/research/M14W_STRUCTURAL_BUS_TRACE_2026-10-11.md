# M14W structural IS/PHI observation contract

M14V was owner-validated and integrated at 5dfdbffbd301eba8a923d5883e356258b5382ff0. No new hardware behavior or physical timing is claimed by this slice.

## Scope

The opt-in library API executes the resolved-net structural display/fetch word and records each committed PHI transition. Columns are ordinal tick, transport bit b0..b55, PHI1, PHI2, resolved IS, ACT drive and aggregate ROM responder drive. TSV enum labels preserve High/Low/Floating/Contention and HighZ rather than coercing uncertainty into binary zero. Trace appends across words; callers own storage/reset. Failed words retain only already committed transitions, without padding a word or inventing its missing response.

Each successful word produces 224 observations. The last transition of b55 advances the backplane into the next word, but the observation explicitly retains the processed transport bit 55. The PHI rows describe the existing four-transition scaffold. They do not locate physical receiver launch/sample edges, despite showing the structural IS value carried during those scaffold phases.

## Evidence and limits

Existing evidence is unchanged: docs/HARDWARE_SOURCES.md and research/M14A_TIMING_EVIDENCE_LOCK_2026-09-21.md document the Classic Notes HP-67 captures. Address b16..b27, ROM response b46..b55, display b0..b7, wired-high IS and alternating active-low PHI topology retain their existing source classification. Exact PHI widths/dead time and receiver edges remain SOURCE-BLOCKED. Ordinal ticks must not be scaled into physical edge timing. No physical DATA, SYNC, STR/RCD, propagation, magnetic sense or per-chip ROM selection is exported. Aggregate ROM driver ownership is a structural abstraction, not proof of a specific physical responder.

This is not a capture of the desktop restore/oracle/authority-commit bridge; it is an explicit structural diagnostic API, capable of advancing a caller-bound ACT serial instruction. No calibrated waveform or live firmware capture is claimed.

## Performance boundary and validation

The existing no-DATA and DATA production transport bodies are unchanged. The diagnostic twin is deliberately separate rather than generalizing the historical fast loop; trace allocation and export occur only when this API is explicitly called. Its duplication requires continued differential coverage when transport semantics change.

Focused tests compare fifteen slots to the established transport with bound serial DISPLAY-toggle execution; reconstruct all display bytes, addresses and returned words from captured resolved IS; check ownership release windows, non-overlapping PHI, 224 transitions, word wrapping and completed serial mode image parity. A missing ROM address must stop before b27 clocks, leaving exactly 108 committed samples. Normal and ignored owner gates remain required; only rustfmt/check/save runs in Actions.

## Next comparison work

Connect this observation contract to bounded live authority capture without changing production scheduling, then accept explicit measured-recording input with alignment and mismatch reports. Physical waveform comparison must preserve sample provenance and distinguish bit/order mismatch from unknown physical edge timing. No additional Pac acceptance expansion is planned.
