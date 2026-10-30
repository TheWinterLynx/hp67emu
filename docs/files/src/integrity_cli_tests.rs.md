# src/integrity_cli_tests.rs

## Purpose

Exercise native ownership audit exit statuses and file safety.

## Why it exists

The offline auditor must remain usable without enabling runtime capture and must not overwrite inputs or existing reports.

## Relationships

Child of pin_trace_cli; uses temporary files, normalized sparse traces and optional actual firmware capture.

## Responsibilities

Check 0/2/3 distinctions, command routing, input/output preservation and invalid arguments.

## Implementation

The always-enabled test audits a synthetic nanosecond sparse input as incomplete, writes a new report, rejects overwrite and preserves bytes. The capture-feature test audits sixteen real firmware words, injects an agreeing CRC contribution on PGMSW for exit 2, then removes MOTOR observation for exit 3. No emulator payload or semantic oracle is altered.
