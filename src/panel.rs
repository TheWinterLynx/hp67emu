use eframe::egui::{Align2, Color32, Painter, Pos2, Sense, Stroke, Ui, Vec2};

use crate::hp67::{Hp67State, RunMode, UiEvent};

use crate::ui::geometry::{scale_for, Transform, DESIGN_H, DESIGN_W};

const PANEL: Color32 = Color32::from_rgb(39, 40, 36);
const PANEL_DARK: Color32 = Color32::from_rgb(25, 25, 23);
const CASE_GREEN: Color32 = Color32::from_rgb(79, 82, 58);
const CASE_GREEN_DARK: Color32 = Color32::from_rgb(59, 61, 44);
const SILVER: Color32 = Color32::from_rgb(171, 172, 166);
const SILVER_LIGHT: Color32 = Color32::from_rgb(225, 227, 221);
const WHITE: Color32 = Color32::from_rgb(230, 233, 229);

pub struct Hp67Panel;

impl Hp67Panel {
    pub fn show(ui: &mut Ui, state: &Hp67State) -> Vec<UiEvent> {
        // Do not let egui replace small circular marks (division/decimal dots)
        // with cached font-atlas discs as the window scale changes.
        ui.ctx()
            .tessellation_options_mut(|options| options.prerasterized_discs = false);
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
    use crate::ui::materials::{outline, panel_left, panel_right, surface};
    // Molded case and rolled metal rim share a tapered, nonrectangular perimeter.
    for (inset, color) in [
        (0.0, Color32::from_rgb(27, 28, 22)),
        (1.0, Color32::from_rgb(116, 116, 92)),
        (2.4, Color32::from_rgb(81, 83, 64)),
        (4.0, CASE_GREEN),
        (7.5, CASE_GREEN_DARK),
        (10.0, Color32::from_rgb(27, 30, 28)),
        (11.5, Color32::from_rgb(115, 122, 116)),
        (13.0, SILVER_LIGHT),
        (14.5, SILVER),
        (16.0, Color32::from_rgb(62, 68, 64)),
        (17.0, PANEL),
    ] {
        if inset >= 10.0 {
            // The upper rolled rim ends at the fold; the nose supplies its own
            // sloping continuation. Do not leave a second old rim behind it.
            let clip = eframe::egui::Rect::from_min_max(
                p.clip_rect().min,
                Pos2::new(p.clip_rect().max.x, t.pos(0.0, 583.5).y),
            );
            outline(&p.with_clip_rect(clip), t, inset, color);
        } else {
            outline(p, t, inset, color);
        }
    }
    surface(
        p,
        t,
        29.0,
        583.5,
        panel_left,
        panel_right,
        PANEL,
        3.5,
        1.8,
        |u, v| 3.0 * (1.0 - u) - 5.0 * v + 2.0 * (v * 8.0).sin(),
    );
    draw_lower_case(p, t);
    // Recessed card/legend rail above the A-E row.
    surface(
        p,
        t,
        118.0,
        166.0,
        panel_left,
        panel_right,
        Color32::from_rgb(42, 43, 39),
        5.0,
        2.0,
        |_, v| 9.0 * (-v * 12.0).exp() - 5.0 * (-(1.0 - v) * 18.0).exp(),
    );
    for y in [119.0, 139.0, 165.5] {
        p.line_segment(
            [t.pos(panel_left(y), y), t.pos(panel_right(y), y)],
            Stroke::new(t.s(0.65), PANEL_DARK),
        );
    }
}

fn draw_display(p: &Painter, t: Transform, text_value: &str) {
    use crate::ui::materials::{panel_left, panel_right, surface};
    surface(
        p,
        t,
        24.0,
        73.0,
        |y| {
            30.4 + if y < 28.0 {
                4.0 - (16.0 - (y - 28.0).powi(2)).max(0.0).sqrt()
            } else {
                0.0
            }
        },
        |y| {
            299.6
                - if y < 28.0 {
                    4.0 - (16.0 - (y - 28.0).powi(2)).max(0.0).sqrt()
                } else {
                    0.0
                }
        },
        Color32::from_rgb(30, 16, 15),
        0.7,
        2.0,
        |u, v| 3.0 * (1.0 - v) + 2.0 * (1.0 - u),
    );
    surface(
        p,
        t,
        73.0,
        84.0,
        panel_left,
        panel_right,
        Color32::from_rgb(47, 46, 41),
        5.0,
        1.4,
        |_, v| 9.0 * (1.0 - v) - 6.0 * v,
    );
    surface(
        p,
        t,
        87.0,
        110.0,
        panel_left,
        panel_right,
        Color32::from_rgb(47, 49, 43),
        5.0,
        1.4,
        |_, v| 5.0 * (1.0 - v) - 5.0 * v,
    );
    surface(
        p,
        t,
        110.0,
        117.0,
        panel_left,
        panel_right,
        Color32::from_rgb(18, 19, 16),
        2.0,
        2.0,
        |_, v| -5.0 * (1.0 - v),
    );
    // Die positions remain fixed regardless of the length of the number.
    draw_segment_string(p, t, text_value, 49.0, 40.0, 232.0);
}

fn draw_power_switch(p: &Painter, t: Transform, on: bool, hovered: bool) {
    switch_label(p, t, 48.0, 99.0, "OFF");
    switch_label(p, t, 115.0, 99.0, "ON");
    draw_slider(p, t, 70.0, 98.0, 39.0, on, hovered);
}
fn draw_mode_switch(p: &Painter, t: Transform, mode: RunMode, hovered: bool) {
    switch_label(p, t, 167.0, 99.0, "W/PRGM");
    switch_label(p, t, 268.0, 99.0, "RUN");
    draw_slider(
        p,
        t,
        223.0,
        98.0,
        39.0,
        matches!(mode, RunMode::Run),
        hovered,
    );
}
fn switch_label(p: &Painter, t: Transform, x: f32, y: f32, value: &str) {
    crate::ui::glyphs::text(
        p,
        t.pos(x, y),
        t.s(8.6),
        WHITE,
        value,
        Align2::LEFT_CENTER,
        700,
    );
}
fn draw_slider(p: &Painter, t: Transform, x: f32, y: f32, w: f32, right: bool, hovered: bool) {
    use crate::ui::materials::surface;
    p.rect_filled(t.rect(x, y, w, 5.2), t.s(0.6), Color32::from_rgb(8, 9, 7));
    p.line_segment(
        [t.pos(x, y + 5.3), t.pos(x + w, y + 5.3)],
        Stroke::new(t.s(0.5), Color32::from_rgb(80, 79, 63)),
    );
    let knob_x = if right { x + w - 14.0 } else { x + 1.0 };
    p.rect_filled(
        t.rect(knob_x - 0.6, y - 3.1, 13.8, 9.0),
        t.s(1.1),
        Color32::from_rgb(16, 19, 18),
    );
    surface(
        p,
        t,
        y - 3.0,
        y + 3.7,
        |_| knob_x,
        |_| knob_x + 12.4,
        Color32::from_rgb(33, 39, 39),
        0.0,
        1.0,
        |_, v| 29.0 * (1.0 - v) - 8.0 * v,
    );
    // Ribs are on the moving black cursor, not across the empty slot.
    for i in 0..7 {
        let xx = knob_x + 0.9 + i as f32 * 1.7;
        p.line_segment(
            [t.pos(xx, y - 2.6), t.pos(xx - 0.5, y + 2.0)],
            Stroke::new(t.s(0.55), Color32::from_rgb(110, 117, 110)),
        );
        p.line_segment(
            [t.pos(xx + 0.6, y - 2.1), t.pos(xx + 0.1, y + 2.2)],
            Stroke::new(t.s(0.5), Color32::from_rgb(12, 15, 15)),
        );
    }
    if hovered {
        p.rect_stroke(
            t.rect(x - 0.8, y - 3.8, w + 1.6, 10.0),
            t.s(1.0),
            Stroke::new(t.s(0.4), Color32::from_gray(96)),
        );
    }
}

// The keyboard deck ends at the fold. The nose, cheeks and return lip belong
// to the chassis; the thin printed nameplate is inset into that larger face.
fn draw_lower_case(p: &Painter, t: Transform) {
    use crate::ui::materials::surface;
    use eframe::egui::Shape;
    let facet = |points: &[(f32, f32)], color| {
        p.add(Shape::convex_polygon(
            points.iter().map(|&(x, y)| t.pos(x, y)).collect(),
            color,
            Stroke::NONE,
        ));
    };
    facet(
        &[(26.0, 583.5), (28.0, 583.5), (33.0, 604.0), (24.0, 610.0)],
        Color32::from_rgb(64, 68, 55),
    );
    facet(
        &[
            (302.0, 583.5),
            (304.0, 583.5),
            (306.0, 610.0),
            (297.0, 604.0),
        ],
        Color32::from_rgb(35, 39, 31),
    );
    surface(
        p,
        t,
        583.5,
        604.0,
        |y| 28.0 + (y - 583.5) * 5.0 / 20.5,
        |y| 302.0 - (y - 583.5) * 5.0 / 20.5,
        Color32::from_rgb(48, 49, 42),
        0.8,
        1.0,
        |u, v| 9.0 * (1.0 - u) - 19.0 * v,
    );
    // Straight fold, diagonal continuation of the rolled rim, and lower return.
    p.line_segment(
        [t.pos(28.0, 583.5), t.pos(302.0, 583.5)],
        Stroke::new(t.s(0.65), Color32::from_rgb(92, 94, 82)),
    );
    for (a, b, c) in [
        ((26.0, 583.5), (31.0, 604.5), SILVER_LIGHT),
        ((304.0, 583.5), (299.0, 604.5), SILVER),
    ] {
        p.line_segment([t.pos(a.0, a.1), t.pos(b.0, b.1)], Stroke::new(t.s(1.7), c));
    }
    facet(
        &[(31.0, 604.0), (299.0, 604.0), (306.0, 610.0), (24.0, 610.0)],
        Color32::from_rgb(48, 51, 39),
    );
    p.line_segment(
        [t.pos(31.0, 604.5), t.pos(299.0, 604.5)],
        Stroke::new(t.s(1.0), SILVER),
    );
}

// Affine projection: every horizontal baseline and diagonal remains straight.
fn nose_point(x: f32, y: f32) -> (f32, f32) {
    (38.0 + x * 254.0 / 268.0, 585.6 + y * 0.76)
}

fn draw_branding(p: &Painter, t: Transform) {
    use crate::ui::glyphs;
    use eframe::egui::Shape;
    let point = |x, y| {
        let (x, y) = nose_point(x, y);
        t.pos(x, y)
    };
    let quad = |x: f32, y: f32, w: f32, h: f32, color, stroke| {
        p.add(Shape::convex_polygon(
            vec![
                point(x, y),
                point(x + w, y),
                point(x + w, y + h),
                point(x, y + h),
            ],
            color,
            stroke,
        ));
    };
    // A fine aluminum outline inset into the dark sloping face.
    quad(
        1.0,
        1.5,
        266.0,
        18.0,
        Color32::from_rgb(24, 25, 23),
        Stroke::new(t.s(0.35), Color32::from_rgb(113, 116, 105)),
    );
    // Period badge: dark left field, blue right field, silver circular hp mark.
    let ink = Color32::from_rgb(25, 31, 33);
    let silver = Color32::from_rgb(190, 190, 175);
    quad(10.0, 3.0, 31.0, 15.0, ink, Stroke::new(t.s(0.4), silver));
    quad(
        27.0,
        3.2,
        13.8,
        14.6,
        Color32::from_rgb(38, 112, 156),
        Stroke::NONE,
    );
    let circle = (0..64)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 64.0;
            point(23.0 + 6.9 * a.cos(), 10.5 + 6.9 * a.sin())
        })
        .collect();
    p.add(Shape::convex_polygon(circle, silver, Stroke::NONE));
    // Parallel slanted stems and open counters of the vintage circular mark.
    for path in [
        vec![(22.0, 3.4), (18.3, 14.8)],
        vec![(20.1, 9.1), (23.3, 9.1), (21.5, 14.8)],
        vec![(22.8, 17.5), (26.3, 6.4)],
        vec![(25.8, 8.0), (28.6, 8.0), (27.4, 11.8), (24.6, 11.8)],
    ] {
        p.add(Shape::line(
            path.into_iter().map(|(x, y)| point(x, y)).collect(),
            Stroke::new(t.s(1.12), ink),
        ));
    }
    // Explicit tracking, with the model number as one unspaced pair.
    let mut cursor = 54.0;
    for ch in "HEWLETT-PACKARD".chars() {
        let value = ch.to_string();
        let mut mesh = glyphs::text_mesh(
            Pos2::new(cursor, 10.2),
            8.0,
            silver,
            &value,
            Align2::LEFT_CENTER,
            400,
            p.ctx().pixels_per_point() * t.scale,
        );
        for vertex in &mut mesh.vertices {
            vertex.pos = point(vertex.pos.x, vertex.pos.y);
        }
        p.add(Shape::mesh(mesh));
        cursor += glyphs::width(&value, 8.0, 400) + 6.1;
    }
    let mut mesh = glyphs::text_mesh(
        Pos2::new(246.0, 10.2),
        10.0,
        silver,
        "67",
        Align2::CENTER_CENTER,
        400,
        p.ctx().pixels_per_point() * t.scale,
    );
    for vertex in &mut mesh.vertices {
        vertex.pos = point(vertex.pos.x, vertex.pos.y);
    }
    p.add(Shape::mesh(mesh));
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

fn display_cells(value: &str) -> [char; 15] {
    let mut cells = [' '; 15];
    if value.is_empty() {
        return cells;
    }
    let (mantissa, exponent) = value.split_once(['e', 'E']).unwrap_or((value, ""));
    cells[0] = if mantissa.starts_with('-') { '-' } else { ' ' };
    let magnitude = mantissa.trim_start_matches(['-', '+']);
    for (slot, ch) in cells[1..12].iter_mut().zip(magnitude.chars()) {
        *slot = ch;
    }
    if !magnitude.contains('.') && magnitude.len() < 11 {
        cells[1 + magnitude.len()] = '.';
    }
    if !exponent.is_empty() {
        cells[12] = if exponent.starts_with('-') { '-' } else { ' ' };
        let digits: Vec<_> = exponent
            .trim_start_matches(['-', '+'])
            .chars()
            .rev()
            .take(2)
            .collect();
        for (i, ch) in digits.into_iter().enumerate() {
            cells[14 - i] = ch;
        }
    }
    cells
}
fn draw_segment_string(p: &Painter, t: Transform, value: &str, x: f32, y: f32, width: f32) {
    let cells = display_cells(value);
    for (index, ch) in cells.into_iter().enumerate() {
        let tx = x + index as f32 * width / 15.0;
        let die = Transform {
            origin: t.pos(tx, y),
            scale: t.scale * 0.68,
        };
        draw_segment_digit(p, die, 0.0, 0.0, ch);
        if ch == '.' {
            for (radius, alpha) in [(1.9, 15), (1.2, 45), (0.65, 255)] {
                p.circle_filled(
                    t.pos(tx + 3.2, y + 12.6),
                    t.s(radius),
                    Color32::from_rgba_unmultiplied(255, 58, 29, alpha),
                );
            }
        }
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
                Color32::from_rgb(33, 18, 17)
            },
            Stroke::NONE,
        ));
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn complete_panel_uses_only_untextured_vector_geometry_at_large_sizes() {
        use eframe::egui::{
            self,
            epaint::{Primitive, WHITE_UV},
        };
        for scale in [1.0, 4.0] {
            let ctx = egui::Context::default();
            let output = ctx.run(
                egui::RawInput {
                    screen_rect: Some(egui::Rect::from_min_size(
                        Pos2::ZERO,
                        Vec2::new(330.0, 620.0) * scale,
                    )),
                    ..Default::default()
                },
                |ctx| {
                    egui::CentralPanel::default()
                        .frame(egui::Frame::none())
                        .show(ctx, |ui| {
                            Hp67Panel::show(ui, &Hp67State::default());
                        });
                },
            );
            let primitives = ctx.tessellate(output.shapes, ctx.pixels_per_point());
            assert!(!primitives.is_empty());
            for primitive in primitives {
                let Primitive::Mesh(mesh) = primitive.primitive else {
                    panic!("non-vector callback")
                };
                assert!(
                    mesh.vertices.iter().all(|v| v.uv == WHITE_UV),
                    "raster texture used in panel"
                );
            }
        }
    }

    #[test]
    fn display_has_separate_decimal_sign_and_exponent_cells() {
        assert_eq!(
            display_cells("-1.234567890e-67"),
            ['-', '1', '.', '2', '3', '4', '5', '6', '7', '8', '9', '0', '-', '6', '7']
        );
        assert_eq!(&display_cells("3.14")[..5], &[' ', '3', '.', '1', '4']);
        assert_eq!(&display_cells("0")[..3], &[' ', '0', '.']);
        assert_eq!(display_cells(""), [' '; 15]);
    }

    #[test]
    fn resizing_has_no_upper_scale_limit_and_keeps_the_panel_inside_host() {
        for available in [
            Vec2::new(165.0, 310.0),
            Vec2::new(330.0, 2000.0),
            Vec2::new(3840.0, 2160.0),
            Vec2::new(3300.0, 6200.0),
        ] {
            let scale = scale_for(available);
            let size = Vec2::new(DESIGN_W, DESIGN_H) * scale;
            assert!(size.x <= available.x + 0.001 && size.y <= available.y + 0.001);
            assert!((size.x / size.y - DESIGN_W / DESIGN_H).abs() < 0.00001);
        }
        assert_eq!(scale_for(Vec2::new(3300.0, 6200.0)), 10.0);
    }

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
