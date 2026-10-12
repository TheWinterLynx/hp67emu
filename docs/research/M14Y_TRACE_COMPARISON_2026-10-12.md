# M14Y ordinal trace comparison

M14X owner FULL GATE GREEN, integrated at a4c8ffa8d7e82b982bdabeda6757285e639fd47c. Live firmware idle benchmark 0.858 us/word (372.86x nominal hardware headroom). Throughput is not evidence of calibrated timing.

## Input contract
Native seven-column TSV: tick, transport_bit, phi1, phi2, isa, act_drive, rom_drive.
Normalized pin-only five-column TSV: tick, transport_bit, phi1, phi2, isa.
Fields are tab separated; LogicLevel names High/Low/Floating/Contention and driver names High/Low/HighZ are exact. Comments beginning # preserve source/provenance. Empty traces, malformed headers/columns, invalid values, bits beyond b55 and non-increasing ordinal ticks are rejected. Input limits: 64 MiB and 4096*224 rows.

Pin-only input supports independently normalized recordings without inventing driver ownership. It is not a raw analyser CSV importer. The producer must explicitly document source, normalization, observed sampling points and chosen transport-bit coordinates. Missing provenance is reported UNSPECIFIED, not promoted to physical evidence.

## Comparison and limits
Native CLI: `hp67emu --compare-is expected.tsv actual.tsv report.txt [expected_start actual_start]`.
The two explicit zero-based start indices select suffixes. No automatic best-fit alignment, polarity inversion, resampling or discarded trailing rows occurs. Absolute origins are ignored and disclosed; relative ordinal tick spacing, transport bit and all three net levels are compared. Drivers are compared only where both recordings supply them. Missing driver coverage and Floating/Contention rows are disclosed.

Reports include file paths, source comments, origin/start coordinates, both lengths, compared/differing rows, driver coverage, unresolved rows and up to sixteen mismatch examples. Exit 0 means scoped observation equality, 2 means mismatch, 1 means parse/read/write/argument failure. Report cannot overwrite an input file.

This closes an offline diagnostic comparison facility, not a physical fidelity claim. No real analyser recording is bundled or compared in this slice. Existing HARDWARE_SOURCES and M14A/M14W/M14X evidence classifications are unchanged: exact PHI widths/dead time, launch/sample edges, DATA electrical polarity/ownership/chip mapping and STR/RCD propagation remain SOURCE-BLOCKED. Normalized bit observations can be compared independently of those gaps; physical-time validation requires actual recordings and source-backed normalization.

## Validation
Four focused library regressions cover native export import, explicit origin/index alignment, all comparison fields/unequal lengths, provenance/pin-only unknown levels, CRLF and malformed input. Native owner gate captures two real firmware words, compares identical observations, deliberately changes one IS level and requires mismatch report/exit 2. Existing full tests, release build, diagnostic and benchmarks remain owner-local. Actions executes only fmt/check/save.

Production transport and live execution are unchanged.
