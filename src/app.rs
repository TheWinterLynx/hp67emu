use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::{
    hp67::Hp67State,
    panel::Hp67Panel,
    ui::{sliders, top_keys},
};

pub struct Hp67App {
    state: Hp67State,
    photo: TextureHandle,
}

impl Hp67App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(17, 18, 16);
        visuals.window_fill = Color32::from_rgb(17, 18, 16);
        cc.egui_ctx.set_visuals(visuals);

        let decoded = image::load_from_memory_with_format(
            include_bytes!("../hp67.png"),
            image::ImageFormat::Png,
        )
        .expect("embedded hp67.png must be a valid PNG")
        .to_rgba8();
        let size = [decoded.width() as usize, decoded.height() as usize];
        let color = ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
        let photo = cc
            .egui_ctx
            .load_texture("hp67-photorealistic-body", color, TextureOptions::LINEAR);

        Self {
            state: Hp67State::default(),
            photo,
        }
    }
}

impl eframe::App for Hp67App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(17, 18, 16)))
            .show(ctx, |ui| {
                let host = ui.available_rect_before_wrap();
                for event in Hp67Panel::show(ui, &self.state, &self.photo) {
                    self.state.handle(event);
                }
                // The photographed A-E wells are deeper than the other key rows.
                // Correct only those five keys after the generic key pass, leaving
                // the already-good animation of every other key untouched.
                top_keys::paint(ui, host, &self.photo);
                sliders::paint(ui, host, &self.photo, &self.state);
            });

        // Keep mouse-down motion responsive even on platforms that throttle
        // otherwise-idle windows. egui's animation system schedules the release
        // frames after the pointer comes up.
        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}
