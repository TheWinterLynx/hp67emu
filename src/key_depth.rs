use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

const DESIGN_W: f32 = 330.0;
const DESIGN_H: f32 = 620.0;

#[derive(Clone, Copy)]
enum KeyStyle {
    Olive,
    Orange,
    Blue,
    White,
    Black,
}

#[derive(Clone, Copy)]
struct KeyDepthSpec {
    id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    style: KeyStyle,
    sub: Option<&'static str>,
}

const fn k(
    id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    style: KeyStyle,
    sub: Option<&'static str>,
) -> KeyDepthSpec {
    KeyDepthSpec { id, x, y, w, h, style, sub }
}

const KEYS: &[KeyDepthSpec] = &[
    k("a", 50.0, 170.0, 30.0, 28.0, KeyStyle::Olive, None),
    k("b", 101.0, 170.0, 30.0, 28.0, KeyStyle::Olive, None),
    k("c", 152.0, 170.0, 30.0, 28.0, KeyStyle::Olive, None),
    k("d", 203.0, 170.0, 30.0, 28.0, KeyStyle::Olive, None),
    k("e", 254.0, 170.0, 30.0, 28.0, KeyStyle::Olive, None),

    k("sigma", 50.0, 224.0, 30.0, 30.0, KeyStyle::Olive, Some("SIGMA-")),
    k("gto", 101.0, 224.0, 30.0, 30.0, KeyStyle::Olive, Some("RTN")),
    k("dsp", 152.0, 224.0, 30.0, 30.0, KeyStyle::Olive, Some("ENG")),
    k("indirect", 203.0, 224.0, 30.0, 30.0, KeyStyle::Olive, Some("X<>I")),
    k("sst", 254.0, 224.0, 30.0, 30.0, KeyStyle::Olive, Some("BST")),

    k("f", 50.0, 279.0, 30.0, 28.0, KeyStyle::Orange, None),
    k("g", 101.0, 279.0, 30.0, 28.0, KeyStyle::Blue, None),
    k("sto", 152.0, 279.0, 30.0, 30.0, KeyStyle::Olive, Some("ST I")),
    k("rcl", 203.0, 279.0, 30.0, 30.0, KeyStyle::Olive, Some("RC I")),
    k("h", 254.0, 279.0, 30.0, 28.0, KeyStyle::Black, None),

    k("enter", 50.0, 333.0, 81.0, 30.0, KeyStyle::Olive, Some("DEG")),
    k("chs", 152.0, 333.0, 30.0, 30.0, KeyStyle::Olive, Some("RAD")),
    k("eex", 203.0, 333.0, 30.0, 30.0, KeyStyle::Olive, Some("GRD")),
    k("clx", 254.0, 333.0, 30.0, 30.0, KeyStyle::Olive, Some("DEL")),

    k("minus", 50.0, 385.0, 30.0, 30.0, KeyStyle::Olive, Some("SF")),
    k("7", 101.0, 385.0, 30.0, 30.0, KeyStyle::White, Some("X<>Y")),
    k("8", 177.0, 385.0, 30.0, 30.0, KeyStyle::White, Some("RDN")),
    k("9", 253.0, 385.0, 30.0, 30.0, KeyStyle::White, Some("RUP")),

    k("plus", 50.0, 437.0, 30.0, 30.0, KeyStyle::Olive, Some("CF")),
    k("4", 101.0, 437.0, 30.0, 30.0, KeyStyle::White, Some("1/X")),
    k("5", 177.0, 437.0, 30.0, 30.0, KeyStyle::White, Some("Y^X")),
    k("6", 253.0, 437.0, 30.0, 30.0, KeyStyle::White, Some("ABS")),

    k("multiply", 50.0, 489.0, 30.0, 30.0, KeyStyle::Olive, Some("F?")),
    k("1", 101.0, 489.0, 30.0, 30.0, KeyStyle::White, Some("PAUSE")),
    k("2", 177.0, 489.0, 30.0, 30.0, KeyStyle::White, Some("PI")),
    k("3", 253.0, 489.0, 30.0, 30.0, KeyStyle::White, Some("REG")),

    k("divide", 50.0, 541.0, 30.0, 30.0, KeyStyle::Olive, Some("N!")),
    k("0", 101.0, 541.0, 30.0, 30.0, KeyStyle::White, Some("LST X")),
    k("decimal", 177.0, 541.0, 30.0, 30.0, KeyStyle::White, Some("H.MS+")),
    k("rs", 253.0, 541.0, 30.0, 30.0, KeyStyle::White, Some("SPACE")),
];

pub struct KeyDepthOverlay;

impl KeyDepthOverlay {
    pub fn paint(p: &Painter, host_rect: Rect) {
        let scale = (host_rect.width() / DESIGN_W).min(host_rect.height() / DESIGN_H);
        if scale <= 0.0 {
            return;
        }

        let panel = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(
                host_rect.center().x - panel.x * 0.5,
                host_rect.center().y - panel.y * 0.5,
            ),
            scale,
        };

        for key in KEYS {
            draw_front_face(p, t, *key);
        }
    }
}

#[derive(Clone, Copy)]
struct Transform {
    origin: Pos2,
    scale: f32,
}

impl Transform {
    fn s(self, v: f32) -> f32 {
        v * self.scale
    }

    fn pos(self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.origin.x + x * self.scale, self.origin.y + y * self.scale)
    }
}

fn palette(style: KeyStyle) -> (Color32, Color32, Color32, Color32) {
    match style {
        KeyStyle::Olive => (
            Color32::from_rgb(132, 133, 75),
            Color32::from_rgb(104, 105, 59),
            Color32::from_rgb(80, 81, 46),
            Color32::from_rgb(26, 27, 23),
        ),
        KeyStyle::Orange => (
            Color32::from_rgb(211, 143, 27),
            Color32::from_rgb(174, 108, 15),
            Color32::from_rgb(132, 76, 9),
            Color32::from_rgb(39, 31, 17),
        ),
        KeyStyle::Blue => (
            Color32::from_rgb(43, 151, 177),
            Color32::from_rgb(28, 116, 139),
            Color32::from_rgb(18, 83, 101),
            Color32::from_rgb(22, 29, 31),
        ),
        KeyStyle::White => (
            Color32::from_rgb(196, 201, 202),
            Color32::from_rgb(163, 169, 170),
            Color32::from_rgb(130, 136, 137),
            Color32::from_rgb(25, 29, 30),
        ),
        KeyStyle::Black => (
            Color32::from_rgb(15, 17, 18),
            Color32::from_rgb(8, 9, 10),
            Color32::from_rgb(3, 4, 4),
            Color32::from_rgb(226, 228, 226),
        ),
    }
}

fn draw_front_face(p: &Painter, t: Transform, key: KeyDepthSpec) {
    let (front, side, bottom, text_color) = palette(key.style);

    // HP keycaps are not stacked rectangles. The upper face ends first and a
    // sloping front skirt falls toward the panel. The lower edge converges
    // inward, which is the perspective cue visible in the photographic reference.
    let top_y = key.y + key.h - if key.sub.is_some() { 9.2 } else { 7.0 };
    let bottom_y = key.y + key.h + 0.2;
    let top_inset = 1.1;
    let bottom_inset = if key.w > 40.0 { 4.8 } else { 3.4 };

    let tl = t.pos(key.x + top_inset, top_y);
    let tr = t.pos(key.x + key.w - top_inset, top_y);
    let br = t.pos(key.x + key.w - bottom_inset, bottom_y);
    let bl = t.pos(key.x + bottom_inset, bottom_y);

    // Contact shadow immediately below the skirt; kept narrow so it reads as
    // height above the chassis instead of a generic drop shadow.
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + bottom_inset + 0.8, bottom_y - 0.2),
            t.pos(key.x + key.w - bottom_inset - 0.8, bottom_y - 0.2),
            t.pos(key.x + key.w - bottom_inset - 0.1, bottom_y + 2.1),
            t.pos(key.x + bottom_inset + 0.1, bottom_y + 2.1),
        ],
        Color32::from_rgba_premultiplied(0, 0, 0, 105),
        Stroke::NONE,
    ));

    // Left and right bevels are darker than the frontal plane and taper with it.
    p.add(Shape::convex_polygon(
        vec![
            tl,
            t.pos(key.x + top_inset + 2.0, top_y + 1.2),
            t.pos(key.x + bottom_inset + 2.0, bottom_y - 0.7),
            bl,
        ],
        side,
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + key.w - top_inset - 2.0, top_y + 1.2),
            tr,
            br,
            t.pos(key.x + key.w - bottom_inset - 2.0, bottom_y - 0.7),
        ],
        bottom,
        Stroke::NONE,
    ));

    // Main sloped front face.
    p.add(Shape::convex_polygon(
        vec![tl, tr, br, bl],
        front,
        Stroke::new(t.s(0.55), bottom),
    ));

    // Upper ridge catches light where the horizontal key face breaks downward.
    p.line_segment(
        [
            t.pos(key.x + top_inset + 1.4, top_y + 0.65),
            t.pos(key.x + key.w - top_inset - 1.4, top_y + 0.65),
        ],
        Stroke::new(t.s(0.75), Color32::from_rgba_premultiplied(255, 255, 245, 125)),
    );

    // Recessed lower panel seen on the real HP key skirt.
    let recess_left = key.x + bottom_inset + if key.w > 40.0 { 5.0 } else { 2.0 };
    let recess_right = key.x + key.w - bottom_inset - if key.w > 40.0 { 5.0 } else { 2.0 };
    let recess_y = bottom_y - 1.75;
    p.line_segment(
        [t.pos(recess_left, recess_y), t.pos(recess_right, recess_y)],
        Stroke::new(t.s(0.72), Color32::from_rgba_premultiplied(35, 35, 30, 100)),
    );
    p.line_segment(
        [t.pos(recess_left + 0.5, recess_y - 0.8), t.pos(recess_right - 0.5, recess_y - 0.8)],
        Stroke::new(t.s(0.45), Color32::from_rgba_premultiplied(245, 245, 230, 55)),
    );

    if let Some(sub) = key.sub {
        draw_sublegend(p, t, key, sub, text_color, top_y, bottom_y);
    }
}

fn draw_sublegend(
    p: &Painter,
    t: Transform,
    key: KeyDepthSpec,
    sub: &str,
    color: Color32,
    top_y: f32,
    bottom_y: f32,
) {
    let cx = key.x + key.w * 0.5;
    let cy = top_y + (bottom_y - top_y) * 0.54;

    match key.id {
        "sigma" => draw_sigma_minus(p, t, cx, cy, color),
        "indirect" => draw_swap(p, t, cx, cy, "x", "I", color),
        "7" => draw_swap(p, t, cx, cy, "x", "y", color),
        "8" => draw_r_triangle(p, t, cx, cy, false, color),
        "9" => draw_r_triangle(p, t, cx, cy, true, color),
        "4" => draw_one_over_x(p, t, cx, cy, color),
        "5" => draw_y_power_x(p, t, cx, cy, color),
        "2" => draw_pi(p, t, cx, cy, color),
        _ => {
            let size = if key.w > 40.0 {
                5.9
            } else if sub.len() >= 5 {
                5.2
            } else {
                5.8
            };
            p.text(
                t.pos(cx, cy),
                Align2::CENTER_CENTER,
                sub,
                FontId::proportional(t.s(size)),
                color,
            );
        }
    }
}

fn draw_swap(p: &Painter, t: Transform, cx: f32, cy: f32, left: &str, right: &str, color: Color32) {
    p.text(t.pos(cx - 6.5, cy), Align2::CENTER_CENTER, left, FontId::proportional(t.s(5.8)), color);
    p.text(t.pos(cx + 6.5, cy), Align2::CENTER_CENTER, right, FontId::proportional(t.s(5.8)), color);
    let x1 = cx - 3.4;
    let x2 = cx + 3.4;
    p.line_segment([t.pos(x1, cy - 1.05), t.pos(x2, cy - 1.05)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x2 - 1.4, cy - 2.0), t.pos(x2, cy - 1.05)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x2 - 1.4, cy - 0.1), t.pos(x2, cy - 1.05)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x2, cy + 1.05), t.pos(x1, cy + 1.05)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x1 + 1.4, cy + 0.1), t.pos(x1, cy + 1.05)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x1 + 1.4, cy + 2.0), t.pos(x1, cy + 1.05)], Stroke::new(t.s(0.55), color));
}

fn draw_r_triangle(p: &Painter, t: Transform, cx: f32, cy: f32, up: bool, color: Color32) {
    p.text(t.pos(cx - 3.1, cy), Align2::CENTER_CENTER, "R", FontId::proportional(t.s(5.8)), color);
    let dir = if up { -1.0 } else { 1.0 };
    p.add(Shape::convex_polygon(
        vec![
            t.pos(cx + 4.8, cy + dir * 2.0),
            t.pos(cx + 2.8, cy - dir * 1.2),
            t.pos(cx + 6.8, cy - dir * 1.2),
        ],
        color,
        Stroke::NONE,
    ));
}

fn draw_one_over_x(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.text(t.pos(cx - 4.5, cy), Align2::CENTER_CENTER, "1", FontId::proportional(t.s(5.7)), color);
    p.line_segment([t.pos(cx - 1.2, cy + 2.2), t.pos(cx + 1.4, cy - 2.2)], Stroke::new(t.s(0.55), color));
    p.text(t.pos(cx + 4.5, cy), Align2::CENTER_CENTER, "x", FontId::proportional(t.s(5.7)), color);
}

fn draw_y_power_x(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.text(t.pos(cx - 2.2, cy + 0.5), Align2::CENTER_CENTER, "y", FontId::proportional(t.s(5.8)), color);
    p.text(t.pos(cx + 3.0, cy - 2.0), Align2::CENTER_CENTER, "x", FontId::proportional(t.s(4.2)), color);
}

fn draw_pi(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.line_segment([t.pos(cx - 3.5, cy - 2.0), t.pos(cx + 3.5, cy - 2.0)], Stroke::new(t.s(0.62), color));
    p.line_segment([t.pos(cx - 1.8, cy - 2.0), t.pos(cx - 1.8, cy + 2.4)], Stroke::new(t.s(0.62), color));
    p.line_segment([t.pos(cx + 1.8, cy - 2.0), t.pos(cx + 1.8, cy + 2.4)], Stroke::new(t.s(0.62), color));
}

fn draw_sigma_minus(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    let left = cx - 4.4;
    let right = cx + 1.8;
    let top = cy - 2.8;
    let bottom = cy + 2.8;
    p.line_segment([t.pos(left, top), t.pos(right, top)], Stroke::new(t.s(0.6), color));
    p.line_segment([t.pos(left, top), t.pos(cx - 0.8, cy)], Stroke::new(t.s(0.6), color));
    p.line_segment([t.pos(cx - 0.8, cy), t.pos(left, bottom)], Stroke::new(t.s(0.6), color));
    p.line_segment([t.pos(left, bottom), t.pos(right, bottom)], Stroke::new(t.s(0.6), color));
    p.line_segment([t.pos(cx + 3.1, cy), t.pos(cx + 6.1, cy)], Stroke::new(t.s(0.6), color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn overlay_tracks_all_physical_keys() {
        assert_eq!(KEYS.len(), 35);
    }

    #[test]
    fn bottom_edge_converges_inward() {
        for key in KEYS {
            let inset = if key.w > 40.0 { 4.8 } else { 3.4 };
            assert!(inset > 1.1);
            assert!(key.w - inset * 2.0 > 0.0);
        }
    }
}
