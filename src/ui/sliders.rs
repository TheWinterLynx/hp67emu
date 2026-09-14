use eframe::egui::{pos2, vec2, Color32, Rect, TextureHandle, Ui};

use crate::hp67::{Hp67State, RunMode};

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
struct PhotoSlider {
    id: &'static str,
    clip: PxRect,
    right_knob: PxRect,
    erase_right: PxRect,
    clean_track_sample: PxRect,
    travel_px: f32,
}

// hp67.png was photographed with both switches in their right-hand positions
// (ON and RUN). Only the ribbed actuator moves. The vacated right side is rebuilt
// from a narrow, clean piece of the real empty slot and stretched horizontally;
// this avoids copying the small molded/reflection feature that previously looked
// like a plastic tab next to the moving actuator.
const POWER: PhotoSlider = PhotoSlider {
    id: "power",
    clip: PxRect::new(222.0, 286.0, 330.0, 321.0),
    right_knob: PxRect::new(284.0, 290.0, 327.0, 316.0),
    erase_right: PxRect::new(274.0, 286.0, 329.0, 321.0),
    clean_track_sample: PxRect::new(225.0, 286.0, 235.0, 321.0),
    travel_px: 52.0,
};

const MODE: PhotoSlider = PhotoSlider {
    id: "mode",
    // Stop before the RUN legend: the previous 706px erase reached into the R.
    clip: PxRect::new(593.0, 286.0, 702.0, 321.0),
    right_knob: PxRect::new(655.0, 290.0, 698.0, 316.0),
    erase_right: PxRect::new(645.0, 286.0, 700.0, 321.0),
    clean_track_sample: PxRect::new(596.0, 286.0, 606.0, 321.0),
    travel_px: 52.0,
};

pub fn paint(ui: &Ui, host: Rect, photo: &TextureHandle, state: &Hp67State) {
    if host.width() <= 0.0 || host.height() <= 0.0 {
        return;
    }

    let photo_rect = fit_photo(host);
    let painter = ui.painter_at(host).with_clip_rect(photo_rect);

    let power_right = ui.ctx().animate_bool_with_time(
        ui.make_persistent_id(("photo-slider-position", POWER.id)),
        state.power_on,
        0.095,
    );
    paint_slider(&painter, photo, photo_rect, POWER, power_right);

    let run = matches!(state.mode, RunMode::Run);
    let mode_right = ui.ctx().animate_bool_with_time(
        ui.make_persistent_id(("photo-slider-position", MODE.id)),
        run,
        0.095,
    );
    paint_slider(&painter, photo, photo_rect, MODE, mode_right);
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

fn paint_slider(
    painter: &eframe::egui::Painter,
    photo: &TextureHandle,
    photo_rect: Rect,
    slider: PhotoSlider,
    rightness: f32,
) {
    // At the photographed endpoint there is nothing to synthesize: leaving the
    // base image alone gives exact original ON/RUN pixels.
    if rightness >= 0.9995 {
        return;
    }

    let clip = source_to_screen(photo_rect, slider.clip);
    let p = painter.with_clip_rect(clip);

    // Completely remove the photographed right-position actuator and its contact
    // highlight/shadow, but never touch the OFF/ON/W/PRGM/RUN lettering. A clean
    // vertical slice of the genuine empty recess is stretched across the vacated
    // area, so there is no copied molded bump or leftover plastic fragment.
    let erase = source_to_screen(photo_rect, slider.erase_right);
    p.image(
        photo.id(),
        erase,
        source_uv(slider.clean_track_sample),
        Color32::WHITE,
    );

    let original_knob = source_to_screen(photo_rect, slider.right_knob);
    let scale = photo_rect.width() / PHOTO_W;
    let shift = slider.travel_px * scale * (1.0 - rightness.clamp(0.0, 1.0));
    let moved = original_knob.translate(vec2(-shift, 0.0));
    p.image(
        photo.id(),
        moved,
        source_uv(slider.right_knob),
        Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_endpoints_and_cleanup_stay_inside_the_tracks() {
        for slider in [POWER, MODE] {
            let left = slider.right_knob.x0 - slider.travel_px;
            let right = slider.right_knob.x1 - slider.travel_px;
            assert!(left >= slider.clip.x0);
            assert!(right <= slider.clip.x1);
            assert!(slider.right_knob.x0 >= slider.clip.x0);
            assert!(slider.right_knob.x1 <= slider.clip.x1);
            assert!(slider.erase_right.x0 <= slider.right_knob.x0);
            assert!(slider.erase_right.x1 >= slider.right_knob.x1);
            assert!(slider.erase_right.x0 >= slider.clip.x0);
            assert!(slider.erase_right.x1 <= slider.clip.x1);
            assert!(slider.clean_track_sample.x1 < slider.erase_right.x0);
        }
    }
}
