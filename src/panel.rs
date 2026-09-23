use eframe::egui::{
    pos2, vec2, Color32, CursorIcon, Painter, Pos2, Rect, Sense, Stroke, TextureHandle, Ui,
};

use crate::{
    hp67::{HardwareDisplayFrame, Hp67State, KeyAction, UiEvent},
    ui::{
        classic_display,
        program_card::{self, ProgramCardView},
    },
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

// The glass is only the optical clipping aperture.  LED geometry is registered
// independently to the projected calculator case so it cannot be stretched to
// fit this rectangle.  Measurements are source-image pixels in assets/hp67.png.
const DISPLAY_GLASS: PxRect = PxRect::new(178.0, 105.0, 750.0, 208.0);
const HP67_CASE_WIDTH_MM: f32 = 81.0;
const DISPLAY_CASE_CENTER_X: f32 = 453.0;
const DISPLAY_CASE_WIDTH_SOURCE_PX: f32 = 792.0;
const DISPLAY_LED_CENTER_Y: f32 = 156.0;
const DISPLAY_SOURCE_PX_PER_MM: f32 = DISPLAY_CASE_WIDTH_SOURCE_PX / HP67_CASE_WIDTH_MM;

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
        KeyAction::Exponent,
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

pub struct Hp67PanelOutput {
    pub events: Vec<UiEvent>,
    pub key_contact: Option<KeyAction>,
    pub card_reader_clicked: bool,
    pub blank_card_requested: bool,
    pub card_waiting_reader_clicked: bool,
    pub card_parked_left_clicked: bool,
    pub card_parked_left_double_clicked: bool,
    pub card_window_clicked: bool,
}

pub struct Hp67Panel;

impl Hp67Panel {
    pub fn show(
        ui: &mut Ui,
        state: &Hp67State,
        display: &HardwareDisplayFrame,
        photo: &TextureHandle,
        card_view: ProgramCardView<'_>,
        minimum_touch_target: f32,
    ) -> Hp67PanelOutput {
        let available = ui.available_size();
        let (host, _) = ui.allocate_exact_size(available, Sense::hover());
        if host.width() <= 0.0 || host.height() <= 0.0 {
            return Hp67PanelOutput {
                events: Vec::new(),
                key_contact: None,
                card_reader_clicked: false,
                blank_card_requested: false,
                card_waiting_reader_clicked: false,
                card_parked_left_clicked: false,
                card_parked_left_double_clicked: false,
                card_window_clicked: false,
            };
        }

        let photo_rect = fit_photo(host);
        let painter = ui.painter_at(host).with_clip_rect(photo_rect);
        painter.image(
            photo.id(),
            photo_rect,
            Rect::from_min_max(Pos2::ZERO, pos2(1.0, 1.0)),
            Color32::WHITE,
        );

        let card_ui = program_card::paint(ui, photo_rect, photo, card_view);

        let mut events = Vec::new();

        // Capture the physical key on the mouse-down edge and keep that same
        // contact closed until the primary button is released. A real HP-67 key
        // does not spring back merely because the host pointer moves a few pixels
        // or egui stops considering the interaction a potential click.
        let pointer_capture_id = ui.make_persistent_id("hp67-held-pointer-key");
        let (pointer_pos, pointer_pressed, pointer_down) = ui.ctx().input(|input| {
            (
                input.pointer.interact_pos(),
                input.pointer.primary_pressed(),
                input.pointer.primary_down(),
            )
        });
        let previous_key_contact = ui.data(|data| {
            data.get_temp::<Option<KeyAction>>(pointer_capture_id)
                .flatten()
        });
        let pressed_key = pointer_pressed.then(|| {
            pointer_pos.and_then(|position| {
                key_at_position(photo_rect, position, minimum_touch_target)
            })
        });
        let key_contact = held_pointer_key(
            previous_key_contact,
            pointer_pressed,
            pointer_down,
            pressed_key.flatten(),
        );
        ui.data_mut(|data| data.insert_temp(pointer_capture_id, key_contact));

        // The two mechanical slide switches remain part of the photograph, but
        // retain their emulator hit areas.  The display state makes power changes
        // immediately visible even before a later dedicated slider sprite pass.
        let power = ui.interact(
            minimum_hit_rect(
                source_to_screen(photo_rect, PxRect::new(150.0, 270.0, 386.0, 334.0)),
                minimum_touch_target,
            ),
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
            minimum_hit_rect(
                source_to_screen(photo_rect, PxRect::new(455.0, 270.0, 775.0, 334.0)),
                minimum_touch_target,
            ),
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
                minimum_hit_rect(rect, minimum_touch_target),
                ui.make_persistent_id(("photo-key", key.id)),
                Sense::click(),
            );
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }

            let down = key_contact == Some(key.action);
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
        // Register the LED assembly to the calculator's projected case width with
        // one uniform physical scale. DISPLAY_GLASS only clips emitted light.
        if state.power_on {
            let photo_scale = photo_rect.width() / PHOTO_W;
            classic_display::paint_segments(
                &painter,
                source_to_screen(photo_rect, DISPLAY_GLASS),
                source_point_to_screen(photo_rect, DISPLAY_CASE_CENTER_X, DISPLAY_LED_CENTER_Y),
                DISPLAY_SOURCE_PX_PER_MM * photo_scale,
                display.segments(),
            );
        }
        Hp67PanelOutput {
            events,
            key_contact,
            card_reader_clicked: card_ui.reader_clicked,
            blank_card_requested: card_ui.blank_card_requested,
            card_waiting_reader_clicked: card_ui.waiting_reader_clicked,
            card_parked_left_clicked: card_ui.parked_left_clicked,
            card_parked_left_double_clicked: card_ui.parked_left_double_clicked,
            card_window_clicked: card_ui.window_clicked,
        }
    }
}

fn held_pointer_key(
    previous: Option<KeyAction>,
    pointer_pressed: bool,
    pointer_down: bool,
    pressed_key: Option<KeyAction>,
) -> Option<KeyAction> {
    if !pointer_down {
        None
    } else if pointer_pressed {
        pressed_key
    } else {
        previous
    }
}

pub fn height_for_width(width: f32) -> f32 {
    width * PHOTO_H / PHOTO_W
}

fn minimum_hit_rect(rect: Rect, minimum_size: f32) -> Rect {
    if minimum_size <= 0.0 {
        return rect;
    }
    Rect::from_center_size(
        rect.center(),
        vec2(
            rect.width().max(minimum_size),
            rect.height().max(minimum_size),
        ),
    )
}

fn key_at_position(photo: Rect, position: Pos2, minimum_touch_target: f32) -> Option<KeyAction> {
    let mut best: Option<(f32, KeyAction)> = None;
    for key in KEYS {
        let hit = minimum_hit_rect(source_to_screen(photo, key.src), minimum_touch_target);
        if !hit.contains(position) {
            continue;
        }
        let distance = (hit.center() - position).length_sq();
        if best.map_or(true, |(best_distance, _)| distance < best_distance) {
            best = Some((distance, key.action));
        }
    }
    best.map(|(_, action)| action)
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

fn source_point_to_screen(photo: Rect, x: f32, y: f32) -> Pos2 {
    let sx = photo.width() / PHOTO_W;
    let sy = photo.height() / PHOTO_H;
    pos2(photo.left() + x * sx, photo.top() + y * sy)
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
    fn touch_hit_rect_expands_without_moving_control_center() {
        let rect = Rect::from_min_size(pos2(10.0, 20.0), vec2(30.0, 35.0));
        let expanded = minimum_hit_rect(rect, 44.0);
        assert_eq!(expanded.center(), rect.center());
        assert_eq!(expanded.size(), vec2(44.0, 44.0));
        assert_eq!(minimum_hit_rect(rect, 0.0), rect);
    }

    #[test]
    fn natural_photo_height_preserves_source_aspect_ratio() {
        assert_eq!(height_for_width(PHOTO_W), PHOTO_H);
        assert!((height_for_width(528.0) - 964.655_15).abs() < 0.001);
    }

    #[test]
    fn held_pointer_key_stays_pressed_until_mouse_up() {
        assert_eq!(
            held_pointer_key(None, true, true, Some(KeyAction::A)),
            Some(KeyAction::A)
        );
        assert_eq!(
            held_pointer_key(Some(KeyAction::A), false, true, None),
            Some(KeyAction::A)
        );
        assert_eq!(
            held_pointer_key(Some(KeyAction::A), false, false, None),
            None
        );
    }

    #[test]
    fn dragging_onto_a_key_does_not_create_a_new_contact() {
        assert_eq!(held_pointer_key(None, true, true, None), None);
        assert_eq!(
            held_pointer_key(None, false, true, Some(KeyAction::A)),
            None
        );
    }

    #[test]
    fn photo_key_table_contains_all_35_keys() {
        assert_eq!(KEYS.len(), 35);
        assert_eq!(
            KEYS.iter()
                .find(|key| key.id == "eex")
                .map(|key| key.action),
            Some(KeyAction::Exponent)
        );
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

    #[test]
    fn led_registration_uses_case_scale_and_keeps_all_centres_inside_glass() {
        assert!((DISPLAY_SOURCE_PX_PER_MM - 9.777_778).abs() < 0.0001);
        let half_span = 7.0 * classic_display::CHARACTER_PITCH_MM * DISPLAY_SOURCE_PX_PER_MM;
        let leftmost = DISPLAY_CASE_CENTER_X - half_span;
        let rightmost = DISPLAY_CASE_CENTER_X + half_span;
        assert!((leftmost - 192.23).abs() < 0.05);
        assert!((rightmost - 713.77).abs() < 0.05);
        assert!(leftmost > DISPLAY_GLASS.x0);
        assert!(rightmost < DISPLAY_GLASS.x1);

        // Power-on 0.00 starts at physical position 2 (index 1).  This regression
        // prevents the old, too-centred placement from returning.
        let first_power_on_zero = DISPLAY_CASE_CENTER_X
            + classic_display::character_offset_mm(1) * DISPLAY_SOURCE_PX_PER_MM;
        assert!((first_power_on_zero - 229.48).abs() < 0.05);
    }
}
