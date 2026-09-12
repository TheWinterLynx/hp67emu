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

// Geometry mirrors the panel renderer.  The lower legend belongs to the
// sloping front skirt of the physical keycap on the HP-67, not to the top face.
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
    fn s(self, v: f32) -> f32 { v * self.scale }
    fn pos(self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.origin.x + x * self.scale, self.origin.y + y * self.scale)
    }
}

fn palette(style: KeyStyle) -> (Color32, Color32, Color32, Color32) {
    match style {
        KeyStyle::Olive => (
            Color32::from_rgb(126, 127, 70),
            Color32::from_rgb(96, 97, 53),
            Color32::from_rgb(72, 73, 40),
            Color32::from_rgb(24, 25, 21),
        ),
        KeyStyle::Orange => (
            Color32::from_rgb(212, 142, 23),
            Color32::from_rgb(169, 103, 12),
            Color32::from_rgb(126, 72, 7),
            Color32::from_rgb(37, 29, 15),
        ),
        KeyStyle::Blue => (
            Color32::from_rgb(39, 151, 177),
            Color32::from_rgb(25, 113, 136),
            Color32::from_rgb(16, 80, 97),
            Color32::from_rgb(20, 28, 30),
        ),
        KeyStyle::White => (
            Color32::from_rgb(190, 196, 198),
            Color32::from_rgb(156, 163, 165),
            Color32::from_rgb(122, 128, 130),
            Color32::from_rgb(23, 27, 29),
        ),
        KeyStyle::Black => (
            Color32::from_rgb(14, 16, 17),
            Color32::from_rgb(7, 8, 9),
            Color32::from_rgb(2, 3, 3),
            Color32::from_rgb(224, 227, 224),
        ),
    }
}

fn draw_front_face(p: &Painter, t: Transform, key: KeyDepthSpec) {
    let (front, side, bottom, text_color) = palette(key.style);

    // The HP-67 lower legend sits on the sloping front face.  Start the skirt
    // high enough to completely cover the legacy flat-face sublegend beneath it.
    // This also removes the doubled text visible in the previous pass.
    let has_sub = key.sub.is_some();
    let top_y = key.y + key.h - if has_sub { 12.0 } else { 6.7 };
    let bottom_y = key.y + key.h + 0.6;
    let top_inset = if key.w > 40.0 { 1.5 } else { 1.0 };
    let bottom_inset = if key.w > 40.0 { 5.3 } else { 3.6 };

    let tl = t.pos(key.x + top_inset, top_y);
    let tr = t.pos(key.x + key.w - top_inset, top_y);
    let br = t.pos(key.x + key.w - bottom_inset, bottom_y);
    let bl = t.pos(key.x + bottom_inset, bottom_y);

    // Tight contact shadow: depth, not a floating-card shadow.
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + bottom_inset + 0.6, bottom_y),
            t.pos(key.x + key.w - bottom_inset - 0.6, bottom_y),
            t.pos(key.x + key.w - bottom_inset + 0.2, bottom_y + 2.0),
            t.pos(key.x + bottom_inset - 0.2, bottom_y + 2.0),
        ],
        Color32::from_rgba_premultiplied(0, 0, 0, 115),
        Stroke::NONE,
    ));

    // Dark side facets reinforce the inward convergence of the lower edge.
    p.add(Shape::convex_polygon(
        vec![
            tl,
            t.pos(key.x + top_inset + 2.0, top_y + 0.9),
            t.pos(key.x + bottom_inset + 2.0, bottom_y - 0.8),
            bl,
        ],
        side,
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + key.w - top_inset - 2.0, top_y + 0.9),
            tr,
            br,
            t.pos(key.x + key.w - bottom_inset - 2.0, bottom_y - 0.8),
        ],
        bottom,
        Stroke::NONE,
    ));

    // Opaque trapezoid deliberately covers the old flat sublegend first.
    p.add(Shape::convex_polygon(
        vec![tl, tr, br, bl],
        front,
        Stroke::new(t.s(0.55), bottom),
    ));

    // Break line between top key face and sloping skirt.
    p.line_segment(
        [
            t.pos(key.x + top_inset + 1.2, top_y + 0.55),
            t.pos(key.x + key.w - top_inset - 1.2, top_y + 0.55),
        ],
        Stroke::new(t.s(0.75), Color32::from_rgba_premultiplied(255, 255, 244, 135)),
    );

    // A subtle lower edge, as seen on the real moulded keycap.
    p.line_segment(
        [
            t.pos(key.x + bottom_inset + 1.0, bottom_y - 1.0),
            t.pos(key.x + key.w - bottom_inset - 1.0, bottom_y - 1.0),
        ],
        Stroke::new(t.s(0.55), Color32::from_rgba_premultiplied(245, 245, 230, 55)),
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
    // Optical centre is slightly low on the sloped face in the HP-67 photo.
    let cy = top_y + (bottom_y - top_y) * 0.61;

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
                5.7
            } else if sub.len() >= 5 {
                5.0
            } else {
                5.7
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
    p.text(t.pos(cx - 6.3, cy), Align2::CENTER_CENTER, left, FontId::proportional(t.s(5.6)), color);
    p.text(t.pos(cx + 6.3, cy), Align2::CENTER_CENTER, right, FontId::proportional(t.s(5.6)), color);
    let x1 = cx - 3.2;
    let x2 = cx + 3.2;
    let yu = cy - 1.25;
    let yd = cy + 1.25;
    p.line_segment([t.pos(x1, yu), t.pos(x2, yu)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x2 - 1.5, yu - 1.0), t.pos(x2, yu)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x2 - 1.5, yu + 1.0), t.pos(x2, yu)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x2, yd), t.pos(x1, yd)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x1 + 1.5, yd - 1.0), t.pos(x1, yd)], Stroke::new(t.s(0.55), color));
    p.line_segment([t.pos(x1 + 1.5, yd + 1.0), t.pos(x1, yd)], Stroke::new(t.s(0.55), color));
}

fn draw_r_triangle(p: &Painter, t: Transform, cx: f32, cy: f32, up: bool, color: Color32) {
    p.text(t.pos(cx - 3.0, cy), Align2::CENTER_CENTER, "R", FontId::proportional(t.s(5.7)), color);
    let dir = if up { -1.0 } else { 1.0 };
    let x = cx + 3.7;
    let h = 2.0;
    p.add(Shape::convex_polygon(
        vec![
            t.pos(x, cy + dir * h),
            t.pos(x - h, cy - dir * h * 0.7),
            t.pos(x + h, cy - dir * h * 0.7),
        ],
        color,
        Stroke::NONE,
    ));
}

fn draw_one_over_x(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.text(t.pos(cx - 4.4, cy), Align2::CENTER_CENTER, "1", FontId::proportional(t.s(5.6)), color);
    p.line_segment([t.pos(cx - 1.7, cy + 2.3), t.pos(cx + 1.6, cy - 2.3)], Stroke::new(t.s(0.55), color));
    p.text(t.pos(cx + 4.4, cy), Align2::CENTER_CENTER, "x", FontId::proportional(t.s(5.6)), color);
}

fn draw_y_power_x(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.text(t.pos(cx - 2.0, cy + 0.5), Align2::CENTER_CENTER, "y", FontId::proportional(t.s(5.7)), color);
    p.text(t.pos(cx + 2.3, cy - 2.2), Align2::CENTER_CENTER, "x", FontId::proportional(t.s(3.8)), color);
}

fn draw_pi(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.line_segment([t.pos(cx - 4.0, cy - 2.6), t.pos(cx + 4.0, cy - 2.6)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(cx - 2.1, cy - 2.6), t.pos(cx - 2.1, cy + 2.8)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(cx + 2.1, cy - 2.6), t.pos(cx + 2.1, cy + 2.8)], Stroke::new(t.s(0.65), color));
}

fn draw_sigma_minus(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    let left = cx - 5.5;
    let right = cx + 1.6;
    let top = cy - 2.6;
    let bottom = cy + 2.6;
    p.line_segment([t.pos(left, top), t.pos(right, top)], Stroke::new(t.s(0.58), color));
    p.line_segment([t.pos(left, top), t.pos(cx - 1.1, cy)], Stroke::new(t.s(0.58), color));
    p.line_segment([t.pos(cx - 1.1, cy), t.pos(left, bottom)], Stroke::new(t.s(0.58), color));
    p.line_segment([t.pos(left, bottom), t.pos(right, bottom)], Stroke::new(t.s(0.58), color));
    p.line_segment([t.pos(cx + 3.0, cy), t.pos(cx + 6.2, cy)], Stroke::new(t.s(0.6), color));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_secondary_legends_live_on_skirt_specs() {
        assert_eq!(KEYS.len(), 35);
        assert_eq!(KEYS.iter().filter(|k| k.sub.is_some()).count(), 25);
        assert!(KEYS.iter().any(|k| k.id == "gto" && k.sub == Some("RTN")));
        assert!(KEYS.iter().any(|k| k.id == "rs" && k.sub == Some("SPACE")));
    }
}
