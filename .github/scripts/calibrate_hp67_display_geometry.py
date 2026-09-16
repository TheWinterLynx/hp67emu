from pathlib import Path

classic = r'''//! Physically registered HP-67 Classic-series LED emission.
//!
//! The renderer consumes only the raw A..G/DP masks produced by the emulated
//! ROM0/cathode path.  Macro geometry is expressed in millimetres from the
//! 5082-7400/7405 family data and is mapped to the photographed HP-67 with one
//! uniform pixels-per-millimetre scale.  The display-glass rectangle is only a
//! clip mask; it must never stretch the LED geometry independently in X and Y.

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Shape, Stroke};

pub(crate) const CHARACTER_COUNT: usize = 15;
const MODULE_COUNT: usize = 3;
const CHARACTERS_PER_MODULE: usize = 5;
pub(crate) const CHARACTER_PITCH_MM: f32 = 3.81;
pub(crate) const CHARACTER_HEIGHT_MM: f32 = 2.794;
pub(crate) const CHARACTER_WIDTH_MM: f32 = 1.5748; // .062 in
pub(crate) const DECIMAL_DIAMETER_MM: f32 = 0.5334; // .021 in
const MODULE_WIDTH_MM: f32 = CHARACTER_PITCH_MM * CHARACTERS_PER_MODULE as f32;
const ASSEMBLY_WIDTH_MM: f32 = MODULE_WIDTH_MM * MODULE_COUNT as f32;

// Preserve the already-reviewed internal three-bar artwork dimensions while
// expressing them in millimetres.  These are optical artwork dimensions, not a
// claim about an unmeasured die-mask feature.
const ARTWORK_UNIT_MM: f32 = 152.4 / 614.0;

const SEG_A: u8 = 1 << 0;
const SEG_B: u8 = 1 << 1;
const SEG_C: u8 = 1 << 2;
const SEG_D: u8 = 1 << 3;
const SEG_E: u8 = 1 << 4;
const SEG_F: u8 = 1 << 5;
const SEG_G: u8 = 1 << 6;
const SEG_DP: u8 = 1 << 7;

#[derive(Clone, Copy)]
struct DisplayTransform {
    center: Pos2,
    pixels_per_mm: f32,
}

impl DisplayTransform {
    const fn new(center: Pos2, pixels_per_mm: f32) -> Self {
        Self {
            center,
            pixels_per_mm,
        }
    }

    fn pos(self, x_mm: f32, y_mm: f32) -> Pos2 {
        pos2(
            self.center.x + x_mm * self.pixels_per_mm,
            self.center.y + y_mm * self.pixels_per_mm,
        )
    }

    fn stroke(self, width_mm: f32, minimum_px: f32) -> f32 {
        (width_mm * self.pixels_per_mm).max(minimum_px)
    }
}

pub(crate) const fn character_offset_mm(index: usize) -> f32 {
    (index as f32 - 7.0) * CHARACTER_PITCH_MM
}

pub(crate) fn paint_segments(
    painter: &Painter,
    clip_rect: Rect,
    optical_center: Pos2,
    pixels_per_mm: f32,
    segments: &[u8; CHARACTER_COUNT],
) {
    if clip_rect.width() <= 0.0 || clip_rect.height() <= 0.0 || pixels_per_mm <= 0.0 {
        return;
    }

    let t = DisplayTransform::new(optical_center, pixels_per_mm);
    let p = painter.with_clip_rect(clip_rect);

    for (index, mask) in segments.iter().copied().enumerate() {
        if mask == 0 {
            continue;
        }
        draw_segment_mask(&p, t, character_offset_mm(index), 0.0, mask);
    }
}

fn draw_segment_mask(p: &Painter, t: DisplayTransform, cx_mm: f32, cy_mm: f32, mask: u8) {
    if mask == 0 {
        return;
    }

    let h = CHARACTER_HEIGHT_MM;
    let w = CHARACTER_WIDTH_MM;
    let x = w * 0.43;
    let upper_y = h * 0.245;
    let lower_y = h * 0.245;
    let horizontal_half = w * 0.405;
    let vertical_half = h * 0.185;

    for (bit, x0, y0, horizontal, half_len, serif) in [
        (SEG_A, cx_mm, cy_mm - h * 0.47, true, horizontal_half, true),
        (SEG_B, cx_mm + x, cy_mm - upper_y, false, vertical_half, false),
        (SEG_C, cx_mm + x, cy_mm + lower_y, false, vertical_half, false),
        (SEG_D, cx_mm, cy_mm + h * 0.47, true, horizontal_half, true),
        (SEG_E, cx_mm - x, cy_mm + lower_y, false, vertical_half, false),
        (SEG_F, cx_mm - x, cy_mm - upper_y, false, vertical_half, false),
        (SEG_G, cx_mm, cy_mm, true, horizontal_half * 0.98, false),
    ] {
        if mask & bit != 0 {
            draw_monolithic_segment(p, t, x0, y0, horizontal, half_len, serif);
        }
    }

    if mask & SEG_DP != 0 {
        draw_center_decimal(p, t, cx_mm, cy_mm);
    }
}

fn draw_monolithic_segment(
    p: &Painter,
    t: DisplayTransform,
    cx_mm: f32,
    cy_mm: f32,
    horizontal: bool,
    half_len_mm: f32,
    serif: bool,
) {
    let offsets_mm = [
        -0.22_f32 * ARTWORK_UNIT_MM,
        0.0,
        0.22 * ARTWORK_UNIT_MM,
    ];
    let inks = [
        Color32::from_rgb(242, 13, 51),
        Color32::from_rgb(255, 21, 56),
        Color32::from_rgb(220, 9, 44),
    ];
    let glow = Color32::from_rgba_unmultiplied(255, 0, 42, 25);

    let (glow_from, glow_to) = if horizontal {
        (
            t.pos(cx_mm - half_len_mm, cy_mm),
            t.pos(cx_mm + half_len_mm, cy_mm),
        )
    } else {
        (
            t.pos(cx_mm, cy_mm - half_len_mm),
            t.pos(cx_mm, cy_mm + half_len_mm),
        )
    };
    p.line_segment(
        [glow_from, glow_to],
        Stroke::new(t.stroke(0.82 * ARTWORK_UNIT_MM, 0.72), glow),
    );

    for (index, offset_mm) in offsets_mm.into_iter().enumerate() {
        if horizontal {
            let left_extra_mm = if serif {
                (0.18 - index as f32 * 0.055) * ARTWORK_UNIT_MM
            } else {
                0.0
            };
            p.line_segment(
                [
                    t.pos(cx_mm - half_len_mm - left_extra_mm, cy_mm + offset_mm),
                    t.pos(cx_mm + half_len_mm, cy_mm + offset_mm),
                ],
                Stroke::new(t.stroke(0.155 * ARTWORK_UNIT_MM, 0.34), inks[index]),
            );
        } else {
            p.line_segment(
                [
                    t.pos(cx_mm + offset_mm, cy_mm - half_len_mm),
                    t.pos(cx_mm + offset_mm, cy_mm + half_len_mm),
                ],
                Stroke::new(t.stroke(0.155 * ARTWORK_UNIT_MM, 0.34), inks[index]),
            );
        }
    }
}

fn draw_center_decimal(p: &Painter, t: DisplayTransform, cx_mm: f32, cy_mm: f32) {
    // The catalogue fixes the emitter diameter.  The vertical optical offset is
    // retained from the measured project reference photograph; it is not used
    // to infer any electrical behaviour.
    let decimal_y_mm = cy_mm + CHARACTER_HEIGHT_MM * 0.16;
    let radius_mm = DECIMAL_DIAMETER_MM * 0.5;

    ellipse(
        p,
        t,
        cx_mm,
        decimal_y_mm,
        radius_mm * 1.65,
        radius_mm * 1.65,
        Color32::from_rgba_unmultiplied(255, 0, 42, 24),
    );
    ellipse(
        p,
        t,
        cx_mm,
        decimal_y_mm,
        radius_mm,
        radius_mm,
        Color32::from_rgb(252, 18, 54),
    );
}

fn ellipse(
    p: &Painter,
    t: DisplayTransform,
    cx_mm: f32,
    cy_mm: f32,
    rx_mm: f32,
    ry_mm: f32,
    color: Color32,
) {
    let points = (0..24)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 24.0;
            t.pos(
                cx_mm + rx_mm * angle.cos(),
                cy_mm + ry_mm * angle.sin(),
            )
        })
        .collect();
    p.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn catalogue_dimensions_define_the_fifteen_position_assembly() {
        assert!((CHARACTER_PITCH_MM - 3.81).abs() < 0.0001);
        assert!((CHARACTER_HEIGHT_MM - 2.794).abs() < 0.0001);
        assert!((CHARACTER_WIDTH_MM - 1.5748).abs() < 0.0001);
        assert!((DECIMAL_DIAMETER_MM - 0.5334).abs() < 0.0001);
        assert!((ASSEMBLY_WIDTH_MM - 57.15).abs() < 0.001);
        assert_eq!(CHARACTER_COUNT, 15);
        assert_eq!(MODULE_COUNT * CHARACTERS_PER_MODULE, CHARACTER_COUNT);
    }

    #[test]
    fn all_character_centres_are_on_exact_3_81_mm_pitch() {
        assert!((character_offset_mm(0) + 26.67).abs() < 0.0001);
        assert_eq!(character_offset_mm(7), 0.0);
        assert!((character_offset_mm(14) - 26.67).abs() < 0.0001);
        for index in 0..CHARACTER_COUNT - 1 {
            assert!(
                (character_offset_mm(index + 1) - character_offset_mm(index)
                    - CHARACTER_PITCH_MM)
                    .abs()
                    < 0.0001
            );
        }
    }

    #[test]
    fn millimetre_transform_is_isotropic() {
        let t = DisplayTransform::new(pos2(100.0, 50.0), 10.0);
        let origin = t.pos(0.0, 0.0);
        let x = t.pos(1.0, 0.0);
        let y = t.pos(0.0, 1.0);
        assert!((x.x - origin.x - 10.0).abs() < 0.0001);
        assert!((y.y - origin.y - 10.0).abs() < 0.0001);
        assert_eq!(x.y, origin.y);
        assert_eq!(y.x, origin.x);
    }

    #[test]
    fn raw_hardware_segment_bits_match_renderer_wiring() {
        assert_eq!(SEG_A, 0x01);
        assert_eq!(SEG_G, 0x40);
        assert_eq!(SEG_DP, 0x80);
        assert_eq!(SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F, 0x3f);
    }
}
'''
Path('src/ui/classic_display.rs').write_text(classic)

panel = Path('src/panel.rs')
text = panel.read_text()
text = text.replace(
    'const DISPLAY_GLASS: PxRect = PxRect::new(178.0, 105.0, 750.0, 208.0);\n',
    '''// The glass is only the optical clipping aperture.  LED geometry is registered\n// independently to the projected calculator case so it cannot be stretched to\n// fit this rectangle.  Measurements are source-image pixels in assets/hp67.png.\nconst DISPLAY_GLASS: PxRect = PxRect::new(178.0, 105.0, 750.0, 208.0);\nconst HP67_CASE_WIDTH_MM: f32 = 81.0;\nconst DISPLAY_CASE_CENTER_X: f32 = 453.0;\nconst DISPLAY_CASE_WIDTH_SOURCE_PX: f32 = 792.0;\nconst DISPLAY_LED_CENTER_Y: f32 = 156.0;\nconst DISPLAY_SOURCE_PX_PER_MM: f32 = DISPLAY_CASE_WIDTH_SOURCE_PX / HP67_CASE_WIDTH_MM;\n'''
)
old_call = '''        // hp67.png already contains the real filter, glass, bezel and reflections.\n        // Add only the physically calibrated 5082-7405 LED emission on top.\n        if state.power_on {\n            classic_display::paint_segments(\n                &painter,\n                source_to_screen(photo_rect, DISPLAY_GLASS),\n                display.segments(),\n            );\n        }\n'''
new_call = '''        // hp67.png already contains the real filter, glass, bezel and reflections.\n        // Register the LED assembly to the calculator's projected case width with\n        // one uniform physical scale. DISPLAY_GLASS only clips emitted light.\n        if state.power_on {\n            let photo_scale = photo_rect.width() / PHOTO_W;\n            classic_display::paint_segments(\n                &painter,\n                source_to_screen(photo_rect, DISPLAY_GLASS),\n                source_point_to_screen(\n                    photo_rect,\n                    DISPLAY_CASE_CENTER_X,\n                    DISPLAY_LED_CENTER_Y,\n                ),\n                DISPLAY_SOURCE_PX_PER_MM * photo_scale,\n                display.segments(),\n            );\n        }\n'''
if old_call not in text:
    raise SystemExit('panel display call anchor not found')
text = text.replace(old_call, new_call)
anchor = '''fn source_to_screen(photo: Rect, src: PxRect) -> Rect {\n    let sx = photo.width() / PHOTO_W;\n    let sy = photo.height() / PHOTO_H;\n    Rect::from_min_max(\n        pos2(photo.left() + src.x0 * sx, photo.top() + src.y0 * sy),\n        pos2(photo.left() + src.x1 * sx, photo.top() + src.y1 * sy),\n    )\n}\n\n'''
replacement = anchor + '''fn source_point_to_screen(photo: Rect, x: f32, y: f32) -> Pos2 {\n    let sx = photo.width() / PHOTO_W;\n    let sy = photo.height() / PHOTO_H;\n    pos2(photo.left() + x * sx, photo.top() + y * sy)\n}\n\n'''
if anchor not in text:
    raise SystemExit('panel source mapping anchor not found')
text = text.replace(anchor, replacement)
test_anchor = '''    #[test]\n    fn display_glass_stays_inside_source_photo() {\n        assert!(DISPLAY_GLASS.x0 >= 0.0 && DISPLAY_GLASS.y0 >= 0.0);\n        assert!(DISPLAY_GLASS.x1 <= PHOTO_W && DISPLAY_GLASS.y1 <= PHOTO_H);\n    }\n'''
extra_test = test_anchor + '''\n    #[test]\n    fn led_registration_uses_case_scale_and_keeps_all_centres_inside_glass() {\n        assert!((DISPLAY_SOURCE_PX_PER_MM - 9.777_778).abs() < 0.0001);\n        let half_span =\n            7.0 * classic_display::CHARACTER_PITCH_MM * DISPLAY_SOURCE_PX_PER_MM;\n        let leftmost = DISPLAY_CASE_CENTER_X - half_span;\n        let rightmost = DISPLAY_CASE_CENTER_X + half_span;\n        assert!((leftmost - 192.23).abs() < 0.05);\n        assert!((rightmost - 713.77).abs() < 0.05);\n        assert!(leftmost > DISPLAY_GLASS.x0);\n        assert!(rightmost < DISPLAY_GLASS.x1);\n\n        // Power-on 0.00 starts at physical position 2 (index 1).  This regression\n        // prevents the old, too-centred placement from returning.\n        let first_power_on_zero = DISPLAY_CASE_CENTER_X\n            + classic_display::character_offset_mm(1) * DISPLAY_SOURCE_PX_PER_MM;\n        assert!((first_power_on_zero - 229.48).abs() < 0.05);\n    }\n'''
if test_anchor not in text:
    raise SystemExit('panel test anchor not found')
text = text.replace(test_anchor, extra_test)
panel.write_text(text)

Path('docs/files/src/ui/classic_display.rs.md').write_text(r'''# `src/ui/classic_display.rs`

## Purpose
Draws photorealistic HP Classic-series LED emission directly from the physical A..G/DP masks produced by the emulated display hardware.

## Why it exists
The photographed filter and bezel are static, while the LED emitters are dynamic. The display geometry must therefore be rendered independently, but it must preserve the real module pitch and character dimensions instead of being stretched to fill an arbitrary UI rectangle.

## Relationships
Called by `panel.rs`. `panel.rs` owns registration to `assets/hp67.png`; this module owns the 5082-7400/7405-family physical dimensions and raw segment artwork. The electrical source remains `machines::hp67` via `HardwareDisplayFrame`.

## Responsibilities
Map the fifteen raw A..G/DP masks onto fifteen physical LED positions at 3.81 mm pitch, preserve the 2.794 mm character height, 1.5748 mm character width and 0.5334 mm decimal emitter diameter, and draw emitted light without formatting a number or text string.

## Implementation
`paint_segments()` takes a clipping rectangle, an optical centre and one pixels-per-millimetre factor. X and Y always use the same scale. Character centres are `(index - 7) * 3.81 mm`, so the complete fifteen-position pitch span is symmetric about the optical axis. The photographed display glass is deliberately not used as a scaling surface. Internal three-bar artwork dimensions are retained from the previous reviewed renderer but are explicitly treated as optical artwork rather than a new die-mask claim.
''')

Path('docs/files/src/panel.rs.md').write_text(r'''# `src/panel.rs`

## Purpose
Renders the embedded HP-67 photograph, maps photographed control coordinates to the current window, handles key/switch hit regions and registers physical LED emission to the photographed calculator.

## Why it exists
The production UI is photo-based. Dynamic LEDs must align to the physical front plane of that photograph at every window size without inheriting distortion from the display-glass crop.

## Relationships
Reads mechanical `Hp67State`, receives `HardwareDisplayFrame` from `app.rs`, emits `UiEvent` values, and calls `ui::classic_display::paint_segments()` with the raw A..G/DP masks plus physical photo registration.

## Responsibilities
Fit the 928x1695 source photograph uniformly, map controls, animate key travel, provide the display clip aperture, derive the LED optical centre and pixels-per-millimetre scale from the projected HP-67 case, and forward raw hardware segment masks unchanged.

## Implementation
The source-photo display registration uses a projected case centre at x=453 px, an LED optical centre at y=156 px and a local case width of 792 px. With the documented 81.0 mm calculator width this yields 9.777778 source pixels/mm. `DISPLAY_GLASS` remains only the clipping aperture. Because the photo itself is uniformly scaled into the window, multiplying this source calibration by the photo scale preserves one isotropic physical transform at every UI size.
''')

Path('docs/research/HP67_DISPLAY_GEOMETRY.md').write_text(r'''# HP-67 display geometry and front-plane registration

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
''')
