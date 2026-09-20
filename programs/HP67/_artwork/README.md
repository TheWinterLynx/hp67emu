# HP-67 program-card artwork cache

This directory contains PNG card faces derived from the checked-in HP pack PDFs.

Generate them from the repository root:

```powershell
python -m pip install pymupdf; python tools/extract_hp67_card_artwork.py --force
```

The emulator looks for `programs/HP67/_artwork/<reference>.png` when a Program Library entry is loaded. If no PNG exists, the magnetic card still loads normally and the UI falls back to the procedural title/reference face.

The extractor searches the program title inside the searchable pack scan and preserves the physical HP-67 card aspect ratio (71.1 x 11.4 mm). If an automatic crop is wrong, add a normalized page/crop override to `overrides.json` and regenerate.
