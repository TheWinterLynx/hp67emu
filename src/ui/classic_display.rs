//! Physically calibrated HP-67 Classic-series LED display overlay.
//!
//! The HP-67 uses three end-stackable five-character HP 5082-7405 style
//! clusters: 15 character positions total, 3.81 mm pitch and a 2.794 mm
//! magnified character height.  The 7405 uses HP's centered decimal point;
//! the decimal therefore consumes a complete character time/position.
//! Each monolithic LED segment is visibly split into three emitting bars and
//! the decimal point into two bars.  The front red filter/lenses remain part of
//! the optical model rather than being approximated as a modern flat 7-seg font.

use eframe::egui::{Color32, Painter, Rect, Shape, Stroke, Ui};

use super::geometry::{
    Transform, CASE_MAX_WIDTH, CASE_WIDTH_MM, DESIGN_H, DESIGN_W, DISPLAY_BOUNDS,
};

pub(crate) const CHARACTER_COUNT: usize = 15;
pub(crate) const MODULE_COUNT: usize = 3;
pub(crate) const CHARACTERS_PER_MODULE: usize = 5;
pub(crate) const CHARACTER_PITCH_MM: f32 = 3.81; // .150 in, 4/5-digit 5082-7400 package
pub(crate) const CHARACTER_HEIGHT_MM: f32 = 2.794; // .110 in magnified character

const UNITS_PER_MM: f32 = CASE_MAX_WIDTH / CASE_WIDTH_MM;
pub(crate) const CHARACTER_PITCH: f32 = CHARACTER_PITCH_MM * UNITS_PER_MM;
pub(crate) const CHARACTER_HEIGHT: f32 = CHARACTER_HEIGHT_MM * UNITS_PER_MM;
pub(crate) const MODULE_WIDTH: f32 = CHARACTER_PITCH * CHARACTERS_PER_MODULE as f32;
pub(crate) const ASSEMBLY_WIDTH: f32 = MODULE_WIDTH * MODULE_COUNT as f32;

const SEG_A: u8 = 1 << 0;
const SEG_B: u8 = 1 << 1;
const SEG_C: u8 = 1 << 2;
const SEG_D: u8 = 1 << 3;
const SEG_E: u8 = 1 << 4;
const SEG_F: u8 = 1 << 5;
const SEG_G: u8 = 1 << 6;

pub(crate) fn paint(ui: &Ui, panel_rect: Rect, value: &str) {
    if panel_rect.width() <= 0.0 || panel_rect.height() <= 0.0 {
        return;
    }
    let scale = (panel_rect.width() / DESIGN_W).min(panel_rect.height() / DESIGN_H);
    let size = eframe::egui::vec2(DESIGN_W * scale, DESIGN_H * scale);
    let t = Transform {
        origin: panel_rect.center() - size * 0.5,
        scale,
    };
    let p = ui.painter().with_clip_rect(panel_rect);
    paint_glass(&p, t);
    paint_modules_and_digits(&p, t, &display_cells(value));
}

fn paint_glass(p: &Painter, t: Transform) {
    let [left, top, width, height] = DISPLAY_BOUNDS;
    let glass = t.rect(left, top, width, height);

    // The measured front-plane opening is already 282 x 57 design units.
    // Repaint it here so the older generic segment renderer below cannot leak
    // through; the WGPU optical pass is composited afterwards by app.rs.
    p.rect_filled(glass, t.s(3.8), Color32::from_rgb(30, 13, 12));

    // Red contrast filter: broad absorption plus a weak upper-room reflection.
    p.rect_filled(
        t.rect(left + 1.0, top + height * 0.54, width - 2.0, height * 0.45),
        t.s(1.0),
        Color32::from_rgba_unmultiplied(0, 0, 0, 30),
    );
    p.rect_filled(
        t.rect(left + 4.0, top + 2.2, width - 8.0, 2.0),
        t.s(1.0),
        Color32::from_rgba_unmultiplied(104, 50, 42, 18),
    );

    // A dark inner edge is visible through the filter; keep it restrained so
    // the subsequent Fresnel/reflection shader owns the final optical sheen.
    p.rect_stroke(
        glass.shrink(t.s(0.6)),
        t.s(3.2),
        Stroke::new(t.s(0.45), Color32::from_rgba_unmultiplied(8, 3, 3, 105)),
    );
}

fn paint_modules_and_digits(p: &Painter, t: Transform, cells: &[char; CHARACTER_COUNT]) {
    let [left, top, width, height] = DISPLAY_BOUNDS;
    let cx = left + width * 0.5;
    let cy = top + height * 0.5;
    let first_center = cx - CHARACTER_PITCH * 7.0;

    // Three five-character packages are end-stackable.  Their physical package
    // width is five 3.81 mm pitches, so the 15-character assembly is 57.15 mm.
    let assembly_left = cx - ASSEMBLY_WIDTH * 0.5;
    for module in 0..MODULE_COUNT {
        let x = assembly_left + module as f32 * MODULE_WIDTH;
        p.rect_filled(
            t.rect(x, cy - CHARACTER_HEIGHT * 0.98, MODULE_WIDTH, CHARACTER_HEIGHT * 1.96),
            t.s(1.2),
            Color32::from_rgba_unmultiplied(54, 11, 10, 16),
        );
        if module > 0 {
            p.line_segment(
                [
                    t.pos(x, cy - CHARACTER_HEIGHT * 0.91),
                    t.pos(x, cy + CHARACTER_HEIGHT * 0.91),
                ],
                Stroke::new(t.s(0.22), Color32::from_rgba_unmultiplied(6, 2, 2, 44)),
            );
        }
    }

    for (index, &ch) in cells.iter().enumerate() {
        let x = first_center + index as f32 * CHARACTER_PITCH;
        draw_bubble_lens(p, t, x, cy, index);
        if ch == '.' {
            draw_center_decimal(p, t, x, cy, true);
        } else {
            draw_digit(p, t, x, cy, ch);
        }
    }
}

fn draw_bubble_lens(p: &Painter, t: Transform, cx: f32, cy: f32, index: usize) {
    // Integral molded magnifier over each tiny die.  The minute package-to-
    // package tint variation is deterministic and intentionally sub-visible.
    let module = index / CHARACTERS_PER_MODULE;
    let tint = match module {
        0 => (61, 17, 15),
        1 => (58, 16, 14),
        _ => (60, 16, 14),
    };
    ellipse(
        p,
        t,
        cx,
        cy,
        CHARACTER_PITCH * 0.405,
        CHARACTER_HEIGHT * 0.78,
        Color32::from_rgba_unmultiplied(tint.0, tint.1, tint.2, 27),
        Stroke::new(t.s(0.20), Color32::from_rgba_unmultiplied(112, 36, 27, 18)),
    );
    // Tiny asymmetric lens catchlight. It must disappear at normal viewing
    // distance rather than read as a modern glossy UI highlight.
    ellipse(
        p,
        t,
        cx - CHARACTER_PITCH * 0.105,
        cy - CHARACTER_HEIGHT * 0.29,
        CHARACTER_PITCH * 0.18,
        CHARACTER_HEIGHT * 0.095,
        Color32::from_rgba_unmultiplied(170, 68, 48, 10),
        Stroke::NONE,
    );
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

fn draw_digit(p: &Painter, t: Transform, cx: f32, cy: f32, ch: char) {
    let mask = segment_mask(ch);
    let h = CHARACTER_HEIGHT;
    let w = h * 0.59;
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
        draw_monolithic_segment(
            p,
            t,
            x0,
            y0,
            horizontal,
            half_len,
            mask & bit != 0,
            serif,
        );
    }
}

fn draw_monolithic_segment(
    p: &Painter,
    t: Transform,
    cx: f32,
    cy: f32,
    horizontal: bool,
    half_len: f32,
    on: bool,
    serif: bool,
) {
    // HP's die has three emitting bars per segment.  At normal scale the three
    // merge optically; at high DPI the authentic internal structure resolves.
    let offsets = [-0.22_f32, 0.0, 0.22];
    let stripe_width = 0.155;
    let glow = if on {
        Color32::from_rgba_unmultiplied(255, 39, 18, 24)
    } else {
        Color32::from_rgba_unmultiplied(57, 13, 12, 15)
    };
    let ink = if on {
        [
            Color32::from_rgb(248, 46, 24),
            Color32::from_rgb(255, 55, 27),
            Color32::from_rgb(244, 39, 20),
        ]
    } else {
        [
            Color32::from_rgb(45, 15, 14),
            Color32::from_rgb(47, 16, 14),
            Color32::from_rgb(43, 14, 13),
        ]
    };

    if horizontal {
        p.line_segment(
            [t.pos(cx - half_len, cy), t.pos(cx + half_len, cy)],
            Stroke::new(t.s(0.78), glow),
        );
        for (i, offset) in offsets.into_iter().enumerate() {
            let left_extra = if serif { 0.18 - i as f32 * 0.055 } else { 0.0 };
            p.line_segment(
                [
                    t.pos(cx - half_len - left_extra, cy + offset),
                    t.pos(cx + half_len, cy + offset),
                ],
                Stroke::new(t.s(stripe_width), ink[i]),
            );
        }
    } else {
        p.line_segment(
            [t.pos(cx, cy - half_len), t.pos(cx, cy + half_len)],
            Stroke::new(t.s(0.78), glow),
        );
        for (i, offset) in offsets.into_iter().enumerate() {
            p.line_segment(
                [
                    t.pos(cx + offset, cy - half_len),
                    t.pos(cx + offset, cy + half_len),
                ],
                Stroke::new(t.s(stripe_width), ink[i]),
            );
        }
    }
}

fn draw_center_decimal(p: &Painter, t: Transform, cx: f32, cy: f32, on: bool) {
    // 5082-7405 center-decimal option: the decimal is a complete character,
    // not a lower-right dot attached to a numeral.  The die itself has two bars.
    let color_a = if on {
        Color32::from_rgb(255, 55, 27)
    } else {
        Color32::from_rgb(46, 15, 14)
    };
    let color_b = if on {
        Color32::from_rgb(244, 39, 20)
    } else {
        Color32::from_rgb(43, 14, 13)
    };
    if on {
        ellipse(
            p,
            t,
            cx,
            cy + CHARACTER_HEIGHT * 0.02,
            1.05,
            0.92,
            Color32::from_rgba_unmultiplied(255, 35, 18, 18),
            Stroke::NONE,
        );
    }
    let half = 0.72;
    for (dy, color) in [(-0.17, color_a), (0.17, color_b)] {
        p.line_segment(
            [t.pos(cx - half, cy + dy), t.pos(cx + half, cy + dy)],
            Stroke::new(t.s(0.18), color),
        );
    }
}

fn display_cells(value: &str) -> [char; CHARACTER_COUNT] {
    let mut cells = [' '; CHARACTER_COUNT];
    if value.is_empty() {
        return cells;
    }

    // Program listings may eventually supply already-spaced numeric fields.
    // Preserve those verbatim instead of forcing numeric mantissa formatting.
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

fn ellipse(
    p: &Painter,
    t: Transform,
    cx: f32,
    cy: f32,
    rx: f32,
    ry: f32,
    fill: Color32,
    stroke: Stroke,
) {
    let points = (0..32)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 32.0;
            t.pos(cx + rx * a.cos(), cy + ry * a.sin())
        })
        .collect();
    p.add(Shape::convex_polygon(points, fill, stroke));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn hardware_dimensions_follow_hp_5082_7405_datasheet() {
        assert!((CHARACTER_PITCH / UNITS_PER_MM - 3.81).abs() < 0.0001);
        assert!((CHARACTER_HEIGHT / UNITS_PER_MM - 2.794).abs() < 0.0001);
        assert!((ASSEMBLY_WIDTH / UNITS_PER_MM - 57.15).abs() < 0.001);
        assert_eq!(CHARACTER_COUNT, 15);
        assert_eq!(MODULE_COUNT * CHARACTERS_PER_MODULE, CHARACTER_COUNT);
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
        assert_eq!(&display_cells("0")[..3], &[' ', '0', '.']);
        assert_eq!(display_cells(""), [' '; 15]);
    }

    #[test]
    fn classic_numeric_masks_match_seven_segment_wiring() {
        assert_eq!(segment_mask('1'), SEG_B | SEG_C);
        assert_eq!(segment_mask('8'), 0x7f);
        assert_eq!(segment_mask('-'), SEG_G);
        assert_eq!(segment_mask('.'), 0);
    }
}
