//! Physically registered HP-67 Classic-series LED emission.
//!
//! The renderer consumes only the raw A..G/DP masks produced by the emulated
//! ROM0/cathode path.  Macro geometry is expressed in millimetres from the
//! 5082-7400/7405 family data and is mapped to the photographed HP-67 with one
//! uniform pixels-per-millimetre scale.  The display-glass rectangle is only a
//! clip mask; it must never stretch the LED geometry independently in X and Y.

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Shape, Stroke};

pub(crate) const CHARACTER_COUNT: usize = 15;
pub(crate) const CHARACTER_PITCH_MM: f32 = 3.81;
pub(crate) const CHARACTER_HEIGHT_MM: f32 = 2.794;
pub(crate) const CHARACTER_WIDTH_MM: f32 = 1.5748; // .062 in
pub(crate) const DECIMAL_DIAMETER_MM: f32 = 0.5334; // .021 in

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
        (
            SEG_B,
            cx_mm + x,
            cy_mm - upper_y,
            false,
            vertical_half,
            false,
        ),
        (
            SEG_C,
            cx_mm + x,
            cy_mm + lower_y,
            false,
            vertical_half,
            false,
        ),
        (SEG_D, cx_mm, cy_mm + h * 0.47, true, horizontal_half, true),
        (
            SEG_E,
            cx_mm - x,
            cy_mm + lower_y,
            false,
            vertical_half,
            false,
        ),
        (
            SEG_F,
            cx_mm - x,
            cy_mm - upper_y,
            false,
            vertical_half,
            false,
        ),
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
    let offsets_mm = [-0.22_f32 * ARTWORK_UNIT_MM, 0.0, 0.22 * ARTWORK_UNIT_MM];
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
            t.pos(cx_mm + rx_mm * angle.cos(), cy_mm + ry_mm * angle.sin())
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
        let module_count = 3usize;
        let characters_per_module = 5usize;
        let assembly_width_mm =
            CHARACTER_PITCH_MM * characters_per_module as f32 * module_count as f32;
        assert!((assembly_width_mm - 57.15).abs() < 0.001);
        assert_eq!(CHARACTER_COUNT, module_count * characters_per_module);
    }

    #[test]
    fn all_character_centres_are_on_exact_3_81_mm_pitch() {
        assert!((character_offset_mm(0) + 26.67).abs() < 0.0001);
        assert_eq!(character_offset_mm(7), 0.0);
        assert!((character_offset_mm(14) - 26.67).abs() < 0.0001);
        for index in 0..CHARACTER_COUNT - 1 {
            assert!(
                (character_offset_mm(index + 1) - character_offset_mm(index) - CHARACTER_PITCH_MM)
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
