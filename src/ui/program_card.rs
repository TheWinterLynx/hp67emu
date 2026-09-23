use std::f32::consts::PI;

use eframe::egui::{
    emath::Rot2,
    epaint::{TextShape, Vertex},
    pos2, Align2, Color32, CursorIcon, FontId, Mesh, Painter, Rect, Sense, Shape, Stroke,
    TextureHandle, Ui,
};

const PHOTO_W: f32 = 928.0;
const PHOTO_H: f32 = 1695.0;

const CARD_WINDOW: SourceRect = SourceRect::new(138.0, 345.0, 796.0, 454.0);
const HOLDER_CARD_CENTER_X_OFFSET_SOURCE_PX: f32 = -1.5;
const HOLDER_RIGHT_FRAME_TOP_X: f32 = 787.0;
const HOLDER_RIGHT_FRAME_BOTTOM_X: f32 = 792.0;
const HOLDER_BACKING_FRAME_GAP_SOURCE_PX: f32 = 0.75;
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

pub const GENERIC_MAGNETIC_CARD: ProgramCardArtwork = ProgramCardArtwork {
    title: "",
    reference: "",
    primary_labels: ["", "", "", "", ""],
    shifted_labels: ["", "", "", "", ""],
    top_marks: &[],
    show_hp_logo: false,
};

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
    WaitingAtReader,
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
    pub face_texture: Option<&'a TextureHandle>,
    pub phase: ProgramCardPhase,
    pub phase_progress: f32,
    pub opposite_track_requested: bool,
    pub rotated_180: bool,
    pub reader_enabled: bool,
    pub reader_free_for_new_blank: bool,
    pub minimum_touch_target: f32,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct ProgramCardUiOutput {
    pub reader_clicked: bool,
    pub blank_card_requested: bool,
    pub waiting_reader_clicked: bool,
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
            let reader_hit = minimum_hit_rect(
                source_to_screen(photo, CARD_READER_HIT),
                view.minimum_touch_target,
            );
            let reader_sense = if view.reader_enabled {
                Sense::click()
            } else {
                Sense::hover()
            };
            let hover_text = if view.reader_enabled {
                "Click: insert loaded card, or a new blank card if none is loaded. Right-click: insert a new blank card"
            } else {
                "Power on the calculator before inserting a magnetic card"
            };
            let reader_response = ui
                .interact(
                    reader_hit,
                    ui.make_persistent_id("hp67-magnetic-card-reader"),
                    reader_sense,
                )
                .on_hover_text(hover_text);
            if reader_response.hovered() && view.reader_enabled {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
                paint_reader_hint(ui.painter(), photo, reader_hit, scale);
            }
            if view.reader_enabled {
                output.reader_clicked = reader_response.clicked();
                output.blank_card_requested = reader_response.secondary_clicked();
            }
        }
        ProgramCardPhase::WaitingAtReader => {
            paint_reader_motion(
                ui,
                photo,
                view.artwork,
                view.logo,
                view.face_texture,
                0.0,
                scale,
                view.rotated_180,
            );
            let waiting_hint = if view.minimum_touch_target > 0.0 {
                "Card is waiting for the reader motor. Tap to withdraw it"
            } else {
                "Card is waiting for the reader motor. Right-click to withdraw it"
            };
            let response = ui
                .interact(
                    minimum_hit_rect(
                        waiting_reader_hit_rect(photo, scale, ui.clip_rect()),
                        view.minimum_touch_target,
                    )
                    .intersect(ui.clip_rect()),
                    ui.make_persistent_id("hp67-magnetic-card-waiting"),
                    Sense::click(),
                )
                .on_hover_text(waiting_hint);
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }
            output.waiting_reader_clicked = if view.minimum_touch_target > 0.0 {
                response.clicked() || response.secondary_clicked()
            } else {
                response.secondary_clicked()
            };
        }
        ProgramCardPhase::ReadingFromRight => {
            paint_reader_motion(
                ui,
                photo,
                view.artwork,
                view.logo,
                view.face_texture,
                view.phase_progress.clamp(0.0, 1.0),
                scale,
                view.rotated_180,
            );
        }
        ProgramCardPhase::ParkedLeft => {
            let visible = minimum_hit_rect(
                parked_left_visible_rect(photo, scale),
                view.minimum_touch_target,
            )
            .intersect(ui.clip_rect());
            let hover_text = if view.opposite_track_requested {
                "Crd: click to rotate the same card 180° and reinsert the opposite end"
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
            paint_parked_left(
                ui,
                photo,
                view.artwork,
                view.logo,
                view.face_texture,
                scale,
                view.rotated_180,
            );
        }
        ProgramCardPhase::InsertingWindowFromRight => {
            paint_window_insertion_from_right(
                ui,
                photo,
                body,
                window,
                view.artwork,
                view.logo,
                view.face_texture,
                view.phase_progress.clamp(0.0, 1.0),
                scale,
                false,
            );
        }
        ProgramCardPhase::InWindow => {
            let response = ui
                .interact(
                    minimum_hit_rect(window, view.minimum_touch_target),
                    ui.make_persistent_id("hp67-program-card-window"),
                    Sense::click(),
                )
                .on_hover_text("Remove program card from holder");
            if response.hovered() {
                ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            }
            output.window_clicked = response.clicked();
            let card_rect = holder_card_rect(window, scale);
            // The real holder/slot behind the chamfered card is dark. Back the
            // card only up to (but not over) the photographed right-hand frame,
            // otherwise a few black pixels can cover its white highlight.
            paint_holder_dark_backing(ui.painter(), photo, window, card_rect);
            paint_card(
                &ui.painter().with_clip_rect(window),
                card_rect,
                view.artwork,
                view.logo,
                view.face_texture,
                CardPalette::holder(),
                scale,
                false,
            );
            paint_holder_right_frame_mask(ui.painter(), photo, body);
        }
    }

    if view.reader_enabled && view.reader_free_for_new_blank && view.phase != ProgramCardPhase::Idle
    {
        let reader_hit = minimum_hit_rect(
            source_to_screen(photo, CARD_READER_HIT),
            view.minimum_touch_target,
        );
        let response = ui
            .interact(
                reader_hit,
                ui.make_persistent_id("hp67-magnetic-card-reader-new-blank"),
                Sense::click(),
            )
            .on_hover_text("Right-click: insert a new blank magnetic card");
        if response.hovered() {
            ui.output_mut(|o| o.cursor_icon = CursorIcon::PointingHand);
            paint_reader_hint(ui.painter(), photo, reader_hit, scale);
        }
        output.blank_card_requested |= response.secondary_clicked();
    }

    output
}

fn minimum_hit_rect(rect: Rect, minimum_size: f32) -> Rect {
    if minimum_size <= 0.0 {
        return rect;
    }
    Rect::from_center_size(
        rect.center(),
        eframe::egui::vec2(
            rect.width().max(minimum_size),
            rect.height().max(minimum_size),
        ),
    )
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
    face_texture: Option<&TextureHandle>,
    palette: CardPalette,
    scale: f32,
    rotated_180: bool,
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
        points.clone(),
        palette.body,
        Stroke::new((1.0 * scale).max(0.5), palette.edge),
    ));

    if let Some(face_texture) = face_texture {
        let mut mesh = Mesh::with_texture(face_texture.id());
        let chamfer_u = chamfer / rect.width();
        let chamfer_v = chamfer / rect.height();
        let mut uvs = [
            pos2(chamfer_u, 0.0),
            pos2(1.0, 0.0),
            pos2(1.0, 1.0 - chamfer_v),
            pos2(1.0 - chamfer_u, 1.0),
            pos2(0.0, 1.0),
            pos2(0.0, chamfer_v),
        ];
        if rotated_180 {
            for uv in &mut uvs {
                *uv = pos2(1.0 - uv.x, 1.0 - uv.y);
            }
        }
        for (position, uv) in points.iter().copied().zip(uvs) {
            mesh.vertices.push(Vertex {
                pos: position,
                uv,
                color: Color32::WHITE,
            });
        }
        mesh.indices
            .extend_from_slice(&[0, 1, 2, 0, 2, 3, 0, 3, 4, 0, 4, 5]);
        painter.add(mesh);
        painter.add(Shape::closed_line(
            points,
            Stroke::new((1.0 * scale).max(0.5), palette.edge),
        ));
        return;
    }

    let notch_width = CARD_TOP_MARK_WIDTH_MM * SOURCE_PX_PER_MM * scale;
    let notch_height = CARD_TOP_MARK_HEIGHT_MM * SOURCE_PX_PER_MM * scale;
    for &fraction in card.top_marks {
        let x = rect.left() + rect.width() * fraction;
        let mark = Rect::from_min_max(
            pos2(x - notch_width * 0.5, rect.top()),
            pos2(x + notch_width * 0.5, rect.top() + notch_height),
        );
        let mark = if rotated_180 {
            rotate_rect_180(rect, mark)
        } else {
            mark
        };
        painter.rect_filled(mark, 0.0, palette.title);
    }

    if card.show_hp_logo {
        paint_hp_card_logo(painter, rect, logo, scale, rotated_180);
    }

    let title_font = FontId::proportional((15.0 * scale).max(7.5));
    let label_font = FontId::proportional((12.5 * scale).max(6.5));
    let reference_font = FontId::proportional((10.5 * scale).max(6.0));

    paint_card_text(
        painter,
        rect,
        pos2(
            rect.center().x,
            rect.top() + rect.height() * CARD_TITLE_Y_FRACTION,
        ),
        Align2::CENTER_CENTER,
        card.title,
        title_font,
        palette.title,
        rotated_180,
    );
    paint_card_text(
        painter,
        rect,
        pos2(
            rect.left() + rect.width() * CARD_REFERENCE_X_FRACTION,
            rect.top() + rect.height() * CARD_TITLE_Y_FRACTION,
        ),
        Align2::RIGHT_CENTER,
        card.reference,
        reference_font,
        palette.shifted,
        rotated_180,
    );

    for (index, fraction) in CARD_LABEL_X_FRACTIONS.iter().copied().enumerate() {
        let x = rect.left() + rect.width() * fraction;
        let shifted = card.shifted_labels[index];
        if !shifted.is_empty() {
            paint_card_text(
                painter,
                rect,
                pos2(x, rect.top() + rect.height() * CARD_SHIFTED_Y_FRACTION),
                Align2::CENTER_CENTER,
                shifted,
                label_font.clone(),
                palette.shifted,
                rotated_180,
            );
        }
        let primary = card.primary_labels[index];
        if !primary.is_empty() {
            paint_card_text(
                painter,
                rect,
                pos2(x, rect.top() + rect.height() * CARD_PRIMARY_Y_FRACTION),
                Align2::CENTER_CENTER,
                primary,
                label_font.clone(),
                palette.primary,
                rotated_180,
            );
        }
    }
}

fn rotate_point_180(card_rect: Rect, point: eframe::egui::Pos2) -> eframe::egui::Pos2 {
    pos2(
        card_rect.center().x * 2.0 - point.x,
        card_rect.center().y * 2.0 - point.y,
    )
}

fn rotate_rect_180(card_rect: Rect, rect: Rect) -> Rect {
    Rect::from_center_size(rotate_point_180(card_rect, rect.center()), rect.size())
}

fn paint_card_text(
    painter: &Painter,
    card_rect: Rect,
    position: eframe::egui::Pos2,
    anchor: Align2,
    text: &str,
    font: FontId,
    color: Color32,
    rotated_180: bool,
) {
    if !rotated_180 {
        painter.text(position, anchor, text, font, color);
        return;
    }

    let galley = painter.layout_no_wrap(text.to_owned(), font, color);
    let unrotated = anchor.anchor_size(position, galley.size());
    let center = rotate_point_180(card_rect, unrotated.center());
    let pivot = center + galley.size() * 0.5;
    painter.add(TextShape::new(pivot, galley, color).with_angle(PI));
}

fn paint_hp_card_logo(
    painter: &Painter,
    rect: Rect,
    logo: &TextureHandle,
    scale: f32,
    rotated_180: bool,
) {
    let height = CARD_HP_LOGO_HEIGHT_MM * SOURCE_PX_PER_MM * scale;
    let width = height * CARD_HP_LOGO_ASPECT;
    let center = pos2(
        rect.left() + rect.width() * CARD_HP_LOGO_X_FRACTION,
        rect.top() + rect.height() * CARD_HP_LOGO_Y_FRACTION,
    );
    let center = if rotated_180 {
        rotate_point_180(rect, center)
    } else {
        center
    };
    let logo_rect = Rect::from_center_size(center, eframe::egui::vec2(width, height));
    let mut mesh = Mesh::with_texture(logo.id());
    mesh.add_rect_with_uv(
        logo_rect,
        Rect::from_min_max(pos2(0.0, 0.0), pos2(1.0, 1.0)),
        Color32::WHITE,
    );
    if rotated_180 {
        mesh.rotate(Rot2::from_angle(PI), logo_rect.center());
    }
    painter.add(mesh);
}

fn paint_reader_motion(
    ui: &Ui,
    photo: Rect,
    card: &ProgramCardArtwork,
    logo: &TextureHandle,
    face_texture: Option<&TextureHandle>,
    progress: f32,
    scale: f32,
    rotated_180: bool,
) {
    let reader_mouth_x = source_x_to_screen(photo, CARD_READER_MOUTH_X);
    let exit_mouth_x = source_x_to_screen(photo, CARD_EXIT_MOUTH_X);
    let rect = reader_motion_rect(photo, progress, scale);

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
            face_texture,
            CardPalette::holder(),
            scale,
            rotated_180,
        );
    }
    if left_clip.intersects(rect) {
        paint_card(
            &ui.painter().with_clip_rect(left_clip),
            rect,
            card,
            logo,
            face_texture,
            CardPalette::holder(),
            scale,
            rotated_180,
        );
    }
}

fn waiting_reader_hit_rect(photo: Rect, scale: f32, clip: Rect) -> Rect {
    let reader_hit = source_to_screen(photo, CARD_READER_HIT).intersect(clip);
    let card_hit = reader_motion_rect(photo, 0.0, scale).intersect(clip);
    Rect::from_min_max(
        pos2(
            reader_hit.left().min(card_hit.left()),
            reader_hit.top().min(card_hit.top()),
        ),
        pos2(
            reader_hit.right().max(card_hit.right()),
            reader_hit.bottom().max(card_hit.bottom()),
        ),
    )
}

fn reader_motion_rect(photo: Rect, progress: f32, scale: f32) -> Rect {
    let reader_mouth_x = source_x_to_screen(photo, CARD_READER_MOUTH_X);
    let exit_mouth_x = source_x_to_screen(photo, CARD_EXIT_MOUTH_X);
    let card_width = CARD_PHYSICAL_WIDTH * scale;
    let card_height = CARD_PHYSICAL_HEIGHT * scale;
    let start_left = reader_mouth_x + 42.0 * scale;
    let end_left = exit_mouth_x - CARD_LEFT_VISIBLE_WIDTH * scale;
    let left = start_left + (end_left - start_left) * ease_in_out(progress);
    let top = photo.top() + 370.0 * scale;
    Rect::from_min_max(pos2(left, top), pos2(left + card_width, top + card_height))
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
    face_texture: Option<&TextureHandle>,
    scale: f32,
    rotated_180: bool,
) {
    let rect = parked_left_rect(photo, scale);
    let visible = parked_left_visible_rect(photo, scale);
    paint_card(
        &ui.painter().with_clip_rect(visible),
        rect,
        card,
        logo,
        face_texture,
        CardPalette::holder(),
        scale,
        rotated_180,
    );
}

fn paint_window_insertion_from_right(
    ui: &Ui,
    photo: Rect,
    body: &TextureHandle,
    window: Rect,
    card: &ProgramCardArtwork,
    logo: &TextureHandle,
    face_texture: Option<&TextureHandle>,
    progress: f32,
    scale: f32,
    rotated_180: bool,
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
            face_texture,
            CardPalette::holder(),
            scale,
            rotated_180,
        );
    }

    if holder_window.intersects(rect) {
        paint_holder_dark_backing(ui.painter(), photo, holder_window, rect);
        paint_card(
            &ui.painter().with_clip_rect(holder_window),
            rect,
            card,
            logo,
            face_texture,
            CardPalette::holder(),
            scale,
            rotated_180,
        );
        paint_holder_right_frame_mask(ui.painter(), photo, body);
    }

    debug_assert!(holder_mouth_x <= case_right_x);
}

fn holder_frame_x_source(y_source: f32) -> f32 {
    let t = ((y_source - CARD_WINDOW.y0) / (CARD_WINDOW.y1 - CARD_WINDOW.y0)).clamp(0.0, 1.0);
    HOLDER_RIGHT_FRAME_TOP_X + (HOLDER_RIGHT_FRAME_BOTTOM_X - HOLDER_RIGHT_FRAME_TOP_X) * t
}

fn screen_y_to_source(photo: Rect, y: f32) -> f32 {
    ((y - photo.top()) / photo.height()) * PHOTO_H
}

fn paint_holder_dark_backing(painter: &Painter, photo: Rect, window: Rect, card_rect: Rect) {
    let clipped = window.intersect(card_rect);
    if clipped.width() <= 0.0 || clipped.height() <= 0.0 {
        return;
    }

    let y0_source = screen_y_to_source(photo, clipped.top());
    let y1_source = screen_y_to_source(photo, clipped.bottom());
    let right_top = source_x_to_screen(
        photo,
        holder_frame_x_source(y0_source) - HOLDER_BACKING_FRAME_GAP_SOURCE_PX,
    );
    let right_bottom = source_x_to_screen(
        photo,
        holder_frame_x_source(y1_source) - HOLDER_BACKING_FRAME_GAP_SOURCE_PX,
    );
    let left = clipped.left();
    let right_top = right_top.min(clipped.right());
    let right_bottom = right_bottom.min(clipped.right());
    if right_top <= left || right_bottom <= left {
        return;
    }

    painter.add(Shape::convex_polygon(
        vec![
            pos2(left, clipped.top()),
            pos2(right_top, clipped.top()),
            pos2(right_bottom, clipped.bottom()),
            pos2(left, clipped.bottom()),
        ],
        Color32::BLACK,
        Stroke::NONE,
    ));
}

fn paint_holder_right_frame_mask(painter: &Painter, photo: Rect, body: &TextureHandle) {
    // Repaint the photographed sloping frame as one textured trapezoid. The
    // old 109-band reconstruction could expose one-pixel seams that appeared
    // as tiny lines extending into the calculator shell.
    let source_points = [
        (HOLDER_RIGHT_FRAME_TOP_X, CARD_WINDOW.y0),
        (CARD_WINDOW.x1, CARD_WINDOW.y0),
        (CARD_WINDOW.x1, CARD_WINDOW.y1),
        (HOLDER_RIGHT_FRAME_BOTTOM_X, CARD_WINDOW.y1),
    ];

    let mut mesh = Mesh::with_texture(body.id());
    for (x, y) in source_points {
        mesh.vertices.push(Vertex {
            pos: pos2(
                photo.left() + x * photo.width() / PHOTO_W,
                photo.top() + y * photo.height() / PHOTO_H,
            ),
            uv: pos2(x / PHOTO_W, y / PHOTO_H),
            color: Color32::WHITE,
        });
    }
    mesh.indices.extend_from_slice(&[0, 1, 2, 0, 2, 3]);
    painter.add(mesh);
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
    fn touch_hit_rect_preserves_large_targets_and_expands_small_ones() {
        let small = Rect::from_min_size(pos2(10.0, 20.0), eframe::egui::vec2(28.0, 40.0));
        let expanded = minimum_hit_rect(small, 44.0);
        assert_eq!(expanded.center(), small.center());
        assert_eq!(expanded.size(), eframe::egui::vec2(44.0, 44.0));

        let large = Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(80.0, 50.0));
        assert_eq!(minimum_hit_rect(large, 44.0), large);
    }

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
    fn holder_backing_stops_before_photographed_right_frame() {
        let photo = Rect::from_min_size(pos2(0.0, 0.0), eframe::egui::vec2(PHOTO_W, PHOTO_H));
        let window = source_to_screen(photo, CARD_WINDOW);
        let card = holder_card_rect(window, 1.0);

        for y in [window.top(), window.center().y, window.bottom()] {
            let y_source = screen_y_to_source(photo, y);
            let frame_x = source_x_to_screen(photo, holder_frame_x_source(y_source));
            let backing_x = frame_x - HOLDER_BACKING_FRAME_GAP_SOURCE_PX;
            assert!(backing_x < frame_x);
            assert!(backing_x > card.left());
        }
    }

    #[test]
    fn holder_right_frame_is_one_linear_trapezoid() {
        assert_eq!(
            holder_frame_x_source(CARD_WINDOW.y0),
            HOLDER_RIGHT_FRAME_TOP_X
        );
        assert_eq!(
            holder_frame_x_source(CARD_WINDOW.y1),
            HOLDER_RIGHT_FRAME_BOTTOM_X
        );
        let middle = holder_frame_x_source((CARD_WINDOW.y0 + CARD_WINDOW.y1) * 0.5);
        assert!(
            (middle - (HOLDER_RIGHT_FRAME_TOP_X + HOLDER_RIGHT_FRAME_BOTTOM_X) * 0.5).abs() < 0.001
        );
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
    fn opposite_end_rotation_is_exactly_involutive_about_card_center() {
        let card = Rect::from_min_max(pos2(10.0, 20.0), pos2(110.0, 60.0));
        let point = pos2(25.0, 27.0);
        let rotated = rotate_point_180(card, point);
        assert_eq!(rotated, pos2(95.0, 53.0));
        assert_eq!(rotate_point_180(card, rotated), point);
    }

    #[test]
    fn opposite_end_rotation_moves_top_marks_to_mirrored_bottom_positions() {
        let card = Rect::from_min_max(pos2(0.0, 0.0), pos2(100.0, 20.0));
        let mark = Rect::from_min_max(pos2(9.0, 0.0), pos2(11.0, 2.0));
        let rotated = rotate_rect_180(card, mark);
        assert_eq!(rotated.min, pos2(89.0, 18.0));
        assert_eq!(rotated.max, pos2(91.0, 20.0));
    }

    #[test]
    fn waiting_reader_hitbox_covers_reader_and_visible_card() {
        let photo = Rect::from_min_max(pos2(0.0, 0.0), pos2(PHOTO_W, PHOTO_H));
        let clip = photo;
        let scale = 1.0;
        let hit = waiting_reader_hit_rect(photo, scale, clip);
        let reader = source_to_screen(photo, CARD_READER_HIT);
        let card = reader_motion_rect(photo, 0.0, scale).intersect(clip);

        assert!(hit.contains(reader.center()));
        assert!(hit.contains(card.center()));
    }

    #[test]
    fn card_phase_animation_only_covers_motion_states() {
        assert!(!ProgramCardPhase::Idle.is_animating());
        assert!(!ProgramCardPhase::WaitingAtReader.is_animating());
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
