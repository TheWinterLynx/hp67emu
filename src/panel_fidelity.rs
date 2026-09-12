use eframe::egui::{Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::hp67::{Hp67State, KeyAction, RunMode, UiEvent};

pub const DESIGN_W: f32 = 330.0;
pub const DESIGN_H: f32 = 620.0;

const PANEL: Color32 = Color32::from_rgb(45, 46, 43);
const PANEL_DARK: Color32 = Color32::from_rgb(25, 25, 23);
const CASE_GREEN: Color32 = Color32::from_rgb(79, 87, 57);
const CASE_GREEN_DARK: Color32 = Color32::from_rgb(55, 61, 43);
const SILVER: Color32 = Color32::from_rgb(171, 172, 166);
const SILVER_LIGHT: Color32 = Color32::from_rgb(225, 227, 221);
const WHITE: Color32 = Color32::from_rgb(230, 233, 229);
const F_YELLOW: Color32 = Color32::from_rgb(214, 205, 61);
const G_CYAN: Color32 = Color32::from_rgb(81, 203, 223);
const OLIVE: Color32 = Color32::from_rgb(171, 173, 105);
const ORANGE: Color32 = Color32::from_rgb(236, 176, 44);
const BLUE: Color32 = Color32::from_rgb(59, 186, 213);
const KEY_WHITE: Color32 = Color32::from_rgb(225, 230, 232);
const KEY_BLACK: Color32 = Color32::from_rgb(24, 27, 29);

#[derive(Debug, Clone, Copy)]
enum KeyStyle { Olive, Orange, Blue, White, Black }

#[derive(Debug, Clone, Copy)]
struct KeySpec {
    id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    main: &'static str,
    sub: Option<&'static str>,
    style: KeyStyle,
    action: KeyAction,
}

const fn key(id: &'static str, x: f32, y: f32, w: f32, h: f32, main: &'static str, sub: Option<&'static str>, style: KeyStyle, action: KeyAction) -> KeySpec {
    KeySpec { id, x, y, w, h, main, sub, style, action }
}

const KEYS: &[KeySpec] = &[
    key("a", 50.0, 170.0, 30.0, 28.0, "A", None, KeyStyle::Olive, KeyAction::A),
    key("b", 101.0, 170.0, 30.0, 28.0, "B", None, KeyStyle::Olive, KeyAction::B),
    key("c", 152.0, 170.0, 30.0, 28.0, "C", None, KeyStyle::Olive, KeyAction::C),
    key("d", 203.0, 170.0, 30.0, 28.0, "D", None, KeyStyle::Olive, KeyAction::D),
    key("e", 254.0, 170.0, 30.0, 28.0, "E", None, KeyStyle::Olive, KeyAction::E),

    key("sigma", 50.0, 224.0, 30.0, 30.0, "SIGMA+", Some("SIGMA-"), KeyStyle::Olive, KeyAction::SigmaPlus),
    key("gto", 101.0, 224.0, 30.0, 30.0, "GTO", Some("RTN"), KeyStyle::Olive, KeyAction::Gto),
    key("dsp", 152.0, 224.0, 30.0, 30.0, "DSP", Some("ENG"), KeyStyle::Olive, KeyAction::Dsp),
    key("indirect", 203.0, 224.0, 30.0, 30.0, "(i)", Some("X<>I"), KeyStyle::Olive, KeyAction::Indirect),
    key("sst", 254.0, 224.0, 30.0, 30.0, "SST", Some("BST"), KeyStyle::Olive, KeyAction::Sst),

    key("f", 50.0, 279.0, 30.0, 28.0, "f", None, KeyStyle::Orange, KeyAction::FunctionF),
    key("g", 101.0, 279.0, 30.0, 28.0, "g", None, KeyStyle::Blue, KeyAction::FunctionG),
    key("sto", 152.0, 279.0, 30.0, 30.0, "STO", Some("ST I"), KeyStyle::Olive, KeyAction::Sto),
    key("rcl", 203.0, 279.0, 30.0, 30.0, "RCL", Some("RC I"), KeyStyle::Olive, KeyAction::Rcl),
    key("h", 254.0, 279.0, 30.0, 28.0, "h", None, KeyStyle::Black, KeyAction::FunctionH),

    key("enter", 50.0, 333.0, 81.0, 30.0, "ENTER", Some("DEG"), KeyStyle::Olive, KeyAction::Enter),
    key("chs", 152.0, 333.0, 30.0, 30.0, "CHS", Some("RAD"), KeyStyle::Olive, KeyAction::ChangeSign),
    key("eex", 203.0, 333.0, 30.0, 30.0, "EEX", Some("GRD"), KeyStyle::Olive, KeyAction::Enter),
    key("clx", 254.0, 333.0, 30.0, 30.0, "CLX", Some("DEL"), KeyStyle::Olive, KeyAction::ClearX),

    key("minus", 50.0, 385.0, 30.0, 30.0, "-", Some("SF"), KeyStyle::Olive, KeyAction::Subtract),
    key("7", 101.0, 385.0, 30.0, 30.0, "7", Some("X<>Y"), KeyStyle::White, KeyAction::Digit(7)),
    key("8", 177.0, 385.0, 30.0, 30.0, "8", Some("RDN"), KeyStyle::White, KeyAction::Digit(8)),
    key("9", 253.0, 385.0, 30.0, 30.0, "9", Some("RUP"), KeyStyle::White, KeyAction::Digit(9)),

    key("plus", 50.0, 437.0, 30.0, 30.0, "+", Some("CF"), KeyStyle::Olive, KeyAction::Add),
    key("4", 101.0, 437.0, 30.0, 30.0, "4", Some("1/X"), KeyStyle::White, KeyAction::Digit(4)),
    key("5", 177.0, 437.0, 30.0, 30.0, "5", Some("Y^X"), KeyStyle::White, KeyAction::Digit(5)),
    key("6", 253.0, 437.0, 30.0, 30.0, "6", Some("ABS"), KeyStyle::White, KeyAction::Digit(6)),

    key("multiply", 50.0, 489.0, 30.0, 30.0, "x", Some("F?"), KeyStyle::Olive, KeyAction::Multiply),
    key("1", 101.0, 489.0, 30.0, 30.0, "1", Some("PAUSE"), KeyStyle::White, KeyAction::Digit(1)),
    key("2", 177.0, 489.0, 30.0, 30.0, "2", Some("PI"), KeyStyle::White, KeyAction::Digit(2)),
    key("3", 253.0, 489.0, 30.0, 30.0, "3", Some("REG"), KeyStyle::White, KeyAction::Digit(3)),

    key("divide", 50.0, 541.0, 30.0, 30.0, "/", Some("N!"), KeyStyle::Olive, KeyAction::Divide),
    key("0", 101.0, 541.0, 30.0, 30.0, "0", Some("LST X"), KeyStyle::White, KeyAction::Digit(0)),
    key("decimal", 177.0, 541.0, 30.0, 30.0, ".", Some("H.MS+"), KeyStyle::White, KeyAction::Decimal),
    key("rs", 253.0, 541.0, 30.0, 30.0, "R/S", Some("SPACE"), KeyStyle::White, KeyAction::RunStop),
];

pub struct Hp67Panel;

impl Hp67Panel {
    pub fn show(ui: &mut Ui, state: &Hp67State) -> Vec<UiEvent> {
        let available = ui.available_size();
        let (host_rect, _) = ui.allocate_exact_size(available, Sense::hover());
        let scale = scale_for(host_rect.size());
        if scale <= 0.0 { return Vec::new(); }

        let panel_size = Vec2::new(DESIGN_W * scale, DESIGN_H * scale);
        let t = Transform {
            origin: Pos2::new(host_rect.center().x - panel_size.x * 0.5, host_rect.center().y - panel_size.y * 0.5),
            scale,
        };
        let painter = ui.painter_at(host_rect);
        let mut events = Vec::new();

        draw_chassis(&painter, t);
        draw_display(&painter, t, state.display_text());

        let power_response = ui.interact(t.rect(44.0, 92.0, 91.0, 28.0), ui.make_persistent_id("hp67-power-switch"), Sense::click());
        if power_response.clicked() { events.push(UiEvent::TogglePower); }
        draw_power_switch(&painter, t, state.power_on, power_response.hovered());

        let mode_response = ui.interact(t.rect(166.0, 92.0, 121.0, 28.0), ui.make_persistent_id("hp67-mode-switch"), Sense::click());
        if mode_response.clicked() { events.push(UiEvent::ToggleMode); }
        draw_mode_switch(&painter, t, state.mode, mode_response.hovered());

        draw_keyboard_legends(&painter, t);
        for (index, spec) in KEYS.iter().enumerate() {
            let hit = t.rect(spec.x - 1.0, spec.y - 1.0, spec.w + 2.0, spec.h + 4.0);
            let response = ui.interact(hit, ui.make_persistent_id(("hp67-key", index, spec.id)), Sense::click());
            if response.clicked() { events.push(UiEvent::Key(spec.action)); }
            draw_key(&painter, t, *spec, response.hovered(), response.is_pointer_button_down_on());
        }
        draw_branding(&painter, t);
        events
    }
}

pub fn scale_for(available: Vec2) -> f32 {
    if available.x <= 0.0 || available.y <= 0.0 { 0.0 } else { (available.x / DESIGN_W).min(available.y / DESIGN_H) }
}

#[derive(Debug, Clone, Copy)]
struct Transform { origin: Pos2, scale: f32 }
impl Transform {
    fn s(self, v: f32) -> f32 { v * self.scale }
    fn pos(self, x: f32, y: f32) -> Pos2 { Pos2::new(self.origin.x + x * self.scale, self.origin.y + y * self.scale) }
    fn rect(self, x: f32, y: f32, w: f32, h: f32) -> Rect { Rect::from_min_size(self.pos(x, y), Vec2::new(self.s(w), self.s(h))) }
}

fn draw_chassis(p: &Painter, t: Transform) {
    p.rect_filled(t.rect(4.0, 3.0, 322.0, 614.0), t.s(19.0), Color32::from_rgb(15, 16, 13));
    p.rect_filled(t.rect(7.0, 1.0, 316.0, 615.0), t.s(18.0), CASE_GREEN_DARK);
    p.rect_filled(t.rect(11.0, 2.0, 308.0, 612.0), t.s(15.0), CASE_GREEN);
    p.rect_filled(t.rect(23.0, 5.0, 284.0, 609.0), t.s(11.0), Color32::from_rgb(108, 108, 100));
    p.rect_filled(t.rect(26.0, 7.0, 278.0, 605.0), t.s(9.0), SILVER_LIGHT);
    p.rect_filled(t.rect(30.0, 11.0, 270.0, 596.0), t.s(7.0), PANEL);
    p.rect_stroke(t.rect(28.0, 8.5, 274.0, 601.0), t.s(8.0), Stroke::new(t.s(1.2), SILVER));
    p.line_segment([t.pos(36.0, 126.0), t.pos(294.0, 126.0)], Stroke::new(t.s(2.0), PANEL_DARK));
}

fn draw_display(p: &Painter, t: Transform, text_value: &str) {
    p.rect_filled(t.rect(37.0, 18.0, 256.0, 77.0), t.s(4.0), Color32::from_rgb(83, 84, 79));
    p.rect_filled(t.rect(39.0, 20.0, 252.0, 73.0), t.s(3.0), SILVER_LIGHT);
    p.rect_filled(t.rect(42.0, 24.0, 246.0, 66.0), t.s(1.8), Color32::from_rgb(43, 30, 27));
    p.rect_filled(t.rect(45.0, 27.0, 240.0, 59.0), t.s(1.0), Color32::from_rgb(55, 31, 29));
    p.rect_filled(t.rect(47.0, 29.0, 236.0, 3.0), t.s(1.0), Color32::from_rgb(79, 48, 44));
    if !text_value.is_empty() { draw_segment_string(p, t, text_value, 49.0, 39.0, 232.0); }
}

fn draw_power_switch(p: &Painter, t: Transform, on: bool, hovered: bool) {
    label(p, t, 48.0, 106.0, Align2::LEFT_CENTER, 7.4, WHITE, "OFF");
    label(p, t, 116.0, 106.0, Align2::RIGHT_CENTER, 7.4, WHITE, "ON");
    draw_slider(p, t, 72.0, 101.0, 42.0, on, hovered);
}

fn draw_mode_switch(p: &Painter, t: Transform, mode: RunMode, hovered: bool) {
    label(p, t, 166.0, 106.0, Align2::LEFT_CENTER, 7.2, WHITE, "W/PRGM");
    label(p, t, 286.0, 106.0, Align2::RIGHT_CENTER, 7.2, WHITE, "RUN");
    draw_slider(p, t, 225.0, 101.0, 39.0, matches!(mode, RunMode::Run), hovered);
}

fn draw_slider(p: &Painter, t: Transform, x: f32, y: f32, w: f32, right: bool, hovered: bool) {
    let track = t.rect(x, y, w, 8.0);
    p.rect_filled(track, t.s(1.0), Color32::from_rgb(7, 8, 8));
    p.rect_stroke(track, t.s(1.0), Stroke::new(t.s(if hovered { 1.0 } else { 0.7 }), if hovered { Color32::from_rgb(156, 158, 151) } else { Color32::from_rgb(69, 70, 66) }));
    for i in 0..10 {
        let xx = x + 2.0 + i as f32 * (w - 4.0) / 9.0;
        p.line_segment([t.pos(xx, y + 1.2), t.pos(xx, y + 6.8)], Stroke::new(t.s(0.45), Color32::from_rgb(52, 53, 50)));
    }
    let knob_x = if right { x + w - 13.0 } else { x + 2.0 };
    p.rect_filled(t.rect(knob_x + 0.8, y + 0.4, 11.0, 10.0), t.s(1.3), Color32::from_rgb(19, 20, 19));
    p.rect_filled(t.rect(knob_x, y - 1.0, 11.0, 10.0), t.s(1.3), Color32::from_rgb(102, 104, 99));
    p.line_segment([t.pos(knob_x + 1.5, y), t.pos(knob_x + 9.5, y)], Stroke::new(t.s(0.75), Color32::from_rgb(160, 161, 155)));
}

fn draw_keyboard_legends(p: &Painter, t: Transform) {
    draw_one_over_x(p, t, 65.0, 151.5, 9.0, WHITE);
    draw_sqrt_x(p, t, 116.0, 151.5, 9.0, WHITE);
    draw_power(p, t, 167.0, 151.5, "y", "x", 9.0, WHITE, WHITE);
    draw_r_arrow(p, t, 218.0, 151.5, false, 9.0, WHITE);
    draw_swap_text(p, t, 269.0, 151.5, "x", "y", 9.0, WHITE, WHITE);

    for (x, value) in [(65.0, "a"), (116.0, "b"), (167.0, "c"), (218.0, "d"), (269.0, "e")] { centered(p, t, x, 207.0, 7.0, F_YELLOW, value); }

    draw_x_bar(p, t, 59.0, 263.0, 6.5, F_YELLOW); centered(p, t, 72.0, 263.0, 6.8, G_CYAN, "s");
    centered(p, t, 109.0, 263.0, 6.8, F_YELLOW, "GSB"); centered(p, t, 125.0, 263.0, 6.8, G_CYAN, "f");
    centered(p, t, 158.0, 263.0, 6.8, F_YELLOW, "FIX"); centered(p, t, 178.0, 263.0, 6.8, G_CYAN, "SCI");
    centered(p, t, 218.0, 263.0, 6.8, F_YELLOW, "RND");
    centered(p, t, 260.0, 263.0, 6.8, F_YELLOW, "LBL"); centered(p, t, 278.0, 263.0, 6.8, G_CYAN, "f");

    centered(p, t, 161.0, 317.0, 6.8, F_YELLOW, "DSZ"); centered(p, t, 180.0, 317.0, 6.8, G_CYAN, "(i)");
    centered(p, t, 212.0, 317.0, 6.8, F_YELLOW, "ISZ"); centered(p, t, 231.0, 317.0, 6.8, G_CYAN, "(i)");

    centered(p, t, 64.0, 374.0, 6.8, F_YELLOW, "W/DATA"); centered(p, t, 116.0, 374.0, 6.8, G_CYAN, "MERGE");
    draw_swap_text(p, t, 167.0, 374.0, "P", "S", 6.8, F_YELLOW, F_YELLOW);
    centered(p, t, 218.0, 374.0, 6.8, F_YELLOW, "CL REG"); centered(p, t, 269.0, 374.0, 6.8, F_YELLOW, "CL PRGM");

    draw_equal_pair(p, t, 65.0, 425.0, false);
    centered(p, t, 112.0, 425.0, 6.8, F_YELLOW, "LN"); draw_power(p, t, 127.0, 425.0, "e", "x", 6.8, G_CYAN, G_CYAN);
    centered(p, t, 183.0, 425.0, 6.8, F_YELLOW, "LOG"); draw_power(p, t, 205.0, 425.0, "10", "x", 6.8, G_CYAN, G_CYAN);
    draw_sqrt_x(p, t, 258.0, 425.0, 6.8, F_YELLOW); draw_power(p, t, 277.0, 425.0, "x", "2", 6.8, G_CYAN, G_CYAN);

    draw_equal_pair(p, t, 65.0, 477.0, true);
    draw_inverse_trig(p, t, 116.0, 477.0, "SIN"); draw_inverse_trig(p, t, 192.0, 477.0, "COS"); draw_inverse_trig(p, t, 268.0, 477.0, "TAN");

    draw_relational_pair(p, t, 65.0, 529.0, '<', true, 6.3);
    draw_swap_text(p, t, 116.0, 529.0, "R", "P", 6.5, F_YELLOW, G_CYAN); draw_swap_text(p, t, 192.0, 529.0, "D", "R", 6.5, F_YELLOW, G_CYAN); draw_swap_text(p, t, 268.0, 529.0, "H", "H.MS", 6.2, F_YELLOW, G_CYAN);

    draw_relational_pair(p, t, 65.0, 581.0, '>', false, 6.3);
    centered(p, t, 108.0, 581.0, 6.6, F_YELLOW, "%"); centered(p, t, 126.0, 581.0, 6.6, G_CYAN, "%CH");
    centered(p, t, 182.0, 581.0, 6.6, F_YELLOW, "INT"); centered(p, t, 205.0, 581.0, 6.6, G_CYAN, "FRAC");
    centered(p, t, 258.0, 581.0, 6.6, F_YELLOW, "-x-"); centered(p, t, 280.0, 581.0, 6.6, G_CYAN, "STK");
}

fn key_palette(style: KeyStyle, hovered: bool) -> (Color32, Color32, Color32, Color32) {
    let (face, lip, text) = match style {
        KeyStyle::Olive => (OLIVE, Color32::from_rgb(120, 121, 71), WHITE),
        KeyStyle::Orange => (ORANGE, Color32::from_rgb(179, 121, 22), Color32::from_rgb(35, 34, 25)),
        KeyStyle::Blue => (BLUE, Color32::from_rgb(30, 126, 148), Color32::from_rgb(25, 32, 35)),
        KeyStyle::White => (KEY_WHITE, Color32::from_rgb(172, 177, 179), Color32::from_rgb(24, 28, 30)),
        KeyStyle::Black => (KEY_BLACK, Color32::from_rgb(8, 10, 11), WHITE),
    };
    let face = if hovered { Color32::from_rgb(face.r().saturating_add(13), face.g().saturating_add(13), face.b().saturating_add(13)) } else { face };
    (face, lip, text, Color32::from_rgb(23, 24, 23))
}

fn draw_key(p: &Painter, t: Transform, spec: KeySpec, hovered: bool, pressed: bool) {
    let press = if pressed { 1.6 } else { 0.0 };
    let y = spec.y + press;
    let (face, lip, main_text, lower_text) = key_palette(spec.style, hovered);

    p.rect_filled(t.rect(spec.x + 1.7, y + 3.0, spec.w, spec.h), t.s(3.0), Color32::from_rgba_premultiplied(0, 0, 0, 150));
    p.rect_filled(t.rect(spec.x - 0.8, y + 0.8, spec.w + 1.6, spec.h + 1.5), t.s(3.2), Color32::from_rgb(20, 21, 20));
    p.rect_filled(t.rect(spec.x + 0.5, y + 3.0, spec.w - 1.0, spec.h - 0.5), t.s(3.0), lip);

    let face_rect = t.rect(spec.x + 1.0, y, spec.w - 2.0, spec.h - 4.2);
    p.rect_filled(face_rect, t.s(2.6), face);
    p.line_segment([Pos2::new(face_rect.left() + t.s(2.3), face_rect.top() + t.s(1.1)), Pos2::new(face_rect.right() - t.s(2.3), face_rect.top() + t.s(1.1))], Stroke::new(t.s(1.0), Color32::from_rgba_premultiplied(255, 255, 255, 125)));
    p.line_segment([Pos2::new(face_rect.right() - t.s(0.9), face_rect.top() + t.s(2.0)), Pos2::new(face_rect.right() - t.s(0.9), face_rect.bottom() - t.s(1.5))], Stroke::new(t.s(0.9), Color32::from_rgba_premultiplied(0, 0, 0, 55)));

    if spec.sub.is_some() {
        let sy = y + 15.2;
        p.line_segment([t.pos(spec.x + 2.1, sy), t.pos(spec.x + spec.w - 2.1, sy)], Stroke::new(t.s(1.05), Color32::from_rgba_premultiplied(246, 248, 243, 170)));
        p.line_segment([t.pos(spec.x + 2.3, sy + 1.1), t.pos(spec.x + spec.w - 2.3, sy + 1.1)], Stroke::new(t.s(0.8), Color32::from_rgba_premultiplied(0, 0, 0, 75)));
    }

    draw_key_main(p, t, spec, y, main_text);
    if let Some(sub) = spec.sub { draw_key_sub(p, t, spec, y, lower_text, sub); }
}

fn draw_key_main(p: &Painter, t: Transform, spec: KeySpec, y: f32, color: Color32) {
    if spec.id == "enter" {
        label(p, t, spec.x + 34.0, y + 8.0, Align2::CENTER_CENTER, 9.6, color, "ENTER");
        draw_triangle(p, t, spec.x + 64.5, y + 7.9, 3.2, true, color);
        return;
    }
    let size = match spec.id {
        "f" | "g" | "h" => 11.5,
        "sigma" => 10.0,
        "minus" | "plus" | "multiply" | "divide" => 13.5,
        "7" | "8" | "9" | "4" | "5" | "6" | "1" | "2" | "3" | "0" => 12.2,
        _ => 9.9,
    };
    if spec.id == "sigma" {
        draw_sigma(p, t, spec.x + spec.w * 0.5 - 2.2, y + 7.8, 8.7, color);
        label(p, t, spec.x + spec.w * 0.5 + 4.0, y + 7.8, Align2::CENTER_CENTER, 9.0, color, "+");
    } else if spec.id == "multiply" {
        draw_cross(p, t, spec.x + spec.w * 0.5, y + 7.8, 4.3, color);
    } else if spec.id == "divide" {
        draw_divide(p, t, spec.x + spec.w * 0.5, y + 7.7, 4.4, color);
    } else {
        label(p, t, spec.x + spec.w * 0.5, y + if spec.sub.is_some() { 7.8 } else { 11.8 }, Align2::CENTER_CENTER, size, color, spec.main);
    }
}

fn draw_key_sub(p: &Painter, t: Transform, spec: KeySpec, y: f32, color: Color32, sub: &str) {
    let cy = y + 21.8;
    match spec.id {
        "sigma" => { draw_sigma(p, t, spec.x + spec.w * 0.5 - 2.0, cy, 6.2, color); label(p, t, spec.x + spec.w * 0.5 + 3.4, cy, Align2::CENTER_CENTER, 6.3, color, "-"); }
        "indirect" => draw_swap_text(p, t, spec.x + spec.w * 0.5, cy, "x", "I", 6.4, color, color),
        "7" => draw_swap_text(p, t, spec.x + spec.w * 0.5, cy, "x", "y", 6.2, color, color),
        "8" => draw_r_arrow(p, t, spec.x + spec.w * 0.5, cy, false, 6.4, color),
        "9" => draw_r_arrow(p, t, spec.x + spec.w * 0.5, cy, true, 6.4, color),
        "4" => draw_one_over_x(p, t, spec.x + spec.w * 0.5, cy, 6.4, color),
        "5" => draw_power(p, t, spec.x + spec.w * 0.5, cy, "y", "x", 6.4, color, color),
        "2" => draw_pi(p, t, spec.x + spec.w * 0.5, cy, 7.0, color),
        _ => centered(p, t, spec.x + spec.w * 0.5, cy, if sub.len() > 4 { 5.7 } else { 6.4 }, color, sub),
    }
}

fn draw_branding(p: &Painter, t: Transform) {
    p.rect_filled(t.rect(43.0, 592.0, 244.0, 12.0), 0.0, Color32::from_rgb(16, 20, 22));
    p.rect_stroke(t.rect(43.0, 592.0, 244.0, 12.0), 0.0, Stroke::new(t.s(0.65), Color32::from_rgb(189, 191, 185)));
    p.rect_filled(t.rect(47.0, 594.0, 20.0, 8.0), 0.0, Color32::from_rgb(38, 158, 204));
    centered(p, t, 57.0, 598.0, 5.0, WHITE, "hp");
    centered(p, t, 178.0, 598.0, 7.3, Color32::from_rgb(205, 209, 205), "H E W L E T T - P A C K A R D   6 7");
}

fn centered(p: &Painter, t: Transform, x: f32, y: f32, size: f32, color: Color32, value: &str) {
    label(p, t, x, y, Align2::CENTER_CENTER, size, color, value);
}

fn label(p: &Painter, t: Transform, x: f32, y: f32, align: Align2, size: f32, color: Color32, value: &str) {
    p.text(t.pos(x, y), align, value, FontId::proportional(t.s(size)), color);
}

fn draw_one_over_x(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, color: Color32) {
    let s = size / 9.0;
    label(p, t, cx - 6.2 * s, cy, Align2::CENTER_CENTER, size, color, "1");
    p.line_segment([t.pos(cx - 1.6 * s, cy + 4.0 * s), t.pos(cx + 2.4 * s, cy - 4.0 * s)], Stroke::new(t.s(0.8 * s), color));
    label(p, t, cx + 6.0 * s, cy, Align2::CENTER_CENTER, size, color, "x");
}

fn draw_sqrt_x(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, color: Color32) {
    let s = size / 9.0;
    let x0 = cx - 8.5 * s;
    p.line_segment([t.pos(x0, cy + 0.5 * s), t.pos(x0 + 2.5 * s, cy + 4.0 * s)], Stroke::new(t.s(1.0 * s), color));
    p.line_segment([t.pos(x0 + 2.5 * s, cy + 4.0 * s), t.pos(x0 + 5.7 * s, cy - 5.0 * s)], Stroke::new(t.s(1.0 * s), color));
    p.line_segment([t.pos(x0 + 5.7 * s, cy - 5.0 * s), t.pos(x0 + 15.0 * s, cy - 5.0 * s)], Stroke::new(t.s(0.8 * s), color));
    label(p, t, cx + 3.5 * s, cy + 0.5 * s, Align2::CENTER_CENTER, size, color, "x");
}

fn draw_power(p: &Painter, t: Transform, cx: f32, cy: f32, base: &str, exp: &str, size: f32, base_color: Color32, exp_color: Color32) {
    let base_w = if base.len() > 1 { 5.2 } else { 2.5 };
    label(p, t, cx - base_w, cy + 1.2, Align2::CENTER_CENTER, size, base_color, base);
    label(p, t, cx + base_w + 2.2, cy - size * 0.42, Align2::CENTER_CENTER, size * 0.65, exp_color, exp);
}

fn draw_r_arrow(p: &Painter, t: Transform, cx: f32, cy: f32, up: bool, size: f32, color: Color32) {
    label(p, t, cx - 3.0, cy, Align2::CENTER_CENTER, size, color, "R");
    draw_triangle(p, t, cx + 5.2, cy + 0.3, size * 0.32, up, color);
}

fn draw_swap_text(p: &Painter, t: Transform, cx: f32, cy: f32, left: &str, right: &str, size: f32, left_color: Color32, right_color: Color32) {
    let span = if right.len() > 1 { 10.5 } else { 8.0 };
    label(p, t, cx - span, cy, Align2::CENTER_CENTER, size, left_color, left);
    label(p, t, cx + span, cy, Align2::CENTER_CENTER, size, right_color, right);
    draw_double_arrow(p, t, cx, cy, size * 0.55, if left_color == right_color { left_color } else { G_CYAN });
}

fn draw_double_arrow(p: &Painter, t: Transform, cx: f32, cy: f32, half: f32, color: Color32) {
    let x1 = cx - half; let x2 = cx + half; let yu = cy - 1.4; let yd = cy + 1.4;
    p.line_segment([t.pos(x1, yu), t.pos(x2, yu)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(x2 - 2.0, yu - 1.5), t.pos(x2, yu)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(x2 - 2.0, yu + 1.5), t.pos(x2, yu)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(x2, yd), t.pos(x1, yd)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(x1 + 2.0, yd - 1.5), t.pos(x1, yd)], Stroke::new(t.s(0.65), color));
    p.line_segment([t.pos(x1 + 2.0, yd + 1.5), t.pos(x1, yd)], Stroke::new(t.s(0.65), color));
}

fn draw_triangle(p: &Painter, t: Transform, cx: f32, cy: f32, half: f32, up: bool, color: Color32) {
    let dir = if up { -1.0 } else { 1.0 };
    p.add(eframe::egui::Shape::convex_polygon(vec![t.pos(cx, cy + dir * half), t.pos(cx - half, cy - dir * half * 0.75), t.pos(cx + half, cy - dir * half * 0.75)], color, Stroke::NONE));
}

fn draw_x_bar(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, color: Color32) {
    centered(p, t, cx, cy + 0.5, size, color, "x");
    p.line_segment([t.pos(cx - 3.2, cy - 4.0), t.pos(cx + 3.2, cy - 4.0)], Stroke::new(t.s(0.75), color));
}

fn draw_equal_pair(p: &Painter, t: Transform, cx: f32, cy: f32, not_equal: bool) {
    centered(p, t, cx - 13.0, cy, 6.3, F_YELLOW, "x");
    if not_equal { draw_not_equal(p, t, cx - 5.0, cy, F_YELLOW); } else { centered(p, t, cx - 5.0, cy, 6.3, F_YELLOW, "="); }
    centered(p, t, cx + 1.0, cy, 6.3, F_YELLOW, "0");
    centered(p, t, cx + 7.0, cy, 6.3, G_CYAN, "x");
    if not_equal { draw_not_equal(p, t, cx + 15.0, cy, G_CYAN); } else { centered(p, t, cx + 15.0, cy, 6.3, G_CYAN, "="); }
    centered(p, t, cx + 22.0, cy, 6.3, G_CYAN, "y");
}

fn draw_not_equal(p: &Painter, t: Transform, cx: f32, cy: f32, color: Color32) {
    p.line_segment([t.pos(cx - 2.7, cy - 1.5), t.pos(cx + 2.7, cy - 1.5)], Stroke::new(t.s(0.7), color));
    p.line_segment([t.pos(cx - 2.7, cy + 1.5), t.pos(cx + 2.7, cy + 1.5)], Stroke::new(t.s(0.7), color));
    p.line_segment([t.pos(cx - 2.2, cy + 3.2), t.pos(cx + 2.2, cy - 3.2)], Stroke::new(t.s(0.7), color));
}

fn draw_relational_pair(p: &Painter, t: Transform, cx: f32, cy: f32, op: char, right_inclusive: bool, size: f32) {
    let op_s = if op == '<' { "<" } else { ">" };
    label(p, t, cx - 13.0, cy, Align2::LEFT_CENTER, size, F_YELLOW, "x");
    label(p, t, cx - 7.8, cy, Align2::LEFT_CENTER, size, F_YELLOW, op_s);
    label(p, t, cx - 1.3, cy, Align2::LEFT_CENTER, size, F_YELLOW, "0");
    label(p, t, cx + 4.0, cy, Align2::LEFT_CENTER, size, G_CYAN, "x");
    draw_less_greater_equal(p, t, cx + 12.2, cy, op, right_inclusive, G_CYAN);
    label(p, t, cx + 18.2, cy, Align2::LEFT_CENTER, size, G_CYAN, "y");
}

fn draw_less_greater_equal(p: &Painter, t: Transform, cx: f32, cy: f32, op: char, inclusive: bool, color: Color32) {
    let flip = if op == '<' { 1.0 } else { -1.0 };
    p.line_segment([t.pos(cx + 2.5 * flip, cy - 3.0), t.pos(cx - 2.0 * flip, cy)], Stroke::new(t.s(0.75), color));
    p.line_segment([t.pos(cx - 2.0 * flip, cy), t.pos(cx + 2.5 * flip, cy + 3.0)], Stroke::new(t.s(0.75), color));
    if inclusive { p.line_segment([t.pos(cx - 2.5, cy + 4.4), t.pos(cx + 2.8, cy + 4.4)], Stroke::new(t.s(0.75), color)); }
}

fn draw_inverse_trig(p: &Painter, t: Transform, cx: f32, cy: f32, name: &str) {
    label(p, t, cx - 2.0, cy, Align2::CENTER_CENTER, 6.5, F_YELLOW, name);
    label(p, t, cx + 12.0, cy - 3.5, Align2::CENTER_CENTER, 4.6, G_CYAN, "-1");
}

fn draw_sigma(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, color: Color32) {
    let s = size / 8.0; let left = cx - 4.0 * s; let right = cx + 4.0 * s; let top = cy - 4.2 * s; let bottom = cy + 4.2 * s;
    p.line_segment([t.pos(left, top), t.pos(right, top)], Stroke::new(t.s(0.9 * s), color));
    p.line_segment([t.pos(left, top), t.pos(cx + 1.0 * s, cy)], Stroke::new(t.s(0.9 * s), color));
    p.line_segment([t.pos(cx + 1.0 * s, cy), t.pos(left, bottom)], Stroke::new(t.s(0.9 * s), color));
    p.line_segment([t.pos(left, bottom), t.pos(right, bottom)], Stroke::new(t.s(0.9 * s), color));
}

fn draw_cross(p: &Painter, t: Transform, cx: f32, cy: f32, half: f32, color: Color32) {
    p.line_segment([t.pos(cx - half, cy - half), t.pos(cx + half, cy + half)], Stroke::new(t.s(0.95), color));
    p.line_segment([t.pos(cx - half, cy + half), t.pos(cx + half, cy - half)], Stroke::new(t.s(0.95), color));
}

fn draw_divide(p: &Painter, t: Transform, cx: f32, cy: f32, half: f32, color: Color32) {
    p.line_segment([t.pos(cx - half, cy), t.pos(cx + half, cy)], Stroke::new(t.s(0.95), color));
    p.circle_filled(t.pos(cx, cy - 4.0), t.s(0.85), color); p.circle_filled(t.pos(cx, cy + 4.0), t.s(0.85), color);
}

fn draw_pi(p: &Painter, t: Transform, cx: f32, cy: f32, size: f32, color: Color32) {
    let s = size / 7.0;
    p.line_segment([t.pos(cx - 4.0 * s, cy - 3.2 * s), t.pos(cx + 4.0 * s, cy - 3.2 * s)], Stroke::new(t.s(0.85 * s), color));
    p.line_segment([t.pos(cx - 2.1 * s, cy - 3.2 * s), t.pos(cx - 2.1 * s, cy + 3.4 * s)], Stroke::new(t.s(0.85 * s), color));
    p.line_segment([t.pos(cx + 2.1 * s, cy - 3.2 * s), t.pos(cx + 2.1 * s, cy + 3.4 * s)], Stroke::new(t.s(0.85 * s), color));
}

const SEG_A: u8 = 1 << 0; const SEG_B: u8 = 1 << 1; const SEG_C: u8 = 1 << 2; const SEG_D: u8 = 1 << 3; const SEG_E: u8 = 1 << 4; const SEG_F: u8 = 1 << 5; const SEG_G: u8 = 1 << 6;
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
    let cell_w = 17.0; let visible_cells = value.chars().filter(|&c| c != '.').count().min(12); let total_w = visible_cells as f32 * cell_w;
    let mut cursor_x = x + (width - total_w).max(0.0); let mut last_digit_x: Option<f32> = None;
    for ch in value.chars().take(16) {
        if ch == '.' {
            if let Some(dx) = last_digit_x { p.circle_filled(t.pos(dx + 13.3, y + 27.0), t.s(1.35), Color32::from_rgb(248, 72, 57)); }
            continue;
        }
        draw_segment_digit(p, t, cursor_x, y, ch); last_digit_x = Some(cursor_x); cursor_x += cell_w;
    }
}

fn draw_segment_digit(p: &Painter, t: Transform, x: f32, y: f32, ch: char) {
    let mask = segment_mask(ch); let on = Color32::from_rgb(249, 70, 54); let off = Color32::from_rgb(71, 35, 34); let color = |bit| if mask & bit != 0 { on } else { off };
    p.rect_filled(t.rect(x + 2.0, y, 8.0, 2.1), t.s(0.8), color(SEG_A));
    p.rect_filled(t.rect(x + 10.0, y + 2.0, 2.1, 9.0), t.s(0.8), color(SEG_B));
    p.rect_filled(t.rect(x + 10.0, y + 13.0, 2.1, 9.0), t.s(0.8), color(SEG_C));
    p.rect_filled(t.rect(x + 2.0, y + 22.0, 8.0, 2.1), t.s(0.8), color(SEG_D));
    p.rect_filled(t.rect(x, y + 13.0, 2.1, 9.0), t.s(0.8), color(SEG_E));
    p.rect_filled(t.rect(x, y + 2.0, 2.1, 9.0), t.s(0.8), color(SEG_F));
    p.rect_filled(t.rect(x + 2.0, y + 11.0, 8.0, 2.1), t.s(0.8), color(SEG_G));
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
    fn key_layout_is_complete() {
        assert_eq!(KEYS.len(), 35);
        assert!(KEYS.iter().any(|key| key.id == "enter" && key.sub == Some("DEG")));
        assert!(KEYS.iter().any(|key| key.id == "rs" && key.sub == Some("SPACE")));
    }

    #[test]
    fn seven_segment_masks_are_sane() {
        assert_eq!(segment_mask('1'), SEG_B | SEG_C);
        assert_eq!(segment_mask('8'), 0x7f);
        assert_eq!(segment_mask('-'), SEG_G);
    }
}
