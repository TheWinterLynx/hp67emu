//! Authentic HP-67 Classic-series LED emission for the photographed display.
//!
//! The physical HP-67 display is a 15-position assembly made from three
//! end-stackable five-character HP 5082-7405-style modules.  Character pitch is
//! 3.81 mm (.150 in), magnified character height is 2.794 mm (.110 in), and the
//! centered decimal occupies its own character position.  Each logical segment
//! is formed by three narrow emitting bars; the decimal die uses two bars.
//!
//! In the photorealistic renderer the photograph already provides the red
//! contrast filter, bezel, reflections, module/lens structure and black level.
//! This module therefore adds only the light emitted by the real LED geometry.

use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Stroke};

pub(crate) const CHARACTER_COUNT: usize = 15;
const MODULE_COUNT: usize = 3;
const CHARACTERS_PER_MODULE: usize = 5;
const CHARACTER_PITCH_MM: f32 = 3.81;
const CHARACTER_HEIGHT_MM: f32 = 2.794;

// The measured vector reconstruction uses 614 logical units for HP's published
// 152.4 mm case length and a 282 x 57 display opening.  Keeping those reference
// dimensions here preserves the physical LED-to-glass ratio while the final
// mapping is made directly into the photographed display rectangle.
const REFERENCE_UNITS_PER_MM: f32 = 614.0 / 152.4;
const REFERENCE_DISPLAY_WIDTH: f32 = 282.0;
const REFERENCE_DISPLAY_HEIGHT: f32 = 57.0;
const CHARACTER_PITCH: f32 = CHARACTER_PITCH_MM * REFERENCE_UNITS_PER_MM;
const CHARACTER_HEIGHT: f32 = CHARACTER_HEIGHT_MM * REFERENCE_UNITS_PER_MM;
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
        pos2(self.rect.left() + x * self.sx, self.rect.top() + y * self.sy)
    }

    fn stroke(self, reference_width: f32, minimum: f32) -> f32 {
        (reference_width * self.stroke_scale).max(minimum)
    }
}

/// Paint only authentic LED emission into the already-photographed glass.
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
    // The three bars should merge optically at ordinary size but remain visible
    // when enlarged, as on the real monolithic HP LED die.
    let offsets = [-0.22_f32, 0.0, 0.22];
    let inks = [
        Color32::from_rgb(244, 38, 22),
        Color32::from_rgb(255, 49, 25),
        Color32::from_rgb(235, 31, 18),
    ];
    let glow = Color32::from_rgba_unmultiplied(255, 32, 18, 27);

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
    // 5082-7405 center-decimal die: two short horizontal emitting bars in its
    // own character position, not a dot attached to the previous numeral.
    let half = 0.72;
    let glow = Color32::from_rgba_unmultiplied(255, 32, 18, 25);
    p.line_segment(
        [t.pos(cx - half, cy), t.pos(cx + half, cy)],
        Stroke::new(t.stroke(0.85, 0.72), glow),
    );
    for (dy, color) in [
        (-0.17, Color32::from_rgb(255, 49, 25)),
        (0.17, Color32::from_rgb(235, 31, 18)),
    ] {
        p.line_segment(
            [t.pos(cx - half, cy + dy), t.pos(cx + half, cy + dy)],
            Stroke::new(t.stroke(0.18, 0.36), color),
        );
    }
}

fn display_cells(value: &str) -> [char; CHARACTER_COUNT] {
    let mut cells = [' '; CHARACTER_COUNT];
    if value.is_empty() {
        return cells;
    }

    // A future calculator core may supply an already-spaced hardware field.
    // Preserve such fields verbatim rather than reformatting them.
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

    // The temporary UI model can emit operator characters even though the real
    // HP-67 display cannot.  Do not invent non-hardware glyphs for those cases.
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
    fn hardware_dimensions_follow_hp_5082_7405_datasheet() {
        assert!((CHARACTER_PITCH / REFERENCE_UNITS_PER_MM - 3.81).abs() < 0.0001);
        assert!((CHARACTER_HEIGHT / REFERENCE_UNITS_PER_MM - 2.794).abs() < 0.0001);
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
        assert_eq!(&display_cells("0")[..3], &[' ', '0', '.']);
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
