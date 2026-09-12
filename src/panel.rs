use eframe::egui::{self, Align2, Color32, FontId, Painter, Pos2, Rect, Sense, Stroke, Ui, Vec2};

use crate::hp67::{Hp67State, KeyAction, RunMode, UiEvent};

pub const DESIGN_W: f32 = 330.0;
pub const DESIGN_H: f32 = 620.0;

const PANEL_BLACK: Color32 = Color32::from_rgb(38, 33, 30);
const PANEL_BLACK_2: Color32 = Color32::from_rgb(25, 23, 21);
const CASE_GREEN: Color32 = Color32::from_rgb(79, 88, 50);
const CASE_GREEN_DARK: Color32 = Color32::from_rgb(53, 61, 37);
const SILVER: Color32 = Color32::from_rgb(188, 185, 171);
const SILVER_LIGHT: Color32 = Color32::from_rgb(224, 222, 208);
const LEGEND: Color32 = Color32::from_rgb(222, 219, 202);
const GOLD: Color32 = Color32::from_rgb(164, 151, 84);
const ORANGE: Color32 = Color32::from_rgb(231, 144, 54);
const BLUE: Color32 = Color32::from_rgb(92, 157, 184);

#[derive(Debug, Clone, Copy)]
enum KeyStyle {
    Gold,
    Orange,
    Blue,
    White,
    Black,
}

#[derive(Debug, Clone, Copy)]
struct KeySpec {
    id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &'static str,
    style: KeyStyle,
    action: KeyAction,
}

const KEYS: &[KeySpec] = &[
    // A-E row
    key("a", 50.0, 170.0, 30.0, 28.0, "A", KeyStyle::Gold, KeyAction::A),
    key("b", 101.0, 170.0, 30.0, 28.0, "B", KeyStyle::Gold, KeyAction::B),
    key("c", 152.0, 170.0, 30.0, 28.0, "C", KeyStyle::Gold, KeyAction::C),
    key("d", 203.0, 170.0, 30.0, 28.0, "D", KeyStyle::Gold, KeyAction::D),
    key("e", 254.0, 170.0, 30.0, 28.0, "E", KeyStyle::Gold, KeyAction::E),
    // Function row
    key("sigma", 50.0, 224.0, 30.0, 28.0, "Σ+", KeyStyle::Gold, KeyAction::SigmaPlus),
    key("gto", 101.0, 224.0, 30.0, 28.0, "GTO", KeyStyle::Gold, KeyAction::Gto),
    key("dsp", 152.0, 224.0, 30.0, 28.0, "DSP", KeyStyle::Gold, KeyAction::Dsp),
    key("indirect", 203.0, 224.0, 30.0, 28.0, "(i)", KeyStyle::Gold, KeyAction::Indirect),
    key("sst", 254.0, 224.0, 30.0, 28.0, "SST", KeyStyle::Gold, KeyAction::Sst),
    // Shift/storage row
    key("f", 50.0, 278.0, 30.0, 28.0, "f", KeyStyle::Orange, KeyAction::FunctionF),
    key("g", 101.0, 278.0, 30.0, 28.0, "g", KeyStyle::Blue, KeyAction::FunctionG),
    key("sto", 152.0, 278.0, 30.0, 28.0, "STO", KeyStyle::Gold, KeyAction::Sto),
    key("rcl", 203.0, 278.0, 30.0, 28.0, "RCL", KeyStyle::Gold, KeyAction::Rcl),
    key("h", 254.0, 278.0, 30.0, 28.0, "h", KeyStyle::Black, KeyAction::FunctionH),
    // ENTER row
    key("enter", 50.0, 333.0, 81.0, 29.0, "ENTER ↑", KeyStyle::White, KeyAction::Enter),
    key("chs", 152.0, 333.0, 30.0, 29.0, "CHS", KeyStyle::White, KeyAction::ChangeSign),
    key("eex", 203.0, 333.0, 30.0, 29.0, "EEX", KeyStyle::White, KeyAction::Enter),
    key("clx", 254.0, 333.0, 30.0, 29.0, "CLx", KeyStyle::White, KeyAction::ClearX),
    // Numeric block
    key("minus", 50.0, 385.0, 30.0, 27.0, "−", KeyStyle::White, KeyAction::Subtract),
    key("7", 101.0, 385.0, 30.0, 27.0, "7", KeyStyle::White, KeyAction::Digit(7)),
    key("8", 177.0, 385.0, 30.0, 27.0, "8", KeyStyle::White, KeyAction::Digit(8)),
    key("9", 253.0, 385.0, 30.0, 27.0, "9", KeyStyle::White, KeyAction::Digit(9)),
    key("plus", 50.0, 437.0, 30.0, 27.0, "+", KeyStyle::White, KeyAction::Add),
    key("4", 101.0, 437.0, 30.0, 27.0, "4", KeyStyle::White, KeyAction::Digit(4)),
    key("5", 177.0, 437.0, 30.0, 27.0, "5", KeyStyle::White, KeyAction::Digit(5)),
    key("6", 253.0, 437.0, 30.0, 27.0, "6", KeyStyle::White, KeyAction::Digit(6)),
    key("multiply", 50.0, 489.0, 30.0, 27.0, "×", KeyStyle::White, KeyAction::Multiply),
    key("1", 101.0, 489.0, 30.0, 27.0, "1", KeyStyle::White, KeyAction::Digit(1)),
    key("2", 177.0, 489.0, 30.0, 27.0, "2", KeyStyle::White, KeyAction::Digit(2)),
    key("3", 253.0, 489.0, 30.0, 27.0, "3", KeyStyle::White, KeyAction::Digit(3)),
    key("divide", 50.0, 541.0, 30.0, 27.0, "÷", KeyStyle::White, KeyAction::Divide),
    key("0", 101.0, 541.0, 30.0, 27.0, "0", KeyStyle::White, KeyAction::Digit(0)),
    key("decimal", 177.0, 541.0, 30.0, 27.0, ".", KeyStyle::White, KeyAction::Decimal),
    key("rs", 253.0, 541.0, 30.0, 27.0, "R/S", KeyStyle::White, KeyAction::RunStop),
];

const fn key(
    id: &'static str,
    x: f32,
    y: f32,
    w: f32,
    h: f32,
    label: &'static str,
    style: KeyStyle,
    action: KeyAction,
) -> KeySpec {
    KeySpec {
        id,
        x,
        y,
        w,
        h,
        label,
        style,
        action,
    }
}

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
        let origin = Pos2::new(
            host_rect.center().x - panel_size.x * 0.5,
            host_rect.center().y - panel_size.y * 0.5,
        );
        let t = Transform { origin, scale };
        let painter = ui.painter_at(host_rect);
        let mut events = Vec::new();

        draw_chassis(&painter, t);
        draw_display(&painter, t, state.display_text());

        let power_hit = t.rect(44.0, 92.0, 91.0, 28.0);
        let power_response = ui.interact(
            power_hit,
            ui.make_persistent_id("hp67-power-switch"),
            Sense::click(),
        );
        if power_response.clicked() {
            events.push(UiEvent::TogglePower);
        }
        draw_power_switch(&painter, t, state.power_on, power_response.hovered());

        let mode_hit = t.rect(166.0, 92.0, 121.0, 28.0);
        let mode_response = ui.interact(
            mode_hit,
            ui.make_persistent_id("hp67-mode-switch"),
            Sense::click(),
        );
        if mode_response.clicked() {
            events.push(UiEvent::ToggleMode);
        }
        draw_mode_switch(&painter, t, state.mode, mode_response.hovered());

        draw_keyboard_legends(&painter, t);

        for (index, spec) in KEYS.iter().enumerate() {
            let rect = t.rect(spec.x, spec.y, spec.w, spec.h);
            let response = ui.interact(
                rect,
                ui.make_persistent_id(("hp67-key", index, spec.id)),
                Sense::click(),
            );

            if response.clicked() {
                events.push(UiEvent::Key(spec.action));
            }

            draw_key(
                &painter,
                t,
                *spec,
                response.hovered(),
                response.is_pointer_button_down_on(),
            );
        }

        draw_branding(&painter, t);
        events
    }
}

pub fn scale_for(available: Vec2) -> f32 {
    if available.x <= 0.0 || available.y <= 0.0 {
        return 0.0;
    }
    (available.x / DESIGN_W).min(available.y / DESIGN_H)
}

#[derive(Debug, Clone, Copy)]
struct Transform {
    origin: Pos2,
    scale: f32,
}

impl Transform {
    fn s(self, value: f32) -> f32 {
        value * self.scale
    }

    fn pos(self, x: f32, y: f32) -> Pos2 {
        Pos2::new(self.origin.x + x * self.scale, self.origin.y + y * self.scale)
    }

    fn rect(self, x: f32, y: f32, w: f32, h: f32) -> Rect {
        Rect::from_min_size(self.pos(x, y), Vec2::new(self.s(w), self.s(h)))
    }
}

fn draw_chassis(p: &Painter, t: Transform) {
    // Soft shadow and the olive-green outer shell.
    p.rect_filled(t.rect(4.0, 3.0, 322.0, 614.0), t.s(19.0), Color32::from_rgb(17, 18, 14));
    p.rect_filled(t.rect(7.0, 1.0, 316.0, 615.0), t.s(18.0), CASE_GREEN_DARK);
    p.rect_filled(t.rect(11.0, 2.0, 308.0, 612.0), t.s(15.0), CASE_GREEN);

    // Metallic frame around the actual calculator face.
    p.rect_filled(t.rect(24.0, 5.0, 282.0, 609.0), t.s(11.0), Color32::from_rgb(120, 117, 105));
    p.rect_filled(t.rect(27.0, 7.0, 276.0, 605.0), t.s(9.0), SILVER_LIGHT);
    p.rect_filled(t.rect(30.0, 11.0, 270.0, 596.0), t.s(7.0), PANEL_BLACK);

    // Inner highlights imitate the slightly stepped metal bezel.
    p.rect_stroke(
        t.rect(28.5, 8.5, 273.0, 601.0),
        t.s(8.0),
        Stroke::new(t.s(1.2), Color32::from_rgb(103, 100, 91)),
    );
    p.line_segment(
        [t.pos(34.0, 126.0), t.pos(296.0, 126.0)],
        Stroke::new(t.s(1.0), Color32::from_rgb(9, 8, 8)),
    );
}

fn draw_display(p: &Painter, t: Transform, text_value: &str) {
    // Chrome/silver display frame and dark red glass.
    p.rect_filled(t.rect(37.0, 18.0, 256.0, 77.0), t.s(4.0), Color32::from_rgb(90, 87, 79));
    p.rect_filled(t.rect(39.0, 20.0, 252.0, 73.0), t.s(3.0), SILVER_LIGHT);
    p.rect_filled(t.rect(42.0, 24.0, 246.0, 66.0), t.s(1.5), Color32::from_rgb(55, 36, 31));
    p.rect_filled(t.rect(44.0, 27.0, 242.0, 60.0), t.s(1.0), Color32::from_rgb(62, 39, 33));

    // Subtle glass reflection.
    p.rect_filled(
        t.rect(46.0, 29.0, 238.0, 3.0),
        t.s(1.0),
        Color32::from_rgb(83, 56, 47),
    );

    if !text_value.is_empty() {
        draw_segment_string(p, t, text_value, 49.0, 40.0, 232.0);
    }
}

fn draw_power_switch(p: &Painter, t: Transform, on: bool, hovered: bool) {
    text(p, t, 48.0, 106.0, Align2::LEFT_CENTER, 7.2, LEGEND, "OFF");
    text(p, t, 116.0, 106.0, Align2::RIGHT_CENTER, 7.2, LEGEND, "ON");
    draw_slider(p, t, 72.0, 101.0, 42.0, on, hovered);
}

fn draw_mode_switch(p: &Painter, t: Transform, mode: RunMode, hovered: bool) {
    text(p, t, 166.0, 106.0, Align2::LEFT_CENTER, 7.0, LEGEND, "W/ PRGM");
    text(p, t, 286.0, 106.0, Align2::RIGHT_CENTER, 7.0, LEGEND, "RUN");
    draw_slider(p, t, 225.0, 101.0, 39.0, matches!(mode, RunMode::Run), hovered);
}

fn draw_slider(p: &Painter, t: Transform, x: f32, y: f32, w: f32, right: bool, hovered: bool) {
    let track = t.rect(x, y, w, 8.0);
    p.rect_filled(track, t.s(1.0), Color32::from_rgb(9, 9, 9));
    p.rect_stroke(
        track,
        t.s(1.0),
        Stroke::new(
            t.s(if hovered { 1.0 } else { 0.7 }),
            if hovered { Color32::from_rgb(150, 147, 135) } else { Color32::from_rgb(69, 67, 62) },
        ),
    );

    for i in 0..10 {
        let xx = x + 2.0 + i as f32 * (w - 4.0) / 9.0;
        p.line_segment(
            [t.pos(xx, y + 1.0), t.pos(xx, y + 7.0)],
            Stroke::new(t.s(0.45), Color32::from_rgb(55, 53, 49)),
        );
    }

    let knob_x = if right { x + w - 13.0 } else { x + 2.0 };
    p.rect_filled(t.rect(knob_x, y - 1.0, 11.0, 10.0), t.s(1.3), Color32::from_rgb(81, 79, 73));
    p.rect_stroke(
        t.rect(knob_x, y - 1.0, 11.0, 10.0),
        t.s(1.3),
        Stroke::new(t.s(0.8), Color32::from_rgb(28, 27, 25)),
    );
}

fn draw_keyboard_legends(p: &Painter, t: Transform) {
    // Top A-E functions.
    centered(p, t, 65.0, 151.0, 9.0, LEGEND, "1/x");
    centered(p, t, 116.0, 151.0, 9.0, LEGEND, "√x");
    centered(p, t, 167.0, 151.0, 9.0, LEGEND, "yˣ");
    centered(p, t, 218.0, 151.0, 9.0, LEGEND, "R↓");
    centered(p, t, 269.0, 151.0, 9.0, LEGEND, "x↔y");

    for (x, label) in [(65.0, "a"), (116.0, "b"), (167.0, "c"), (218.0, "d"), (269.0, "e")] {
        centered(p, t, x, 207.0, 7.0, GOLD, label);
    }

    centered(p, t, 65.0, 261.0, 6.7, GOLD, "Σ−   s");
    centered(p, t, 116.0, 261.0, 6.7, GOLD, "GSB  f");
    centered(p, t, 167.0, 261.0, 6.7, LEGEND, "FIX  SCI");
    centered(p, t, 218.0, 261.0, 6.7, LEGEND, "RND");
    centered(p, t, 269.0, 261.0, 6.7, GOLD, "LBL  f");

    centered(p, t, 167.0, 315.0, 6.6, LEGEND, "DSZ (i)");
    centered(p, t, 218.0, 315.0, 6.6, LEGEND, "ISZ (i)");

    // ENTER-row auxiliary labels.
    centered(p, t, 65.0, 374.0, 6.6, GOLD, "W/DATA");
    centered(p, t, 116.0, 374.0, 6.6, BLUE, "MERGE");
    centered(p, t, 167.0, 374.0, 6.6, GOLD, "P↔S");
    centered(p, t, 218.0, 374.0, 6.6, GOLD, "CL REG");
    centered(p, t, 269.0, 374.0, 6.6, GOLD, "CL PRGM");

    // Numeric/function legends. These are separate vector text elements so
    // their placement remains correct at every scale.
    centered(p, t, 65.0, 425.0, 6.4, GOLD, "x=0  x=y");
    centered(p, t, 116.0, 425.0, 6.7, ORANGE, "LN");
    centered(p, t, 192.0, 425.0, 6.7, BLUE, "eˣ");
    centered(p, t, 268.0, 425.0, 6.7, GOLD, "LOG  10ˣ");

    centered(p, t, 65.0, 477.0, 6.3, GOLD, "x≠0  x≠y");
    centered(p, t, 116.0, 477.0, 6.4, LEGEND, "SIN⁻¹");
    centered(p, t, 192.0, 477.0, 6.4, LEGEND, "COS⁻¹");
    centered(p, t, 268.0, 477.0, 6.4, LEGEND, "TAN⁻¹");

    centered(p, t, 65.0, 529.0, 6.3, GOLD, "x<0  x≤y");
    centered(p, t, 116.0, 529.0, 6.4, LEGEND, "R↔P");
    centered(p, t, 192.0, 529.0, 6.4, LEGEND, "D↔R");
    centered(p, t, 268.0, 529.0, 6.2, LEGEND, "H↔H.MS");

    centered(p, t, 65.0, 581.0, 6.3, GOLD, "x>0  x≥y");
    centered(p, t, 116.0, 581.0, 6.4, LEGEND, "%CH");
    centered(p, t, 192.0, 581.0, 6.4, LEGEND, "INT  FRAC");
    centered(p, t, 268.0, 581.0, 6.4, LEGEND, "←  STK");
}

fn draw_key(p: &Painter, t: Transform, spec: KeySpec, hovered: bool, pressed: bool) {
    let offset = if pressed { t.s(1.4) } else { 0.0 };
    let rect = t
        .rect(spec.x, spec.y, spec.w, spec.h)
        .translate(Vec2::new(0.0, offset));

    let shadow = rect.translate(Vec2::new(t.s(1.4), t.s(2.0)));
    p.rect_filled(shadow, t.s(2.0), Color32::from_rgb(8, 8, 8));

    let (base, face, text_color) = match spec.style {
        KeyStyle::Gold => (
            Color32::from_rgb(74, 68, 39),
            if hovered { Color32::from_rgb(188, 173, 102) } else { GOLD },
            Color32::from_rgb(242, 240, 220),
        ),
        KeyStyle::Orange => (
            Color32::from_rgb(112, 63, 22),
            if hovered { Color32::from_rgb(246, 159, 67) } else { ORANGE },
            Color32::from_rgb(255, 247, 225),
        ),
        KeyStyle::Blue => (
            Color32::from_rgb(40, 75, 87),
            if hovered { Color32::from_rgb(112, 177, 200) } else { BLUE },
            Color32::from_rgb(21, 30, 31),
        ),
        KeyStyle::White => (
            Color32::from_rgb(111, 108, 98),
            if hovered { Color32::from_rgb(237, 234, 216) } else { Color32::from_rgb(216, 213, 195) },
            Color32::from_rgb(42, 39, 34),
        ),
        KeyStyle::Black => (
            Color32::from_rgb(5, 5, 5),
            if hovered { Color32::from_rgb(42, 42, 39) } else { Color32::from_rgb(20, 20, 19) },
            Color32::from_rgb(231, 228, 211),
        ),
    };

    p.rect_filled(rect, t.s(2.5), base);
    let face_rect = rect.shrink(t.s(2.0));
    p.rect_filled(face_rect, t.s(1.8), face);

    // Top-edge highlight adds shape without raster textures.
    p.line_segment(
        [
            Pos2::new(face_rect.left() + t.s(2.0), face_rect.top() + t.s(1.0)),
            Pos2::new(face_rect.right() - t.s(2.0), face_rect.top() + t.s(1.0)),
        ],
        Stroke::new(t.s(0.7), Color32::from_rgba_premultiplied(255, 255, 255, 80)),
    );

    p.text(
        face_rect.center(),
        Align2::CENTER_CENTER,
        spec.label,
        FontId::proportional(t.s(if spec.w > 40.0 { 10.7 } else { 10.5 })),
        text_color,
    );
}

fn draw_branding(p: &Painter, t: Transform) {
    p.rect_filled(t.rect(43.0, 592.0, 244.0, 12.0), 0.0, Color32::from_rgb(205, 202, 188));
    p.circle_filled(t.pos(59.0, 598.0), t.s(4.4), Color32::from_rgb(50, 65, 73));
    centered(p, t, 59.0, 598.0, 5.0, Color32::WHITE, "hp");
    centered(p, t, 171.0, 598.0, 7.3, Color32::from_rgb(48, 47, 43), "H E W L E T T  ·  P A C K A R D   6 7");
}

fn centered(p: &Painter, t: Transform, x: f32, y: f32, size: f32, color: Color32, value: &str) {
    text(p, t, x, y, Align2::CENTER_CENTER, size, color, value);
}

fn text(
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
        '-' | '−' => SEG_G,
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
                    t.pos(dx + 13.3, y + 27.0),
                    t.s(1.35),
                    Color32::from_rgb(244, 74, 42),
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
    let on = Color32::from_rgb(242, 67, 37);
    let off = Color32::from_rgb(76, 38, 32);
    let color = |bit| if mask & bit != 0 { on } else { off };

    // All seven segments are vector rectangles; no bitmap font is involved.
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
    fn seven_segment_masks_are_sane() {
        assert_eq!(segment_mask('1'), SEG_B | SEG_C);
        assert_eq!(segment_mask('8'), 0x7f);
        assert_eq!(segment_mask('-'), SEG_G);
    }
}
