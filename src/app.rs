use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::{
    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State},
    panel::Hp67Panel,
    ui::{sliders, top_keys},
};

pub struct Hp67App {
    state: Hp67State,
    photo: TextureHandle,
    live_machine: Option<Hp67LiveMachine>,
}

impl Hp67App {
    pub fn new(cc: &eframe::CreationContext<'_>) -> Self {
        let mut visuals = egui::Visuals::dark();
        visuals.panel_fill = Color32::from_rgb(17, 18, 16);
        visuals.window_fill = Color32::from_rgb(17, 18, 16);
        cc.egui_ctx.set_visuals(visuals);

        let decoded = image::load_from_memory_with_format(
            include_bytes!("../assets/hp67.png"),
            image::ImageFormat::Png,
        )
        .expect("embedded assets/hp67.png must be a valid PNG")
        .to_rgba8();
        let size = [decoded.width() as usize, decoded.height() as usize];
        let color = ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
        let photo =
            cc.egui_ctx
                .load_texture("hp67-photorealistic-body", color, TextureOptions::LINEAR);

        let live_machine = match Hp67LiveMachine::boot_default() {
            Ok(machine) => Some(machine),
            Err(error) => {
                eprintln!("HP-67 live display disabled: {error}");
                None
            }
        };

        Self {
            state: Hp67State::default(),
            photo,
            live_machine,
        }
    }
}

impl eframe::App for Hp67App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(17, 18, 16)))
            .show(ctx, |ui| {
                let host = ui.available_rect_before_wrap();
                let display = self
                    .live_machine
                    .as_ref()
                    .map_or(HardwareDisplayFrame::BLANK, Hp67LiveMachine::display_frame);
                for event in Hp67Panel::show(ui, &self.state, &display, &self.photo) {
                    self.state.handle(event);
                }
                top_keys::paint(ui, host, &self.photo);
                sliders::paint(ui, host, &self.photo, &self.state);
            });

        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}
