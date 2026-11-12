# Standard Pac polynomial sampler

## Purpose

Decode complete optical display frames for desktop PLOT. The SD1-09A evaluation in an isolated machine is retained as a test fixture.

## Why it exists

The graph should reflect the original program, including firmware numeric entry and the displayed result.

## Relationships

Uses Hp67LiveMachine, the built-in program library, physical key contacts, and complete optical display frames.

## Responsibilities

For the regression only: boot a separate machine, insert both card ends through the emulated reader, enter coefficients through f A and B through E, and evaluate X with A. Return the entered X and the numeric display result, or a bounded error.

## Implementation

Input values are rounded to at most nine significant digits for entry through digit, decimal, sign and exponent keys. Firmware key dispatch and return to idle are observed before the next operation. Wait for a new full display scan and decode its numeric segments; results have the precision of the displayed format. No host polynomial arithmetic or changes to the primary session are used. The release regression checks the three temperatures in the original Standard Pac example.
