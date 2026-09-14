use eframe::egui::{Align2, Color32, Painter, Rect, Shape, Stroke, Ui, Vec2};

use super::geometry::{KeySpec, KeyStyle, SubAlign, Transform, DESIGN_H, DESIGN_W, KEYS};

use crate::hp67::UiEvent;
const WHITE: Color32 = Color32::from_rgb(218, 228, 215);
const YELLOW: Color32 = Color32::from_rgb(185, 183, 75);
const CYAN: Color32 = Color32::from_rgb(74, 174, 220);
const LEGEND_SIZE: f32 = 8.8;
const TOP_LEGEND_SIZE: f32 = 10.5;
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
    fn exchange_spacing_tracks_the_printed_letters_at_both_legend_sizes() {
        for size in [6.7, 6.9, 7.7, 10.0] {
            let (left, right, arrow) = exchange_positions("x", "y", size);
            let left_edge = left - print_width("x", size) * 0.5;
            let right_edge = right + print_width("y", size) * 0.5;
            assert!((left_edge + right_edge).abs() < 0.00001);
            assert!(right_edge - left_edge < size * 1.85);
            assert!(arrow - size * 0.25 > left + print_width("x", size) * 0.5);
            assert!(arrow + size * 0.25 < right - print_width("y", size) * 0.5);
        }
    }

    #[test]
    fn exchange_mark_has_two_filled_heads_and_no_shafts() {
        let ctx = egui::Context::default();
        let output = ctx.run(egui::RawInput::default(), |ctx| {
            exchange_heads(
                &ctx.layer_painter(egui::LayerId::background()),
                Transform {
                    origin: Pos2::ZERO,
                    scale: 1.0,
                },
                20.0,
                20.0,
                10.0,
                WHITE,
                WHITE,
            );
        });
        assert_eq!(output.shapes.len(), 2);
        for shape in output.shapes {
            let Shape::Path(path) = shape.shape else {
                panic!("exchange shaft or unexpected shape")
            };
            assert_eq!(path.points.len(), 3);
            assert!(path.closed);
            assert_eq!(path.stroke, Stroke::NONE);
            assert_eq!(path.fill, WHITE);
        }
    }

    #[test]
    fn all_printed_key_marks_fit_inside_their_own_face() {
        for key in KEYS {
            let key = KeySpec {
                cx: 0.0,
                y: 0.0,
                ..*key
            };
            let skirt = key.top_h - 0.4;
            for front in [false, true] {
                if front && key.sub.is_none() {
                    continue;
                }
                let ctx = egui::Context::default();
                let output = ctx.run(
                    egui::RawInput {
                        screen_rect: Some(Rect::from_min_size(Pos2::ZERO, Vec2::splat(200.0))),
                        ..Default::default()
                    },
                    |ctx| {
                        ctx.tessellation_options_mut(|o| o.prerasterized_discs = false);
                        let p = ctx.layer_painter(egui::LayerId::background());
                        let t = Transform {
                            origin: Pos2::new(100.0, 50.0),
                            scale: 1.0,
                        };
                        if front {
                            draw_sub(&p, t, key, DARK, skirt + (key.h - skirt) * 0.5);
                        } else {
                            draw_main(&p, t, key, WHITE, skirt * 0.5);
                        }
                    },
                );
                let safe = Rect::from_min_max(
                    Pos2::new(
                        100.0 - key.w * 0.5 + 2.0,
                        50.0 + if front { skirt + 0.25 } else { 1.0 },
                    ),
                    Pos2::new(
                        100.0 + key.w * 0.5 - 2.0,
                        50.0 + if front { key.h - 0.25 } else { skirt - 1.0 },
                    ),
                );
                for primitive in ctx.tessellate(output.shapes, 1.0) {
                    let egui::epaint::Primitive::Mesh(mesh) = primitive.primitive else {
                        panic!()
                    };
                    for v in mesh.vertices.iter().filter(|v| v.color.a() >= 128) {
                        assert!(
                            safe.contains(v.pos),
                            "{} front={front}: {:?} outside {:?}",
                            key.id,
                            v.pos,
                            safe
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn panel_math_ink_matches_the_shared_legend_cap_height() {
        for letter in ["x", "y", "e"] {
            let ctx = egui::Context::default();
            let output = ctx.run(egui::RawInput::default(), |ctx| {
                matched_math_text(
                    &ctx.layer_painter(egui::LayerId::background()),
                    Transform {
                        origin: Pos2::ZERO,
                        scale: 1.0,
                    },
                    50.0,
                    50.0,
                    LEGEND_SIZE,
                    WHITE,
                    letter,
                    Align2::CENTER_CENTER,
                );
            });
            let mut lo = f32::INFINITY;
            let mut hi = f32::NEG_INFINITY;
            for shape in output.shapes {
                let Shape::Mesh(mesh) = shape.shape else {
                    panic!()
                };
                for vertex in mesh.vertices.iter().filter(|v| v.color.a() == 255) {
                    lo = lo.min(vertex.pos.y);
                    hi = hi.max(vertex.pos.y);
                }
            }
            assert!((hi - lo - LEGEND_SIZE * 0.78).abs() < 0.001, "{letter}");
            assert!(((hi + lo) * 0.5 - 50.0).abs() < 0.001);
        }
    }

    #[test]
    fn reciprocal_x_print_extends_below_the_one() {
        for size in [6.8, 10.5] {
            let placements = reciprocal_positions(size);
            let bottom = |index: usize, ch| {
                let (_, y, height) = placements[index];
                let glyph = super::super::lettering_data::glyph(ch, 700);
                y - height * 0.78 * 0.5
                    + glyph
                        .vertices
                        .iter()
                        .map(|v| v[1])
                        .fold(f32::NEG_INFINITY, f32::max)
                        * height
                        * 0.78
            };
            assert!(bottom(1, 'x') - bottom(0, '1') > size * 0.20);
        }
    }

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

    use super::materials::{antialiased_surface as surface, shade};
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
    draw_main(p, kt, key, text, key.y + (key.top_h - 0.4) * 0.5);
    if key.sub.is_some() {
        draw_sub(p, kt, key, subtext, skirt_top + skirt_h * 0.5);
    }
}

fn draw_main(p: &Painter, t: Transform, key: KeySpec, color: Color32, cy: f32) {
    match key.id {
        "sigma" => {
            sigma(p, t, key.cx - 2.2, cy, 8.7, color);
            bold_txt(p, t, key.cx + 4.8, cy, 12.5, color, "+");
        }
        "decimal" => {
            p.circle_filled(t.pos(key.cx, cy), t.s(0.85), color);
        }
        "multiply" => cross(p, t, key.cx, cy, 4.7, color),
        "divide" => divide(p, t, key.cx, cy, 4.7, color),
        "clx" => {
            bold_txt(p, t, key.cx - 4.0, cy, 12.5, color, "CL");
            bold_txt(p, t, key.cx + 8.0, cy - 0.9, 14.8, color, "x");
        }
        "enter" => {
            let text_width = print_width("ENTER", 11.8);
            let arrow_width = 3.1 * 0.8 * 2.0;
            let gap = 6.0;
            bold_txt(
                p,
                t,
                key.cx - (arrow_width + gap) * 0.5,
                cy,
                11.8,
                color,
                "ENTER",
            );
            vertical_arrow(
                p,
                t,
                key.cx + (text_width + gap) * 0.5,
                cy,
                3.1,
                true,
                color,
            );
        }
        _ => {
            let size = match key.id {
                "f" | "g" | "h" => 14.0,
                "7" | "8" | "9" | "4" | "5" | "6" | "1" | "2" | "3" | "0" => 16.0,
                "minus" | "plus" => 16.0,
                "rs" => 12.7,
                "a" | "b" | "c" | "d" | "e" => 13.0,
                _ => 12.5,
            };
            bold_txt(p, t, key.cx, cy, size, color, key.main);
        }
    }
}

fn draw_sub(p: &Painter, t: Transform, key: KeySpec, color: Color32, cy: f32) {
    // Enlarge front printing together, preserving the original font weights
    // and compound-symbol proportions rather than changing size categories.
    let scale = t.scale * 1.075;
    let t = Transform {
        origin: t.pos(key.cx, cy) - Vec2::new(key.cx, cy) * scale,
        scale,
    };
    let sub = key.sub.unwrap_or("");
    match key.id {
        "sigma" => {
            sigma(p, t, key.cx - 2.1, cy, 6.5, color);
            bold_txt(p, t, key.cx + 3.9, cy, 6.7, color, "−");
        }
        "indirect" => swap(p, t, key.cx, cy, "x", "I", 6.7, color),
        "7" => swap(p, t, key.cx, cy - 0.45, "x", "y", 6.9, color),
        "8" => r_arrow(p, t, key.cx, cy, false, 6.8, color),
        "9" => r_arrow(p, t, key.cx, cy, true, 6.8, color),
        "4" => one_over_x(p, t, key.cx, cy, 6.8, color),
        "5" => power(p, t, key.cx, cy + 0.3, "y", "x", 6.8, color, color),
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
    one_over_x(p, t, 58.0, 151.8, TOP_LEGEND_SIZE, WHITE);
    sqrt_x(p, t, 112.0, 151.7, TOP_LEGEND_SIZE, WHITE);
    power(p, t, 166.0, 151.7, "y", "x", TOP_LEGEND_SIZE, WHITE, WHITE);
    r_arrow(p, t, 220.0, 151.7, false, TOP_LEGEND_SIZE, WHITE);
    swap(p, t, 274.0, 151.7, "x", "y", TOP_LEGEND_SIZE, WHITE);
    for (x, s) in [
        (58.0, "a"),
        (112.0, "b"),
        (166.0, "c"),
        (220.0, "d"),
        (274.0, "e"),
    ] {
        bold_txt(p, t, x, 214.0, LEGEND_SIZE, YELLOW, s);
    }
    xbar(p, t, 50.0, 267.0, LEGEND_SIZE, YELLOW);
    bold_txt(p, t, 66.0, 267.0, LEGEND_SIZE, CYAN, "s");
    paired_legend(p, t, 112.0, 267.0, "GSB", "f", 5.0);
    paired_legend(p, t, 166.0, 267.0, "FIX", "SCI", 5.0);
    bold_txt(p, t, 220.0, 267.0, LEGEND_SIZE, YELLOW, "RND");
    paired_legend(p, t, 274.0, 267.0, "LBL", "f", 5.0);
    paired_legend(p, t, 166.0, 321.0, "DSZ", "(i)", 4.0);
    paired_legend(p, t, 220.0, 321.0, "ISZ", "(i)", 4.0);
    bold_txt(p, t, 56.0, 372.0, LEGEND_SIZE, YELLOW, "W/DATA");
    bold_txt(p, t, 119.0, 372.0, LEGEND_SIZE, CYAN, "MERGE");
    swap_two_color(p, t, 166.0, 372.0, "P", "S", LEGEND_SIZE, YELLOW, YELLOW);
    bold_txt(p, t, 220.0, 372.0, LEGEND_SIZE, YELLOW, "CL REG");
    bold_txt(p, t, 274.0, 372.0, LEGEND_SIZE, YELLOW, "CL PRGM");
    eq_pair(p, t, 56.0, 423.0, false);
    bold_txt(p, t, 108.0, 423.0, LEGEND_SIZE, YELLOW, "LN");
    power(p, t, 128.0, 423.0, "e", "x", LEGEND_SIZE, CYAN, CYAN);
    bold_txt(p, t, 181.0, 423.0, LEGEND_SIZE, YELLOW, "LOG");
    power(p, t, 208.0, 423.0, "10", "x", LEGEND_SIZE, CYAN, CYAN);
    sqrt_x(p, t, 252.0, 423.0, LEGEND_SIZE, YELLOW);
    power(p, t, 279.0, 423.0, "x", "2", LEGEND_SIZE, CYAN, CYAN);
    eq_pair(p, t, 56.0, 475.0, true);
    inverse_trig(p, t, 119.0, 475.0, "SIN");
    inverse_trig(p, t, 194.0, 475.0, "COS");
    inverse_trig(p, t, 270.0, 475.0, "TAN");
    relation_pair(p, t, 56.0, 526.0, '<', true);
    swap_two_color(p, t, 119.0, 526.0, "R", "P", LEGEND_SIZE, YELLOW, CYAN);
    swap_two_color(p, t, 194.0, 526.0, "D", "R", LEGEND_SIZE, YELLOW, CYAN);
    swap_two_color(p, t, 270.0, 526.0, "H", "H.MS", LEGEND_SIZE, YELLOW, CYAN);
    relation_pair(p, t, 56.0, 577.0, '>', false);
    bold_txt(p, t, 106.0, 577.0, LEGEND_SIZE, YELLOW, "%");
    bold_txt(p, t, 126.5, 577.0, LEGEND_SIZE, CYAN, "%CH");
    bold_txt(p, t, 181.0, 577.0, LEGEND_SIZE, YELLOW, "INT");
    bold_txt(p, t, 207.0, 577.0, LEGEND_SIZE, CYAN, "FRAC");
    bold_txt(p, t, 247.0, 577.0, LEGEND_SIZE, YELLOW, "\u{2212}");
    bold_txt(p, t, 253.0, 577.0, LEGEND_SIZE, YELLOW, "x");
    bold_txt(p, t, 259.0, 577.0, LEGEND_SIZE, YELLOW, "\u{2212}");
    bold_txt(p, t, 281.0, 577.0, LEGEND_SIZE, CYAN, "STK");
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
    if (size == LEGEND_SIZE || size == TOP_LEGEND_SIZE || size <= 7.1)
        && matches!(s, "x" | "y" | "\u{03c0}")
    {
        matched_math_text(p, t, x, y, size, color, s, align);
        return;
    }
    super::glyphs::text(
        p,
        t.pos(x, y),
        t.s(size),
        color,
        s,
        align,
        printed_weight(size, s),
    );
}
// Matching nominal font sizes is insufficient: the custom math letters have
// smaller ink bounds. Normalize their visible height to the row's cap height.
fn matched_math_text(
    p: &Painter,
    t: Transform,
    x: f32,
    y: f32,
    size: f32,
    color: Color32,
    value: &str,
    align: Align2,
) {
    let weight = printed_weight(size, value);
    let g = super::lettering_data::glyph(value.chars().next().unwrap(), weight);
    let lo = g
        .vertices
        .iter()
        .map(|v| v[1])
        .fold(f32::INFINITY, f32::min);
    let hi = g
        .vertices
        .iter()
        .map(|v| v[1])
        .fold(f32::NEG_INFINITY, f32::max);
    let cap = t.s(size) * 0.78;
    let center = t.pos(x, y);
    let mut mesh = super::glyphs::text_mesh(
        center,
        t.s(size),
        color,
        value,
        align,
        weight,
        p.ctx().pixels_per_point(),
    );
    let old_mid = center.y - cap * 0.5 + (lo + hi) * cap * 0.5;
    for v in &mut mesh.vertices {
        v.pos.y = center.y + (v.pos.y - old_mid) / (hi - lo);
    }
    p.add(Shape::mesh(mesh));
}

fn printed_weight(size: f32, value: &str) -> u16 {
    if matches!(value, "x" | "y" | "π") {
        600
    } else if size <= 7.1 {
        700
    } else {
        400
    }
}
fn print_width(value: &str, size: f32) -> f32 {
    super::glyphs::width(value, size, printed_weight(size, value))
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
    bold_txt(p, t, cx, cy, size, c, "π");
}
// Photographed reciprocal: a smaller raised 1, descending slash, lower x.
// Shared by the white panel print and the front of the 4 key.
fn reciprocal_positions(size: f32) -> [(f32, f32, f32); 2] {
    let s = size / 9.0;
    [
        (-5.4 * s, -1.1 * s, size * 0.86),
        (5.3 * s, 0.8 * s, size * 0.98),
    ]
}
fn one_over_x(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    let s = size / 9.0;
    for ((x, y, height), letter) in reciprocal_positions(size).into_iter().zip(["1", "x"]) {
        bold_txt(p, t, cx + x, cy + y, height, c, letter);
    }
    p.line_segment(
        [
            t.pos(cx - 1.6 * s, cy + 4.0 * s),
            t.pos(cx + 2.3 * s, cy - 4.0 * s),
        ],
        Stroke::new(t.s(0.85 * s), c),
    );
}

fn sqrt_x(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    let s = size / 9.0;
    let x0 = cx - 8.2 * s;
    let st = Stroke::new(t.s(1.05 * s), c);
    p.add(Shape::line(
        vec![
            t.pos(x0, cy + 0.4 * s),
            t.pos(x0 + 1.3 * s, cy - 0.1 * s),
            t.pos(x0 + 3.0 * s, cy + 3.8 * s),
            t.pos(x0 + 6.0 * s, cy - 4.1 * s),
            t.pos(x0 + 15.0 * s, cy - 4.1 * s),
        ],
        st,
    ));
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
    let exp_size = size * 0.78;
    let base_width = print_width(base, size);
    let exp_width = print_width(exp, exp_size);
    let gap = size * 0.025;
    let left = cx - (base_width + gap + exp_width) * 0.5;
    if size == LEGEND_SIZE && base == "e" {
        matched_math_text(
            p,
            t,
            left + base_width * 0.5,
            cy,
            size,
            bc,
            base,
            Align2::CENTER_CENTER,
        );
    } else {
        bold_txt(p, t, left + base_width * 0.5, cy, size, bc, base);
    }
    bold_txt(
        p,
        t,
        left + base_width + gap + exp_width * 0.5,
        cy - size * 0.30,
        exp_size,
        ec,
        exp,
    );
}
fn r_arrow(p: &Painter, t: Transform, cx: f32, cy: f32, up: bool, size: f32, c: Color32) {
    let height = size * 0.3;
    let arrow_width = height * if size > 8.0 { 1.6 } else { 2.0 };
    let gap = size * 0.10;
    bold_txt(p, t, cx - (arrow_width + gap) * 0.5, cy, size, c, "R");
    let arrow_x = cx + (print_width("R", size) + gap) * 0.5;
    if size > 8.0 {
        vertical_arrow(p, t, arrow_x, cy + 0.2, height, up, c);
    } else {
        triangle(p, t, arrow_x, cy + 0.2, height, up, c);
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
    let (left, right, arrow) = exchange_positions(l, r, size);
    bold_txt(p, t, cx + left, cy, size, lc, l);
    bold_txt(p, t, cx + right, cy, size, rc, r);
    exchange_heads(p, t, cx + arrow, cy, size, lc, rc);
}

// Width ratios from the supplied close-up: mark ~0.14 key widths, complete
// x/exchange/y group ~0.52 key widths. No fixed eight-unit letter offsets.
fn exchange_positions(l: &str, r: &str, size: f32) -> (f32, f32, f32) {
    let lw = print_width(l, size);
    let rw = print_width(r, size);
    let arrow = size * 0.50;
    let gap = size * 0.035;
    let start = -(lw + rw + arrow + 2.0 * gap) * 0.5;
    (
        start + lw * 0.5,
        start + lw + 2.0 * gap + arrow + rw * 0.5,
        start + lw + gap + arrow * 0.5,
    )
}
fn exchange_heads(
    p: &Painter,
    t: Transform,
    cx: f32,
    cy: f32,
    size: f32,
    upper: Color32,
    lower: Color32,
) {
    let half = size * 0.25;
    let head = size * 0.30;
    let dy = size * 0.17;
    let arm = size * 0.17;
    for (sign, y, color) in [(1.0, cy - dy, upper), (-1.0, cy + dy, lower)] {
        let tip = cx + sign * half;
        p.add(Shape::convex_polygon(
            vec![
                t.pos(tip, y),
                t.pos(tip - sign * head, y - arm),
                t.pos(tip - sign * head, y + arm),
            ],
            color,
            Stroke::NONE,
        ));
    }
}
fn xbar(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, c: Color32) {
    bold_txt(p, t, cx, cy + 0.4, size, c, "x");
    p.line_segment(
        [t.pos(cx - 3.4, cy - 4.2), t.pos(cx + 3.4, cy - 4.2)],
        Stroke::new(t.s(0.8), c),
    );
}
fn paired_legend(p: &Painter, t: Transform, cx: f32, cy: f32, left: &str, right: &str, gap: f32) {
    let lw = print_width(left, LEGEND_SIZE);
    let rw = print_width(right, LEGEND_SIZE);
    bold_txt(p, t, cx - (rw + gap) * 0.5, cy, LEGEND_SIZE, YELLOW, left);
    bold_txt(p, t, cx + (lw + gap) * 0.5, cy, LEGEND_SIZE, CYAN, right);
}
fn inverse_trig(p: &Painter, t: Transform, cx: f32, cy: f32, name: &str) {
    let lw = print_width(name, LEGEND_SIZE);
    let rw = print_width("\u{2212}1", 5.8);
    bold_txt(p, t, cx - (rw + 0.3) * 0.5, cy, LEGEND_SIZE, YELLOW, name);
    bold_txt(
        p,
        t,
        cx + (lw + 0.3) * 0.5,
        cy - 3.4,
        5.8,
        CYAN,
        "\u{2212}1",
    );
}
fn eq_pair(p: &Painter, t: Transform, cx: f32, cy: f32, ne: bool) {
    bold_txt(p, t, cx - 14.0, cy, LEGEND_SIZE, YELLOW, "x");
    if ne {
        not_eq(p, t, cx - 8.0, cy, YELLOW)
    } else {
        bold_txt(p, t, cx - 8.0, cy, LEGEND_SIZE, YELLOW, "=")
    };
    bold_txt(p, t, cx - 2.0, cy, LEGEND_SIZE, YELLOW, "0");
    bold_txt(p, t, cx + 10.0, cy, LEGEND_SIZE, CYAN, "x");
    if ne {
        not_eq(p, t, cx + 16.0, cy, CYAN)
    } else {
        bold_txt(p, t, cx + 16.0, cy, LEGEND_SIZE, CYAN, "=")
    };
    bold_txt(p, t, cx + 22.0, cy, LEGEND_SIZE, CYAN, "y");
}
fn not_eq(p: &Painter, t: Transform, cx: f32, cy: f32, c: Color32) {
    let st = Stroke::new(t.s(0.75), c);
    p.line_segment([t.pos(cx - 2.8, cy - 1.5), t.pos(cx + 2.8, cy - 1.5)], st);
    p.line_segment([t.pos(cx - 2.8, cy + 1.5), t.pos(cx + 2.8, cy + 1.5)], st);
    p.line_segment([t.pos(cx - 2.3, cy + 3.2), t.pos(cx + 2.3, cy - 3.2)], st);
}
fn relation_pair(p: &Painter, t: Transform, cx: f32, cy: f32, op: char, incl: bool) {
    let os = if op == '<' { "<" } else { ">" };
    bold_txt(p, t, cx - 14.0, cy, LEGEND_SIZE, YELLOW, "x");
    bold_txt(p, t, cx - 8.0, cy, LEGEND_SIZE, YELLOW, os);
    bold_txt(p, t, cx - 2.0, cy, LEGEND_SIZE, YELLOW, "0");
    bold_txt(p, t, cx + 10.0, cy, LEGEND_SIZE, CYAN, "x");
    relop(p, t, cx + 16.0, cy, op, incl, CYAN);
    bold_txt(p, t, cx + 22.0, cy, LEGEND_SIZE, CYAN, "y");
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
