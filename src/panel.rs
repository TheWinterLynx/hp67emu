use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Sense, Stroke, Ui, Vec2};

use crate::hp67::{Hp67State, RunMode, UiEvent};

use crate::ui::geometry::{scale_for, Transform, DESIGN_H, DESIGN_W};

const PANEL: Color32 = Color32::from_rgb(33, 43, 53);
const PANEL_DARK: Color32 = Color32::from_rgb(25, 25, 23);
const CASE_GREEN: Color32 = Color32::from_rgb(48, 58, 46);
const CASE_GREEN_DARK: Color32 = Color32::from_rgb(55, 61, 43);
const SILVER: Color32 = Color32::from_rgb(171, 172, 166);
const SILVER_LIGHT: Color32 = Color32::from_rgb(225, 227, 221);
const WHITE: Color32 = Color32::from_rgb(230, 233, 229);

pub struct Hp67Panel;

impl Hp67Panel {
    pub fn show(ui: &mut Ui, state: &Hp67State) -> Vec<UiEvent> {
        let available = ui.available_size();
        let (host_rect, _) = ui.allocate_exact_size(available, Sense::hover());
        let scale = scale_for(host_rect.size());
        if scale <= 0.0 {
            return Vec::new();
        }

        let panel_size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(
                host_rect.center().x - panel_size.x * 0.5,
                host_rect.center().y - panel_size.y * 0.5,
            ),
            scale,
        };
        let painter = ui.painter_at(host_rect);
        let mut events = Vec::new();

        draw_chassis(&painter, t);
        draw_display(&painter, t, state.display_text());

        let power_response = ui.interact(
            t.rect(44.0, 92.0, 91.0, 28.0),
            ui.make_persistent_id("hp67-power-switch"),
            Sense::click(),
        );
        if power_response.clicked() {
            events.push(UiEvent::TogglePower);
        }
        draw_power_switch(&painter, t, state.power_on, power_response.hovered());

        let mode_response = ui.interact(
            t.rect(166.0, 92.0, 121.0, 28.0),
            ui.make_persistent_id("hp67-mode-switch"),
            Sense::click(),
        );
        if mode_response.clicked() {
            events.push(UiEvent::ToggleMode);
        }
        draw_mode_switch(&painter, t, state.mode, mode_response.hovered());

        events.extend(crate::ui::keyboard::show(ui, host_rect));
        draw_branding(&painter, t);
        events
    }
}

fn draw_chassis(p: &Painter, t: Transform) {
    p.rect_filled(
        t.rect(4.0, 3.0, 322.0, 614.0),
        t.s(19.0),
        Color32::from_rgb(15, 16, 13),
    );
    p.rect_filled(t.rect(7.0, 1.0, 316.0, 615.0), t.s(18.0), CASE_GREEN_DARK);
    p.rect_filled(t.rect(11.0, 2.0, 308.0, 612.0), t.s(15.0), CASE_GREEN);
    p.rect_filled(
        t.rect(23.0, 5.0, 284.0, 609.0),
        t.s(11.0),
        Color32::from_rgb(108, 108, 100),
    );
    p.rect_filled(t.rect(26.0, 7.0, 278.0, 605.0), t.s(9.0), SILVER_LIGHT);
    p.rect_filled(t.rect(30.0, 11.0, 270.0, 596.0), t.s(7.0), PANEL);
    p.rect_stroke(
        t.rect(28.0, 8.5, 274.0, 601.0),
        t.s(8.0),
        Stroke::new(t.s(1.2), SILVER),
    );
    p.line_segment(
        [t.pos(31.0, 166.0), t.pos(299.0, 166.0)],
        Stroke::new(t.s(2.0), PANEL_DARK),
    );
}

fn draw_display(p: &Painter, t: Transform, text_value: &str) {
    // The glass runs into the inner perimeter; there is no separate silver box.
    p.rect_filled(
        t.rect(31.0, 12.0, 268.0, 64.0),
        t.s(3.0),
        Color32::from_rgb(19, 20, 26),
    );
    p.line_segment(
        [t.pos(32.0, 76.0), t.pos(298.0, 76.0)],
        Stroke::new(t.s(1.0), PANEL_DARK),
    );
    p.rect_filled(
        t.rect(31.0, 87.0, 268.0, 24.0),
        0.0,
        Color32::from_rgb(35, 45, 48),
    );
    p.rect_filled(t.rect(31.0, 113.0, 268.0, 6.0), 0.0, PANEL_DARK);
    if !text_value.is_empty() {
        draw_segment_string(p, t, text_value, 43.0, 31.0, 240.0);
    }
}

fn draw_power_switch(p: &Painter, t: Transform, on: bool, hovered: bool) {
    label(p, t, 42.0, 101.0, Align2::LEFT_CENTER, 7.4, WHITE, "OFF");
    label(p, t, 127.0, 101.0, Align2::RIGHT_CENTER, 7.4, WHITE, "ON");
    draw_slider(p, t, 68.0, 96.0, 40.0, on, hovered);
}

fn draw_mode_switch(p: &Painter, t: Transform, mode: RunMode, hovered: bool) {
    label(
        p,
        t,
        166.0,
        101.0,
        Align2::LEFT_CENTER,
        7.2,
        WHITE,
        "W/PRGM",
    );
    label(p, t, 286.0, 101.0, Align2::RIGHT_CENTER, 7.2, WHITE, "RUN");
    draw_slider(
        p,
        t,
        225.0,
        96.0,
        39.0,
        matches!(mode, RunMode::Run),
        hovered,
    );
}

fn draw_slider(p: &Painter, t: Transform, x: f32, y: f32, w: f32, right: bool, hovered: bool) {
    let track = t.rect(x, y, w, 8.0);
    p.rect_filled(track, t.s(1.0), Color32::from_rgb(7, 8, 8));
    p.rect_stroke(
        track,
        t.s(1.0),
        Stroke::new(
            t.s(if hovered { 1.0 } else { 0.7 }),
            if hovered {
                Color32::from_rgb(156, 158, 151)
            } else {
                Color32::from_rgb(69, 70, 66)
            },
        ),
    );
    for i in 0..10 {
        let xx = x + 2.0 + i as f32 * (w - 4.0) / 9.0;
        p.line_segment(
            [t.pos(xx, y + 1.2), t.pos(xx, y + 6.8)],
            Stroke::new(t.s(0.45), Color32::from_rgb(52, 53, 50)),
        );
    }
    let knob_x = if right { x + w - 13.0 } else { x + 2.0 };
    p.rect_filled(
        t.rect(knob_x + 0.8, y + 0.4, 11.0, 10.0),
        t.s(1.3),
        Color32::from_rgb(19, 20, 19),
    );
    p.rect_filled(
        t.rect(knob_x, y - 1.0, 11.0, 10.0),
        t.s(1.3),
        Color32::from_rgb(102, 104, 99),
    );
    p.line_segment(
        [t.pos(knob_x + 1.5, y), t.pos(knob_x + 9.5, y)],
        Stroke::new(t.s(0.75), Color32::from_rgb(160, 161, 155)),
    );
}

fn draw_branding(p: &Painter, t: Transform) {
    p.rect_filled(
        t.rect(31.0, 586.0, 268.0, 17.0),
        0.0,
        Color32::from_rgb(16, 20, 22),
    );
    p.rect_stroke(
        t.rect(31.0, 586.0, 268.0, 17.0),
        0.0,
        Stroke::new(t.s(0.65), Color32::from_rgb(189, 191, 185)),
    );
    p.rect_filled(
        t.rect(43.0, 588.0, 31.0, 11.0),
        0.0,
        Color32::from_rgb(47, 91, 128),
    );
    p.circle_filled(
        t.pos(55.5, 593.5),
        t.s(5.1),
        Color32::from_rgb(176, 169, 144),
    );
    // Slanted connected h/p mark, expressed in the same logical coordinates.
    for path in [
        vec![(51.5, 597.0), (54.2, 589.0)],
        vec![(52.8, 593.0), (56.0, 593.0), (54.6, 597.0)],
        vec![
            (56.0, 599.0),
            (59.2, 589.5),
            (61.0, 589.5),
            (59.8, 593.4),
            (57.8, 593.4),
        ],
    ] {
        p.add(eframe::egui::Shape::line(
            path.into_iter().map(|(x, y)| t.pos(x, y)).collect(),
            Stroke::new(t.s(1.0), Color32::from_rgb(30, 37, 45)),
        ));
    }
    centered(
        p,
        t,
        178.0,
        594.0,
        7.3,
        Color32::from_rgb(205, 209, 205),
        "H E W L E T T - P A C K A R D   6 7",
    );
}

fn centered(p: &Painter, t: Transform, x: f32, y: f32, size: f32, color: Color32, value: &str) {
    label(p, t, x, y, Align2::CENTER_CENTER, size, color, value);
}

fn label(
    p: &Painter,
    t: Transform,
    x: f32,
    y: f32,
    align: Align2,
    size: f32,
    color: Color32,
    value: &str,
) {
    p.text(
        t.pos(x, y),
        align,
        value,
        FontId::proportional(t.s(size)),
        color,
    );
}

const SEG_A: u8 = 1 << 0;
const SEG_B: u8 = 1 << 1;
const SEG_C: u8 = 1 << 2;
const SEG_D: u8 = 1 << 3;
const SEG_E: u8 = 1 << 4;
const SEG_F: u8 = 1 << 5;
const SEG_G: u8 = 1 << 6;
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
        'E' | 'e' => SEG_A | SEG_D | SEG_E | SEG_F | SEG_G,
        _ => 0,
    }
}

fn draw_segment_string(p: &Painter, t: Transform, value: &str, x: f32, y: f32, width: f32) {
    let cell_w = 17.0;
    let visible_cells = value.chars().filter(|&c| c != '.').count().min(12);
    let total_w = visible_cells as f32 * cell_w;
    let mut cursor_x = x + (width - total_w).max(0.0);
    let mut last_digit_x: Option<f32> = None;
    for ch in value.chars().take(16) {
        if ch == '.' {
            if let Some(dx) = last_digit_x {
                p.circle_filled(
                    t.pos(dx + 11.5, y + 19.5),
                    t.s(0.85),
                    Color32::from_rgb(248, 72, 57),
                );
            }
            continue;
        }
        draw_segment_digit(p, t, cursor_x, y, ch);
        last_digit_x = Some(cursor_x);
        cursor_x += cell_w;
    }
}

fn draw_segment_digit(p: &Painter, t: Transform, x: f32, y: f32, ch: char) {
    let mask = segment_mask(ch);
    // Narrow LED dies with tapered ends and a low, red halo under the glass.
    for (bit, sx, sy, vertical) in [
        (SEG_A, 1.4, 0.0, false),
        (SEG_B, 8.0, 1.5, true),
        (SEG_C, 8.0, 10.5, true),
        (SEG_D, 1.4, 19.0, false),
        (SEG_E, 0.0, 10.5, true),
        (SEG_F, 0.0, 1.5, true),
        (SEG_G, 1.4, 9.5, false),
    ] {
        let on = mask & bit != 0;
        let shape = [
            (0.0, 0.7),
            (0.7, 0.0),
            (5.5, 0.0),
            (6.2, 0.7),
            (5.5, 1.4),
            (0.7, 1.4),
        ];
        let points: Vec<_> = shape
            .iter()
            .map(|&(a, b)| {
                if vertical {
                    t.pos(x + sx + b, y + sy + a * 1.25)
                } else {
                    t.pos(x + sx + a, y + sy + b)
                }
            })
            .collect();
        if on {
            p.add(eframe::egui::Shape::convex_polygon(
                points.clone(),
                Color32::from_rgb(91, 24, 24),
                Stroke::new(t.s(1.1), Color32::from_rgba_premultiplied(56, 5, 3, 70)),
            ));
        }
        p.add(eframe::egui::Shape::convex_polygon(
            points,
            if on {
                Color32::from_rgb(249, 65, 39)
            } else {
                Color32::from_rgb(38, 23, 28)
            },
            Stroke::NONE,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn aspect_ratio_scaling_is_uniform() {
        assert!((scale_for(Vec2::new(660.0, 1240.0)) - 2.0).abs() < f32::EPSILON);
        assert!((scale_for(Vec2::new(1000.0, 900.0)) - (900.0 / 620.0)).abs() < 0.0001);
    }

    #[test]
    fn zero_sized_host_has_zero_scale() {
        assert_eq!(scale_for(Vec2::new(0.0, 620.0)), 0.0);
        assert_eq!(scale_for(Vec2::new(330.0, 0.0)), 0.0);
    }

    #[test]
    fn seven_segment_masks_are_sane() {
        assert_eq!(segment_mask('1'), SEG_B | SEG_C);
        assert_eq!(segment_mask('8'), 0x7f);
        assert_eq!(segment_mask('-'), SEG_G);
    }
}
