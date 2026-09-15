//! Authentic HP-67 Classic-series LED emission for the photographed display.
//!
//! The HP-67 uses three end-stackable five-character HP 1990-0335 modules.
//! HP 5082-7405 is a documented drop-in equivalent: 3.81 mm character pitch,
//! 2.794 mm magnified height and a centrally located decimal that consumes its
//! own character position. "Centered" describes the decimal's horizontal
//! placement in that position; photographs of the real HP-67 show the dot below
//! the digit midline. Each logical segment is formed by three emitting bars.
//!
//! The photograph supplies the red filter, bezel, reflections and black level;
//! this module paints only LED emission.

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Shape, Stroke};

pub(crate) const CHARACTER_COUNT: usize = 15;
const MODULE_COUNT: usize = 3;
const CHARACTERS_PER_MODULE: usize = 5;
const CHARACTER_PITCH_MM: f32 = 3.81;
const CHARACTER_HEIGHT_MM: f32 = 2.794;
const CHARACTER_WIDTH_MM: f32 = 1.5748; // .062 in
const DECIMAL_DIAMETER_MM: f32 = 0.5334; // .021 in, 5082-7400 family font drawing

const REFERENCE_UNITS_PER_MM: f32 = 614.0 / 152.4;
const REFERENCE_DISPLAY_WIDTH: f32 = 282.0;
const REFERENCE_DISPLAY_HEIGHT: f32 = 57.0;
const CHARACTER_PITCH: f32 = CHARACTER_PITCH_MM * REFERENCE_UNITS_PER_MM;
const CHARACTER_HEIGHT: f32 = CHARACTER_HEIGHT_MM * REFERENCE_UNITS_PER_MM;
const CHARACTER_WIDTH: f32 = CHARACTER_WIDTH_MM * REFERENCE_UNITS_PER_MM;
const DECIMAL_DIAMETER: f32 = DECIMAL_DIAMETER_MM * REFERENCE_UNITS_PER_MM;
const MODULE_WIDTH: f32 = CHARACTER_PITCH * CHARACTERS_PER_MODULE as f32;
const ASSEMBLY_WIDTH: f32 = MODULE_WIDTH * MODULE_COUNT as f32;

const SEG_A: u8 = 1 << 0;
const SEG_B: u8 = 1 << 1;
const SEG_C: u8 = 1 << 2;
const SEG_D: u8 = 1 << 3;
const SEG_E: u8 = 1 << 4;
const SEG_F: u8 = 1 << 5;
const SEG_G: u8 = 1 << 6;

#[derive(Clone, Copy)]
struct DisplayTransform {
    rect: Rect,
    sx: f32,
    sy: f32,
    stroke_scale: f32,
}

impl DisplayTransform {
    fn new(rect: Rect) -> Self {
        let sx = rect.width() / REFERENCE_DISPLAY_WIDTH;
        let sy = rect.height() / REFERENCE_DISPLAY_HEIGHT;
        Self {
            rect,
            sx,
            sy,
            stroke_scale: sx.min(sy),
        }
    }

    fn pos(self, x: f32, y: f32) -> Pos2 {
        pos2(
            self.rect.left() + x * self.sx,
            self.rect.top() + y * self.sy,
        )
    }

    fn stroke(self, reference_width: f32, minimum: f32) -> f32 {
        (reference_width * self.stroke_scale).max(minimum)
    }
}

pub(crate) fn paint(painter: &Painter, display_rect: Rect, value: &str) {
    if value.is_empty() || display_rect.width() <= 0.0 || display_rect.height() <= 0.0 {
        return;
    }

    let t = DisplayTransform::new(display_rect);
    let p = painter.with_clip_rect(display_rect);
    let cells = display_cells(value);
    let center_x = REFERENCE_DISPLAY_WIDTH * 0.5;
    let center_y = REFERENCE_DISPLAY_HEIGHT * 0.5;
    let assembly_left = center_x - ASSEMBLY_WIDTH * 0.5;
    let first_center = assembly_left + CHARACTER_PITCH * 0.5;

    for (index, ch) in cells.into_iter().enumerate() {
        if ch == ' ' {
            continue;
        }
        let x = first_center + index as f32 * CHARACTER_PITCH;
        if ch == '.' {
            draw_center_decimal(&p, t, x, center_y);
        } else {
            draw_digit(&p, t, x, center_y, ch);
        }
    }
}

fn segment_mask(ch: char) -> u8 {
    match ch {
        '0' => SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F,
        '1' => SEG_B | SEG_C,
        '2' => SEG_A | SEG_B | SEG_D | SEG_E | SEG_G,
        '3' => SEG_A | SEG_B | SEG_C | SEG_D | SEG_G,
        '4' => SEG_B | SEG_C | SEG_F | SEG_G,
        '5' => SEG_A | SEG_C | SEG_D | SEG_F | SEG_G,
        '6' => SEG_A | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G,
        '7' => SEG_A | SEG_B | SEG_C,
        '8' => SEG_A | SEG_B | SEG_C | SEG_D | SEG_E | SEG_F | SEG_G,
        '9' => SEG_A | SEG_B | SEG_C | SEG_D | SEG_F | SEG_G,
        '-' => SEG_G,
        _ => 0,
    }
}

fn draw_digit(p: &Painter, t: DisplayTransform, cx: f32, cy: f32, ch: char) {
    let mask = segment_mask(ch);
    if mask == 0 {
        return;
    }

    let h = CHARACTER_HEIGHT;
    let w = CHARACTER_WIDTH;
    let x = w * 0.43;
    let upper_y = h * 0.245;
    let lower_y = h * 0.245;
    let horizontal_half = w * 0.405;
    let vertical_half = h * 0.185;

    for (bit, x0, y0, horizontal, half_len, serif) in [
        (SEG_A, cx, cy - h * 0.47, true, horizontal_half, true),
        (SEG_B, cx + x, cy - upper_y, false, vertical_half, false),
        (SEG_C, cx + x, cy + lower_y, false, vertical_half, false),
        (SEG_D, cx, cy + h * 0.47, true, horizontal_half, true),
        (SEG_E, cx - x, cy + lower_y, false, vertical_half, false),
        (SEG_F, cx - x, cy - upper_y, false, vertical_half, false),
        (SEG_G, cx, cy, true, horizontal_half * 0.98, false),
    ] {
        if mask & bit != 0 {
            draw_monolithic_segment(p, t, x0, y0, horizontal, half_len, serif);
        }
    }
}

fn draw_monolithic_segment(
    p: &Painter,
    t: DisplayTransform,
    cx: f32,
    cy: f32,
    horizontal: bool,
    half_len: f32,
    serif: bool,
) {
    let offsets = [-0.22_f32, 0.0, 0.22];
    let inks = [
        Color32::from_rgb(242, 13, 51),
        Color32::from_rgb(255, 21, 56),
        Color32::from_rgb(220, 9, 44),
    ];
    let glow = Color32::from_rgba_unmultiplied(255, 0, 42, 25);

    let (glow_from, glow_to) = if horizontal {
        (t.pos(cx - half_len, cy), t.pos(cx + half_len, cy))
    } else {
        (t.pos(cx, cy - half_len), t.pos(cx, cy + half_len))
    };
    p.line_segment(
        [glow_from, glow_to],
        Stroke::new(t.stroke(0.82, 0.72), glow),
    );

    for (index, offset) in offsets.into_iter().enumerate() {
        if horizontal {
            let left_extra = if serif {
                0.18 - index as f32 * 0.055
            } else {
                0.0
            };
            p.line_segment(
                [
                    t.pos(cx - half_len - left_extra, cy + offset),
                    t.pos(cx + half_len, cy + offset),
                ],
                Stroke::new(t.stroke(0.155, 0.34), inks[index]),
            );
        } else {
            p.line_segment(
                [
                    t.pos(cx + offset, cy - half_len),
                    t.pos(cx + offset, cy + half_len),
                ],
                Stroke::new(t.stroke(0.155, 0.34), inks[index]),
            );
        }
    }
}

fn draw_center_decimal(p: &Painter, t: DisplayTransform, cx: f32, cy: f32) {
    // Measurement from the supplied real HP-67 photograph: compared with the
    // adjacent zero, the decimal's optical center is ~0.16 character heights
    // below the digit center.  The 5082-7400 family drawing gives a ~.021 in
    // decimal element, which also matches the photographed dot-to-digit ratio.
    let decimal_y = cy + CHARACTER_HEIGHT * 0.16;
    let radius = DECIMAL_DIAMETER * 0.5;

    // The real point reads as a compact luminous dot, not as a miniature dash.
    // Draw a restrained larger glow first, then the measured emitting element.
    ellipse(
        p,
        t,
        cx,
        decimal_y,
        radius * 1.65,
        radius * 1.65,
        Color32::from_rgba_unmultiplied(255, 0, 42, 24),
    );
    ellipse(
        p,
        t,
        cx,
        decimal_y,
        radius,
        radius,
        Color32::from_rgb(252, 18, 54),
    );
}

fn ellipse(p: &Painter, t: DisplayTransform, cx: f32, cy: f32, rx: f32, ry: f32, color: Color32) {
    let points = (0..24)
        .map(|i| {
            let angle = i as f32 * std::f32::consts::TAU / 24.0;
            t.pos(cx + rx * angle.cos(), cy + ry * angle.sin())
        })
        .collect();
    p.add(Shape::convex_polygon(points, color, Stroke::NONE));
}

fn display_cells(value: &str) -> [char; CHARACTER_COUNT] {
    let mut cells = [' '; CHARACTER_COUNT];
    if value.is_empty() {
        return cells;
    }

    // When the calculator core supplies an already-spaced 15-position field,
    // preserve it verbatim. Eventually the renderer should be fed directly by
    // the emulated display scan state rather than by formatted text.
    if value.contains(' ') && !value.contains(['e', 'E']) {
        for (cell, ch) in cells.iter_mut().zip(value.chars()) {
            *cell = ch;
        }
        return cells;
    }

    let (mantissa, exponent) = value.split_once(['e', 'E']).unwrap_or((value, ""));
    cells[0] = if mantissa.starts_with('-') { '-' } else { ' ' };
    let magnitude = mantissa.trim_start_matches(['-', '+']);

    let mut out = 1usize;
    let mut digits = 0usize;
    let mut decimal = false;
    for ch in magnitude.chars() {
        if ch.is_ascii_digit() && digits < 10 && out < 12 {
            cells[out] = ch;
            out += 1;
            digits += 1;
        } else if ch == '.' && !decimal && out < 12 {
            cells[out] = '.';
            out += 1;
            decimal = true;
        }
    }

    if digits == 0 {
        return cells;
    }

    if !decimal && out < 12 {
        cells[out] = '.';
    }

    if !exponent.is_empty() {
        cells[12] = if exponent.starts_with('-') { '-' } else { ' ' };
        let exp_digits: Vec<char> = exponent
            .trim_start_matches(['-', '+'])
            .chars()
            .filter(|c| c.is_ascii_digit())
            .collect();
        let tens = if exp_digits.len() >= 2 {
            exp_digits[exp_digits.len() - 2]
        } else {
            '0'
        };
        let ones = exp_digits.last().copied().unwrap_or('0');
        cells[13] = tens;
        cells[14] = ones;
    }

    cells
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_dimensions_follow_original_classic_display() {
        assert!((CHARACTER_PITCH / REFERENCE_UNITS_PER_MM - 3.81).abs() < 0.0001);
        assert!((CHARACTER_HEIGHT / REFERENCE_UNITS_PER_MM - 2.794).abs() < 0.0001);
        assert!((CHARACTER_WIDTH / REFERENCE_UNITS_PER_MM - 1.5748).abs() < 0.0001);
        assert!((DECIMAL_DIAMETER / REFERENCE_UNITS_PER_MM - 0.5334).abs() < 0.0001);
        assert!((ASSEMBLY_WIDTH / REFERENCE_UNITS_PER_MM - 57.15).abs() < 0.001);
        assert_eq!(CHARACTER_COUNT, 15);
        assert_eq!(MODULE_COUNT * CHARACTERS_PER_MODULE, CHARACTER_COUNT);
    }

    #[test]
    fn photographed_glass_keeps_original_led_to_aperture_ratio() {
        let pitch_ratio = CHARACTER_PITCH / REFERENCE_DISPLAY_WIDTH;
        let height_ratio = CHARACTER_HEIGHT / REFERENCE_DISPLAY_HEIGHT;
        assert!((pitch_ratio - 0.05444).abs() < 0.0001);
        assert!((height_ratio - 0.19749).abs() < 0.0001);
        assert!((ASSEMBLY_WIDTH / REFERENCE_DISPLAY_WIDTH - 0.8165).abs() < 0.0002);
    }

    #[test]
    fn hp67_fixed_display_partition_is_exact() {
        assert_eq!(
            display_cells("-1.234567890e-7"),
            ['-', '1', '.', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '0', '7']
        );
        assert_eq!(
            display_cells("1.234567890e12"),
            [' ', '1', '.', '2', '3', '4', '5', '6', '7', '8', '9', '0', ' ', '1', '2']
        );
        assert_eq!(&display_cells("0.00")[..5], &[' ', '0', '.', '0', '0']);
        assert_eq!(display_cells("+"), [' '; CHARACTER_COUNT]);
        assert_eq!(display_cells(""), [' '; CHARACTER_COUNT]);
    }

    #[test]
    fn classic_numeric_masks_match_seven_segment_wiring() {
        assert_eq!(segment_mask('1'), SEG_B | SEG_C);
        assert_eq!(segment_mask('8'), 0x7f);
        assert_eq!(segment_mask('-'), SEG_G);
        assert_eq!(segment_mask('.'), 0);
    }
}
