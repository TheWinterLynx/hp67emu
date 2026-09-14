use eframe::egui::{pos2, vec2, Color32, Rect, TextureHandle, Ui};

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
struct TopKey {
    id: &'static str,
    hit: PxRect,
    restore: PxRect,
    cap: PxRect,
}

const TOP_KEYS: &[TopKey] = &[
    TopKey {
        id: "a",
        hit: PxRect::new(166.0, 467.0, 256.0, 550.0),
        restore: PxRect::new(162.0, 464.0, 260.0, 556.0),
        cap: PxRect::new(171.0, 472.0, 251.0, 545.0),
    },
    TopKey {
        id: "b",
        hit: PxRect::new(293.0, 467.0, 383.0, 550.0),
        restore: PxRect::new(289.0, 464.0, 387.0, 556.0),
        cap: PxRect::new(298.0, 472.0, 378.0, 545.0),
    },
    TopKey {
        id: "c",
        hit: PxRect::new(420.0, 467.0, 510.0, 550.0),
        restore: PxRect::new(416.0, 464.0, 514.0, 556.0),
        cap: PxRect::new(425.0, 472.0, 505.0, 545.0),
    },
    TopKey {
        id: "d",
        hit: PxRect::new(547.0, 467.0, 637.0, 550.0),
        restore: PxRect::new(543.0, 464.0, 641.0, 556.0),
        cap: PxRect::new(552.0, 472.0, 632.0, 545.0),
    },
    TopKey {
        id: "e",
        hit: PxRect::new(674.0, 467.0, 764.0, 550.0),
        restore: PxRect::new(670.0, 464.0, 768.0, 556.0),
        cap: PxRect::new(679.0, 472.0, 759.0, 545.0),
    },
];

/// Corrects the A-E row after the generic photographic key pass.
///
/// Those five keys sit in unusually deep photographed wells. The generic crop
/// contains part of that black well, so moving the whole crop makes the recess
/// move with the key and exaggerates the apparent hole. We first restore the
/// untouched photograph, then animate only the physical olive keycap itself.
pub fn paint(ui: &Ui, host: Rect, photo: &TextureHandle) {
    if host.width() <= 0.0 || host.height() <= 0.0 {
        return;
    }

    let photo_rect = fit_photo(host);
    let painter = ui.painter_at(host).with_clip_rect(photo_rect);
    let (pointer_pos, pointer_down) = ui
        .ctx()
        .input(|i| (i.pointer.interact_pos(), i.pointer.primary_down()));

    for key in TOP_KEYS {
        let hit = source_to_screen(photo_rect, key.hit);
        let down = pointer_down && pointer_pos.is_some_and(|p| hit.contains(p));
        let press = ui.ctx().animate_bool_with_time(
            ui.make_persistent_id(("photo-top-key-correction", key.id)),
            down,
            if down { 0.040 } else { 0.075 },
        );
        if press <= 0.001 {
            continue;
        }

        // Remove the generic animation, including its shadow just below the
        // source crop, by restoring the exact original pixels in this small area.
        let restore = source_to_screen(photo_rect, key.restore);
        painter.image(photo.id(), restore, source_uv(key.restore), Color32::WHITE);

        let cap = source_to_screen(photo_rect, key.cap);
        let scale = photo_rect.height() / PHOTO_H;
        let travel = 2.6 * scale * press;

        // The top edge moves much less than the lower edge. This reads as the
        // short Classic-series key travel while avoiding a newly exposed black
        // strip above the A-E caps.
        let moved = Rect::from_min_max(
            pos2(cap.left(), cap.top() + travel * 0.22),
            pos2(cap.right(), cap.bottom() + travel),
        );
        let shade = (255.0 - 5.0 * press).round() as u8;
        painter.image(
            photo.id(),
            moved,
            source_uv(key.cap),
            Color32::from_rgb(shade, shade, shade),
        );

        // Very restrained contact shadow at the bottom; no synthetic top hole.
        painter.line_segment(
            [
                pos2(moved.left() + scale * 7.0, moved.bottom()),
                pos2(moved.right() - scale * 7.0, moved.bottom()),
            ],
            eframe::egui::Stroke::new(
                (0.9 * scale).max(0.55),
                Color32::from_rgba_unmultiplied(
                    0,
                    0,
                    0,
                    (24.0 + 24.0 * press).round() as u8,
                ),
            ),
        );
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn top_row_corrections_cover_exactly_five_keys() {
        assert_eq!(TOP_KEYS.len(), 5);
        for key in TOP_KEYS {
            assert!(key.cap.x0 > key.hit.x0);
            assert!(key.cap.x1 < key.hit.x1);
            assert!(key.cap.y0 > key.hit.y0);
            assert!(key.cap.y1 < key.hit.y1);
        }
    }
}
