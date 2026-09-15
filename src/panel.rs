use eframe::egui::{
    pos2, vec2, Color32, CursorIcon, Painter, Pos2, Rect, Sense, Stroke, TextureHandle, Ui,
};

use crate::{
    hp67::{HardwareDisplayFrame, Hp67State, KeyAction, UiEvent},
    ui::classic_display,
};

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

const DISPLAY_GLASS: PxRect = PxRect::new(178.0, 105.0, 750.0, 208.0);

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
    key(
        "gto",
        PxRect::new(293.0, 601.0, 383.0, 685.0),
        KeyAction::Gto,
    ),
    key(
        "dsp",
        PxRect::new(420.0, 601.0, 510.0, 685.0),
        KeyAction::Dsp,
    ),
    key(
        "indirect",
        PxRect::new(547.0, 601.0, 637.0, 685.0),
        KeyAction::Indirect,
    ),
    key(
        "sst",
        PxRect::new(674.0, 601.0, 764.0, 685.0),
        KeyAction::Sst,
    ),
    key(
        "f",
        PxRect::new(164.0, 735.0, 257.0, 819.0),
        KeyAction::FunctionF,
    ),
    key(
        "g",
        PxRect::new(291.0, 735.0, 384.0, 819.0),
        KeyAction::FunctionG,
    ),
    key(
        "sto",
        PxRect::new(420.0, 735.0, 510.0, 819.0),
        KeyAction::Sto,
    ),
    key(
        "rcl",
        PxRect::new(547.0, 735.0, 637.0, 819.0),
        KeyAction::Rcl,
    ),
    key(
        "h",
        PxRect::new(674.0, 735.0, 764.0, 819.0),
        KeyAction::FunctionH,
    ),
    key(
        "enter",
        PxRect::new(164.0, 868.0, 385.0, 952.0),
        KeyAction::Enter,
    ),
    key(
        "chs",
        PxRect::new(420.0, 868.0, 510.0, 952.0),
        KeyAction::ChangeSign,
    ),
    key(
        "eex",
        PxRect::new(547.0, 868.0, 637.0, 952.0),
        KeyAction::Enter,
    ),
    key(
        "clx",
        PxRect::new(674.0, 868.0, 764.0, 952.0),
        KeyAction::ClearX,
    ),
    key(
        "minus",
        PxRect::new(166.0, 1003.0, 241.0, 1088.0),
        KeyAction::Subtract,
    ),
    key(
        "7",
        PxRect::new(295.0, 1003.0, 405.0, 1088.0),
        KeyAction::Digit(7),
    ),
    key(
        "8",
        PxRect::new(476.0, 1003.0, 587.0, 1088.0),
        KeyAction::Digit(8),
    ),
    key(
        "9",
        PxRect::new(657.0, 1003.0, 768.0, 1088.0),
        KeyAction::Digit(9),
    ),
    key(
        "plus",
        PxRect::new(166.0, 1138.0, 241.0, 1223.0),
        KeyAction::Add,
    ),
    key(
        "4",
        PxRect::new(295.0, 1138.0, 405.0, 1223.0),
        KeyAction::Digit(4),
    ),
    key(
        "5",
        PxRect::new(476.0, 1138.0, 587.0, 1223.0),
        KeyAction::Digit(5),
    ),
    key(
        "6",
        PxRect::new(657.0, 1138.0, 768.0, 1223.0),
        KeyAction::Digit(6),
    ),
    key(
        "multiply",
        PxRect::new(166.0, 1273.0, 241.0, 1358.0),
        KeyAction::Multiply,
    ),
    key(
        "1",
        PxRect::new(295.0, 1273.0, 405.0, 1358.0),
        KeyAction::Digit(1),
    ),
    key(
        "2",
        PxRect::new(476.0, 1273.0, 587.0, 1358.0),
        KeyAction::Digit(2),
    ),
    key(
        "3",
        PxRect::new(657.0, 1273.0, 768.0, 1358.0),
        KeyAction::Digit(3),
    ),
    key(
        "divide",
        PxRect::new(166.0, 1409.0, 241.0, 1495.0),
        KeyAction::Divide,
    ),
    key(
        "0",
        PxRect::new(295.0, 1409.0, 405.0, 1495.0),
        KeyAction::Digit(0),
    ),
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
    pub fn show(
        ui: &mut Ui,
        state: &Hp67State,
        display: &HardwareDisplayFrame,
        photo: &TextureHandle,
    ) -> Vec<UiEvent> {
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

        // hp67.png already contains the real filter, glass, bezel and reflections.
        // Add only the physically calibrated 5082-7405 LED emission on top.
        if state.power_on {
            classic_display::paint_segments(
                &painter,
                source_to_screen(photo_rect, DISPLAY_GLASS),
                display.segments(),
            );
        }
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
        [
            pos2(original.left() + scale * 3.0, edge_y),
            pos2(original.right() - scale * 3.0, edge_y),
        ],
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn photo_key_table_contains_all_35_keys() {
        assert_eq!(KEYS.len(), 35);
    }

    #[test]
    fn source_mapping_preserves_photo_edges() {
        let photo = Rect::from_min_size(Pos2::ZERO, vec2(PHOTO_W, PHOTO_H));
        let mapped = source_to_screen(photo, PxRect::new(0.0, 0.0, PHOTO_W, PHOTO_H));
        assert_eq!(mapped, photo);
    }

    #[test]
    fn display_glass_stays_inside_source_photo() {
        assert!(DISPLAY_GLASS.x0 >= 0.0 && DISPLAY_GLASS.y0 >= 0.0);
        assert!(DISPLAY_GLASS.x1 <= PHOTO_W && DISPLAY_GLASS.y1 <= PHOTO_H);
    }
}
