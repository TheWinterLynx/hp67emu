# Firmware plot viewer

## Purpose

Draw a desktop chart of samples obtained from the HP-67 Standard Pac polynomial evaluator.

## Why it exists

The desktop can render and export the numeric results while the original calculator program performs the evaluation.

## Relationships

Opened from the main menu in app.rs. The worker owns a sampler from hp67/plot_sampler.rs and sends plain coordinates back to egui.

## Responsibilities

Offer editable cubic coefficients and an X interval, draw a dark chart with sampled-coordinate inspection, show progress or an error, and copy successful runs as CSV.

## Implementation

A native worker boots a separate machine and sends up to 201 points. The UI never reads or mutates the foreground machine. Dropping the receiver stops delivery and the worker exits no later than its next completed sample or error. Chart geometry and axis scaling are host presentation only. The WASM frontend does not expose this worker-based viewer.
