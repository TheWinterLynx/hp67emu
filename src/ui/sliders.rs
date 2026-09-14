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
    actuator: PxRect,
    clean_track: PxRect,
    travel_px: f32,
}

// hp67.png was photographed with both switches in their right-hand positions
// (ON and RUN). Only the ribbed actuator moves. The sloped dark piece visible
// beneath/left of it in the source image must not travel with the actuator; that
// was the small plastic-looking tab that appeared on the wrong side after the
// switch moved. For animated states we rebuild the slot as a clean recess, then
// draw only the ribbed piece at its new position.
const POWER: PhotoSlider = PhotoSlider {
    id: "power",
    clip: PxRect::new(224.0, 292.0, 336.0, 319.0),
    actuator: PxRect::new(286.0, 292.0, 327.0, 309.0),
    clean_track: PxRect::new(234.0, 292.0, 275.0, 309.0),
    travel_px: 52.0,
};

const MODE: PhotoSlider = PhotoSlider {
    id: "mode",
    // Ends before the RUN lettering; this can never erase the R.
    clip: PxRect::new(595.0, 292.0, 707.0, 319.0),
    actuator: PxRect::new(657.0, 292.0, 698.0, 309.0),
    clean_track: PxRect::new(605.0, 292.0, 646.0, 309.0),
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
    // At the photographed endpoint the PNG already contains the exact real
    // appearance, so leave it untouched.
    if rightness >= 0.9995 {
        return;
    }

    let slot = source_to_screen(photo_rect, slider.clip);
    let p = painter.with_clip_rect(slot);

    // Replace the whole slot interior with a clean sample from the same real
    // recess. This removes both the source actuator and the sloped/rounded piece
    // that used to remain visible as a little tab beside the moved ribbed knob.
    p.image(
        photo.id(),
        slot,
        source_uv(slider.clean_track),
        Color32::WHITE,
    );

    // The moving sprite is deliberately cropped to the ribbed actuator only.
    let actuator = source_to_screen(photo_rect, slider.actuator);
    let scale = photo_rect.width() / PHOTO_W;
    let shift = slider.travel_px * scale * (1.0 - rightness.clamp(0.0, 1.0));
    let moved = actuator.translate(vec2(-shift, 0.0));
    p.image(
        photo.id(),
        moved,
        source_uv(slider.actuator),
        Color32::WHITE,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn slider_endpoints_stay_inside_their_tracks() {
        for slider in [POWER, MODE] {
            let left = slider.actuator.x0 - slider.travel_px;
            let right = slider.actuator.x1 - slider.travel_px;
            assert!(left >= slider.clip.x0);
            assert!(right <= slider.clip.x1);
            assert!(slider.actuator.x0 >= slider.clip.x0);
            assert!(slider.actuator.x1 <= slider.clip.x1);
            assert!(slider.actuator.y0 >= slider.clip.y0);
            assert!(slider.actuator.y1 <= slider.clip.y1);
        }
    }

    #[test]
    fn clean_track_matches_actuator_source_size() {
        for slider in [POWER, MODE] {
            assert_eq!(
                (slider.clean_track.x1 - slider.clean_track.x0) as i32,
                (slider.actuator.x1 - slider.actuator.x0) as i32
            );
            assert_eq!(
                (slider.clean_track.y1 - slider.clean_track.y0) as i32,
                (slider.actuator.y1 - slider.actuator.y0) as i32
            );
        }
    }
}
