# M14V — optical publication epochs

## Problem and resulting behavior

M14U suppresses the panel while DISPLAY is disabled but retains the last completed scan. A rapid disable/enable could therefore reveal an older image before a new full scan completes. M14V marks previous scans ineligible whenever firmware executes its existing DISPLAY control instructions. It waits through the interrupted scan and one following complete scan before publishing again.

## Scope and evidence

This is a WORKING APPROXIMATION for conservative optical snapshot publication, not a new HP-67 latch or persistence claim. M14T's nominal 320 us words and fifteen-slot scans are unchanged. The cutoff is the current completed-scan count plus two: one possibly interrupted scan and one complete subsequent scan. Worst-case nominal added blank interval is two 4800 us scans, not a claimed physical reset delay.

No ACT phase, cathode phase, IS driver, PHI transition, firmware successor, raw dwell matrix, diagnostic prompt detection or card flow changes. Snapshot reads remain pure. Power reset clears the cutoff. No invented current, flux serialization or source-blocked edge is introduced.

## Validation

Actual boot firmware DISPLAY controls must invalidate optics. Every one of the fifteen scan phases is exercised through complete real-firmware word cycles: no stale publication, unchanged raw phase on invalidation, fresh complete output after the cutoff, and reset cleanup. Existing M14T/M14U regressions retain their contracts. Owner runs full warnings/release/diagnostic and realtime benchmarks; Actions formats Rust only.
