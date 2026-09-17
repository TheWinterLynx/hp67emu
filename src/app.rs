use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::{
    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State, UiEvent},
    panel::Hp67Panel,
    ui::{sliders, top_keys},
};

pub struct Hp67App {
    state: Hp67State,
    photo: TextureHandle,
    live_machine: Option<Hp67LiveMachine>,
    last_live_tick: Option<Instant>,
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

        let live_machine = match Hp67LiveMachine::power_on_default() {
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
            last_live_tick: None,
        }
    }
}

impl eframe::App for Hp67App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        let elapsed = self
            .last_live_tick
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
            });

        let mut live_error = None;
        if self.state.power_on {
            if let Some(machine) = self.live_machine.as_mut() {
                if let Err(error) = machine.advance(elapsed) {
                    live_error = Some(error);
                }
            }
        }
        if let Some(error) = live_error {
            eprintln!("HP-67 live display disabled: {error}");
            self.live_machine = None;
        }

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(17, 18, 16)))
            .show(ctx, |ui| {
                let host = ui.available_rect_before_wrap();
                let display = self
                    .live_machine
                    .as_ref()
                    .map_or(HardwareDisplayFrame::BLANK, Hp67LiveMachine::display_frame);
                for event in Hp67Panel::show(ui, &self.state, &display, &self.photo) {
                    let was_power_on = self.state.power_on;
                    let key_contact = match event {
                        UiEvent::KeyContact(action) => Some(action),
                        _ => None,
                    };
                    self.state.handle(event);

                    if let Some(action) = key_contact {
                        if let Some(machine) = self.live_machine.as_mut() {
                            machine.set_key_contact(action);
                        }
                    }

                    if matches!(event, UiEvent::TogglePower) {
                        self.last_live_tick = None;
                        if !was_power_on && self.state.power_on {
                            let reset_error = self
                                .live_machine
                                .as_mut()
                                .and_then(|machine| machine.reset_power_on().err());
                            if let Some(error) = reset_error {
                                eprintln!("HP-67 live power-on reset failed: {error}");
                                self.live_machine = None;
                            }
                        }
                    }
                }
                top_keys::paint(ui, host, &self.photo);
                sliders::paint(ui, host, &self.photo, &self.state);
            });

        if self.state.power_on && self.live_machine.is_some() {
            // Once the no-key wait loop is reached the real calculator does not
            // halt: firmware keeps polling the keyboard and refreshing display
            // hardware. Keep advancing on approximately one display-refresh
            // cadence so a held physical contact can be sampled by ACT.
            ctx.request_repaint_after(Duration::from_micros(4_800));
        }
        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}
