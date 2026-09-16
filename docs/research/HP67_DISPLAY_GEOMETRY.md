# HP-67 display geometry and front-plane registration

## Scope

This note records the evidence used to place the fifteen HP-67 LED positions in the photo-based UI. It concerns physical geometry only; it does not change ROM0 decode, cathode scan order, ACT state or electrical timing.

## Hardware dimensions

The HP 5082-7400-series catalogue describes the 5082-7405 five-digit centre-decimal display used as a documented replacement for the HP 1990-0335 Classic-series module. The dimensions used by the renderer are:

- digit pitch: 0.150 in = 3.81 mm;
- magnified character height: 0.110 in = 2.794 mm;
- character width: 0.062 in = 1.5748 mm;
- decimal emitter diameter: 0.021 in = 0.5334 mm;
- five positions per module; three modules give fifteen physical positions.

Primary/technical references:

- HP Optoelectronics Designer's Catalog, 5082-7400 series: https://bitsavers.org/components/hp/LEDs/1981_HP_Optoelectronics_Designers_Catalog.pdf
- HP-67 physical display discussion and inspected module inventory: https://www.keesvandersanden.nl/calculators/classic_display.php
- HP-67/97 brochure dimensions used by the project's earlier proportion study: https://www.vintage-calculators.nl/HP-67-97-Brochure-1.pdf

## Photo registration

`assets/hp67.png` is 928x1695 pixels and is always fitted to the UI with a uniform scale. The display-plane calibration is therefore kept in source-image coordinates:

- projected case centre: x = 453 px;
- projected case width at the LED plane: 792 px;
- LED optical centre: y = 156 px;
- physical calculator width used by the project: 81.0 mm;
- resulting source scale: 792 / 81 = 9.777778 px/mm.

The case edge measurements carry a few source pixels of uncertainty because the photographed body is rounded and perspective/lens effects make an "outer edge" non-singular. They are nevertheless a better registration datum than the glass aperture because the glass is not 81 mm wide and cannot define a physical millimetre scale.

The glass rectangle `(178,105)..(750,208)` is now used only to clip LED emission. It no longer maps a synthetic 282x57 coordinate plane to the screen. This removes the previous anisotropic scaling error that made the LED characters too narrow, too short and too close to the centre.

## Deterministic position model

With the centre position numbered index 7, every LED cell centre is

`x(index) = optical_center_x + (index - 7) * 3.81 mm * scale`.

At native source-photo scale this produces outer position centres near x=192.23 and x=713.77. The first visible zero in the source-backed power-on `0.00` pattern is physical index 1 and therefore lands near x=229.48 rather than the old, too-centred location.

## Claims deliberately not made

This calibration does not claim sub-pixel recovery of camera lens distortion, the exact package tilt angle in a particular manufactured calculator, or unmeasured LED die-mask dimensions. Those require additional direct photographic or hardware measurement evidence. Electrical RCD/STR/PHI timing remains governed by the separate hardware-timing evidence policy.
