use eframe::egui::{Align2, Color32, Painter, Rect, Shape, Stroke, Ui, Vec2};

use super::geometry::{KeySpec, KeyStyle, SubAlign, Transform, DESIGN_H, DESIGN_W, KEYS};

use crate::hp67::UiEvent;
const WHITE: Color32 = Color32::from_rgb(218, 228, 215);
const YELLOW: Color32 = Color32::from_rgb(185, 183, 75);
const CYAN: Color32 = Color32::from_rgb(74, 174, 220);
const DARK: Color32 = Color32::from_rgb(27, 29, 28);

pub fn show(ui: &Ui, host: Rect) -> Vec<UiEvent> {
    let scale = (host.width() / DESIGN_W).min(host.height() / DESIGN_H);
    if scale <= 0.0 {
        return Vec::new();
    }
    let size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
    let t = Transform {
        origin: host.center() - size * 0.5,
        scale,
    };
    let p = ui.painter_at(host);
    draw_legends(&p, t);
    let mut events = Vec::new();
    for key in KEYS {
        let hit = t.rect(
            key.cx - key.w * 0.5 - 1.0,
            key.y - 1.0,
            key.w + 2.0,
            key.h + 4.0,
        );
        let response = ui.interact(
            hit,
            ui.make_persistent_id(("hp67-key", key.id)),
            eframe::egui::Sense::click(),
        );
        if response.clicked() {
            events.push(UiEvent::Key(key.action));
        }
        let down = response.is_pointer_button_down_on();
        let press = ui.ctx().animate_bool_with_time(
            response.id.with("travel"),
            down,
            if down { 0.050 } else { 0.070 },
        );
        draw_key(&p, t, *key, press);
    }
    events
}

#[cfg(test)]
mod tests {
    use super::*;
    use eframe::egui::{self, Event, Modifiers, PointerButton, Pos2};

    #[test]
    fn every_visible_key_including_its_outer_edge_dispatches_its_own_action() {
        assert_eq!(KEYS.len(), 35);
        for key in KEYS {
            // Exercise the wide numeric-key edge missed by the old underlying matrix.
            let pos = Pos2::new(key.cx + key.w * 0.5 - 0.5, key.y + 5.0);
            let ctx = egui::Context::default();
            let mut received = Vec::new();
            for frame in 0..4 {
                let mut events = vec![Event::PointerMoved(pos)];
                if frame == 1 || frame == 3 {
                    events.push(Event::PointerButton {
                        pos,
                        button: PointerButton::Primary,
                        pressed: frame == 1,
                        modifiers: Modifiers::NONE,
                    });
                }
                let _ = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::new(330.0, 620.0))),
                        events,
                        time: Some(frame as f64 * 0.1),
                        ..Default::default()
                    },
                    |ctx| {
                        egui::CentralPanel::default()
                            .frame(egui::Frame::none())
                            .show(ctx, |ui| {
                                received.extend(show(ui, ui.available_rect_before_wrap()));
                            });
                    },
                );
                if frame < 3 {
                    assert!(received.is_empty(), "early/repeated event: {}", key.id);
                }
            }
            assert_eq!(received, vec![UiEvent::Key(key.action)], "{}", key.id);
        }
    }

    #[test]
    fn pressed_artwork_is_only_a_rigid_physical_pixel_translation() {
        for scale in [1.0, 1.37, 2.0] {
            for density in [1.0, 1.25, 2.0] {
                for key in KEYS {
                    let ctx = egui::Context::default();
                    ctx.set_pixels_per_point(density);
                    let render = |press| {
                        ctx.run(
                            egui::RawInput {
                                screen_rect: Some(Rect::from_min_size(
                                    Pos2::ZERO,
                                    Vec2::splat(3000.0),
                                )),
                                ..Default::default()
                            },
                            |ctx| {
                                let painter = ctx.layer_painter(egui::LayerId::background());
                                draw_key(
                                    &painter,
                                    Transform {
                                        origin: Pos2::new(10.0, 10.0),
                                        scale,
                                    },
                                    *key,
                                    press,
                                );
                            },
                        )
                        .shapes
                    };
                    let rest = render(0.0);
                    for press in [0.35, 1.0] {
                        let held = render(press);
                        assert_eq!(rest.len(), held.len(), "{}", key.id);
                        assert_eq!(rest[0].shape, held[0].shape, "socket moved");
                        let travel = (2.8 * press * scale * ctx.pixels_per_point()).round()
                            / ctx.pixels_per_point();
                        let mut expected = rest[1..].to_vec();
                        for shape in &mut expected {
                            shape.shape.translate(Vec2::new(0.0, travel));
                        }
                        let a = ctx.tessellate(expected, ctx.pixels_per_point());
                        let b = ctx.tessellate(held[1..].to_vec(), ctx.pixels_per_point());
                        assert_eq!(a.len(), b.len());
                        for (a, b) in a.iter().zip(&b) {
                            let (
                                egui::epaint::Primitive::Mesh(a),
                                egui::epaint::Primitive::Mesh(b),
                            ) = (&a.primitive, &b.primitive)
                            else {
                                panic!("unexpected callback")
                            };
                            assert_eq!(a.indices, b.indices, "topology changed: {}", key.id);
                            assert_eq!(a.vertices.len(), b.vertices.len());
                            for (a, b) in a.vertices.iter().zip(&b.vertices) {
                                assert_eq!(a.color, b.color);
                                assert_eq!(a.uv, b.uv);
                                assert!(
                                    (a.pos - b.pos).length() < 0.002,
                                    "geometry changed: {}",
                                    key.id
                                );
                            }
                        }
                    }
                }
            }
        }
    }
}

fn draw_key(p: &Painter, t: Transform, key: KeySpec, press: f32) {
    let x = key.cx - key.w * 0.5;

    // Mechanical travel is quantized to whole physical pixels.  This keeps the
    // rasterized legends bit-for-bit stable while the complete keycap moves.
    let pixels_per_point = p.ctx().pixels_per_point();
    let travel_px = (2.8 * press * t.scale * pixels_per_point).round() / pixels_per_point;
    let travel = if t.scale > 0.0 {
        travel_px / t.scale
    } else {
        0.0
    };

    // Fixed socket beneath the moving key. Nothing printed on the key is ever
    // scaled, recomposed or substituted during a press.
    p.rect_filled(
        t.rect(x - 0.9, key.y + 0.6, key.w + 1.8, key.h + 2.0),
        t.s(2.1),
        Color32::from_rgb(19, 20, 19),
    );

    let local = Transform {
        origin: t.pos(key.cx, key.y),
        scale: t.scale,
    }
    .translated_y(travel);
    draw_artwork(
        p,
        local,
        KeySpec {
            cx: 0.0,
            y: 0.0,
            ..key
        },
    );
}

// Artwork is defined entirely in keycap-local coordinates. No press state is
// passed here: every face, skirt and printed mark uses this exact same path.
fn draw_artwork(p: &Painter, kt: Transform, key: KeySpec) {
    let (face, front, side, text, subtext) = palette(key.style);
    let x = -key.w * 0.5;
    let skirt_top = key.y + key.top_h - 0.4;
    let bottom = key.y + key.h;
    let skirt_h = bottom - skirt_top;

    use super::materials::{shade, surface};
    // One rounded plastic body. The shoulder highlight wraps onto the front;
    // the skirt is a sloping surface, not a second button stacked underneath.
    for i in (1..=3).rev() {
        p.rect_filled(
            kt.rect(x + i as f32 * 0.45, 1.0, key.w, key.h + 0.5),
            kt.s(2.5),
            Color32::from_black_alpha(18 + i * 9),
        );
    }
    p.rect_filled(kt.rect(x, 0.0, key.w, key.h), kt.s(2.3), side);
    let l = |y: f32| {
        let r = 2.5;
        let corner = if y < r {
            r - (r * r - (y - r) * (y - r)).max(0.0).sqrt()
        } else {
            0.0
        };
        x + 0.6 + corner
    };
    let r = |y: f32| -l(y);
    surface(p, kt, 0.0, skirt_top, l, r, face, 0.9, 1.2, |u, v| {
        15.0 * (-v * 25.0).exp() + 25.0 * (-u * 45.0).exp()
            - 19.0 * (-(1.0 - u) * 28.0).exp()
            - 8.0 * (-(1.0 - v) * 14.0).exp()
            + 3.0 * (1.0 - v)
    });
    surface(
        p,
        kt,
        skirt_top,
        bottom - 0.5,
        |y| x + 1.0 + 1.3 * (y - skirt_top) / skirt_h,
        |y| -x - 1.0 - 1.3 * (y - skirt_top) / skirt_h,
        front,
        1.1,
        1.0,
        |u, v| 12.0 * (-v * 18.0).exp() - 10.0 * v - 14.0 * (-(1.0 - u) * 20.0).exp(),
    );
    // Molded front recess and a narrow, curved reflection down the left shoulder.
    p.rect_stroke(
        kt.rect(x + 2.2, skirt_top + 0.6, key.w - 4.4, skirt_h - 1.5),
        kt.s(2.1),
        Stroke::new(kt.s(0.5), shade(front, 19.0)),
    );
    p.line_segment(
        [kt.pos(x + 1.6, 3.1), kt.pos(x + 1.4, skirt_top - 2.0)],
        Stroke::new(
            kt.s(0.9),
            Color32::from_rgba_unmultiplied(255, 255, 230, 155),
        ),
    );

    // Crucially, the exact same label renderer is used in both states.  The
    // translated transform moves every vector/text element as one rigid object.
    draw_main(p, kt, key, text, key.y + key.top_h * 0.47);
    if key.sub.is_some() {
        draw_sub(p, kt, key, subtext, skirt_top + skirt_h * 0.55);
    }
}

fn draw_main(p: &Painter, t: Transform, key: KeySpec, color: Color32, cy: f32) {
    match key.id {
        "sigma" => {
            sigma(p, t, key.cx - 2.4, cy, 9.7, color);
            bold_txt(p, t, key.cx + 5.0, cy, 10.1, color, "+");
        }
        "multiply" => cross(p, t, key.cx, cy, 4.3, color),
        "divide" => divide(p, t, key.cx, cy, 4.3, color),
        "enter" => {
            bold_txt(p, t, key.cx - 6.0, cy, 10.8, color, "ENTER");
            vertical_arrow(p, t, key.cx + 28.0, cy, 3.1, true, color);
        }
        _ => {
            let size = match key.id {
                "f" | "g" | "h" => 13.4,
                "7" | "8" | "9" | "4" | "5" | "6" | "1" | "2" | "3" | "0" => 15.5,
                "minus" | "plus" => 15.2,
                "rs" => 11.8,
                "a" | "b" | "c" | "d" | "e" => 11.9,
                _ => 11.6,
            };
            bold_txt(p, t, key.cx, cy, size, color, key.main);
        }
    }
}

fn draw_sub(p: &Painter, t: Transform, key: KeySpec, color: Color32, cy: f32) {
    let sub = key.sub.unwrap_or("");
    match key.id {
        "sigma" => {
            sigma(p, t, key.cx - 2.1, cy, 6.5, color);
            bold_txt(p, t, key.cx + 3.9, cy, 6.7, color, "−");
        }
        "indirect" => swap(p, t, key.cx, cy, "x", "I", 6.7, color),
        "7" => swap(p, t, key.cx, cy, "x", "y", 6.9, color),
        "8" => r_arrow(p, t, key.cx, cy, false, 6.8, color),
        "9" => r_arrow(p, t, key.cx, cy, true, 6.8, color),
        "4" => one_over_x(p, t, key.cx, cy, 6.8, color),
        "5" => power(p, t, key.cx, cy, "y", "x", 6.8, color, color),
        "2" => pi(p, t, key.cx, cy, 7.0, color),
        "enter" => {
            let x = match key.sub_align {
                SubAlign::Right => key.cx + 30.0,
                SubAlign::Center => key.cx,
            };
            bold_txt_aligned(
                p,
                t,
                x,
                cy,
                6.7,
                color,
                "DEG",
                if matches!(key.sub_align, SubAlign::Right) {
                    Align2::RIGHT_CENTER
                } else {
                    Align2::CENTER_CENTER
                },
            );
        }
        _ => {
            let size = if key.w >= 70.0 {
                6.7
            } else if sub.len() >= 5 {
                6.6
            } else {
                7.0
            };
            bold_txt(p, t, key.cx, cy, size, color, sub);
        }
    }
}

fn draw_legends(p: &Painter, t: Transform) {
    one_over_x(p, t, 58.0, 151.8, 10.5, WHITE);
    sqrt_x(p, t, 112.0, 151.7, 10.6, WHITE);
    power(p, t, 166.0, 151.7, "y", "x", 10.4, WHITE, WHITE);
    r_arrow(p, t, 220.0, 151.7, false, 10.2, WHITE);
    swap(p, t, 274.0, 151.7, "x", "y", 10.0, WHITE);
    for (x, s) in [
        (58.0, "a"),
        (112.0, "b"),
        (166.0, "c"),
        (220.0, "d"),
        (274.0, "e"),
    ] {
        bold_txt(p, t, x, 214.0, 7.9, YELLOW, s);
    }
    xbar(p, t, 55.0, 267.0, 7.4, YELLOW);
    bold_txt(p, t, 71.7, 267.0, 7.8, CYAN, "s");
    bold_txt(p, t, 107.5, 267.0, 7.8, YELLOW, "GSB");
    bold_txt(p, t, 126.2, 267.0, 7.8, CYAN, "f");
    bold_txt(p, t, 156.0, 267.0, 7.8, YELLOW, "FIX");
    bold_txt(p, t, 179.0, 267.0, 7.8, CYAN, "SCI");
    bold_txt(p, t, 220.0, 267.0, 7.8, YELLOW, "RND");
    bold_txt(p, t, 258.5, 267.0, 7.8, YELLOW, "LBL");
    bold_txt(p, t, 280.0, 267.0, 7.8, CYAN, "f");
    bold_txt(p, t, 158.0, 321.0, 7.8, YELLOW, "DSZ");
    bold_txt(p, t, 180.7, 321.0, 7.8, CYAN, "(i)");
    bold_txt(p, t, 209.5, 321.0, 7.8, YELLOW, "ISZ");
    bold_txt(p, t, 232.0, 321.0, 7.8, CYAN, "(i)");
    bold_txt(p, t, 61.0, 372.0, 7.9, YELLOW, "W/DATA");
    bold_txt(p, t, 112.0, 372.0, 7.9, CYAN, "MERGE");
    swap_two_color(p, t, 166.0, 372.0, "P", "S", 7.7, YELLOW, YELLOW);
    bold_txt(p, t, 220.0, 372.0, 7.9, YELLOW, "CL REG");
    bold_txt(p, t, 274.0, 372.0, 7.9, YELLOW, "CL PRGM");
    eq_pair(p, t, 56.0, 423.0, false);
    bold_txt(p, t, 108.0, 423.0, 8.0, YELLOW, "LN");
    power(p, t, 128.0, 423.0, "e", "x", 7.9, CYAN, CYAN);
    bold_txt(p, t, 181.0, 423.0, 8.0, YELLOW, "LOG");
    power(p, t, 208.0, 423.0, "10", "x", 7.9, CYAN, CYAN);
    sqrt_x(p, t, 252.0, 423.0, 8.0, YELLOW);
    power(p, t, 279.0, 423.0, "x", "2", 7.9, CYAN, CYAN);
    eq_pair(p, t, 56.0, 475.0, true);
    inverse_trig(p, t, 112.0, 475.0, "SIN");
    inverse_trig(p, t, 194.0, 475.0, "COS");
    inverse_trig(p, t, 270.0, 475.0, "TAN");
    relation_pair(p, t, 56.0, 526.0, '<', true);
    swap_two_color(p, t, 119.0, 526.0, "R", "P", 7.7, YELLOW, CYAN);
    swap_two_color(p, t, 194.0, 526.0, "D", "R", 7.7, YELLOW, CYAN);
    swap_two_color(p, t, 270.0, 526.0, "H", "H.MS", 7.3, YELLOW, CYAN);
    relation_pair(p, t, 56.0, 577.0, '>', false);
    bold_txt(p, t, 106.0, 577.0, 7.9, YELLOW, "%");
    bold_txt(p, t, 126.5, 577.0, 7.9, CYAN, "%CH");
    bold_txt(p, t, 181.0, 577.0, 7.9, YELLOW, "INT");
    bold_txt(p, t, 207.0, 577.0, 7.9, CYAN, "FRAC");
    bold_txt(p, t, 253.0, 577.0, 7.9, YELLOW, "−x−");
    bold_txt(p, t, 281.0, 577.0, 7.9, CYAN, "STK");
}

fn palette(style: KeyStyle) -> (Color32, Color32, Color32, Color32, Color32) {
    match style {
        KeyStyle::Olive => (
            Color32::from_rgb(139, 143, 86),
            Color32::from_rgb(109, 112, 64),
            Color32::from_rgb(64, 68, 43),
            WHITE,
            DARK,
        ),
        KeyStyle::Orange => (
            Color32::from_rgb(207, 166, 60),
            Color32::from_rgb(197, 126, 18),
            Color32::from_rgb(145, 84, 7),
            DARK,
            DARK,
        ),
        KeyStyle::Blue => (
            Color32::from_rgb(52, 160, 207),
            Color32::from_rgb(31, 137, 162),
            Color32::from_rgb(18, 92, 109),
            DARK,
            DARK,
        ),
        KeyStyle::White => (
            Color32::from_rgb(234, 231, 211),
            Color32::from_rgb(187, 186, 164),
            Color32::from_rgb(148, 153, 151),
            DARK,
            DARK,
        ),
        KeyStyle::Black => (
            Color32::from_rgb(22, 24, 25),
            Color32::from_rgb(10, 11, 12),
            Color32::from_rgb(4, 5, 5),
            WHITE,
            WHITE,
        ),
    }
}

fn bold_txt(p: &Painter, t: Transform, x: f32, y: f32, size: f32, color: Color32, s: &str) {
    bold_txt_aligned(p, t, x, y, size, color, s, Align2::CENTER_CENTER)
}
fn bold_txt_aligned(
    p: &Painter,
    t: Transform,
    x: f32,
    y: f32,
    size: f32,
    color: Color32,
    s: &str,
    align: Align2,
) {
    super::glyphs::text(
        p,
        t.pos(x, y),
        t.s(size),
        color,
        s,
        align,
        if size < 8.1 {
            700
        } else if s.chars().all(|c| c.is_ascii_digit()) {
            400
        } else {
            600
        },
    );
}
fn cross(p: &Painter, t: Transform, cx: f32, cy: f32, h: f32, c: Color32) {
    p.line_segment(
        [t.pos(cx - h, cy - h), t.pos(cx + h, cy + h)],
        Stroke::new(t.s(1.05), c),
    );
    p.line_segment(
        [t.pos(cx - h, cy + h), t.pos(cx + h, cy - h)],
        Stroke::new(t.s(1.05), c),
    );
}
fn divide(p: &Painter, t: Transform, cx: f32, cy: f32, h: f32, c: Color32) {
    p.line_segment(
        [t.pos(cx - h, cy), t.pos(cx + h, cy)],
        Stroke::new(t.s(1.05), c),
    );
    p.circle_filled(t.pos(cx, cy - 4.1), t.s(0.9), c);
    p.circle_filled(t.pos(cx, cy + 4.1), t.s(0.9), c);
}
fn vertical_arrow(p: &Painter, t: Transform, cx: f32, cy: f32, h: f32, up: bool, c: Color32) {
    let direction = if up { -1.0 } else { 1.0 };
    p.line_segment(
        [
            t.pos(cx, cy - direction * h),
            t.pos(cx, cy + direction * h * 0.3),
        ],
        Stroke::new(t.s(h * 0.5), c),
    );
    triangle(p, t, cx, cy + direction * h * 0.4, h * 0.8, up, c);
}
fn triangle(p: &Painter, t: Transform, cx: f32, cy: f32, h: f32, up: bool, c: Color32) {
    let d = if up { -1.0 } else { 1.0 };
    p.add(Shape::convex_polygon(
        vec![
            t.pos(cx, cy + d * h),
            t.pos(cx - h, cy - d * h * 0.72),
            t.pos(cx + h, cy - d * h * 0.72),
        ],
        c,
        Stroke::NONE,
    ));
}
fn sigma(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    let s = size / 8.0;
    let l = cx - 4.0 * s;
    let r = cx + 4.0 * s;
    let top = cy - 4.0 * s;
    let bot = cy + 4.0 * s;
    let st = Stroke::new(t.s(0.95 * s), c);
    p.line_segment([t.pos(l, top), t.pos(r, top)], st);
    p.line_segment([t.pos(l, top), t.pos(cx + 0.8 * s, cy)], st);
    p.line_segment([t.pos(cx + 0.8 * s, cy), t.pos(l, bot)], st);
    p.line_segment([t.pos(l, bot), t.pos(r, bot)], st);
}
fn pi(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    let s = size / 7.0;
    let st = Stroke::new(t.s(0.8 * s), c);
    p.line_segment(
        [
            t.pos(cx - 4.0 * s, cy - 2.7 * s),
            t.pos(cx + 4.0 * s, cy - 2.7 * s),
        ],
        st,
    );
    p.line_segment(
        [
            t.pos(cx - 2.1 * s, cy - 2.7 * s),
            t.pos(cx - 2.1 * s, cy + 2.8 * s),
        ],
        st,
    );
    p.line_segment(
        [
            t.pos(cx + 2.1 * s, cy - 2.7 * s),
            t.pos(cx + 2.1 * s, cy + 2.8 * s),
        ],
        st,
    );
}
fn one_over_x(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    let s = size / 9.0;
    bold_txt(p, t, cx - 6.0 * s, cy, size, c, "1");
    p.line_segment(
        [
            t.pos(cx - 1.6 * s, cy + 4.0 * s),
            t.pos(cx + 2.3 * s, cy - 4.0 * s),
        ],
        Stroke::new(t.s(0.85 * s), c),
    );
    bold_txt(p, t, cx + 6.0 * s, cy, size, c, "x");
}
fn sqrt_x(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    let s = size / 9.0;
    let x0 = cx - 8.2 * s;
    let st = Stroke::new(t.s(1.0 * s), c);
    p.line_segment(
        [t.pos(x0, cy + 0.3 * s), t.pos(x0 + 2.5 * s, cy + 4.0 * s)],
        st,
    );
    p.line_segment(
        [
            t.pos(x0 + 2.5 * s, cy + 4.0 * s),
            t.pos(x0 + 5.8 * s, cy - 5.0 * s),
        ],
        st,
    );
    p.line_segment(
        [
            t.pos(x0 + 5.8 * s, cy - 5.0 * s),
            t.pos(x0 + 15.0 * s, cy - 5.0 * s),
        ],
        Stroke::new(t.s(0.8 * s), c),
    );
    bold_txt(p, t, cx + 3.5 * s, cy + 0.4 * s, size, c, "x");
}
fn power(
    p: &Painter,
    t: Transform,
    cx: f32,
    cy: f32,
    base: &str,
    exp: &str,
    size: f32,
    bc: Color32,
    ec: Color32,
) {
    let bw = if base.len() > 1 { 5.5 } else { 2.6 };
    bold_txt(p, t, cx - bw, cy + 1.0, size, bc, base);
    bold_txt(p, t, cx + bw + 2.4, cy - size * 0.42, size * 0.65, ec, exp);
}
fn r_arrow(p: &Painter, t: Transform, cx: f32, cy: f32, up: bool, size: f32, c: Color32) {
    bold_txt(p, t, cx - 3.2, cy, size, c, "R");
    if size > 8.0 {
        vertical_arrow(p, t, cx + 5.8, cy + 0.2, size * 0.3, up, c);
    } else {
        triangle(p, t, cx + 5.8, cy + 0.2, size * 0.3, up, c);
    }
}
fn swap(p: &Painter, t: Transform, cx: f32, cy: f32, l: &str, r: &str, size: f32, c: Color32) {
    swap_two_color(p, t, cx, cy, l, r, size, c, c)
}
fn swap_two_color(
    p: &Painter,
    t: Transform,
    cx: f32,
    cy: f32,
    l: &str,
    r: &str,
    size: f32,
    lc: Color32,
    rc: Color32,
) {
    let span = if r.len() > 1 { 11.0 } else { 8.0 };
    bold_txt(p, t, cx - span, cy, size, lc, l);
    bold_txt(p, t, cx + span, cy, size, rc, r);
    double_arrow(p, t, cx, cy, size * 0.52, if lc == rc { lc } else { CYAN });
}
fn double_arrow(p: &Painter, t: Transform, cx: f32, cy: f32, h: f32, c: Color32) {
    let x1 = cx - h;
    let x2 = cx + h;
    let yu = cy - 1.35;
    let yd = cy + 1.35;
    let st = Stroke::new(t.s(0.7), c);
    p.line_segment([t.pos(x1, yu), t.pos(x2, yu)], st);
    p.line_segment([t.pos(x2 - 2.0, yu - 1.4), t.pos(x2, yu)], st);
    p.line_segment([t.pos(x2 - 2.0, yu + 1.4), t.pos(x2, yu)], st);
    p.line_segment([t.pos(x2, yd), t.pos(x1, yd)], st);
    p.line_segment([t.pos(x1 + 2.0, yd - 1.4), t.pos(x1, yd)], st);
    p.line_segment([t.pos(x1 + 2.0, yd + 1.4), t.pos(x1, yd)], st);
}
fn xbar(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    bold_txt(p, t, cx, cy + 0.4, size, c, "x");
    p.line_segment(
        [t.pos(cx - 3.4, cy - 4.2), t.pos(cx + 3.4, cy - 4.2)],
        Stroke::new(t.s(0.8), c),
    );
}
fn inverse_trig(p: &Painter, t: Transform, cx: f32, cy: f32, name: &str) {
    bold_txt(p, t, cx - 2.0, cy, 7.9, YELLOW, name);
    bold_txt(p, t, cx + 13.0, cy - 4.1, 5.2, CYAN, "−1");
}
fn eq_pair(p: &Painter, t: Transform, cx: f32, cy: f32, ne: bool) {
    bold_txt(p, t, cx - 13.0, cy, 7.5, YELLOW, "x");
    if ne {
        not_eq(p, t, cx - 5.0, cy, YELLOW)
    } else {
        bold_txt(p, t, cx - 5.0, cy, 7.5, YELLOW, "=")
    };
    bold_txt(p, t, cx + 1.0, cy, 7.5, YELLOW, "0");
    bold_txt(p, t, cx + 7.0, cy, 7.5, CYAN, "x");
    if ne {
        not_eq(p, t, cx + 15.0, cy, CYAN)
    } else {
        bold_txt(p, t, cx + 15.0, cy, 7.5, CYAN, "=")
    };
    bold_txt(p, t, cx + 22.0, cy, 7.5, CYAN, "y");
}
fn not_eq(p: &Painter, t: Transform, cx: f32, cy: f32, c: Color32) {
    let st = Stroke::new(t.s(0.75), c);
    p.line_segment([t.pos(cx - 2.8, cy - 1.5), t.pos(cx + 2.8, cy - 1.5)], st);
    p.line_segment([t.pos(cx - 2.8, cy + 1.5), t.pos(cx + 2.8, cy + 1.5)], st);
    p.line_segment([t.pos(cx - 2.3, cy + 3.2), t.pos(cx + 2.3, cy - 3.2)], st);
}
fn relation_pair(p: &Painter, t: Transform, cx: f32, cy: f32, op: char, incl: bool) {
    let os = if op == '<' { "<" } else { ">" };
    bold_txt(p, t, cx - 13.0, cy, 7.4, YELLOW, "x");
    bold_txt(p, t, cx - 6.4, cy, 7.4, YELLOW, os);
    bold_txt(p, t, cx, cy, 7.4, YELLOW, "0");
    bold_txt(p, t, cx + 7.0, cy, 7.4, CYAN, "x");
    relop(p, t, cx + 15.0, cy, op, incl, CYAN);
    bold_txt(p, t, cx + 23.0, cy, 7.4, CYAN, "y");
}
fn relop(p: &Painter, t: Transform, cx: f32, cy: f32, op: char, incl: bool, c: Color32) {
    let f = if op == '<' { 1.0 } else { -1.0 };
    let st = Stroke::new(t.s(0.75), c);
    p.line_segment([t.pos(cx + 2.5 * f, cy - 3.0), t.pos(cx - 2.0 * f, cy)], st);
    p.line_segment([t.pos(cx - 2.0 * f, cy), t.pos(cx + 2.5 * f, cy + 3.0)], st);
    if incl {
        p.line_segment([t.pos(cx - 2.5, cy + 4.3), t.pos(cx + 2.8, cy + 4.3)], st);
    }
}
