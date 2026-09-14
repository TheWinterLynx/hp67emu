use eframe::egui::{
    pos2, vec2, Color32, CursorIcon, Painter, Pos2, Rect, Sense, Stroke, TextureHandle, Ui,
    Vec2,
};

use crate::hp67::{Hp67State, KeyAction, UiEvent};

const PHOTO_W: f32 = 928.0;
const PHOTO_H: f32 = 1695.0;

#[derive(Clone, Copy)]
struct PxRect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl PxRect {
    const fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

#[derive(Clone, Copy)]
struct PhotoKey {
    id: &'static str,
    src: PxRect,
    action: KeyAction,
}

const fn key(id: &'static str, src: PxRect, action: KeyAction) -> PhotoKey {
    PhotoKey { id, src, action }
}

// Pixel measurements from hp67.png.  The hit regions deliberately follow the
// photographed keycaps instead of the legacy vector geometry, so the controls
// remain aligned with the photo at every window size.
const KEYS: &[PhotoKey] = &[
    key("a", PxRect::new(166.0, 467.0, 256.0, 550.0), KeyAction::A),
    key("b", PxRect::new(293.0, 467.0, 383.0, 550.0), KeyAction::B),
    key("c", PxRect::new(420.0, 467.0, 510.0, 550.0), KeyAction::C),
    key("d", PxRect::new(547.0, 467.0, 637.0, 550.0), KeyAction::D),
    key("e", PxRect::new(674.0, 467.0, 764.0, 550.0), KeyAction::E),
    key(
        "sigma",
        PxRect::new(166.0, 601.0, 256.0, 685.0),
        KeyAction::SigmaPlus,
    ),
    key("gto", PxRect::new(293.0, 601.0, 383.0, 685.0), KeyAction::Gto),
    key("dsp", PxRect::new(420.0, 601.0, 510.0, 685.0), KeyAction::Dsp),
    key(
        "indirect",
        PxRect::new(547.0, 601.0, 637.0, 685.0),
        KeyAction::Indirect,
    ),
    key("sst", PxRect::new(674.0, 601.0, 764.0, 685.0), KeyAction::Sst),
    key("f", PxRect::new(164.0, 735.0, 257.0, 819.0), KeyAction::FunctionF),
    key("g", PxRect::new(291.0, 735.0, 384.0, 819.0), KeyAction::FunctionG),
    key("sto", PxRect::new(420.0, 735.0, 510.0, 819.0), KeyAction::Sto),
    key("rcl", PxRect::new(547.0, 735.0, 637.0, 819.0), KeyAction::Rcl),
    key("h", PxRect::new(674.0, 735.0, 764.0, 819.0), KeyAction::FunctionH),
    key(
        "enter",
        PxRect::new(164.0, 868.0, 385.0, 952.0),
        KeyAction::Enter,
    ),
    key("chs", PxRect::new(420.0, 868.0, 510.0, 952.0), KeyAction::ChangeSign),
    key("eex", PxRect::new(547.0, 868.0, 637.0, 952.0), KeyAction::Enter),
    key("clx", PxRect::new(674.0, 868.0, 764.0, 952.0), KeyAction::ClearX),
    key(
        "minus",
        PxRect::new(166.0, 1003.0, 241.0, 1088.0),
        KeyAction::Subtract,
    ),
    key("7", PxRect::new(295.0, 1003.0, 405.0, 1088.0), KeyAction::Digit(7)),
    key("8", PxRect::new(476.0, 1003.0, 587.0, 1088.0), KeyAction::Digit(8)),
    key("9", PxRect::new(657.0, 1003.0, 768.0, 1088.0), KeyAction::Digit(9)),
    key("plus", PxRect::new(166.0, 1138.0, 241.0, 1223.0), KeyAction::Add),
    key("4", PxRect::new(295.0, 1138.0, 405.0, 1223.0), KeyAction::Digit(4)),
    key("5", PxRect::new(476.0, 1138.0, 587.0, 1223.0), KeyAction::Digit(5)),
    key("6", PxRect::new(657.0, 1138.0, 768.0, 1223.0), KeyAction::Digit(6)),
    key(
        "multiply",
        PxRect::new(166.0, 1273.0, 241.0, 1358.0),
        KeyAction::Multiply,
    ),
    key("1", PxRect::new(295.0, 1273.0, 405.0, 1358.0), KeyAction::Digit(1)),
    key("2", PxRect::new(476.0, 1273.0, 587.0, 1358.0), KeyAction::Digit(2)),
    key("3", PxRect::new(657.0, 1273.0, 768.0, 1358.0), KeyAction::Digit(3)),
    key(
        "divide",
        PxRect::new(166.0, 1409.0, 241.0, 1495.0),
        KeyAction::Divide,
    ),
    key("0", PxRect::new(295.0, 1409.0, 405.0, 1495.0), KeyAction::Digit(0)),
    key(
        "decimal",
        PxRect::new(476.0, 1409.0, 587.0, 1495.0),
        KeyAction::Decimal,
    ),
    key(
        "rs",
        PxRect::new(657.0, 1409.0, 768.0, 1495.0),
        KeyAction::RunStop,
    ),
];

pub struct Hp67Panel;

impl Hp67Panel {
    pub fn show(ui: &mut Ui, state: &Hp67State, photo: &TextureHandle) -> Vec<UiEvent> {
        let available = ui.available_size();
        let (host, _) = ui.allocate_exact_size(available, Sense::hover());
        if host.width() <= 0.0 || host.height() <= 0.0 {
            return Vec::new();
        }

        let photo_rect = fit_photo(host);
        let painter = ui.painter_at(host).with_clip_rect(photo_rect);
        painter.image(
            photo.id(),
            photo_rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let mut events = Vec::new();

        // The two mechanical slide switches remain part of the photograph, but
        // retain their emulator hit areas.  The display state makes power changes
        // immediately visible even before a later dedicated slider sprite pass.
        let power = ui.interact(
            source_to_screen(photo_rect, PxRect::new(150.0, 270.0, 386.0, 334.0)),
            ui.make_persistent_id("photo-power-switch"),
            Sense::click(),
        );
        if power.clicked() {
            events.push(UiEvent::TogglePower);
        }
        if power.hovered() {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
        }

        let mode = ui.interact(
            source_to_screen(photo_rect, PxRect::new(455.0, 270.0, 775.0, 334.0)),
            ui.make_persistent_id("photo-mode-switch"),
            Sense::click(),
        );
        if mode.clicked() {
            events.push(UiEvent::ToggleMode);
        }
        if mode.hovered() {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
        }

        for key in KEYS {
            let rect = source_to_screen(photo_rect, key.src);
            let response = ui.interact(
                rect,
                ui.make_persistent_id(("photo-key", key.id)),
                Sense::click(),
            );
            if response.clicked() {
                events.push(UiEvent::Key(key.action));
            }
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }

            let down = response.is_pointer_button_down_on();
            let press = ui.ctx().animate_bool_with_time(
                response.id.with("travel"),
                down,
                if down { 0.040 } else { 0.075 },
            );
            if press > 0.001 {
                paint_pressed_key(&painter, photo, photo_rect, key.src, press);
            }
        }

        draw_led_display(&painter, photo_rect, state.display_text());
        events
    }
}

fn fit_photo(host: Rect) -> Rect {
    let scale = (host.width() / PHOTO_W).min(host.height() / PHOTO_H);
    Rect::from_center_size(host.center(), vec2(PHOTO_W * scale, PHOTO_H * scale))
}

fn source_to_screen(photo: Rect, src: PxRect) -> Rect {
    let sx = photo.width() / PHOTO_W;
    let sy = photo.height() / PHOTO_H;
    Rect::from_min_max(
        pos2(photo.left() + src.x0 * sx, photo.top() + src.y0 * sy),
        pos2(photo.left() + src.x1 * sx, photo.top() + src.y1 * sy),
    )
}

fn source_uv(src: PxRect) -> Rect {
    Rect::from_min_max(
        pos2(src.x0 / PHOTO_W, src.y0 / PHOTO_H),
        pos2(src.x1 / PHOTO_W, src.y1 / PHOTO_H),
    )
}

fn paint_pressed_key(
    p: &Painter,
    photo: &TextureHandle,
    photo_rect: Rect,
    src: PxRect,
    press: f32,
) {
    let original = source_to_screen(photo_rect, src);
    let scale = photo_rect.height() / PHOTO_H;

    // Classic-series keys have short, firm travel.  Keep the movement visible
    // without opening the exaggerated rectangular cavity of the first pass.
    let travel = 3.8 * scale * press;

    // Reconstruct the newly exposed strip from the photographed panel directly
    // above the key.  This preserves the real texture and lighting instead of
    // inventing a flat black slot, which looked artificial in motion.
    let gap_h = travel + 0.35 * scale;
    let gap = Rect::from_min_max(
        original.min,
        pos2(original.max.x, (original.min.y + gap_h).min(original.max.y)),
    );
    let recess_src = PxRect::new(src.x0, (src.y0 - 4.0).max(0.0), src.x1, src.y0);
    let recess_shade = (248.0 - 8.0 * press).round() as u8;
    p.image(
        photo.id(),
        gap,
        source_uv(recess_src),
        Color32::from_rgb(recess_shade, recess_shade, recess_shade),
    );

    // Only a restrained contact/occlusion cue is needed at the top edge.
    let edge_y = original.top() + travel;
    p.line_segment(
        [pos2(original.left() + scale * 3.0, edge_y), pos2(original.right() - scale * 3.0, edge_y)],
        Stroke::new(
            (0.75 * scale).max(0.45),
            Color32::from_rgba_unmultiplied(0, 0, 0, (42.0 + 34.0 * press).round() as u8),
        ),
    );

    let moved = original.translate(vec2(0.0, travel));
    let shade = (255.0 - 7.0 * press).round() as u8;
    p.image(
        photo.id(),
        moved,
        source_uv(src),
        Color32::from_rgb(shade, shade, shade),
    );

    // A soft lower contact shadow gives depth without making the key look cut out.
    let shadow_alpha = (30.0 + 34.0 * press).round() as u8;
    p.line_segment(
        [
            pos2(moved.left() + scale * 6.0, moved.bottom()),
            pos2(moved.right() - scale * 6.0, moved.bottom()),
        ],
        Stroke::new(
            (1.0 * scale).max(0.6),
            Color32::from_rgba_unmultiplied(0, 0, 0, shadow_alpha),
        ),
    );
}

fn draw_led_display(p: &Painter, photo: Rect, value: &str) {
    if value.is_empty() {
        return;
    }

    // The photo already supplies the glass, bezel, reflections and black level;
    // this overlay paints only the emitting LED dies.
    let display = source_to_screen(photo, PxRect::new(178.0, 105.0, 750.0, 208.0));
    let p = p.with_clip_rect(display);
    let cells = display_cells(value);
    let pitch = display.width() / 15.0;
    let height = display.height() * 0.48;
    let cy = display.center().y + display.height() * 0.03;

    for (i, ch) in cells.iter().enumerate() {
        if *ch == ' ' {
            continue;
        }
        let cx = display.left() + pitch * (i as f32 + 0.5);
        draw_led_char(&p, cx, cy, pitch * 0.62, height, *ch);
    }
}

fn display_cells(value: &str) -> [char; 15] {
    let mut cells = [' '; 15];
    let chars: Vec<char> = value.chars().take(15).collect();
    let start = 15usize.saturating_sub(chars.len());
    for (dst, ch) in cells[start..].iter_mut().zip(chars) {
        *dst = ch;
    }
    cells
}

const A: u8 = 1 << 0;
const B: u8 = 1 << 1;
const C: u8 = 1 << 2;
const D: u8 = 1 << 3;
const E: u8 = 1 << 4;
const F: u8 = 1 << 5;
const G: u8 = 1 << 6;

fn digit_mask(ch: char) -> u8 {
    match ch {
        '0' => A | B | C | D | E | F,
        '1' => B | C,
        '2' => A | B | D | E | G,
        '3' => A | B | C | D | G,
        '4' => B | C | F | G,
        '5' => A | C | D | F | G,
        '6' => A | C | D | E | F | G,
        '7' => A | B | C,
        '8' => A | B | C | D | E | F | G,
        '9' => A | B | C | D | F | G,
        '-' => G,
        _ => 0,
    }
}

fn draw_led_char(p: &Painter, cx: f32, cy: f32, width: f32, height: f32, ch: char) {
    let core = Color32::from_rgb(245, 43, 22);
    let glow = Color32::from_rgba_unmultiplied(255, 45, 22, 44);
    let stroke = (height * 0.070).max(1.1);

    if ch == '.' {
        p.circle_filled(pos2(cx, cy + height * 0.20), stroke * 1.25, glow);
        p.circle_filled(pos2(cx, cy + height * 0.20), stroke * 0.62, core);
        return;
    }
    if ch == 'x' || ch == 'X' {
        led_line(
            p,
            pos2(cx - width * 0.32, cy - height * 0.25),
            pos2(cx + width * 0.32, cy + height * 0.25),
            stroke,
            glow,
            core,
        );
        led_line(
            p,
            pos2(cx + width * 0.32, cy - height * 0.25),
            pos2(cx - width * 0.32, cy + height * 0.25),
            stroke,
            glow,
            core,
        );
        return;
    }
    if ch == '/' {
        led_line(
            p,
            pos2(cx + width * 0.26, cy - height * 0.42),
            pos2(cx - width * 0.26, cy + height * 0.42),
            stroke,
            glow,
            core,
        );
        return;
    }
    if ch == '+' {
        led_line(
            p,
            pos2(cx - width * 0.30, cy),
            pos2(cx + width * 0.30, cy),
            stroke,
            glow,
            core,
        );
        led_line(
            p,
            pos2(cx, cy - height * 0.28),
            pos2(cx, cy + height * 0.28),
            stroke,
            glow,
            core,
        );
        return;
    }

    let mask = digit_mask(ch);
    if mask == 0 {
        return;
    }
    let xl = cx - width * 0.36;
    let xr = cx + width * 0.36;
    let yt = cy - height * 0.46;
    let ym = cy;
    let yb = cy + height * 0.46;
    let inset = width * 0.08;

    let segments = [
        (A, pos2(xl + inset, yt), pos2(xr - inset, yt)),
        (B, pos2(xr, yt + inset), pos2(xr, ym - inset)),
        (C, pos2(xr, ym + inset), pos2(xr, yb - inset)),
        (D, pos2(xl + inset, yb), pos2(xr - inset, yb)),
        (E, pos2(xl, ym + inset), pos2(xl, yb - inset)),
        (F, pos2(xl, yt + inset), pos2(xl, ym - inset)),
        (G, pos2(xl + inset, ym), pos2(xr - inset, ym)),
    ];
    for (bit, from, to) in segments {
        if mask & bit != 0 {
            led_line(p, from, to, stroke, glow, core);
        }
    }
}

fn led_line(
    p: &Painter,
    from: Pos2,
    to: Pos2,
    width: f32,
    glow: Color32,
    core: Color32,
) {
    p.line_segment([from, to], Stroke::new(width * 2.7, glow));
    p.line_segment([from, to], Stroke::new(width, core));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photo_key_table_contains_all_35_keys() {
        assert_eq!(KEYS.len(), 35);
    }

    #[test]
    fn source_mapping_preserves_photo_edges() {
        let photo = Rect::from_min_size(Pos2::ZERO, Vec2::new(PHOTO_W, PHOTO_H));
        let mapped = source_to_screen(photo, PxRect::new(0.0, 0.0, PHOTO_W, PHOTO_H));
        assert_eq!(mapped, photo);
    }
}
