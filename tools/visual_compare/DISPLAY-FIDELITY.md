# HP-67 display fidelity audit

This pass replaces the generic seven-segment artwork at runtime with a renderer calibrated to the real Classic-series display hardware.

## Hardware sources

- Hewlett-Packard 5082-7400 series technical data (April 1976 / later catalog reprints): the five-digit center-decimal device is the 5082-7405 family. Character pitch for the 4/5-digit clusters is 3.81 mm (.150 in), magnified character height is 2.794 mm (.110 in), GaAsP emission is approximately 655 nm, the package has integral molded magnifier lenses and a red contrast filter, and the centered decimal is activated as a separate character position rather than as a lower-right point attached to a numeral.
  - https://www.keesvandersanden.nl/calculators/datasheets/HP_50827400.pdf
  - https://www.bitsavers.org/components/hp/1976_HP_Optoelectronics_Designers_Catalog.pdf
- Kees van der Sanden's Classic display teardown: the 15-character display is three five-digit modules; every digit has seven segments plus the decimal-point element; each segment is split into three emitting bars and the point into two bars. The fixed field allocation is: position 1 mantissa sign, positions 2-12 mantissa plus the separate decimal position, position 13 exponent sign, positions 14-15 exponent.
  - https://www.keesvandersanden.nl/calculators/led.php
- HP Classic display history: the molded lens magnifies the tiny LED die and the top/bottom segment shapes were intentionally tuned with a slight left serif for readability.
  - https://www.hpmemoryproject.org/wb_pages/d_cochran_01.htm
- HP-67/97 brochure: HP-67 plan width 81 mm and length 152.4 mm. Those published dimensions already anchor the emulator's physical design scale.
- Existing project photo registration (`proportion_reference.json` / `PROPORTIONS.md`): the real front-plane display opening measures 281.60 x 56.90 logical units; production uses 282 x 57. This pass intentionally preserves that already-calibrated glass aperture.

## Production geometry

The renderer derives physical LED dimensions from the same case scale used by the chassis:

- design units per mm: `CASE_MAX_WIDTH / 81 mm`
- character pitch: 3.81 mm = 15.35 logical units
- magnified character height: 2.794 mm = 11.2567 logical units
- five-character module width: 19.05 mm = 76.75 logical units
- complete three-module assembly: 57.15 mm = 230.25 logical units
- 15 positions centered exactly in the measured display opening

No photographic bitmap or external font is used. The display remains resolution-independent vector geometry, with the existing WGPU optical pass applied after it.

## Character construction

The renderer does not use a generic beveled seven-segment font. Each logical segment is made from three narrow emissive bars, with the documented slight left-edge serif on the top and bottom bars. The decimal position is its own centered character and consists of two emissive bars. Unlit segment structure and the 15 bubble lenses remain only faintly visible through the red filter.

Exponent formatting is fixed to the physical HP partition: exponent sign in position 13 and a zero-padded two-digit exponent in positions 14-15.
