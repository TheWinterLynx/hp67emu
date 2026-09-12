use eframe::egui::{Align2, Color32, FontFamily, FontId, Painter, Pos2, Rect, Shape, Stroke, Vec2};

const DESIGN_W: f32 = 330.0;
const DESIGN_H: f32 = 620.0;

const LIGHT_TEXT: Color32 = Color32::from_rgb(235, 236, 228);
const DARK_TEXT: Color32 = Color32::from_rgb(31, 34, 33);

#[derive(Clone, Copy)]
enum KeyStyle {
    Olive,
    Orange,
    Blue,
    White,
    Black,
}

#[derive(Clone, Copy)]
enum SubAlign {
    Center,
    Right,
}

#[derive(Clone, Copy)]
struct KeySpec {
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    top_h: f32,
    style: KeyStyle,
    main: &'static str,
    sub: Option<&'static str>,
    sub_align: SubAlign,
}

const fn k(
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    top_h: f32,
    style: KeyStyle,
    main: &'static str,
    sub: Option<&'static str>,
    sub_align: SubAlign,
) -> KeySpec {
    KeySpec { x, y, w, h, top_h, style, main, sub, sub_align }
}

const KEYS: &[KeySpec] = &[
    // Top two rows / upper block
    k(49.0, 170.0, 31.0, 27.5, 18.5, KeyStyle::Olive, "A", None, SubAlign::Center),
    k(99.5, 170.0, 31.0, 27.5, 18.5, KeyStyle::Olive, "B", None, SubAlign::Center),
    k(150.0, 170.0, 31.0, 27.5, 18.5, KeyStyle::Olive, "C", None, SubAlign::Center),
    k(200.5, 170.0, 31.0, 27.5, 18.5, KeyStyle::Olive, "D", None, SubAlign::Center),
    k(251.0, 170.0, 31.0, 27.5, 18.5, KeyStyle::Olive, "E", None, SubAlign::Center),

    k(49.0, 224.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "Σ+", Some("Σ−"), SubAlign::Center),
    k(99.5, 224.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "GTO", Some("RTN"), SubAlign::Center),
    k(150.0, 224.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "DSP", Some("ENG"), SubAlign::Center),
    k(200.5, 224.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "(i)", Some("x↔I"), SubAlign::Center),
    k(251.0, 224.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "SST", Some("BST"), SubAlign::Center),

    k(49.0, 279.0, 31.0, 27.5, 18.5, KeyStyle::Orange, "f", None, SubAlign::Center),
    k(99.5, 279.0, 31.0, 27.5, 18.5, KeyStyle::Blue, "g", None, SubAlign::Center),
    k(150.0, 279.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "STO", Some("ST I"), SubAlign::Center),
    k(200.5, 279.0, 31.0, 29.5, 18.0, KeyStyle::Olive, "RCL", Some("RC I"), SubAlign::Center),
    k(251.0, 279.0, 31.0, 27.5, 18.5, KeyStyle::Black, "h", None, SubAlign::Center),

    k(49.0, 333.5, 79.5, 29.5, 18.2, KeyStyle::Olive, "ENTER", Some("DEG"), SubAlign::Right),
    k(150.0, 333.5, 31.0, 29.5, 18.0, KeyStyle::Olive, "CHS", Some("RAD"), SubAlign::Center),
    k(200.5, 333.5, 31.0, 29.5, 18.0, KeyStyle::Olive, "EEX", Some("GRD"), SubAlign::Center),
    k(251.0, 333.5, 31.0, 29.5, 18.0, KeyStyle::Olive, "CLX", Some("DEL"), SubAlign::Center),

    // Lower matrix: operator column narrower, numeric columns rectangular and wider.
    k(49.0, 386.5, 22.0, 28.0, 17.5, KeyStyle::Olive, "−", Some("SF"), SubAlign::Center),
    k(99.0, 386.5, 36.0, 28.0, 17.5, KeyStyle::White, "7", Some("x↔y"), SubAlign::Center),
    k(172.0, 386.5, 36.0, 28.0, 17.5, KeyStyle::White, "8", Some("R▼"), SubAlign::Center),
    k(245.0, 386.5, 37.0, 28.0, 17.5, KeyStyle::White, "9", Some("R▲"), SubAlign::Center),

    k(49.0, 438.8, 22.0, 28.0, 17.5, KeyStyle::Olive, "+", Some("CF"), SubAlign::Center),
    k(99.0, 438.8, 36.0, 28.0, 17.5, KeyStyle::White, "4", Some("1/x"), SubAlign::Center),
    k(172.0, 438.8, 36.0, 28.0, 17.5, KeyStyle::White, "5", Some("yˣ"), SubAlign::Center),
    k(245.0, 438.8, 37.0, 28.0, 17.5, KeyStyle::White, "6", Some("ABS"), SubAlign::Center),

    k(49.0, 491.1, 22.0, 28.0, 17.5, KeyStyle::Olive, "×", Some("F?"), SubAlign::Center),
    k(99.0, 491.1, 36.0, 28.0, 17.5, KeyStyle::White, "1", Some("PAUSE"), SubAlign::Center),
    k(172.0, 491.1, 36.0, 28.0, 17.5, KeyStyle::White, "2", Some("π"), SubAlign::Center),
    k(245.0, 491.1, 37.0, 28.0, 17.5, KeyStyle::White, "3", Some("REG"), SubAlign::Center),

    k(49.0, 543.4, 22.0, 28.0, 17.5, KeyStyle::Olive, "÷", Some("N!"), SubAlign::Center),
    k(99.0, 543.4, 36.0, 28.0, 17.5, KeyStyle::White, "0", Some("LST x"), SubAlign::Center),
    k(172.0, 543.4, 36.0, 28.0, 17.5, KeyStyle::White, ".", Some("H.MS+"), SubAlign::Center),
    k(245.0, 543.4, 37.0, 28.0, 17.5, KeyStyle::White, "R/S", Some("SPACE"), SubAlign::Center),
];

pub struct KeyDepthOverlay;

impl KeyDepthOverlay {
    pub fn paint(p: &Painter, host: Rect) {
        let scale = (host.width() / DESIGN_W).min(host.height() / DESIGN_H);
        if scale <= 0.0 {
            return;
        }
        let size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(host.center().x - size.x * 0.5, host.center().y - size.y * 0.5),
            scale,
        };
        for key in KEYS {
            draw_key(p, t, *key);
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
    fn rect(self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.pos(x, y), Vec2::new(self.s(w), self.s(h)))
    }
}

fn draw_key(p: &Painter, t: Transform, key: KeySpec) {
    let (top_face, front_face, shell, main_color, sub_color) = palette(key.style);

    // Mask the old flat renderer completely in this footprint.
    p.rect_filled(
        t.rect(key.x - 2.0, key.y - 1.0, key.w + 4.0, key.h + 5.0),
        t.s(2.8),
        shell,
    );

    // Contact shadow under the sloping front.
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + 3.0, key.y + key.h + 0.6),
            t.pos(key.x + key.w - 3.0, key.y + key.h + 0.6),
            t.pos(key.x + key.w - 0.8, key.y + key.h + 3.1),
            t.pos(key.x + 0.8, key.y + key.h + 3.1),
        ],
        Color32::from_rgba_premultiplied(0, 0, 0, 125),
        Stroke::NONE,
    ));

    let top_rect = t.rect(key.x + 0.8, key.y, key.w - 1.6, key.top_h);
    p.rect_filled(top_rect, t.s(2.2), top_face);
    p.rect_stroke(
        top_rect,
        t.s(2.2),
        Stroke::new(t.s(0.7), Color32::from_rgba_premultiplied(0, 0, 0, 55)),
    );

    // Sloping front skirt. No lower reborde line: only the hinge/ridge at the top.
    let top_y = key.y + key.top_h - 0.6;
    let skirt_inset = if key.w >= 70.0 { 5.6 } else if key.w >= 35.0 { 4.0 } else { 3.1 };
    let left_top = key.x + 1.4;
    let right_top = key.x + key.w - 1.4;
    let left_bottom = key.x + skirt_inset;
    let right_bottom = key.x + key.w - skirt_inset;
    let bottom_y = key.y + key.h;

    p.add(Shape::convex_polygon(
        vec![
            t.pos(left_top, top_y),
            t.pos(right_top, top_y),
            t.pos(right_bottom, bottom_y),
            t.pos(left_bottom, bottom_y),
        ],
        front_face,
        Stroke::NONE,
    ));

    // Side facets to suggest perspective towards the chassis.
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + 0.8, key.y + 1.6),
            t.pos(key.x + 0.8, key.y + key.top_h - 0.3),
            t.pos(left_bottom, bottom_y),
            t.pos(key.x + 1.2, key.y + key.h - 0.7),
        ],
        darker(front_face, 18),
        Stroke::NONE,
    ));
    p.add(Shape::convex_polygon(
        vec![
            t.pos(key.x + key.w - 0.8, key.y + 1.6),
            t.pos(key.x + key.w - 0.8, key.y + key.top_h - 0.3),
            t.pos(right_bottom, bottom_y),
            t.pos(key.x + key.w - 1.2, key.y + key.h - 0.7),
        ],
        darker(front_face, 28),
        Stroke::NONE,
    ));

    // Highlight on top face and ridge where the surface breaks.
    p.line_segment(
        [
            t.pos(key.x + 2.6, key.y + 1.4),
            t.pos(key.x + key.w - 2.6, key.y + 1.4),
        ],
        Stroke::new(t.s(0.95), Color32::from_rgba_premultiplied(255, 255, 245, 145)),
    );
    p.line_segment(
        [t.pos(left_top + 0.5, top_y), t.pos(right_top - 0.5, top_y)],
        Stroke::new(t.s(0.75), Color32::from_rgba_premultiplied(255, 255, 238, 115)),
    );

    draw_main_label(p, t, key, main_color);
    if let Some(sub) = key.sub {
        draw_sub_label(p, t, key, sub, sub_color, top_y, bottom_y);
    }
}

fn draw_main_label(p: &Painter, t: Transform, key: KeySpec, color: Color32) {
    let top_center_y = key.y + key.top_h * 0.48;
    if key.main == "ENTER" {
        thick_text(
            p,
            t.pos(key.x + key.w * 0.43, top_center_y),
            Align2::CENTER_CENTER,
            8.7,
            color,
            key.main,
        );
        let tri_x = key.x + key.w - 12.5;
        let tri_y = key.y + key.top_h * 0.42;
        p.add(Shape::convex_polygon(
            vec![t.pos(tri_x, tri_y - 2.4), t.pos(tri_x - 2.9, tri_y + 1.8), t.pos(tri_x + 2.9, tri_y + 1.8)],
            color,
            Stroke::NONE,
        ));
        return;
    }

    let size = if matches!(key.style, KeyStyle::White) {
        if key.main.len() >= 3 { 11.2 } else { 12.6 }
    } else if matches!(key.style, KeyStyle::Black) {
        11.0
    } else if key.main.len() >= 3 {
        9.9
    } else {
        11.4
    };

    thick_text(
        p,
        t.pos(key.x + key.w * 0.5, top_center_y),
        Align2::CENTER_CENTER,
        size,
        color,
        key.main,
    );
}

fn draw_sub_label(
    p: &Painter,
    t: Transform,
    key: KeySpec,
    sub: &str,
    color: Color32,
    top_y: f32,
    bottom_y: f32,
) {
    let cy = top_y + (bottom_y - top_y) * 0.56;
    let (pos, align) = match key.sub_align {
        SubAlign::Center => (t.pos(key.x + key.w * 0.5, cy), Align2::CENTER_CENTER),
        SubAlign::Right => (t.pos(key.x + key.w - 8.0, cy), Align2::RIGHT_CENTER),
    };

    let size = if key.w >= 70.0 {
        5.7
    } else if key.w >= 35.0 {
        if sub.len() >= 5 { 5.1 } else { 5.8 }
    } else if sub.len() >= 3 {
        5.2
    } else {
        5.6
    };

    thick_text(p, pos, align, size, color, sub);
}

fn thick_text(p: &Painter, pos: Pos2, align: Align2, size: f32, color: Color32, text: &str) {
    let font = FontId::new(size, FontFamily::Proportional);
    p.text(pos, align, text, font.clone(), color);
    p.text(Pos2::new(pos.x + 0.22, pos.y), align, text, font, color);
}

fn palette(style: KeyStyle) -> (Color32, Color32, Color32, Color32, Color32) {
    match style {
        KeyStyle::Olive => (
            Color32::from_rgb(179, 178, 106),
            Color32::from_rgb(132, 127, 68),
            Color32::from_rgb(26, 28, 23),
            LIGHT_TEXT,
            DARK_TEXT,
        ),
        KeyStyle::Orange => (
            Color32::from_rgb(243, 188, 52),
            Color32::from_rgb(194, 129, 20),
            Color32::from_rgb(28, 24, 18),
            DARK_TEXT,
            DARK_TEXT,
        ),
        KeyStyle::Blue => (
            Color32::from_rgb(74, 198, 227),
            Color32::from_rgb(31, 139, 168),
            Color32::from_rgb(19, 28, 31),
            DARK_TEXT,
            DARK_TEXT,
        ),
        KeyStyle::White => (
            Color32::from_rgb(235, 236, 232),
            Color32::from_rgb(188, 192, 192),
            Color32::from_rgb(24, 27, 28),
            DARK_TEXT,
            DARK_TEXT,
        ),
        KeyStyle::Black => (
            Color32::from_rgb(19, 20, 23),
            Color32::from_rgb(12, 13, 15),
            Color32::from_rgb(22, 24, 25),
            LIGHT_TEXT,
            LIGHT_TEXT,
        ),
    }
}

fn darker(color: Color32, amount: u8) -> Color32 {
    Color32::from_rgb(
        color.r().saturating_sub(amount),
        color.g().saturating_sub(amount),
        color.b().saturating_sub(amount),
    )
}
