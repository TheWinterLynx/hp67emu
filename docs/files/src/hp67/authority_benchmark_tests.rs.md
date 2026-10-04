# `src/hp67/authority_benchmark_tests.rs`

## Purpose

Measures the actual desktop live authority bridge after M14N, separately from historical structural fixtures.

## Why it exists

The historical dual benchmark never called the live restore guard and result commits. Treating that row as current production throughput concealed the cost of the new authority path.

## Relationships

A test-only child of `src/hp67.rs` uses its real cycle, machine, backplane and port lifecycle. It retains the same HP-67 physical 320 us/word reference and workload environment variables as the historical integration benchmarks.

## Responsibilities

Measure actual firmware idle, synthetic alternating installed-RAM ports and synthetic CRC ports; boot/warm outside timed samples; keep machines continuous between rotating-order rounds; report median/min/max us per word, words/s and hardware multiple. Require exact word, oracle execution and 224-transition counts. Label host fixture overhead and source-blocked electrical scope.

## Implementation

The ignored release-only benchmark directly calls `step_firmware_cycle_with_execution`, including snapshot/binding, architectural oracle, restoration guard, structural transport, exact result comparisons, authority commits and card-transport advance. RAM fixtures retain the continuous logical DATA tail. CRC fixtures refill/drain logical buffers within timed samples, with no inserted physical card; their numbers are stress costs, not card throughput. A normal regression runs odd/even word batches to lock continuous fixture state, tick counts, execution counts, port ownership and reconstructed RAM payload. Invalid workload environment values fail instead of silently using a default. Wall-clock Instant appears only in benchmark instrumentation. No fixed host speed threshold is a correctness claim.
