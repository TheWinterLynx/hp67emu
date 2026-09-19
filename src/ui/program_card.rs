use eframe::egui::{
    pos2, Align2, Color32, CursorIcon, FontId, Painter, Rect, Sense, Shape, Stroke, TextureHandle,
    Ui,
};

const PHOTO_W: f32 = 928.0;
const PHOTO_H: f32 = 1695.0;

const CARD_WINDOW: SourceRect = SourceRect::new(138.0, 345.0, 796.0, 454.0);
const HOLDER_CARD_CENTER_X_OFFSET_SOURCE_PX: f32 = -1.5;
const HOLDER_RIGHT_FRAME_TOP_X: f32 = 787.0;
const HOLDER_RIGHT_FRAME_BOTTOM_X: f32 = 792.0;
const HOLDER_RIGHT_MASK_BANDS: usize = 109;
const CARD_READER_HIT: SourceRect = SourceRect::new(775.0, 356.0, 842.0, 452.0);
const CARD_READER_MOUTH_X: f32 = CARD_READER_HIT.x1;
const CARD_EXIT_MOUTH_X: f32 = 64.0;
const HP67_CASE_WIDTH_MM: f32 = 81.0;
const HP67_CASE_WIDTH_SOURCE_PX: f32 = 792.0;
const SOURCE_PX_PER_MM: f32 = HP67_CASE_WIDTH_SOURCE_PX / HP67_CASE_WIDTH_MM;
const CARD_WIDTH_MM: f32 = 71.1;
const CARD_HEIGHT_MM: f32 = 11.4;
const CARD_END_CHAMFER_MM: f32 = 4.2;
const CARD_PHYSICAL_WIDTH: f32 = CARD_WIDTH_MM * SOURCE_PX_PER_MM;
const CARD_PHYSICAL_HEIGHT: f32 = CARD_HEIGHT_MM * SOURCE_PX_PER_MM;
const CARD_LEFT_VISIBLE_WIDTH: f32 = 10.5 * SOURCE_PX_PER_MM;
const CARD_LABEL_X_FRACTIONS: [f32; 5] = [0.133_918, 0.316_600, 0.499_281, 0.681_962, 0.864_643];
const MOON_ROCKET_LANDER_TOP_MARKS: &[f32] = &[0.095, 0.224, 0.310];
const CARD_TOP_MARK_WIDTH_MM: f32 = 0.9;
const CARD_TOP_MARK_HEIGHT_MM: f32 = 0.75;
const CARD_TITLE_Y_FRACTION: f32 = 0.31;
const CARD_SHIFTED_Y_FRACTION: f32 = 0.61;
const CARD_PRIMARY_Y_FRACTION: f32 = 0.82;
const CARD_REFERENCE_X_FRACTION: f32 = 0.89;
const CARD_HP_LOGO_X_FRACTION: f32 = 0.052;
const CARD_HP_LOGO_Y_FRACTION: f32 = 0.69;
const CARD_HP_LOGO_HEIGHT_MM: f32 = 4.6;
const CARD_HP_LOGO_ASPECT: f32 = 46.0 / 70.0;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ProgramCardArtwork {
    pub title: &'static str,
    pub reference: &'static str,
    pub primary_labels: [&'static str; 5],
    pub shifted_labels: [&'static str; 5],
    pub top_marks: &'static [f32],
    pub show_hp_logo: bool,
}

pub const MOON_ROCKET_LANDER_CARD: ProgramCardArtwork = ProgramCardArtwork {
    title: "MOON ROCKET LANDER",
    reference: "SD-14A",
    primary_labels: ["CNTRL", "RESTART", "", "", ""],
    shifted_labels: ["", "", "", "", ""],
    top_marks: MOON_ROCKET_LANDER_TOP_MARKS,
    show_hp_logo: true,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ProgramCardPhase {
    Idle,
    ReadingFromRight,
    ParkedLeft,
    InsertingWindowFromRight,
    InWindow,
}

impl ProgramCardPhase {
    pub const fn is_animating(self) -> bool {
        matches!(
            self,
            Self::ReadingFromRight | Self::InsertingWindowFromRight
        )
    }
}

#[derive(Clone, Copy)]
pub struct ProgramCardView<'a> {
    pub artwork: &'a ProgramCardArtwork,
    pub logo: &'a TextureHandle,
    pub phase: ProgramCardPhase,
    pub phase_progress: f32,
    pub opposite_track_requested: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgramCardUiOutput {
    pub reader_clicked: bool,
    pub parked_left_clicked: bool,
    pub parked_left_double_clicked: bool,
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

pub fn paint(
    ui: &mut Ui,
    photo: Rect,
    body: &TextureHandle,
    view: ProgramCardView<'_>,
) -> ProgramCardUiOutput {
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
                view.logo,
                view.phase_progress.clamp(0.0, 1.0),
                scale,
            );
        }
        ProgramCardPhase::ParkedLeft => {
            let visible = parked_left_visible_rect(photo, scale);
            let hover_text = if view.opposite_track_requested {
                "Crd: double-click to rotate the same card 180° and reinsert the opposite end"
            } else {
                "Click: move card to the holder above A-E. Double-click: rotate 180° and select the opposite end"
            };
            let response = ui
                .interact(
                    visible,
                    ui.make_persistent_id("hp67-program-card-left-exit"),
                    Sense::click(),
                )
                .on_hover_text(hover_text);
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }
            output.parked_left_double_clicked = response.double_clicked();
            output.parked_left_clicked = response.clicked() && !response.double_clicked();
            paint_parked_left(ui, photo, view.artwork, view.logo, scale);
        }
        ProgramCardPhase::InsertingWindowFromRight => {
            paint_window_insertion_from_right(
                ui,
                photo,
                body,
                window,
                view.artwork,
                view.logo,
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
            let card_rect = holder_card_rect(window, scale);
            paint_card(
                &ui.painter().with_clip_rect(window),
                card_rect,
                view.artwork,
                view.logo,
                CardPalette::holder(),
                scale,
            );
            paint_holder_right_frame_mask(ui.painter(), photo, body);
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
            body: Color32::from_rgb(30, 31, 21),
            edge: Color32::from_rgb(71, 72, 47),
            title: Color32::from_rgb(238, 238, 226),
            primary: Color32::from_rgb(238, 238, 226),
            shifted: Color32::from_rgb(205, 172, 63),
        }
    }
}

fn paint_card(
    painter: &Painter,
    rect: Rect,
    card: &ProgramCardArtwork,
    logo: &TextureHandle,
    palette: CardPalette,
    scale: f32,
) {
    if rect.width() <= 0.0 || rect.height() <= 0.0 {
        return;
    }

    let chamfer = (CARD_END_CHAMFER_MM * SOURCE_PX_PER_MM * scale)
        .min(rect.width() * 0.10)
        .min(rect.height() * 0.42);
    let points = vec![
        pos2(rect.left() + chamfer, rect.top()),
        rect.right_top(),
        pos2(rect.right(), rect.bottom() - chamfer),
        pos2(rect.right() - chamfer, rect.bottom()),
        rect.left_bottom(),
        pos2(rect.left(), rect.top() + chamfer),
    ];
    painter.add(Shape::convex_polygon(
        points,
        palette.body,
        Stroke::new((1.0 * scale).max(0.5), palette.edge),
    ));

    let notch_width = CARD_TOP_MARK_WIDTH_MM * SOURCE_PX_PER_MM * scale;
    let notch_height = CARD_TOP_MARK_HEIGHT_MM * SOURCE_PX_PER_MM * scale;
    for &fraction in card.top_marks {
        let x = rect.left() + rect.width() * fraction;
        painter.rect_filled(
            Rect::from_min_max(
                pos2(x - notch_width * 0.5, rect.top()),
                pos2(x + notch_width * 0.5, rect.top() + notch_height),
            ),
            0.0,
            palette.title,
        );
    }

    if card.show_hp_logo {
        paint_hp_card_logo(painter, rect, logo, scale);
    }

    let title_font = FontId::proportional((15.0 * scale).max(7.5));
    let label_font = FontId::proportional((12.5 * scale).max(6.5));
    let reference_font = FontId::proportional((10.5 * scale).max(6.0));

    painter.text(
        pos2(
            rect.center().x,
            rect.top() + rect.height() * CARD_TITLE_Y_FRACTION,
        ),
        Align2::CENTER_CENTER,
        card.title,
        title_font,
        palette.title,
    );
    painter.text(
        pos2(
            rect.left() + rect.width() * CARD_REFERENCE_X_FRACTION,
            rect.top() + rect.height() * CARD_TITLE_Y_FRACTION,
        ),
        Align2::RIGHT_CENTER,
        card.reference,
        reference_font,
        palette.shifted,
    );

    for (index, fraction) in CARD_LABEL_X_FRACTIONS.iter().copied().enumerate() {
        let x = rect.left() + rect.width() * fraction;
        let shifted = card.shifted_labels[index];
        if !shifted.is_empty() {
            painter.text(
                pos2(x, rect.top() + rect.height() * CARD_SHIFTED_Y_FRACTION),
                Align2::CENTER_CENTER,
                shifted,
                label_font.clone(),
                palette.shifted,
            );
        }
        let primary = card.primary_labels[index];
        if !primary.is_empty() {
            painter.text(
                pos2(x, rect.top() + rect.height() * CARD_PRIMARY_Y_FRACTION),
                Align2::CENTER_CENTER,
                primary,
                label_font.clone(),
                palette.primary,
            );
        }
    }
}

fn paint_hp_card_logo(painter: &Painter, rect: Rect, logo: &TextureHandle, scale: f32) {
    let height = CARD_HP_LOGO_HEIGHT_MM * SOURCE_PX_PER_MM * scale;
    let width = height * CARD_HP_LOGO_ASPECT;
    let center = pos2(
        rect.left() + rect.width() * CARD_HP_LOGO_X_FRACTION,
        rect.top() + rect.height() * CARD_HP_LOGO_Y_FRACTION,
    );
    let logo_rect = Rect::from_center_size(center, eframe::egui::vec2(width, height));
    painter.image(
        logo.id(),
        logo_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
}

fn paint_reader_motion(
    ui: &Ui,
    photo: Rect,
    card: &ProgramCardArtwork,
    logo: &TextureHandle,
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

    // The case hides the middle of the same physical card. At the locked 1:1
    // size there is a short interval where the whole card is hidden inside the
    // reader before its leading edge emerges from the left mouth.
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
            logo,
            CardPalette::holder(),
            scale,
        );
    }
    if left_clip.intersects(rect) {
        paint_card(
            &ui.painter().with_clip_rect(left_clip),
            rect,
            card,
            logo,
            CardPalette::holder(),
            scale,
        );
    }
}

fn holder_card_rect(window: Rect, scale: f32) -> Rect {
    let width = CARD_PHYSICAL_WIDTH * scale;
    let height = CARD_PHYSICAL_HEIGHT * scale;
    let center = pos2(
        window.center().x + HOLDER_CARD_CENTER_X_OFFSET_SOURCE_PX * scale,
        window.center().y,
    );
    Rect::from_center_size(center, eframe::egui::vec2(width, height))
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

fn paint_parked_left(
    ui: &Ui,
    photo: Rect,
    card: &ProgramCardArtwork,
    logo: &TextureHandle,
    scale: f32,
) {
    let rect = parked_left_rect(photo, scale);
    let visible = parked_left_visible_rect(photo, scale);
    paint_card(
        &ui.painter().with_clip_rect(visible),
        rect,
        card,
        logo,
        CardPalette::holder(),
        scale,
    );
}

fn paint_window_insertion_from_right(
    ui: &Ui,
    photo: Rect,
    body: &TextureHandle,
    window: Rect,
    card: &ProgramCardArtwork,
    logo: &TextureHandle,
    progress: f32,
    scale: f32,
) {
    let t = ease_in_out(progress);
    let final_rect = holder_card_rect(window, scale);
    let card_width = final_rect.width();
    let card_height = final_rect.height();
    let holder_mouth_x = window.right();
    let case_right_x = source_x_to_screen(photo, CARD_READER_MOUTH_X);
    let start_left = case_right_x + 28.0 * scale;
    let end_left = final_rect.left();
    let left = start_left + (end_left - start_left) * t;
    let rect = Rect::from_min_max(
        pos2(left, final_rect.top()),
        pos2(left + card_width, final_rect.top() + card_height),
    );

    // The card does not travel over the calculator shell. From the top-down
    // view, the right-hand part remains visible only while it is still outside
    // the case; after crossing the case edge it is hidden beneath the body/lip.
    // It becomes visible again only inside the passive holder window.
    let outside_right = Rect::from_min_max(
        pos2(case_right_x, ui.clip_rect().top()),
        ui.clip_rect().right_bottom(),
    );
    let holder_window = window.intersect(ui.clip_rect());

    if outside_right.intersects(rect) {
        paint_card(
            &ui.painter().with_clip_rect(outside_right),
            rect,
            card,
            logo,
            CardPalette::holder(),
            scale,
        );
    }

    if holder_window.intersects(rect) {
        paint_card(
            &ui.painter().with_clip_rect(holder_window),
            rect,
            card,
            logo,
            CardPalette::holder(),
            scale,
        );
        paint_holder_right_frame_mask(ui.painter(), photo, body);
    }

    debug_assert!(holder_mouth_x <= case_right_x);
}

fn paint_holder_right_frame_mask(painter: &Painter, photo: Rect, body: &TextureHandle) {
    let y_span = CARD_WINDOW.y1 - CARD_WINDOW.y0;
    for band in 0..HOLDER_RIGHT_MASK_BANDS {
        let t0 = band as f32 / HOLDER_RIGHT_MASK_BANDS as f32;
        let t1 = (band + 1) as f32 / HOLDER_RIGHT_MASK_BANDS as f32;
        let tm = (t0 + t1) * 0.5;
        let y0 = CARD_WINDOW.y0 + y_span * t0;
        let y1 = CARD_WINDOW.y0 + y_span * t1;
        let frame_x = HOLDER_RIGHT_FRAME_TOP_X
            + (HOLDER_RIGHT_FRAME_BOTTOM_X - HOLDER_RIGHT_FRAME_TOP_X) * tm;
        let src = SourceRect::new(frame_x, y0, CARD_WINDOW.x1, y1);
        painter.image(
            body.id(),
            source_to_screen(photo, src),
            source_to_uv(src),
            Color32::WHITE,
        );
    }
}

fn source_to_uv(src: SourceRect) -> Rect {
    Rect::from_min_max(
        pos2(src.x0 / PHOTO_W, src.y0 / PHOTO_H),
        pos2(src.x1 / PHOTO_W, src.y1 / PHOTO_H),
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

    const A_E_KEY_CENTERS_X: [f32; 5] = [211.0, 338.0, 465.0, 592.0, 719.0];

    #[test]
    fn moon_rocket_lander_artwork_maps_a_and_b_without_changing_key_identity() {
        assert_eq!(MOON_ROCKET_LANDER_CARD.primary_labels[0], "CNTRL");
        assert_eq!(MOON_ROCKET_LANDER_CARD.primary_labels[1], "RESTART");
        assert_eq!(MOON_ROCKET_LANDER_CARD.reference, "SD-14A");
        assert_eq!(
            MOON_ROCKET_LANDER_CARD.top_marks,
            MOON_ROCKET_LANDER_TOP_MARKS
        );
        assert!(MOON_ROCKET_LANDER_CARD.show_hp_logo);
        assert!(MOON_ROCKET_LANDER_CARD.primary_labels[2..]
            .iter()
            .all(|label| label.is_empty()));
    }

    #[test]
    fn card_window_is_inside_the_photographed_holder_frame() {
        assert_eq!(CARD_WINDOW.x0, 138.0);
        assert_eq!(CARD_WINDOW.x1, 796.0);
        assert_eq!(HOLDER_CARD_CENTER_X_OFFSET_SOURCE_PX, -1.5);
        assert_eq!(HOLDER_RIGHT_FRAME_TOP_X, 787.0);
        assert_eq!(HOLDER_RIGHT_FRAME_BOTTOM_X, 792.0);
        assert!(HOLDER_RIGHT_FRAME_TOP_X < HOLDER_RIGHT_FRAME_BOTTOM_X);
        assert!(HOLDER_RIGHT_FRAME_BOTTOM_X < CARD_WINDOW.x1);
        assert_eq!(CARD_WINDOW.y0, 345.0);
        assert_eq!(CARD_WINDOW.y1, 454.0);
        assert!(CARD_WINDOW.x1 - CARD_WINDOW.x0 < CARD_PHYSICAL_WIDTH);
        assert!(CARD_WINDOW.y1 - CARD_WINDOW.y0 < CARD_PHYSICAL_HEIGHT);
    }

    #[test]
    fn holder_label_anchors_match_the_real_a_to_e_key_centres() {
        let photo = Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(PHOTO_W, PHOTO_H));
        let window = source_to_screen(photo, CARD_WINDOW);
        let card = holder_card_rect(window, 1.0);

        for (fraction, expected_x) in CARD_LABEL_X_FRACTIONS
            .iter()
            .copied()
            .zip(A_E_KEY_CENTERS_X)
        {
            let actual_x = card.left() + card.width() * fraction;
            assert!((actual_x - expected_x).abs() < 0.15);
        }
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
    fn physical_card_matches_hp_magnetic_card_dimensions() {
        assert!((CARD_WIDTH_MM - 71.1).abs() < 0.0001);
        assert!((CARD_HEIGHT_MM - 11.4).abs() < 0.0001);
        assert!((CARD_PHYSICAL_WIDTH - 695.2).abs() < 0.2);
        assert!((CARD_PHYSICAL_HEIGHT - 111.47).abs() < 0.2);
        assert!(
            (CARD_PHYSICAL_WIDTH / CARD_PHYSICAL_HEIGHT - CARD_WIDTH_MM / CARD_HEIGHT_MM).abs()
                < 0.0001
        );
        assert!((CARD_END_CHAMFER_MM - 4.2).abs() < 0.0001);
        assert!(CARD_END_CHAMFER_MM < CARD_HEIGHT_MM * 0.5);
        assert_eq!(MOON_ROCKET_LANDER_CARD.top_marks.len(), 3);
    }

    #[test]
    fn holder_is_an_aperture_not_a_card_resizer() {
        let photo = Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(PHOTO_W, PHOTO_H));
        let window = source_to_screen(photo, CARD_WINDOW);
        let card = holder_card_rect(window, 1.0);
        assert!((card.width() - CARD_PHYSICAL_WIDTH).abs() < 0.001);
        assert!((card.height() - CARD_PHYSICAL_HEIGHT).abs() < 0.001);
        assert!(card.width() > window.width());
        assert!(card.width() - window.width() < CARD_PHYSICAL_WIDTH * 0.07);
        assert!(card.height() > window.height());
        assert!(card.height() - window.height() < CARD_PHYSICAL_HEIGHT * 0.03);
    }

    #[test]
    fn physical_card_keeps_real_length_across_hidden_reader_transit() {
        assert!(CARD_LEFT_VISIBLE_WIDTH < CARD_PHYSICAL_WIDTH);
        assert!(CARD_PHYSICAL_WIDTH < CARD_READER_MOUTH_X - CARD_EXIT_MOUTH_X);
    }

    #[test]
    fn parked_card_exposes_only_a_clickable_left_tab() {
        let photo = Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(PHOTO_W, PHOTO_H));
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
        assert!(ProgramCardPhase::InsertingWindowFromRight.is_animating());
        assert!(!ProgramCardPhase::InWindow.is_animating());
    }

    #[test]
    fn holder_window_stays_inside_the_case_and_occludes_shell_travel() {
        assert!(CARD_WINDOW.x1 < CARD_READER_MOUTH_X);
        assert!(CARD_WINDOW.x0 < CARD_WINDOW.x1);
    }

    #[test]
    fn insertion_easing_has_exact_endpoints() {
        assert_eq!(ease_in_out(0.0), 0.0);
        assert_eq!(ease_in_out(1.0), 1.0);
        assert!((ease_in_out(0.5) - 0.5).abs() < f32::EPSILON);
    }
}
