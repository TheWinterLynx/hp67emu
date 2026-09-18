use eframe::egui::{
    pos2, Align2, Color32, CursorIcon, FontId, Painter, Rect, Sense, Shape, Stroke, Ui,
};

const PHOTO_W: f32 = 928.0;
const PHOTO_H: f32 = 1695.0;

const CARD_WINDOW: SourceRect = SourceRect::new(135.0, 365.0, 795.0, 448.0);
const CARD_READER_HIT: SourceRect = SourceRect::new(775.0, 356.0, 842.0, 452.0);
const CARD_READER_MOUTH_X: f32 = CARD_READER_HIT.x1;
const CARD_EXIT_MOUTH_X: f32 = 64.0;
const CARD_PHYSICAL_WIDTH: f32 = 820.0;
const CARD_PHYSICAL_HEIGHT: f32 = 72.0;
const CARD_LEFT_VISIBLE_WIDTH: f32 = 105.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProgramCardArtwork {
    pub title: &'static str,
    pub reference: &'static str,
    pub primary_labels: [&'static str; 5],
    pub shifted_labels: [&'static str; 5],
}

pub const MOON_ROCKET_LANDER_CARD: ProgramCardArtwork = ProgramCardArtwork {
    title: "MOON ROCKET LANDER",
    reference: "SD-14A",
    primary_labels: ["CNTRL", "RESTART", "", "", ""],
    shifted_labels: ["", "", "", "", ""],
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramCardPhase {
    Idle,
    ReadingFromRight,
    ParkedLeft,
    MovingToWindow,
    InWindow,
}

impl ProgramCardPhase {
    pub const fn is_animating(self) -> bool {
        matches!(self, Self::ReadingFromRight | Self::MovingToWindow)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ProgramCardView<'a> {
    pub artwork: &'a ProgramCardArtwork,
    pub phase: ProgramCardPhase,
    pub phase_progress: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgramCardUiOutput {
    pub reader_clicked: bool,
    pub parked_left_clicked: bool,
    pub window_clicked: bool,
}

#[derive(Debug, Clone, Copy)]
struct SourceRect {
    x0: f32,
    y0: f32,
    x1: f32,
    y1: f32,
}

impl SourceRect {
    const fn new(x0: f32, y0: f32, x1: f32, y1: f32) -> Self {
        Self { x0, y0, x1, y1 }
    }
}

pub fn paint(ui: &mut Ui, photo: Rect, view: ProgramCardView<'_>) -> ProgramCardUiOutput {
    let mut output = ProgramCardUiOutput::default();
    let scale = photo.height() / PHOTO_H;
    let window = source_to_screen(photo, CARD_WINDOW);

    match view.phase {
        ProgramCardPhase::Idle => {
            let reader_hit = source_to_screen(photo, CARD_READER_HIT);
            let reader_response = ui
                .interact(
                    reader_hit,
                    ui.make_persistent_id("hp67-magnetic-card-reader"),
                    Sense::click(),
                )
                .on_hover_text("Insert program card into the magnetic reader");
            if reader_response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                paint_reader_hint(ui.painter(), photo, reader_hit, scale);
            }
            output.reader_clicked = reader_response.clicked();
        }
        ProgramCardPhase::ReadingFromRight => {
            paint_reader_motion(
                ui,
                photo,
                view.artwork,
                view.phase_progress.clamp(0.0, 1.0),
                scale,
            );
        }
        ProgramCardPhase::ParkedLeft => {
            let visible = parked_left_visible_rect(photo, scale);
            let response = ui
                .interact(
                    visible,
                    ui.make_persistent_id("hp67-program-card-left-exit"),
                    Sense::click(),
                )
                .on_hover_text("Move program card to the holder above A-E");
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }
            output.parked_left_clicked = response.clicked();
            paint_parked_left(ui, photo, view.artwork, scale);
        }
        ProgramCardPhase::MovingToWindow => {
            paint_move_to_window(
                ui,
                photo,
                window,
                view.artwork,
                view.phase_progress.clamp(0.0, 1.0),
                scale,
            );
        }
        ProgramCardPhase::InWindow => {
            let response = ui
                .interact(
                    window,
                    ui.make_persistent_id("hp67-program-card-window"),
                    Sense::click(),
                )
                .on_hover_text("Remove program card from holder");
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }
            output.window_clicked = response.clicked();
            paint_card(
                &ui.painter_at(window),
                window,
                view.artwork,
                CardPalette::holder(),
                scale,
            );
        }
    }

    output
}

#[derive(Clone, Copy)]
struct CardPalette {
    body: Color32,
    edge: Color32,
    title: Color32,
    primary: Color32,
    shifted: Color32,
}

impl CardPalette {
    fn holder() -> Self {
        Self {
            body: Color32::from_rgb(83, 80, 57),
            edge: Color32::from_rgb(133, 126, 85),
            title: Color32::from_rgb(232, 232, 220),
            primary: Color32::from_rgb(226, 226, 214),
            shifted: Color32::from_rgb(198, 174, 73),
        }
    }
}

fn paint_card(
    painter: &Painter,
    rect: Rect,
    card: &ProgramCardArtwork,
    palette: CardPalette,
    scale: f32,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let tip = (19.0 * scale).min(rect.width() * 0.08);
    let shoulder = (7.0 * scale).min(rect.height() * 0.18);
    let points = vec![
        rect.left_top(),
        pos2(rect.right() - tip, rect.top()),
        pos2(rect.right() - tip + shoulder, rect.center().y),
        pos2(rect.right() - tip, rect.bottom()),
        rect.left_bottom(),
    ];
    painter.add(Shape::convex_polygon(
        points,
        palette.body,
        Stroke::new((1.2 * scale).max(0.55), palette.edge),
    ));

    let marker_y = rect.top() + 2.0 * scale;
    for index in 0..5 {
        let x = rect.left() + rect.width() * (index as f32 + 0.5) / 5.0;
        painter.line_segment(
            [
                pos2(x - 3.5 * scale, marker_y),
                pos2(x + 3.5 * scale, marker_y),
            ],
            Stroke::new((1.7 * scale).max(0.7), palette.title),
        );
    }

    let title_font = FontId::proportional((15.0 * scale).max(7.5));
    let label_font = FontId::proportional((12.5 * scale).max(6.5));
    let reference_font = FontId::proportional((10.5 * scale).max(6.0));

    painter.text(
        pos2(rect.center().x, rect.top() + rect.height() * 0.29),
        Align2::CENTER_CENTER,
        card.title,
        title_font,
        palette.title,
    );
    painter.text(
        pos2(
            rect.right() - 28.0 * scale,
            rect.top() + rect.height() * 0.29,
        ),
        Align2::RIGHT_CENTER,
        card.reference,
        reference_font,
        palette.shifted,
    );

    for index in 0..5 {
        let x = rect.left() + rect.width() * (index as f32 + 0.5) / 5.0;
        let shifted = card.shifted_labels[index];
        if !shifted.is_empty() {
            painter.text(
                pos2(x, rect.top() + rect.height() * 0.58),
                Align2::CENTER_CENTER,
                shifted,
                label_font.clone(),
                palette.shifted,
            );
        }
        let primary = card.primary_labels[index];
        if !primary.is_empty() {
            painter.text(
                pos2(x, rect.top() + rect.height() * 0.79),
                Align2::CENTER_CENTER,
                primary,
                label_font.clone(),
                palette.primary,
            );
        }
    }
}

fn paint_reader_motion(
    ui: &Ui,
    photo: Rect,
    card: &ProgramCardArtwork,
    progress: f32,
    scale: f32,
) {
    let reader_mouth_x = source_x_to_screen(photo, CARD_READER_MOUTH_X);
    let exit_mouth_x = source_x_to_screen(photo, CARD_EXIT_MOUTH_X);
    let card_width = CARD_PHYSICAL_WIDTH * scale;
    let card_height = CARD_PHYSICAL_HEIGHT * scale;
    let start_left = reader_mouth_x + 42.0 * scale;
    let end_left = exit_mouth_x - CARD_LEFT_VISIBLE_WIDTH * scale;
    let left = start_left + (end_left - start_left) * ease_in_out(progress);
    let top = photo.top() + 370.0 * scale;
    let rect = Rect::from_min_max(pos2(left, top), pos2(left + card_width, top + card_height));

    // The case hides the middle of the same physical card. Because the card is
    // slightly longer than the distance between both mouths, the leading edge
    // starts emerging on the left before the trailing edge vanishes on the right.
    let right_clip = Rect::from_min_max(
        pos2(reader_mouth_x, ui.clip_rect().top()),
        ui.clip_rect().right_bottom(),
    );
    let left_clip = Rect::from_min_max(
        ui.clip_rect().left_top(),
        pos2(exit_mouth_x, ui.clip_rect().bottom()),
    );
    if right_clip.intersects(rect) {
        paint_card(
            &ui.painter().with_clip_rect(right_clip),
            rect,
            card,
            CardPalette::holder(),
            scale,
        );
    }
    if left_clip.intersects(rect) {
        paint_card(
            &ui.painter().with_clip_rect(left_clip),
            rect,
            card,
            CardPalette::holder(),
            scale,
        );
    }
}

fn parked_left_rect(photo: Rect, scale: f32) -> Rect {
    let exit_mouth_x = source_x_to_screen(photo, CARD_EXIT_MOUTH_X);
    let left = exit_mouth_x - CARD_LEFT_VISIBLE_WIDTH * scale;
    let top = photo.top() + 370.0 * scale;
    Rect::from_min_max(
        pos2(left, top),
        pos2(
            left + CARD_PHYSICAL_WIDTH * scale,
            top + CARD_PHYSICAL_HEIGHT * scale,
        ),
    )
}

fn parked_left_visible_rect(photo: Rect, scale: f32) -> Rect {
    let parked = parked_left_rect(photo, scale);
    let exit_mouth_x = source_x_to_screen(photo, CARD_EXIT_MOUTH_X);
    Rect::from_min_max(parked.min, pos2(exit_mouth_x, parked.bottom()))
}

fn paint_parked_left(ui: &Ui, photo: Rect, card: &ProgramCardArtwork, scale: f32) {
    let rect = parked_left_rect(photo, scale);
    let visible = parked_left_visible_rect(photo, scale);
    paint_card(
        &ui.painter().with_clip_rect(visible),
        rect,
        card,
        CardPalette::holder(),
        scale,
    );
}

fn paint_move_to_window(
    ui: &Ui,
    photo: Rect,
    window: Rect,
    card: &ProgramCardArtwork,
    progress: f32,
    scale: f32,
) {
    let t = ease_in_out(progress);
    let start = parked_left_rect(photo, scale);
    let rect = lerp_rect(start, window, t);
    let exit_mouth_x = source_x_to_screen(photo, CARD_EXIT_MOUTH_X);
    let reveal_right = exit_mouth_x + (window.right() - exit_mouth_x) * t;
    let reveal_clip = Rect::from_min_max(
        ui.clip_rect().left_top(),
        pos2(reveal_right, ui.clip_rect().bottom()),
    );
    paint_card(
        &ui.painter().with_clip_rect(reveal_clip),
        rect,
        card,
        CardPalette::holder(),
        scale,
    );
}

fn lerp_rect(from: Rect, to: Rect, t: f32) -> Rect {
    Rect::from_min_max(
        pos2(
            from.left() + (to.left() - from.left()) * t,
            from.top() + (to.top() - from.top()) * t,
        ),
        pos2(
            from.right() + (to.right() - from.right()) * t,
            from.bottom() + (to.bottom() - from.bottom()) * t,
        ),
    )
}

fn source_x_to_screen(photo: Rect, x: f32) -> f32 {
    photo.left() + x * photo.width() / PHOTO_W
}

fn paint_reader_hint(painter: &Painter, photo: Rect, hit: Rect, scale: f32) {
    let x = photo.left() + CARD_READER_MOUTH_X * photo.width() / PHOTO_W;
    let y = hit.center().y;
    let color = Color32::from_rgba_unmultiplied(210, 184, 93, 165);
    painter.line_segment(
        [pos2(x - 9.0 * scale, y), pos2(x + 5.0 * scale, y)],
        Stroke::new((1.6 * scale).max(0.7), color),
    );
    painter.line_segment(
        [
            pos2(x + 5.0 * scale, y),
            pos2(x - 1.0 * scale, y - 5.0 * scale),
        ],
        Stroke::new((1.6 * scale).max(0.7), color),
    );
    painter.line_segment(
        [
            pos2(x + 5.0 * scale, y),
            pos2(x - 1.0 * scale, y + 5.0 * scale),
        ],
        Stroke::new((1.6 * scale).max(0.7), color),
    );
}

fn source_to_screen(photo: Rect, src: SourceRect) -> Rect {
    let sx = photo.width() / PHOTO_W;
    let sy = photo.height() / PHOTO_H;
    Rect::from_min_max(
        pos2(photo.left() + src.x0 * sx, photo.top() + src.y0 * sy),
        pos2(photo.left() + src.x1 * sx, photo.top() + src.y1 * sy),
    )
}

fn ease_in_out(value: f32) -> f32 {
    value * value * (3.0 - 2.0 * value)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn moon_rocket_lander_artwork_maps_a_and_b_without_changing_key_identity() {
        assert_eq!(MOON_ROCKET_LANDER_CARD.primary_labels[0], "CNTRL");
        assert_eq!(MOON_ROCKET_LANDER_CARD.primary_labels[1], "RESTART");
        assert_eq!(MOON_ROCKET_LANDER_CARD.reference, "SD-14A");
        assert!(MOON_ROCKET_LANDER_CARD.primary_labels[2..]
            .iter()
            .all(|label| label.is_empty()));
    }

    #[test]
    fn card_window_sits_above_the_a_to_e_key_row() {
        assert!(CARD_WINDOW.y1 < 467.0);
        assert!(CARD_WINDOW.x0 < 166.0);
        assert!(CARD_WINDOW.x1 > 764.0);
    }

    #[test]
    fn reader_hotspot_is_at_the_right_edge_not_a_second_front_slot() {
        assert!(CARD_READER_HIT.x0 >= 775.0);
        assert!(CARD_READER_HIT.x1 < PHOTO_W);
        assert_eq!(CARD_READER_MOUTH_X, CARD_READER_HIT.x1);
        assert!(CARD_EXIT_MOUTH_X < CARD_WINDOW.x0);
        assert!(CARD_READER_HIT.y0 < CARD_WINDOW.y1);
        assert!(CARD_READER_HIT.y1 > CARD_WINDOW.y0);
    }

    #[test]
    fn physical_card_is_long_enough_to_bridge_reader_and_exit_mouths() {
        assert!(CARD_PHYSICAL_WIDTH > CARD_READER_MOUTH_X - CARD_EXIT_MOUTH_X);
        assert!(CARD_LEFT_VISIBLE_WIDTH < CARD_PHYSICAL_WIDTH);
    }

    #[test]
    fn parked_card_exposes_only_a_clickable_left_tab() {
        let photo = Rect::from_min_size(
            pos2(0.0, 0.0),
            eframe::egui::vec2(PHOTO_W, PHOTO_H),
        );
        let parked = parked_left_rect(photo, 1.0);
        let visible = parked_left_visible_rect(photo, 1.0);
        assert_eq!(visible.width(), CARD_LEFT_VISIBLE_WIDTH);
        assert_eq!(visible.right(), CARD_EXIT_MOUTH_X);
        assert!(parked.right() > CARD_EXIT_MOUTH_X);
    }

    #[test]
    fn card_phase_animation_only_covers_motion_states() {
        assert!(!ProgramCardPhase::Idle.is_animating());
        assert!(ProgramCardPhase::ReadingFromRight.is_animating());
        assert!(!ProgramCardPhase::ParkedLeft.is_animating());
        assert!(ProgramCardPhase::MovingToWindow.is_animating());
        assert!(!ProgramCardPhase::InWindow.is_animating());
    }

    #[test]
    fn insertion_easing_has_exact_endpoints() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        assert!((ease_in_out(0.5) - 0.5).abs() < f32::EPSILON);
    }
}
