use std::time::{Duration, Instant};

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};

use crate::{
    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State, KeyAction, RunMode, UiEvent},
    panel::Hp67Panel,
    ui::{
        program_card::{ProgramCardPhase, ProgramCardView, MOON_ROCKET_LANDER_CARD},
        sliders, top_keys,
    },
};

const PROGRAM_CARD_READ_DURATION: Duration = Duration::from_millis(900);
const PROGRAM_CARD_WINDOW_INSERT_DURATION: Duration = Duration::from_millis(700);

pub struct Hp67App {
    state: Hp67State,
    photo: TextureHandle,
    card_logo: TextureHandle,
    live_machine: Option<Hp67LiveMachine>,
    last_live_tick: Option<Instant>,
    card_phase: ProgramCardPhase,
    card_phase_started: Option<Instant>,
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

        let card_logo_decoded = image::load_from_memory_with_format(
            include_bytes!("../assets/hp67-card-logo.png"),
            image::ImageFormat::Png,
        )
        .expect("embedded assets/hp67-card-logo.png must be a valid PNG")
        .to_rgba8();
        let card_logo_size = [
            card_logo_decoded.width() as usize,
            card_logo_decoded.height() as usize,
        ];
        let card_logo_color =
            ColorImage::from_rgba_unmultiplied(card_logo_size, card_logo_decoded.as_raw());
        let card_logo = cc.egui_ctx.load_texture(
            "hp67-program-card-logo",
            card_logo_color,
            TextureOptions::LINEAR,
        );

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
            card_logo,
            live_machine,
            last_live_tick: None,
            card_phase: ProgramCardPhase::Idle,
            card_phase_started: None,
        }
    }
}

impl eframe::App for Hp67App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();

        match self.card_phase {
            ProgramCardPhase::ReadingFromRight
                if self.card_phase_started.is_some_and(|started| {
                    now.saturating_duration_since(started) >= PROGRAM_CARD_READ_DURATION
                }) =>
            {
                self.card_phase = ProgramCardPhase::ParkedLeft;
                self.card_phase_started = None;
            }
            ProgramCardPhase::InsertingWindowFromRight
                if self.card_phase_started.is_some_and(|started| {
                    now.saturating_duration_since(started) >= PROGRAM_CARD_WINDOW_INSERT_DURATION
                }) =>
            {
                self.card_phase = ProgramCardPhase::InWindow;
                self.card_phase_started = None;
            }
            _ => {}
        }

        let card_phase_progress = match (self.card_phase, self.card_phase_started) {
            (ProgramCardPhase::ReadingFromRight, Some(started)) => {
                (now.saturating_duration_since(started).as_secs_f32()
                    / PROGRAM_CARD_READ_DURATION.as_secs_f32())
                .clamp(0.0, 1.0)
            }
            (ProgramCardPhase::InsertingWindowFromRight, Some(started)) => {
                (now.saturating_duration_since(started).as_secs_f32()
                    / PROGRAM_CARD_WINDOW_INSERT_DURATION.as_secs_f32())
                .clamp(0.0, 1.0)
            }
            _ => 0.0,
        };

        egui::CentralPanel::default()
            .frame(egui::Frame::none().fill(Color32::from_rgb(17, 18, 16)))
            .show(ctx, |ui| {
                let host = ui.available_rect_before_wrap();
                let display = self
                    .live_machine
                    .as_ref()
                    .map_or(HardwareDisplayFrame::BLANK, Hp67LiveMachine::display_frame);
                let panel = Hp67Panel::show(
                    ui,
                    &self.state,
                    &display,
                    &self.photo,
                    ProgramCardView {
                        artwork: &MOON_ROCKET_LANDER_CARD,
                        logo: &self.card_logo,
                        phase: self.card_phase,
                        phase_progress: card_phase_progress,
                    },
                );
                if panel.card_reader_clicked {
                    self.card_phase = ProgramCardPhase::ReadingFromRight;
                    self.card_phase_started = Some(now);
                }
                if panel.card_parked_left_clicked {
                    self.card_phase = ProgramCardPhase::InsertingWindowFromRight;
                    self.card_phase_started = Some(now);
                }
                if panel.card_window_clicked {
                    self.card_phase = ProgramCardPhase::Idle;
                    self.card_phase_started = None;
                }

                for event in panel.events {
                    let was_power_on = self.state.power_on;
                    self.state.handle(event);
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

                let mode_error = self.live_machine.as_mut().and_then(|machine| {
                    machine
                        .set_program_mode(matches!(self.state.mode, RunMode::Program))
                        .err()
                });
                if let Some(error) = mode_error {
                    eprintln!("HP-67 live program-mode update failed: {error}");
                    self.live_machine = None;
                }

                if let Some(machine) = self.live_machine.as_mut() {
                    let contact = if self.state.power_on {
                        panel.key_contact.and_then(KeyAction::physical_key)
                    } else {
                        None
                    };
                    machine.set_key_contact(contact);
                }

                top_keys::paint(ui, host, &self.photo);
                sliders::paint(ui, host, &self.photo, &self.state);
            });

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
            eprintln!("HP-67 live machine disabled: {error}");
            self.live_machine = None;
        }

        if self.card_phase.is_animating() {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if self.state.power_on {
            // Keep the physical firmware machine advancing after boot idle as well
            // as during startup. The same cadence also samples held/released key
            // contacts without introducing host-side calculator semantics.
            ctx.request_repaint_after(Duration::from_micros(4_800));
        }
        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}
