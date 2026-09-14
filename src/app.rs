use eframe::egui::{self, Color32};

use crate::{
    hp67::Hp67State,
    panel::Hp67Panel,
    ui::{
        geometry::{DESIGN_H, DESIGN_W},
        photo_fx,
    },
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

        photo_fx::install(cc);

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
                let host = ui.available_rect_before_wrap();
                let scale = (host.width() / DESIGN_W).min(host.height() / DESIGN_H);
                let panel_rect = egui::Rect::from_center_size(
                    host.center(),
                    egui::vec2(DESIGN_W * scale, DESIGN_H * scale),
                );

                for event in Hp67Panel::show(ui, &self.state) {
                    self.state.handle(event);
                }

                // Paint last: the GPU layer is an optical/material treatment over
                // the existing vector panel, never a replacement for its geometry.
                photo_fx::paint(ui, panel_rect);
            });

        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}
