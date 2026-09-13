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

Quadratic and cubic font contours are adaptively subdivided to a 0.10-font-unit
flatness tolerance; a scanline decomposition emits
vector trapezoids while retaining counters/holes. Vertices are deduplicated.
The original flattened contours also supply a physical-pixel coverage fringe
at rendering time, keeping lettering smooth at small and fractional scales.
This fringe uses the same path for resting and translated keycaps. Positions
are not rounded independently per letter.

The glyph audit adds hand-shaped cubic outlines for mathematical x, y and pi
in `hp_math.py`. They replace those three Arimo contours at generation time.
The x has curved crossing arms; y retains its hooked descender. Nonzero winding
triangulation preserves solid overlaps as well as the font's counters. A Rust
regression checks the x crossing, y descender and availability of pi.
Compound exchange legends use actual outline advances and a mark 0.50 times
the legend size, replacing the former 1.04 ratio and fixed letter offsets.

The panel disables egui's prerasterized-disc optimization. The full-panel test
checks that every tessellated vertex uses the solid white texel at 1x and 4x,
so a future bitmap label, font atlas glyph or cached dot fails the test.
Antialias coverage remains tied to physical pixels, not logical glyph size.
