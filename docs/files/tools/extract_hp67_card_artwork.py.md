# `tools/extract_hp67_card_artwork.py`

## Purpose

Generate visual program-card PNGs from the original HP pack PDFs already stored under `programs/HP67`.

## Method

The tool uses PyMuPDF only as an offline development dependency. It finds a program's first matching manual page, prefers the second exact title occurrence when available (the first is commonly the section heading), and crops around that title using the physical HP-67 card aspect ratio of 71.1 x 11.4 mm.

Generated files go to `programs/HP67/_artwork/<reference>.png`. The emulator never reads the PDF itself.

For scans whose automatic crop is imperfect, `programs/HP67/_artwork/overrides.json` may provide a zero-based page number and normalized crop rectangle. This keeps manual corrections data-driven instead of adding program-specific Rust code.
