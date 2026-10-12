# M14Z structural IS word decode

M14Y owner FULL GATE GREEN, integrated at 4d673754661a5c20f3a7788d5af1a19de075ac8c. Last owner live idle benchmark 0.978 us/word (327.28x nominal reference). This remains computational headroom, not timing proof.

## Contract
Native CLI: `hp67emu --decode-is input.tsv report.txt [start_row words]`.
The optional zero-based row start and word count select a disclosed range. Default decodes all rows only when their count is a multiple of 224. The importer keeps M14Y input/provenance limits and strict format.

Each complete word must have 56 groups of four observations, consecutive ordinal ticks, b0..b55 and the existing four-transition active-low/non-overlapping PHI scaffold. IS must be binary and stable across each group. The decoder rejects partial, shifted, malformed, unresolved or incompatible observations; it does not repair, resample or guess their bit order. Report output never overwrites input.

The existing timing helpers supply display b0..b7, address b16..b27 and ROM return b46..b55. LSB-first reconstruction reports hexadecimal/octal values, source comments, first row/tick and excluded prefix/suffix counts. Address is the 12-bit bus value; bank is not recorded. Returned instruction is fetched for the pipeline, not necessarily the instruction executing concurrently.

## Evidence limits
No new physical timing claim. This is the native ordinal observation profile, not a generic raw analyser decoder or proof that physical IS remains constant throughout a calibrated cell. Existing HARDWARE_SOURCES and M14A/W/X/Y boundaries remain authoritative. PHI widths/dead time, physical launch/sample edges, DATA electronics/chip mapping and STR/RCD propagation remain SOURCE-BLOCKED.

No physical recordings are included. Any externally normalized input must document source and normalization before its decode can support hardware contrast.

## Validation
Three focused regressions decode the actual resolved structural transport at two nontrivial ROM addresses/words; preserve display blank byte; verify explicit subset/provenance scope; reject shifted, partial, overflow, malformed PHI/coordinate/ticks and unresolved IS; and demonstrate that a stable changed bus bit changes the decoded address rather than being hidden.

Owner native gate exports two real live firmware words and requires a two-word decode report. Existing full tests, release build, diagnostic and benchmarks remain owner-local. Actions only fmt/check/save. Production fetch, DATA and live execution remain unchanged.
