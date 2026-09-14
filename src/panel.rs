use eframe::egui::{Align2, Color32, Painter, Pos2, Sense, Stroke, Ui, Vec2};

use crate::hp67::{Hp67State, RunMode, UiEvent};

use crate::ui::geometry::{
    scale_for, Transform, CARD_RAIL_BOTTOM, CARD_RAIL_TOP, CASE_SIDE_EXPANSION, DESIGN_H, DESIGN_W,
    DISPLAY_BOUNDS, SWITCH_CENTER_Y,
};

const PANEL: Color32 = Color32::from_rgb(39, 40, 36);
const PANEL_DARK: Color32 = Color32::from_rgb(25, 25, 23);
const CASE_GREEN: Color32 = Color32::from_rgb(79, 82, 58);
const CASE_GREEN_DARK: Color32 = Color32::from_rgb(59, 61, 44);
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
            t.rect(44.0, SWITCH_CENTER_Y - 10.0, 91.0, 28.0),
            ui.make_persistent_id("hp67-power-switch"),
            Sense::click(),
        );
        if power_response.clicked() {
            events.push(UiEvent::TogglePower);
        }
        draw_power_switch(&painter, t, state.power_on, power_response.hovered());

        let mode_response = ui.interact(
            t.rect(166.0, SWITCH_CENTER_Y - 10.0, 121.0, 28.0),
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
    use crate::ui::materials::{antialiased_surface as surface, outline, panel_left, panel_right};
    // Molded case and rolled metal rim share a tapered, nonrectangular perimeter.
    for (inset, color) in [
        (0.0, Color32::from_rgb(27, 28, 22)),
        (1.0, Color32::from_rgb(116, 116, 92)),
        (2.4, Color32::from_rgb(81, 83, 64)),
        (4.0, CASE_GREEN),
        (7.5, CASE_GREEN_DARK),
        (10.0, Color32::from_rgb(27, 30, 28)),
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
    draw_continuous_rim(p, t);
    // Recessed card/legend rail above the A-E row.
    surface(
        p,
        t,
        CARD_RAIL_TOP,
        CARD_RAIL_BOTTOM,
        panel_left,
        panel_right,
        Color32::from_rgb(42, 43, 39),
        5.0,
        2.0,
        |_, v| 9.0 * (-v * 12.0).exp() - 5.0 * (-(1.0 - v) * 18.0).exp(),
    );
    for y in [CARD_RAIL_TOP + 1.0, 151.0, CARD_RAIL_BOTTOM - 0.5] {
        p.line_segment(
            [t.pos(panel_left(y), y), t.pos(panel_right(y), y)],
            Stroke::new(t.s(0.65), PANEL_DARK),
        );
    }
}

fn draw_display(p: &Painter, t: Transform, text_value: &str) {
    use crate::ui::materials::{panel_left, panel_right, surface};
    let [left, top, width, height] = DISPLAY_BOUNDS;
    let bottom = top + height;
    surface(
        p,
        t,
        top,
        bottom,
        |y| {
            left + if y < top + 4.0 {
                4.0 - (16.0 - (y - top - 4.0).powi(2)).max(0.0).sqrt()
            } else {
                0.0
            }
        },
        |y| {
            left + width
                - if y < top + 4.0 {
                    4.0 - (16.0 - (y - top - 4.0).powi(2)).max(0.0).sqrt()
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
        bottom,
        SWITCH_CENTER_Y - 12.0,
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
        SWITCH_CENTER_Y - 10.0,
        SWITCH_CENTER_Y + 11.0,
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
        SWITCH_CENTER_Y + 11.0,
        CARD_RAIL_TOP,
        panel_left,
        panel_right,
        Color32::from_rgb(18, 19, 16),
        2.0,
        2.0,
        |_, v| -5.0 * (1.0 - v),
    );
    // Die positions remain fixed regardless of the length of the number.
    draw_segment_string(
        p,
        t,
        text_value,
        left + 18.6,
        top + (height - 13.6) * 0.5,
        width - 37.2,
    );
}

fn draw_power_switch(p: &Painter, t: Transform, on: bool, hovered: bool) {
    switch_label(p, t, 40.0, SWITCH_CENTER_Y, "OFF");
    switch_label(p, t, 109.0, SWITCH_CENTER_Y, "ON");
    draw_slider(p, t, 65.0, SWITCH_CENTER_Y - 1.0, 39.0, on, hovered);
}
fn draw_mode_switch(p: &Painter, t: Transform, mode: RunMode, hovered: bool) {
    switch_label(p, t, 172.0, SWITCH_CENTER_Y, "W/PRGM");
    switch_label(p, t, 277.0, SWITCH_CENTER_Y, "RUN");
    draw_slider(
        p,
        t,
        223.0,
        SWITCH_CENTER_Y - 1.0,
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
    use crate::ui::materials::antialiased_surface as surface;
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
    use crate::ui::materials::antialiased_surface as surface;
    // Rounded roll into the lower face: near-parallel sides, no triangular
    // inset corners. The illuminated shoulder darkens continuously underneath.
    surface(
        p,
        t,
        581.8,
        609.0,
        |y| 22.0 - CASE_SIDE_EXPANSION + 2.0 * ((y - 581.8) / 27.2).powi(2),
        |y| 308.0 + CASE_SIDE_EXPANSION - 2.0 * ((y - 581.8) / 27.2).powi(2),
        Color32::from_rgb(55, 57, 45),
        1.2,
        0.65,
        |u, v| 6.0 * (1.0 - u) + 11.0 * (-((v - 0.10) / 0.10).powi(2)).exp() - 20.0 * v,
    );
    surface(
        p,
        t,
        584.0,
        602.0,
        |y| 28.0 - CASE_SIDE_EXPANSION + (y - 584.0) * 0.07,
        |y| 302.0 + CASE_SIDE_EXPANSION - (y - 584.0) * 0.07,
        Color32::from_rgb(24, 25, 23),
        0.8,
        0.6,
        |_, v| 4.0 * (1.0 - v) - 8.0 * v,
    );
}

// One path, one stroke width and one brightness through the folded corners.
fn rim_points() -> Vec<Pos2> {
    let outline = crate::ui::materials::outline_points(13.75);
    let mut points = Vec::new();
    for i in 0..outline.len() {
        let a = outline[i];
        let b = outline[(i + 1) % outline.len()];
        if a.1 <= 583.5 {
            points.push(Pos2::new(a.0, a.1));
        }
        if (a.1 <= 583.5) != (b.1 <= 583.5) {
            let f = (583.5 - a.1) / (b.1 - a.1);
            points.push(Pos2::new(a.0 + (b.0 - a.0) * f, 583.5));
            if a.1 <= 583.5 {
                points.push(Pos2::new(303.0 + CASE_SIDE_EXPANSION, 605.0));
                points.push(Pos2::new(27.0 - CASE_SIDE_EXPANSION, 605.0));
            }
        }
    }
    points
}
fn draw_continuous_rim(p: &Painter, t: Transform) {
    let points: Vec<_> = rim_points()
        .into_iter()
        .map(|point| t.pos(point.x, point.y))
        .collect();
    // Identical metal cross-section on the top, sides, bend and bottom. No
    // independently shaded inset bands to add apparent width along the sides.
    for (width, color) in [
        (4.4, Color32::from_rgb(69, 75, 70)),
        (3.0, Color32::from_rgb(139, 145, 137)),
        (1.4, SILVER_LIGHT),
    ] {
        p.add(eframe::egui::Shape::closed_line(
            points.clone(),
            Stroke::new(t.s(width), color),
        ));
    }
}

// Affine projection: every horizontal baseline and diagonal remains straight.
fn nose_point(x: f32, y: f32) -> (f32, f32) {
    (38.0 + x * 254.0 / 268.0, 584.4 + y * 0.90)
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
        Stroke::NONE,
    );
    // Measured reference badge: silver hairline around a black/blue rectangle;
    // a narrow vintage hp, with curved shoulders and bowl, on a silver disc.
    let ink = Color32::from_rgb(31, 33, 31);
    let silver = Color32::from_rgb(185, 187, 172);
    quad(6.0, 3.0, 31.0, 15.0, ink, Stroke::new(t.s(0.42), silver));
    quad(
        21.5,
        3.25,
        15.2,
        14.5,
        Color32::from_rgb(42, 109, 156),
        Stroke::NONE,
    );
    let circle = (0..128)
        .map(|i| {
            let a = i as f32 * std::f32::consts::TAU / 128.0;
            point(20.0 + 6.25 * a.cos(), 10.5 + 7.0 * a.sin())
        })
        .collect();
    p.add(Shape::convex_polygon(circle, silver, Stroke::NONE));
    // Open h shoulder and curved p counter; thinner than the modern HP mark.
    let line = |a: (f32, f32), b: (f32, f32)| {
        p.line_segment(
            [point(a.0, a.1), point(b.0, b.1)],
            Stroke::new(t.s(0.72), ink),
        );
    };
    let curve = |coords: [(f32, f32); 4]| {
        p.add(Shape::CubicBezier(
            eframe::egui::epaint::CubicBezierShape::from_points_stroke(
                coords.map(|(x, y)| point(x, y)),
                false,
                Color32::TRANSPARENT,
                Stroke::new(t.s(0.72), ink),
            ),
        ));
    };
    line((19.6, 3.7), (16.3, 14.6));
    curve([(18.2, 8.9), (21.9, 6.2), (22.0, 7.7), (21.3, 9.6)]);
    line((21.3, 9.6), (19.7, 14.5));
    line((23.3, 7.8), (20.0, 18.0));
    curve([(23.3, 7.8), (27.6, 6.7), (26.5, 12.8), (21.9, 12.4)]);
    // Explicit tracking, with the model number as one unspaced pair.
    let mut cursor = 54.0;
    for ch in "HEWLETT-PACKARD".chars() {
        let value = ch.to_string();
        if ch == '-' {
            // The original nameplate separates the names with a centered dot.
            let center = cursor + glyphs::width(&value, 8.0, 400) * 0.5;
            p.circle_filled(point(center, 10.2), t.s(0.65), silver);
            cursor += glyphs::width(&value, 8.0, 400) + 6.1;
            continue;
        }
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
