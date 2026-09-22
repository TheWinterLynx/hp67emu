use std::{fs, path::Path, time::Duration};

use web_time::Instant;

use eframe::egui::{self, Color32, ColorImage, TextureHandle, TextureOptions};
use hp67emu::machines::hp67::{
    CardInsertionEnd, Hp67MagneticCard, TeenixHppImport, HP67_CARD_RECORDS_PER_TRACK,
};

use crate::{
    hp67::{HardwareDisplayFrame, Hp67LiveMachine, Hp67State, KeyAction, RunMode, UiEvent},
    panel::Hp67Panel,
    program_library::PROGRAM_LIBRARY,
    ui::{
        program_card::{
            ProgramCardPhase, ProgramCardView, GENERIC_MAGNETIC_CARD, MOON_ROCKET_LANDER_CARD,
        },
        sliders, top_keys,
    },
};

const PROGRAM_CARD_WINDOW_INSERT_DURATION: Duration = Duration::from_millis(700);
#[cfg(not(target_arch = "wasm32"))]
const PROGRAM_LIBRARY_VIEWPORT_KEY: &str = "hp67-program-library-viewport";
#[cfg(not(target_arch = "wasm32"))]
const CARD_SAVE_VIEWPORT_KEY: &str = "hp67-card-save-viewport";
const CARD_ARTWORK_ATLAS_WIDTH: u32 = 499;
const CARD_ARTWORK_ATLAS_ROW_HEIGHT: u32 = 80;
const CARD_ARTWORK_ATLAS_ROWS: u32 = 35;

pub struct Hp67App {
    state: Hp67State,
    photo: TextureHandle,
    card_logo: TextureHandle,
    card_artwork_atlas: Option<image::RgbaImage>,
    live_machine: Option<Hp67LiveMachine>,
    card_media: Option<Hp67MagneticCard>,
    card_import_name: Option<String>,
    card_insertion_end: CardInsertionEnd,
    last_live_tick: Option<Instant>,
    card_phase: ProgramCardPhase,
    card_phase_started: Option<Instant>,
    card_read_progress: f32,
    card_save_dialog_open: bool,
    card_save_path: String,
    card_save_status: Option<String>,
    program_library_open: bool,
    program_library_selected: Option<usize>,
    program_library_status: Option<String>,
    card_library_entry: Option<usize>,
    card_artwork_texture: Option<TextureHandle>,
}

fn decode_embedded_card_artwork_atlas() -> Result<image::RgbaImage, String> {
    let atlas = image::load_from_memory_with_format(
        include_bytes!("../assets/hp67-card-artwork-atlas.png"),
        image::ImageFormat::Png,
    )
    .map_err(|error| format!("invalid embedded PNG: {error}"))?
    .to_rgba8();

    let expected = (
        CARD_ARTWORK_ATLAS_WIDTH,
        CARD_ARTWORK_ATLAS_ROW_HEIGHT * CARD_ARTWORK_ATLAS_ROWS,
    );
    if atlas.dimensions() != expected {
        return Err(format!(
            "unexpected dimensions {:?}; expected {expected:?}",
            atlas.dimensions()
        ));
    }

    Ok(atlas)
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

        let card_artwork_atlas = match decode_embedded_card_artwork_atlas() {
            Ok(atlas) => Some(atlas),
            Err(error) => {
                eprintln!("HP-67 card artwork atlas disabled: {error}");
                None
            }
        };

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
            card_artwork_atlas,
            live_machine,
            card_media: None,
            card_import_name: None,
            card_insertion_end: CardInsertionEnd::End1,
            last_live_tick: None,
            card_phase: ProgramCardPhase::Idle,
            card_phase_started: None,
            card_read_progress: 0.0,
            card_save_dialog_open: false,
            card_save_path: "hp67-card.hp67card".to_owned(),
            card_save_status: None,
            program_library_open: false,
            program_library_selected: None,
            program_library_status: None,
            card_library_entry: None,
            card_artwork_texture: None,
        }
    }
}

impl Hp67App {
    fn import_dropped_card_files(&mut self, ctx: &egui::Context) {
        let paths = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect::<Vec<_>>()
        });

        for path in paths {
            if let Err(error) = self.load_card_file(&path) {
                eprintln!("HP-67 card import failed for {}: {error}", path.display());
            }
        }
    }

    fn load_card_file(&mut self, path: &Path) -> Result<(), String> {
        if self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted)
        {
            return Err("cannot replace card media while a card is inside the reader".to_owned());
        }

        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let bytes =
            fs::read(path).map_err(|error| format!("cannot read {}: {error}", path.display()))?;

        match extension.as_str() {
            "hpp" => {
                let imported = TeenixHppImport::from_bytes(&bytes)
                    .map_err(|error| format!("invalid Teenix .hpp: {error:?}"))?;
                let card_track = imported.card_track;
                let card_name = imported.card_name.clone();
                let length_matches = imported.length_matches;

                if let Some(filename_track) = filename_track_hint(path) {
                    if filename_track != card_track {
                        eprintln!(
                            "Teenix filename hint {:?} disagrees with authoritative header {:?} for {}",
                            filename_track,
                            card_track,
                            path.display()
                        );
                    }
                }

                let same_physical_card = self
                    .card_import_name
                    .as_deref()
                    .is_some_and(|name| name == card_name);
                let mut card = if same_physical_card {
                    self.card_media.take().unwrap_or_default()
                } else {
                    Hp67MagneticCard::default()
                };
                card.set_track(card_track, imported.track);
                self.card_media = Some(card);
                self.card_import_name = Some(card_name.clone());
                self.card_insertion_end = CardInsertionEnd::for_track(card_track);
                self.card_phase = ProgramCardPhase::Idle;
                self.card_phase_started = None;
                self.card_read_progress = 0.0;

                if !length_matches {
                    eprintln!(
                        "Teenix .hpp declared text length differs from decoded content for {}",
                        path.display()
                    );
                }
                eprintln!(
                    "Loaded Teenix HP-{} card '{}' into {:?}",
                    imported.calculator_id, card_name, card_track
                );
            }
            "hp67raw" => {
                self.card_media = Some(
                    Hp67MagneticCard::from_hp67raw_bytes(&bytes)
                        .map_err(|error| format!("invalid .hp67raw: {error:?}"))?,
                );
                self.card_import_name = None;
                self.card_insertion_end = CardInsertionEnd::End1;
                self.card_phase = ProgramCardPhase::Idle;
                self.card_phase_started = None;
                self.card_read_progress = 0.0;
            }
            "hp67card" => {
                self.card_media = Some(
                    Hp67MagneticCard::from_hp67card_bytes(&bytes)
                        .map_err(|error| format!("invalid .hp67card: {error:?}"))?,
                );
                self.card_import_name = None;
                self.card_insertion_end = CardInsertionEnd::End1;
                self.card_phase = ProgramCardPhase::Idle;
                self.card_phase_started = None;
                self.card_read_progress = 0.0;
            }
            _ => {
                return Err(format!(
                    "unsupported card file extension '{}'; expected .hpp, .hp67raw or .hp67card",
                    extension
                ));
            }
        }

        self.card_library_entry = None;
        self.card_artwork_texture = None;
        self.card_save_path = path
            .with_extension("hp67card")
            .to_string_lossy()
            .into_owned();
        self.card_save_status = None;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn save_card_file(&mut self, path: &Path) -> Result<(), String> {
        if self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted)
        {
            return Err("cannot save a magnetic card while it is inside the reader".to_owned());
        }

        let card = self
            .card_media
            .as_ref()
            .ok_or_else(|| "no magnetic card is available to save".to_owned())?;
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase();
        let bytes = serialize_card_for_extension(card, &extension)?;
        fs::write(path, bytes)
            .map_err(|error| format!("cannot write {}: {error}", path.display()))?;

        if let Some(card) = self.card_media.as_mut() {
            card.mark_clean();
        }
        Ok(())
    }

    fn load_program_library_entry(
        &mut self,
        ctx: &egui::Context,
        index: usize,
    ) -> Result<(), String> {
        if self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted)
        {
            return Err("cannot replace card media while a card is inside the reader".to_owned());
        }

        let entry = PROGRAM_LIBRARY
            .get(index)
            .ok_or_else(|| format!("program library index {index} is out of range"))?;
        let loaded = entry.load_card()?;
        let artwork_texture = if let (Some(row), Some(atlas)) =
            (entry.artwork_atlas_row(), self.card_artwork_atlas.as_ref())
        {
            let y = row as u32 * CARD_ARTWORK_ATLAS_ROW_HEIGHT;
            let decoded = image::imageops::crop_imm(
                atlas,
                0,
                y,
                CARD_ARTWORK_ATLAS_WIDTH,
                CARD_ARTWORK_ATLAS_ROW_HEIGHT,
            )
            .to_image();
            let size = [decoded.width() as usize, decoded.height() as usize];
            let image = ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
            Some(ctx.load_texture(
                format!("hp67-program-card-artwork-{index}"),
                image,
                TextureOptions::LINEAR,
            ))
        } else {
            match entry.artwork_path {
                Some(path) if Path::new(path).is_file() => {
                    let bytes = fs::read(path)
                        .map_err(|error| format!("cannot read artwork {}: {error}", path))?;
                    let decoded =
                        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                            .map_err(|error| format!("invalid artwork PNG {}: {error}", path))?
                            .to_rgba8();
                    let size = [decoded.width() as usize, decoded.height() as usize];
                    let image = ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
                    Some(ctx.load_texture(
                        format!("hp67-program-card-artwork-{index}"),
                        image,
                        TextureOptions::LINEAR,
                    ))
                }
                _ => None,
            }
        };

        self.card_media = Some(loaded.card);
        self.card_import_name = Some(loaded.card_name);
        self.card_library_entry = Some(index);
        self.card_artwork_texture = artwork_texture;
        self.card_insertion_end = CardInsertionEnd::End1;
        self.card_phase = ProgramCardPhase::Idle;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        self.card_save_path = format!("{} {}.hp67card", entry.reference, entry.title);
        self.card_save_status = None;
        Ok(())
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn show_program_library(&mut self, ctx: &egui::Context) {
        if !self.program_library_open {
            return;
        }

        let mut keep_open = true;
        let mut selected = self.program_library_selected;
        let mut load_requested = None;
        let status = self.program_library_status.clone();
        let reader_free = !self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted);

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of(PROGRAM_LIBRARY_VIEWPORT_KEY),
            egui::ViewportBuilder::default()
                .with_title("HP-67 Program Card Library")
                .with_inner_size([960.0, 680.0])
                .with_min_inner_size([720.0, 480.0])
                .with_resizable(true),
            |ctx, _class| {
                if ctx.input(|input| input.viewport().close_requested()) {
                    keep_open = false;
                    return;
                }

                egui::TopBottomPanel::top("hp67-program-library-header").show(ctx, |ui| {
                    ui.label(format!(
                        "{} programs from checked-in magnetic-card images",
                        PROGRAM_LIBRARY.len()
                    ));
                });

                egui::TopBottomPanel::bottom("hp67-program-library-actions").show(ctx, |ui| {
                    if let Some(index) = selected {
                        let entry = &PROGRAM_LIBRARY[index];
                        ui.horizontal(|ui| {
                            if ui
                                .add_enabled(reader_free, egui::Button::new("Load card"))
                                .clicked()
                            {
                                load_requested = Some(index);
                            }
                            if !reader_free {
                                ui.label(
                                    "Remove the card from the reader before loading another one.",
                                );
                            } else {
                                ui.label(format!("{} - {}", entry.reference, entry.title));
                            }
                        });
                    }

                    if let Some(status) = status.as_deref() {
                        ui.separator();
                        ui.label(status);
                    }
                });

                egui::SidePanel::left("hp67-program-library-packs")
                    .default_width(330.0)
                    .min_width(240.0)
                    .max_width(480.0)
                    .resizable(true)
                    .show(ctx, |ui| {
                        ui.strong("Program packs");
                        ui.separator();

                        egui::ScrollArea::vertical()
                            .auto_shrink([false, false])
                            .show(ui, |ui| {
                                let mut pack_start = 0usize;
                                while pack_start < PROGRAM_LIBRARY.len() {
                                    let pack = PROGRAM_LIBRARY[pack_start].pack;
                                    let mut pack_end = pack_start + 1;
                                    while pack_end < PROGRAM_LIBRARY.len()
                                        && PROGRAM_LIBRARY[pack_end].pack == pack
                                    {
                                        pack_end += 1;
                                    }

                                    egui::CollapsingHeader::new(format!(
                                        "{} ({})",
                                        pack,
                                        pack_end - pack_start
                                    ))
                                    .id_source(("hp67-program-pack", pack))
                                    .default_open(false)
                                    .show(ui, |ui| {
                                        for (index, entry) in
                                            PROGRAM_LIBRARY[pack_start..pack_end].iter().enumerate()
                                        {
                                            let index = pack_start + index;
                                            let label = if entry.reference.is_empty() {
                                                entry.title.to_owned()
                                            } else {
                                                format!("{} - {}", entry.reference, entry.title)
                                            };
                                            if ui
                                                .selectable_label(selected == Some(index), label)
                                                .clicked()
                                            {
                                                selected = Some(index);
                                            }
                                        }
                                    });

                                    pack_start = pack_end;
                                }
                            });
                    });

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.strong("Program listing");
                    ui.add_space(4.0);

                    if let Some(index) = selected {
                        let entry = &PROGRAM_LIBRARY[index];
                        ui.heading(entry.title);
                        if !entry.reference.is_empty() {
                            ui.label(format!("Reference: {}", entry.reference));
                        }
                        ui.label(format!("Pack: {}", entry.pack));
                        ui.label(format!("Magnetic tracks supplied: {}", entry.track_count()));
                        if let Some(pdf) = entry.source_pdf {
                            let source = if entry.artwork_atlas_row().is_some() {
                                "embedded card crop"
                            } else {
                                "manual only"
                            };
                            ui.label(format!("Artwork/manual source: {pdf} ({source})"));
                        } else {
                            ui.label("Artwork/manual source: no pack PDF checked in");
                        }
                        ui.separator();

                        match entry.program_listing() {
                            Ok(listing) => {
                                egui::ScrollArea::both()
                                    .id_source("hp67-program-listing-scroll")
                                    .auto_shrink([false, false])
                                    .show(ui, |ui| {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(listing).monospace(),
                                            )
                                            .wrap(false),
                                        );
                                    });
                            }
                            Err(error) => {
                                ui.label(format!("Cannot decode listing: {error}"));
                            }
                        }
                    } else {
                        ui.label("Select a program from a pack to view its listing.");
                    }
                });
            },
        );

        self.program_library_open = keep_open;
        self.program_library_selected = selected;

        if let Some(index) = load_requested {
            self.program_library_status = match self.load_program_library_entry(ctx, index) {
                Ok(()) => {
                    let entry = &PROGRAM_LIBRARY[index];
                    Some(format!("Loaded {} - {}", entry.reference, entry.title))
                }
                Err(error) => Some(error),
            };
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn show_program_library(&mut self, ctx: &egui::Context) {
        if !self.program_library_open {
            return;
        }

        let mut keep_open = true;
        let mut close_requested = false;
        let mut selected = self.program_library_selected;
        let mut load_requested = None;
        let status = self.program_library_status.clone();
        let reader_free = !self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted);

        egui::Window::new("HP-67 Program Card Library")
            .id(egui::Id::new("hp67-program-library-web-window"))
            .open(&mut keep_open)
            .collapsible(false)
            .resizable(true)
            .default_size([680.0, 560.0])
            .min_width(420.0)
            .min_height(360.0)
            .show(ctx, |ui| {
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "{} programs from checked-in magnetic-card images",
                        PROGRAM_LIBRARY.len()
                    ));
                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui.button("Close").clicked() {
                            close_requested = true;
                        }
                        let selected_index = selected;
                        let load_enabled = reader_free && selected_index.is_some();
                        if ui
                            .add_enabled(load_enabled, egui::Button::new("Load card"))
                            .clicked()
                        {
                            load_requested = selected_index;
                        }
                    });
                });
                ui.separator();

                ui.columns(2, |columns| {
                    columns[0].strong("Program packs");
                    columns[0].separator();
                    egui::ScrollArea::vertical()
                        .id_source("hp67-program-library-web-packs")
                        .auto_shrink([false, false])
                        .show(&mut columns[0], |ui| {
                            let mut pack_start = 0usize;
                            while pack_start < PROGRAM_LIBRARY.len() {
                                let pack = PROGRAM_LIBRARY[pack_start].pack;
                                let mut pack_end = pack_start + 1;
                                while pack_end < PROGRAM_LIBRARY.len()
                                    && PROGRAM_LIBRARY[pack_end].pack == pack
                                {
                                    pack_end += 1;
                                }

                                egui::CollapsingHeader::new(format!(
                                    "{} ({})",
                                    pack,
                                    pack_end - pack_start
                                ))
                                .id_source(("hp67-program-pack-web", pack))
                                .default_open(false)
                                .show(ui, |ui| {
                                    for (index, entry) in
                                        PROGRAM_LIBRARY[pack_start..pack_end].iter().enumerate()
                                    {
                                        let index = pack_start + index;
                                        let label = if entry.reference.is_empty() {
                                            entry.title.to_owned()
                                        } else {
                                            format!("{} - {}", entry.reference, entry.title)
                                        };
                                        if ui
                                            .selectable_label(selected == Some(index), label)
                                            .clicked()
                                        {
                                            selected = Some(index);
                                        }
                                    }
                                });

                                pack_start = pack_end;
                            }
                        });

                    columns[1].strong("Program listing");
                    columns[1].separator();
                    egui::ScrollArea::both()
                        .id_source("hp67-program-library-web-listing")
                        .auto_shrink([false, false])
                        .show(&mut columns[1], |ui| {
                            if let Some(index) = selected {
                                let entry = &PROGRAM_LIBRARY[index];
                                ui.heading(entry.title);
                                if !entry.reference.is_empty() {
                                    ui.label(format!("Reference: {}", entry.reference));
                                }
                                ui.label(format!("Pack: {}", entry.pack));
                                ui.label(format!(
                                    "Magnetic tracks supplied: {}",
                                    entry.track_count()
                                ));
                                if let Some(pdf) = entry.source_pdf {
                                    let source = if entry.artwork_atlas_row().is_some() {
                                        "embedded card crop"
                                    } else {
                                        "manual only"
                                    };
                                    ui.label(format!("Artwork/manual source: {pdf} ({source})"));
                                } else {
                                    ui.label("Artwork/manual source: no pack PDF checked in");
                                }

                                if !reader_free {
                                    ui.label(
                                        "Remove the card from the reader before loading another one.",
                                    );
                                }

                                if let Some(status) = status.as_deref() {
                                    ui.separator();
                                    ui.label(status);
                                }

                                ui.separator();
                                match entry.program_listing() {
                                    Ok(listing) => {
                                        ui.add(
                                            egui::Label::new(
                                                egui::RichText::new(listing).monospace(),
                                            )
                                            .wrap(false),
                                        );
                                    }
                                    Err(error) => {
                                        ui.label(format!("Cannot decode listing: {error}"));
                                    }
                                }
                            } else {
                                ui.label("Select a program from a pack to view its listing.");
                            }
                        });
                });
            });

        if close_requested {
            keep_open = false;
        }
        self.program_library_open = keep_open;
        self.program_library_selected = selected;

        if let Some(index) = load_requested {
            self.program_library_status = match self.load_program_library_entry(ctx, index) {
                Ok(()) => {
                    self.program_library_open = false;
                    let entry = &PROGRAM_LIBRARY[index];
                    Some(format!("Loaded {} - {}", entry.reference, entry.title))
                }
                Err(error) => Some(error),
            };
        }
    }

    #[cfg(not(target_arch = "wasm32"))]
    fn show_card_save_dialog(&mut self, ctx: &egui::Context) {
        if !self.card_save_dialog_open {
            return;
        }

        let mut keep_open = true;
        let mut save_requested = false;
        let mut save_path = self.card_save_path.clone();
        let status = self.card_save_status.clone();

        ctx.show_viewport_immediate(
            egui::ViewportId::from_hash_of(CARD_SAVE_VIEWPORT_KEY),
            egui::ViewportBuilder::default()
                .with_title("Save magnetic card")
                .with_inner_size([520.0, 150.0])
                .with_resizable(false),
            |ctx, _class| {
                if ctx.input(|input| input.viewport().close_requested()) {
                    keep_open = false;
                    return;
                }

                egui::CentralPanel::default().show(ctx, |ui| {
                    ui.label("Save as .hp67card or .hp67raw");
                    ui.text_edit_singleline(&mut save_path);
                    if let Some(status) = status.as_deref() {
                        ui.label(status);
                    }
                    if ui.button("Save").clicked() {
                        save_requested = true;
                    }
                });
            },
        );

        self.card_save_dialog_open = keep_open;
        self.card_save_path = save_path;

        if save_requested {
            let path = self.card_save_path.clone();
            match self.save_card_file(Path::new(&path)) {
                Ok(()) => {
                    self.card_save_status = Some(format!("Saved {}", Path::new(&path).display()));
                }
                Err(error) => {
                    self.card_save_status = Some(error);
                }
            }
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn show_card_save_dialog(&mut self, ctx: &egui::Context) {
        if !self.card_save_dialog_open {
            return;
        }

        let mut keep_open = true;
        let mut close_requested = false;
        egui::Window::new("Save magnetic card")
            .id(egui::Id::new("hp67-card-save-web-window"))
            .open(&mut keep_open)
            .collapsible(false)
            .resizable(false)
            .default_width(420.0)
            .show(ctx, |ui| {
                ui.label(
                    "Browser card export is not implemented yet. The native build still supports .hp67card and .hp67raw saves.",
                );
                if ui.button("Close").clicked() {
                    close_requested = true;
                }
            });

        if close_requested {
            keep_open = false;
        }
        self.card_save_dialog_open = keep_open;
    }

    fn insert_current_card(&mut self, insertion_end: CardInsertionEnd) -> bool {
        if !card_insertion_allowed(self.state.power_on) {
            return false;
        }

        let Some(card) = self.card_media.take() else {
            return false;
        };
        let restore = card.clone();

        let Some(machine) = self.live_machine.as_mut() else {
            self.card_media = Some(card);
            return false;
        };
        if machine.magnetic_card_inserted() {
            self.card_media = Some(card);
            return false;
        }

        if let Err(error) = machine.insert_magnetic_card(card, insertion_end) {
            eprintln!("HP-67 live card insertion failed: {error}");
            self.card_media = Some(restore);
            return false;
        }

        self.card_insertion_end = insertion_end;
        self.card_phase = ProgramCardPhase::WaitingAtReader;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        true
    }

    fn withdraw_waiting_card(&mut self) -> bool {
        if self.card_phase != ProgramCardPhase::WaitingAtReader {
            return false;
        }

        let Some(machine) = self.live_machine.as_mut() else {
            return false;
        };
        let card = match machine.withdraw_unstarted_magnetic_card() {
            Ok(Some(card)) => card,
            Ok(None) => return false,
            Err(error) => {
                eprintln!("HP-67 card withdrawal failed: {error}");
                return false;
            }
        };

        self.card_media = Some(card);
        self.card_phase = ProgramCardPhase::Idle;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        true
    }

    fn reset_card_presentation_after_power_off(&mut self) {
        self.card_phase = ProgramCardPhase::Idle;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        self.card_insertion_end = CardInsertionEnd::End1;
    }

    fn opposite_track_requested(&self) -> bool {
        let Some(machine) = self.live_machine.as_ref() else {
            return false;
        };
        if !machine.card_prompt_visible() {
            return false;
        }

        self.card_media.as_ref().is_some_and(|card| {
            opposite_track_can_continue(card, self.card_insertion_end, machine.card_write_mode())
        })
    }

    fn insert_new_blank_card(&mut self) -> bool {
        if !card_insertion_allowed(self.state.power_on) {
            return false;
        }

        if self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted)
        {
            return false;
        }

        self.card_media = Some(Hp67MagneticCard::default());
        self.card_import_name = None;
        self.card_library_entry = None;
        self.card_artwork_texture = None;
        self.card_insertion_end = CardInsertionEnd::End1;
        self.card_save_path = "hp67-card.hp67card".to_owned();
        self.card_save_status = None;
        if self.insert_current_card(CardInsertionEnd::End1) {
            return true;
        }

        self.card_media = None;
        false
    }
}

fn card_insertion_allowed(power_on: bool) -> bool {
    power_on
}

fn reader_free_for_new_blank(phase: ProgramCardPhase) -> bool {
    !matches!(
        phase,
        ProgramCardPhase::WaitingAtReader | ProgramCardPhase::ReadingFromRight
    )
}

fn reader_click_requires_new_blank(card_prepared: bool) -> bool {
    !card_prepared
}

#[cfg(not(target_arch = "wasm32"))]
fn serialize_card_for_extension(
    card: &Hp67MagneticCard,
    extension: &str,
) -> Result<Vec<u8>, String> {
    match extension {
        "hp67card" => Ok(card.to_hp67card_bytes().to_vec()),
        "hp67raw" => card
            .to_hp67raw_bytes()
            .map(|bytes| bytes.to_vec())
            .map_err(|error| format!("cannot export .hp67raw: {error:?}")),
        _ => Err(format!(
            "unsupported save extension '{extension}'; expected .hp67card or .hp67raw"
        )),
    }
}

fn imported_artwork_is_moon_rocket_lander(name: Option<&str>) -> bool {
    name.is_some_and(|name| name.trim().eq_ignore_ascii_case("Moon Rocket Lander"))
}

fn opposite_track_can_continue(
    card: &Hp67MagneticCard,
    current_end: CardInsertionEnd,
    write_mode: bool,
) -> bool {
    let track = card.track(current_end.opposite().track());
    if write_mode {
        !track.write_protected()
    } else {
        track.is_recorded()
    }
}

fn filename_track_hint(path: &Path) -> Option<hp67emu::machines::hp67::Hp67CardTrack> {
    use hp67emu::machines::hp67::Hp67CardTrack;

    let stem = path.file_stem()?.to_str()?;
    for token in stem.split_whitespace() {
        if token.ends_with("_1") {
            return Some(Hp67CardTrack::Track1);
        }
        if token.ends_with("_2") {
            return Some(Hp67CardTrack::Track2);
        }
    }
    None
}

impl eframe::App for Hp67App {
    fn update(&mut self, ctx: &egui::Context, _frame: &mut eframe::Frame) {
        let now = Instant::now();
        self.import_dropped_card_files(ctx);
        if ctx.input(|input| input.modifiers.command && input.key_pressed(egui::Key::S)) {
            self.card_save_dialog_open = true;
            self.card_save_status = None;
        }
        let mut open_program_library = false;
        let mut new_blank_from_menu = false;
        let mut save_from_menu = false;
        egui::TopBottomPanel::top("hp67-menu").show(ctx, |ui| {
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Cards", |ui| {
                    if ui.button("Program Library...").clicked() {
                        open_program_library = true;
                        ui.close_menu();
                    }
                    if ui.button("New blank card").clicked() {
                        new_blank_from_menu = true;
                        ui.close_menu();
                    }
                    if ui.button("Save current card...").clicked() {
                        save_from_menu = true;
                        ui.close_menu();
                    }
                });
            });
        });
        if open_program_library {
            self.program_library_open = true;
            self.program_library_status = None;
        }
        if new_blank_from_menu {
            if !self.insert_new_blank_card() {
                self.program_library_status = Some(
                    "Cannot insert a new blank card while power is off or the reader is occupied."
                        .to_owned(),
                );
            }
        }
        if save_from_menu {
            self.card_save_dialog_open = true;
            self.card_save_status = None;
        }

        match self.card_phase {
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
            (ProgramCardPhase::ReadingFromRight, _) => self.card_read_progress,
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
                let opposite_track_requested = self.opposite_track_requested();
                let card_artwork = if let Some(entry) = self
                    .card_library_entry
                    .and_then(|index| PROGRAM_LIBRARY.get(index))
                {
                    &entry.artwork
                } else if imported_artwork_is_moon_rocket_lander(self.card_import_name.as_deref()) {
                    &MOON_ROCKET_LANDER_CARD
                } else {
                    &GENERIC_MAGNETIC_CARD
                };
                let panel = Hp67Panel::show(
                    ui,
                    &self.state,
                    &display,
                    &self.photo,
                    ProgramCardView {
                        artwork: card_artwork,
                        logo: &self.card_logo,
                        face_texture: self.card_artwork_texture.as_ref(),
                        phase: self.card_phase,
                        phase_progress: card_phase_progress,
                        opposite_track_requested,
                        rotated_180: self.card_insertion_end == CardInsertionEnd::End2,
                        reader_enabled: self.state.power_on,
                        reader_free_for_new_blank: reader_free_for_new_blank(self.card_phase),
                    },
                );
                if panel.card_reader_clicked {
                    if reader_click_requires_new_blank(self.card_media.is_some()) {
                        self.insert_new_blank_card();
                    } else {
                        self.insert_current_card(self.card_insertion_end);
                    }
                }
                if panel.blank_card_requested {
                    self.insert_new_blank_card();
                }
                if panel.card_waiting_reader_clicked {
                    self.withdraw_waiting_card();
                }
                if opposite_track_requested
                    && (panel.card_parked_left_clicked || panel.card_parked_left_double_clicked)
                {
                    self.insert_current_card(self.card_insertion_end.opposite());
                } else if panel.card_parked_left_double_clicked {
                    if self.card_media.is_some() {
                        self.card_insertion_end = self.card_insertion_end.opposite();
                        self.card_phase = ProgramCardPhase::Idle;
                        self.card_phase_started = None;
                        self.card_read_progress = 0.0;
                    }
                } else if panel.card_parked_left_clicked {
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
                        if was_power_on && !self.state.power_on {
                            if let Some(machine) = self.live_machine.as_mut() {
                                match machine.take_magnetic_card_for_power_off() {
                                    Ok(Some(card)) => self.card_media = Some(card),
                                    Ok(None) => {}
                                    Err(error) => {
                                        eprintln!("HP-67 power-off card recovery failed: {error}");
                                    }
                                }
                            }
                            self.reset_card_presentation_after_power_off();
                        } else if !was_power_on && self.state.power_on {
                            let reset_result = if let Some(machine) = self.live_machine.as_mut() {
                                machine.reset_power_on()
                            } else {
                                Hp67LiveMachine::power_on_default().map(|machine| {
                                    self.live_machine = Some(machine);
                                })
                            };
                            if let Err(error) = reset_result {
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

                top_keys::paint(ui, host, &self.photo, panel.key_contact);
                sliders::paint(ui, host, &self.photo, &self.state);
            });

        let elapsed = self
            .last_live_tick
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
            });

        let mut live_error = None;
        let mut live_card_active = false;
        if self.state.power_on {
            if let Some(machine) = self.live_machine.as_mut() {
                if let Err(error) = machine.advance(elapsed) {
                    live_error = Some(error);
                } else {
                    live_card_active =
                        machine.card_motor_on() || machine.card_record_stream_active();
                    if machine.magnetic_card_inserted() {
                        if self.card_phase == ProgramCardPhase::WaitingAtReader
                            && machine.card_motor_on()
                        {
                            self.card_phase = ProgramCardPhase::ReadingFromRight;
                        }
                        self.card_read_progress = (machine.card_record_position() as f32
                            / HP67_CARD_RECORDS_PER_TRACK as f32)
                            .clamp(0.0, 1.0);
                    }
                    if machine.card_transport_complete() && self.card_media.is_none() {
                        self.card_read_progress = 1.0;
                        self.card_media = machine.take_completed_magnetic_card();
                        self.card_phase = ProgramCardPhase::ParkedLeft;
                        self.card_phase_started = None;
                    }
                }
            }
        }
        if let Some(error) = live_error {
            eprintln!("HP-67 live machine disabled: {error}");
            self.live_machine = None;
        }

        if self.card_phase.is_animating() || live_card_active {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if self.state.power_on {
            // Keep the physical firmware machine advancing after boot idle as well
            // as during startup. The same cadence also samples held/released key
            // contacts without introducing host-side calculator semantics.
            ctx.request_repaint_after(Duration::from_micros(4_800));
        }
        self.show_card_save_dialog(ctx);
        self.show_program_library(ctx);

        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn embedded_card_artwork_atlas_decodes_and_has_expected_dimensions() {
        let atlas = decode_embedded_card_artwork_atlas()
            .expect("embedded card artwork atlas must decode during the test gate");
        assert_eq!(
            atlas.dimensions(),
            (
                CARD_ARTWORK_ATLAS_WIDTH,
                CARD_ARTWORK_ATLAS_ROW_HEIGHT * CARD_ARTWORK_ATLAS_ROWS,
            )
        );

        for row in 0..CARD_ARTWORK_ATLAS_ROWS {
            let y0 = row * CARD_ARTWORK_ATLAS_ROW_HEIGHT;
            let y1 = y0 + CARD_ARTWORK_ATLAS_ROW_HEIGHT;
            for y in y0..y1 {
                for x in 0..CARD_ARTWORK_ATLAS_WIDTH {
                    assert_eq!(
                        atlas.get_pixel(x, y).0[3],
                        255,
                        "artwork atlas row {row} must be fully opaque at ({x}, {y})"
                    );
                }
            }
        }

        // SD1-01A Moving Average has two compact solid-white registration
        // blocks along its top edge. Detect compact bright runs instead of
        // hard-coding x positions so the regression follows the source crop
        // even if canonical card registration shifts by a few pixels.
        let mut bright_columns = Vec::new();
        for x in 0..CARD_ARTWORK_ATLAS_WIDTH {
            let mut bright = 0usize;
            for y in 0..8 {
                let pixel = atlas.get_pixel(x, y).0;
                if pixel[0] >= 220 && pixel[1] >= 220 && pixel[2] >= 220 {
                    bright += 1;
                }
            }
            if bright >= 2 {
                bright_columns.push(x);
            }
        }

        let mut compact_runs = Vec::new();
        if let Some(&first) = bright_columns.first() {
            let mut start = first;
            let mut previous = first;
            for &x in bright_columns.iter().skip(1) {
                if x <= previous + 1 {
                    previous = x;
                    continue;
                }
                let width = previous - start + 1;
                if (2..=12).contains(&width) {
                    compact_runs.push((start, previous));
                }
                start = x;
                previous = x;
            }
            let width = previous - start + 1;
            if (2..=12).contains(&width) {
                compact_runs.push((start, previous));
            }
        }

        assert!(
            compact_runs.len() >= 2,
            "SD1-01A top registration blocks were lost or hollowed: {compact_runs:?}"
        );
    }

    use hp67emu::machines::hp67::{Hp67CardTrack, Hp67MagneticTrack};

    #[test]
    fn new_blank_is_allowed_when_reader_is_physically_free() {
        assert!(reader_free_for_new_blank(ProgramCardPhase::Idle));
        assert!(reader_free_for_new_blank(ProgramCardPhase::ParkedLeft));
        assert!(reader_free_for_new_blank(
            ProgramCardPhase::InsertingWindowFromRight
        ));
        assert!(reader_free_for_new_blank(ProgramCardPhase::InWindow));
        assert!(!reader_free_for_new_blank(
            ProgramCardPhase::WaitingAtReader
        ));
        assert!(!reader_free_for_new_blank(
            ProgramCardPhase::ReadingFromRight
        ));
    }

    #[test]
    fn magnetic_card_insertion_requires_power_on() {
        assert!(!card_insertion_allowed(false));
        assert!(card_insertion_allowed(true));
    }

    #[test]
    fn reader_primary_click_policy_uses_blank_media_when_none_is_prepared() {
        assert!(reader_click_requires_new_blank(false));
        assert!(!reader_click_requires_new_blank(true));
    }

    #[test]
    fn native_save_serialization_preserves_blank_state_and_raw_rejects_it() {
        let blank = Hp67MagneticCard::default();
        let container = serialize_card_for_extension(&blank, "hp67card").unwrap();
        assert_eq!(
            Hp67MagneticCard::from_hp67card_bytes(&container).unwrap(),
            blank
        );
        assert!(serialize_card_for_extension(&blank, "hp67raw").is_err());
        assert!(serialize_card_for_extension(&blank, "hpp").is_err());
    }

    #[test]
    fn moon_rocket_artwork_requires_matching_imported_card_identity() {
        assert!(imported_artwork_is_moon_rocket_lander(Some(
            "Moon Rocket Lander"
        )));
        assert!(imported_artwork_is_moon_rocket_lander(Some(
            " moon rocket lander "
        )));
        assert!(!imported_artwork_is_moon_rocket_lander(None));
        assert!(!imported_artwork_is_moon_rocket_lander(Some(
            "Another Program"
        )));
    }

    #[test]
    fn opposite_track_continuation_distinguishes_read_and_write_media_requirements() {
        let blank = Hp67MagneticCard::default();
        assert!(!opposite_track_can_continue(
            &blank,
            CardInsertionEnd::End1,
            false
        ));
        assert!(opposite_track_can_continue(
            &blank,
            CardInsertionEnd::End1,
            true
        ));

        let recorded = blank.with_track(
            Hp67CardTrack::Track2,
            Hp67MagneticTrack::from_words([0; HP67_CARD_RECORDS_PER_TRACK]).unwrap(),
        );
        assert!(opposite_track_can_continue(
            &recorded,
            CardInsertionEnd::End1,
            false
        ));

        let protected = Hp67MagneticCard::default().with_track(
            Hp67CardTrack::Track2,
            Hp67MagneticTrack::default().with_write_protected(true),
        );
        assert!(!opposite_track_can_continue(
            &protected,
            CardInsertionEnd::End1,
            true
        ));
    }
}
