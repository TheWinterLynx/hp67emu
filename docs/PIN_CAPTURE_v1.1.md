# HP-67 pin captures v1.1

## Build modes

Normal builds compile without recorder storage or publication hooks. Offline
--audit-pins, --compare-pins and --timing-pins remain available. --trace-pins
fails explicitly without creating an output unless built with the pin-capture
feature (`cargo build --release --features pin-capture`). This is diagnostic
instrumentation selection, not a reduced-fidelity electrical mode. Both builds
retain every PHI transition, resolution, ownership and device operation.

The local gate tests both configurations, runs both diagnostics and benchmarks
the normal build. M15Z's original regressions run in the diagnostic configuration;
new default tests guarantee storage/calls are compiled out. No test is discarded.

## Native commands

- `hp67emu --trace-pins <1..512 words> <new.tsv>` captures real live firmware, with frozen no-key contacts and reset hold excluded.
- `hp67emu --audit-pins <input.tsv> [new-report.txt]` audits topology and intervals in the input domain.
- `hp67emu --compare-pins <left.tsv> <right.tsv> <left-word> <right-word> <1..512 words> <IS,SYNC,...> [new-report.txt]` compares explicitly selected pins at PHI1-rising checkpoints. Word indices are relative to each capture's declared anchor. No automatic alignment/cropping or time scaling occurs.

Reports default to stdout. All new output paths must not exist. Exit 0 means
resolved equality within the selected observation scope (or successful export/
audit), 2 means differing levels/owners, 3 means unresolved selected observations,
and 1 means invalid input/arguments/output. Audit success is not hardware proof.

## Strict normalized TSV

The first two lines are mandatory, followed immediately by the header:

```text
# hp67-pins-v1 domain=ordinal source=emulator first_bit=0
# provenance=live firmware fixture; ordinal observations
at	order	PHI1	PHI2	IS	DATA	SYNC	RCD	STR	F1	F2	KC1	KC2	KC3	KC4	KC5
```

Header separators above are tabs. `first_bit` is b0..b55 at the first PHI1
falling edge; it is an explicit supplied alignment anchor, not an inferred fact.
Provenance must be nonempty. `at` and `order` are unsigned decimal integers.

Ordinal model rows use `source=emulator`. Equal ticks preserve separate committed
publications through contiguous `order=0,1,...`. Whole IS/SYNC/DATA and STR/RCD
bundles each produce one observable image. All fourteen nets are exported, with
`level:high-mask:low-mask` fields. Level is `0`, `1`, `Z` (floating) or `X`
(contention). Masks use hexadecimal Hp67Driver indices from wiring.rs; IS's
passive low bias is preserved. Ownership-only changes are retained even when
the level does not change. Unchanged same-tick images are deduplicated.

Physical-time normalized recordings use `domain=ns source=measured` or
`domain=ns source=synthetic`. Integer timestamps must strictly increase and order
must be zero. A source label is a caller claim; the tool cannot authenticate a
capture. Record acquisition source, instrument, threshold, resolution and anchor
method in provenance. Simultaneous observed pin changes belong in one row; do
not fabricate sub-sample ordering. Driver masks are forbidden because digital
voltage captures do not identify internal driver contributions.

PHI1 and PHI2 columns are mandatory; other columns may be omitted. Plain fields
accept `0`, `1`, `Z`, `X`, or `?` for unobserved. Missing columns become unobserved,
not low. Raw vendor CSV must be normalized explicitly; the importer does not
assume channel names, units, thresholds, interpolation or bit alignment.
Input is capped at 64 MiB and 160,000 rows. Overflow blocks successful export.

## Audit and fidelity scope

The capture begins with both PHI pins high before PHI1 falls. The audit validates
alternating active-low PHI1/PHI2 pulses, forbids overlap/missing/ambiguous edges,
and summarizes PHI1 width, dead time, PHI2 width and dead time in the original
input units. It also reports STR/RCD/SYNC low intervals, STR/RCD cell positions,
coincident clock/pin transitions, partial trailing phase and decoded IS/DATA masks.
Clipped/unobserved low intervals are not assigned fabricated widths.

IS is decoded at b16..b27 and b46..b55; SYNC qualification disagreement remains
visible. DATA bit 0 is b2; next-word b0/b1 complete bits 54/55. Incomplete masks
remain explicit. The PHI1-rising sample checkpoint is the current provisional
model policy; agreement there does not prove a silicon sampling edge.

Comparison checks complete explicit 56-cell words, selected binary pin levels
and masks only where both recordings provide them. Missing/unresolved selected
levels prevent equality status. Physical durations are NOT compared, fitted or
installed into the emulator. The audit's measured units are input timestamps,
not an assertion of exact measurement error or propagation causality.

No new physical recording accompanies M15Z. Synthetic tests cannot close exact
PHI widths/launch/sample, DATA polarity/stage/chip mapping, display propagation,
reset or magnetic timing evidence gaps. Legacy --trace-is TSV is unchanged.

## Four-edge timing/window audit

`hp67emu --timing-pins <input.tsv> <IS,DATA,SYNC,...> [new-report.txt]` reports
observed setup and hold separations against PHI1 falling/rising and PHI2
falling/rising. It also reports changes strictly inside the PHI1-low interval,
changes at its boundaries and disagreements between phase observations. These
are evidence for reviewing candidate sample policies, not a chosen silicon edge.

Nearest pin events are used only when both sides are resolved binary values.
An initial/trailing constant signal or unresolved gap does not become a fake
zero or infinite margin: the bound is censored. A pin transition and clock edge
in one observation/publication cannot be ordered internally, so it is marked
coincident. Physical timestamp resolution/threshold accuracy remains part of
provenance, not an assumed exact measurement error. Ordinal same-tick events may
have distinct order and a zero ordinal separation; this is not zero physical time.

The report keeps source, units, anchor and incomplete trailing phase. Exit 3
marks unresolved or coincident selected observations; 0 means analyzed resolved
observations, not conformance to undocumented setup/hold requirements. Censored
bounds and phase-value disagreements remain visible even on exit 0. No profile
is automatically inferred or installed and no emulator edge policy changes.

M16G extends the named inventory with MOTOR while retaining the v1 TSV schema and legacy sparse import. See [PIN_CAPTURE_v1.2.md](PIN_CAPTURE_v1.2.md).
