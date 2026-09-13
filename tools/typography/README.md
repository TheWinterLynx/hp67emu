# Portable legend contours

`src/ui/lettering.bin` contains float vector coordinates, triangle indices and
outline contours. It contains **no raster pixels**. It is generated from the OFL
Arimo variable font at weights 400, 600 and 700. The source is
https://github.com/google/fonts/tree/main/ofl/arimo and the accompanying OFL is
included here. These are an approximation to HP's printed sans-serif lettering,
not the original HP artwork masters. Mathematical symbols and compound legends
continue to use their purpose-built geometry and placement.

Rebuild with Python/fonttools (development tooling only):

```powershell
python -m pip install --target target/fonttools fonttools==4.65.0
python tools/typography/generate.py target/Arimo.ttf
```

The input font's SHA-256 and download URL are in `source.json`. Download to the
ignored `target/Arimo.ttf`; neither Python nor the font file is needed to build
or run the emulator. The generated vector data and OFL notice are checked in.

Quadratic contours are subdivided in font space; a scanline decomposition emits
vector trapezoids while retaining counters/holes. Vertices are deduplicated.
The original flattened contours also supply a physical-pixel coverage fringe
at rendering time, keeping lettering smooth at small and fractional scales.
This fringe uses the same path for resting and translated keycaps. Positions
are not rounded independently per letter.
