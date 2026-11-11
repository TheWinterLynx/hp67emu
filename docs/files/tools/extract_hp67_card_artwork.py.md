# `tools/extract_hp67_card_artwork.py`

## Purpose

Extract printable card faces from PAC PDFs into canonical PNGs, with an optional fixed-row atlas.

## Why it exists

Scanned manual cards need reviewed crop coordinates and consistent physical dimensions before use in the desktop. This offline tool preserves native raster scale instead of stretching each detected strip to fit.

## Relationships

Uses PyMuPDF for PDF rasterization and Pillow for image processing. Accepts a PDF, a reviewed CSV manifest or reference list, and an output directory. Outputs artwork consumed by the embedding pipeline; it does not generate magnetic program records. PDF viewing in the emulator is a separate feature.

## Responsibilities

Detect wide dark strips, emit candidate PNGs and HTML for review, retain one-based page numbers and PDF-point crop coordinates in CSV, assign references, normalize cards, and build and validate an atlas. Expected counts and duplicate-reference checks detect some input mistakes; they do not certify that a crop contains the correct complete card. Review the images before publishing them.

## Implementation

The `scan`, `extract`, `auto` and `atlas` subcommands separate detection, reviewed extraction and packaging. Extraction defaults to 600 DPI on a 71.1 by 11.4 mm canvas. Native crops are anchored at the top left; excess right/bottom pixels are cropped and missing area is black-filled. Geometric top-registration-mark detection precedes edge cleanup, and detected marks are repainted white. The resulting card is flattened onto an opaque black substrate and decoded again to verify dimensions and RGBA format.

The atlas defaults to 499 by 80 pixels per card and uses BOX area sampling. Validation checks dimensions, RGBA mode, opacity and, when present, the SD1-01A top marks. Built-in reference profiles cover Standard and Games I; other orderings can be supplied explicitly. The CSV manifest is the current crop-correction interface; this version does not read an overrides.json file or locate cards by matching title text.
