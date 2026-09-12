use eframe::egui::{self, Color32};

use crate::{
    hp67::Hp67State,
    key_depth_hp67_v2::KeyDepthOverlay,
    panel::Hp67Panel,
};

pub struct Hp67App {
    state: Hp67State,
}

impl Hp67App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(17, 18, 16);
        visuals.window_fill = Color32::from_rgb(17, 18, 16);
        cc.egui_ctx.set_visuals(visuals);

        Self {
            state: Hp67State::default(),
        }
    }
}

impl eframe::App for Hp67App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(17, 18, 16)))
            .show(ctx, |ui| {
                let host_rect = ui.available_rect_before_wrap();

                for event in Hp67Panel::show(ui, &self.state) {
                    self.state.handle(event);
                }

                KeyDepthOverlay::paint(ui, host_rect);
            });

        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}