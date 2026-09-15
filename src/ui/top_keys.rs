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
    cap: PxRect,
}

const TOP_KEYS: &[TopKey] = &[
    TopKey {
        id: "a",
        hit: PxRect::new(166.0, 467.0, 256.0, 550.0),
        cap: PxRect::new(171.0, 472.0, 251.0, 545.0),
    },
    TopKey {
        id: "b",
        hit: PxRect::new(293.0, 467.0, 383.0, 550.0),
        cap: PxRect::new(298.0, 472.0, 378.0, 545.0),
    },
    TopKey {
        id: "c",
        hit: PxRect::new(420.0, 467.0, 510.0, 550.0),
        cap: PxRect::new(425.0, 472.0, 505.0, 545.0),
    },
    TopKey {
        id: "d",
        hit: PxRect::new(547.0, 467.0, 637.0, 550.0),
        cap: PxRect::new(552.0, 472.0, 632.0, 545.0),
    },
    TopKey {
        id: "e",
        hit: PxRect::new(674.0, 467.0, 764.0, 550.0),
        cap: PxRect::new(679.0, 472.0, 759.0, 545.0),
    },
];

/// Correct only the exposed strip above A-E while leaving the generic key
/// animation completely intact. The previous correction restored the original
/// photographed key before drawing the moved cap, so an olive strip from the
/// unpressed key remained visible above it. Here the A-E row keeps exactly the
/// same 3.8 px travel, timing, shading and shadow as every other key; we only
/// replace the newly exposed strip with pixels from the real black key well.
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

        let scale = photo_rect.height() / PHOTO_H;
        let travel = 3.8 * scale * press;
        let cap = source_to_screen(photo_rect, key.cap);

        // Sample only the dark recess immediately above the olive cap. This is
        // the actual well texture from hp67.png, not a flat procedural fill.
        let recess_src = PxRect::new(
            key.cap.x0,
            (key.cap.y0 - 10.0).max(0.0),
            key.cap.x1,
            (key.cap.y0 - 4.0).max(0.0),
        );
        let gap = Rect::from_min_max(
            cap.min,
            pos2(
                cap.right(),
                (cap.top() + travel + 0.35 * scale).min(cap.bottom()),
            ),
        );
        painter.image(photo.id(), gap, source_uv(recess_src), Color32::WHITE);
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
