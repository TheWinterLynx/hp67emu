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
        program_card::{ProgramCardPhase, ProgramCardView, GENERIC_MAGNETIC_CARD},
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
const CARD_ARTWORK_ATLAS_ROWS: u32 = 349;
const UNLIMITED_SLICE_BUDGET: Duration = Duration::from_millis(8);
const UNLIMITED_BATCH_WORDS: u32 = 64;
#[cfg(any(target_arch = "wasm32", test))]
const WEB_PROGRAM_LIBRARY_COMPACT_WIDTH: f32 = 700.0;
#[cfg(any(target_arch = "wasm32", test))]
const WEB_PROGRAM_LIBRARY_COMPACT_HEIGHT: f32 = 560.0;
const WEB_TOUCH_TARGET: f32 = 44.0;
#[cfg(any(target_arch = "wasm32", test))]
const WEB_LANDSCAPE_PANEL_MAX_WIDTH: f32 = 528.0;

#[cfg(any(target_arch = "wasm32", test))]
fn web_prefers_compact_layout(size: egui::Vec2) -> bool {
    size.x < WEB_PROGRAM_LIBRARY_COMPACT_WIDTH || size.y < WEB_PROGRAM_LIBRARY_COMPACT_HEIGHT
}

#[cfg(any(target_arch = "wasm32", test))]
fn web_prefers_scrollable_landscape(size: egui::Vec2) -> bool {
    web_prefers_compact_layout(size) && size.x > size.y
}

#[cfg(any(target_arch = "wasm32", test))]
fn web_select_program(selected: &mut Option<usize>, listing_visible: &mut bool, index: usize) {
    *selected = Some(index);
    *listing_visible = false;
}

pub struct Hp67App {
    #[cfg(not(target_arch = "wasm32"))]
    pdf_viewer: crate::pdf_viewer::PdfViewer,
    state: Hp67State,
    photo: TextureHandle,
    card_logo: TextureHandle,
    card_artwork_atlas: Option<image::RgbaImage>,
    live_machine: Option<Hp67LiveMachine>,
    live_fault: Option<String>,
    card_media: Option<Hp67MagneticCard>,
    card_insertion_end: CardInsertionEnd,
    last_live_tick: Option<Instant>,
    last_reader_tick: Option<Instant>,
    unlimited_speed: bool,
    card_phase: ProgramCardPhase,
    card_phase_started: Option<Instant>,
    card_read_progress: f32,
    card_save_dialog_open: bool,
    card_save_path: String,
    card_save_status: Option<String>,
    program_library_open: bool,
    program_library_selected: Option<usize>,
    program_library_status: Option<String>,
    #[cfg(target_arch = "wasm32")]
    program_library_listing_visible: bool,
    card_library_entry: Option<usize>,
    card_artwork_texture: Option<TextureHandle>,
}

fn decode_embedded_full_resolution_artwork(
    entry: &crate::program_library::ProgramLibraryEntry,
) -> Result<Option<image::RgbaImage>, String> {
    let Some(bytes) = entry
        .artwork_path
        .and_then(crate::program_library::embedded_card_artwork)
    else {
        return Ok(None);
    };
    image::load_from_memory_with_format(bytes, image::ImageFormat::Png)
        .map(|image| Some(image.to_rgba8()))
        .map_err(|error| {
            format!(
                "invalid embedded source artwork {}: {error}",
                entry.reference
            )
        })
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

        // Decode a full-resolution face only when selected; keep the small atlas lazy.
        let card_artwork_atlas = None;

        let (live_machine, live_fault) = match Hp67LiveMachine::power_on_default() {
            Ok(machine) => (Some(machine), None),
            Err(error) => (None, Some(format!("Power-on failed: {error}"))),
        };

        Self {
            #[cfg(not(target_arch = "wasm32"))]
            pdf_viewer: crate::pdf_viewer::PdfViewer::default(),
            state: Hp67State::default(),
            photo,
            card_logo,
            card_artwork_atlas,
            live_machine,
            live_fault,
            card_media: None,
            card_insertion_end: CardInsertionEnd::End1,
            last_live_tick: None,
            last_reader_tick: None,
            unlimited_speed: false,
            card_phase: ProgramCardPhase::Idle,
            card_phase_started: None,
            card_read_progress: 0.0,
            card_save_dialog_open: false,
            card_save_path: "hp67-card.hp67card".to_owned(),
            card_save_status: None,
            program_library_open: false,
            program_library_selected: None,
            program_library_status: None,
            #[cfg(target_arch = "wasm32")]
            program_library_listing_visible: false,
            card_library_entry: None,
            card_artwork_texture: None,
        }
    }
}

impl Hp67App {
    fn stop_on_fault(&mut self, error: String) {
        eprintln!("HP-67 emulation stopped: {error}");
        if let Some(card) = self
            .live_machine
            .take()
            .and_then(Hp67LiveMachine::into_retained_card)
        {
            self.card_media = Some(card);
            self.card_phase = ProgramCardPhase::Idle;
            self.card_phase_started = None;
            self.card_read_progress = 0.0;
        }
        self.live_fault = Some(error);
    }

    fn import_dropped_card_files(&mut self, ctx: &egui::Context) {
        let paths = ctx.input(|input| {
            input
                .raw
                .dropped_files
                .iter()
                .filter_map(|file| file.path.clone())
                .collect::<Vec<_>>()
        });

        if !paths.is_empty() {
            if let Err(error) = self.load_card_files(ctx, &paths) {
                eprintln!("HP-67 card import failed: {error}");
                self.program_library_status = Some(error);
                self.program_library_open = true;
            }
        }
    }

    fn load_card_files(
        &mut self,
        ctx: &egui::Context,
        paths: &[std::path::PathBuf],
    ) -> Result<(), String> {
        if self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted)
        {
            return Err("cannot replace card media while a card is inside the reader".into());
        }
        let (card, end) = decode_dropped_media(paths)?;
        let index = scanned_card_index(&card);
        let texture = match index {
            Some(i) => self.library_face_texture(ctx, i, &PROGRAM_LIBRARY[i])?,
            None => None,
        };
        self.card_media = Some(card);
        self.card_library_entry = index;
        self.card_artwork_texture = texture;
        self.card_insertion_end = end;
        self.card_phase = ProgramCardPhase::Idle;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        self.card_save_path = paths[0]
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

    fn library_face_texture(
        &mut self,
        ctx: &egui::Context,
        index: usize,
        entry: &crate::program_library::ProgramLibraryEntry,
    ) -> Result<Option<TextureHandle>, String> {
        let decoded_artwork = if let Some(image) = decode_embedded_full_resolution_artwork(entry)? {
            Some(image)
        } else if let Some(row) = entry.artwork_atlas_row() {
            if self.card_artwork_atlas.is_none() {
                self.card_artwork_atlas = Some(decode_embedded_card_artwork_atlas()?);
            }
            let atlas = self
                .card_artwork_atlas
                .as_ref()
                .expect("atlas was initialized");
            Some(
                image::imageops::crop_imm(
                    atlas,
                    0,
                    row as u32 * CARD_ARTWORK_ATLAS_ROW_HEIGHT,
                    CARD_ARTWORK_ATLAS_WIDTH,
                    CARD_ARTWORK_ATLAS_ROW_HEIGHT,
                )
                .to_image(),
            )
        } else {
            match entry.artwork_path {
                Some(path) if Path::new(path).is_file() => {
                    let bytes = fs::read(path)
                        .map_err(|error| format!("cannot read artwork {}: {error}", path))?;
                    Some(
                        image::load_from_memory_with_format(&bytes, image::ImageFormat::Png)
                            .map_err(|error| format!("invalid artwork PNG {}: {error}", path))?
                            .to_rgba8(),
                    )
                }
                _ => None,
            }
        };
        Ok(decoded_artwork.map(|decoded| {
            let size = [decoded.width() as usize, decoded.height() as usize];
            let image = ColorImage::from_rgba_unmultiplied(size, decoded.as_raw());
            ctx.load_texture(
                format!("hp67-program-card-artwork-{index}"),
                image,
                TextureOptions::LINEAR,
            )
        }))
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
        let artwork_texture = self.library_face_texture(ctx, index, entry)?;
        self.card_media = Some(loaded.card);
        self.card_library_entry = Some(index);
        self.card_artwork_texture = artwork_texture;
        self.card_insertion_end = CardInsertionEnd::End1;
        self.card_phase = ProgramCardPhase::Idle;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        self.card_save_path = format!("{} {}.hp67card", entry.reference, loaded.card_name);
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
                                let order = crate::program_library::program_library_display_order();
                                let mut pack_start = 0usize;
                                while pack_start < order.len() {
                                    let pack = PROGRAM_LIBRARY[order[pack_start]].pack;
                                    let mut pack_end = pack_start + 1;
                                    while pack_end < order.len()
                                        && PROGRAM_LIBRARY[order[pack_end]].pack == pack
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
                                        for &index in &order[pack_start..pack_end] {
                                            let entry = &PROGRAM_LIBRARY[index];
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
                        crate::documentation::pack_manual_button(
                            ui,
                            entry.pack,
                            entry.source_pdf,
                            entry.reference,
                        );
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

        if web_prefers_compact_layout(ctx.screen_rect().size()) {
            self.show_program_library_compact(ctx);
        } else {
            self.show_program_library_wide(ctx);
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn show_program_library_wide(&mut self, ctx: &egui::Context) {
        let mut keep_open = true;
        let mut close_requested = false;
        let mut selected = self.program_library_selected;
        let mut listing_visible = self.program_library_listing_visible && selected.is_some();
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
                            let order = crate::program_library::program_library_display_order();
                            let mut pack_start = 0usize;
                            while pack_start < order.len() {
                                let pack = PROGRAM_LIBRARY[order[pack_start]].pack;
                                let mut pack_end = pack_start + 1;
                                while pack_end < order.len()
                                    && PROGRAM_LIBRARY[order[pack_end]].pack == pack
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
                                    for &index in &order[pack_start..pack_end] {
                                        let entry = &PROGRAM_LIBRARY[index];
                                        let label = if entry.reference.is_empty() {
                                            entry.title.to_owned()
                                        } else {
                                            format!("{} - {}", entry.reference, entry.title)
                                        };
                                        if ui
                                            .selectable_label(selected == Some(index), label)
                                            .clicked()
                                        {
                                            web_select_program(
                                                &mut selected,
                                                &mut listing_visible,
                                                index,
                                            );
                                        }
                                    }
                                });

                                pack_start = pack_end;
                            }
                        });

                    columns[1].strong(if listing_visible {
                        "Program listing"
                    } else {
                        "Selected card"
                    });
                    columns[1].separator();
                    egui::ScrollArea::both()
                        .id_source("hp67-program-library-web-selection")
                        .auto_shrink([false, false])
                        .show(&mut columns[1], |ui| {
                            if let Some(index) = selected {
                                let entry = &PROGRAM_LIBRARY[index];
                                ui.heading(entry.title);
                                if !entry.reference.is_empty() {
                                    ui.label(format!("Reference: {}", entry.reference));
                                }
                                ui.label(format!("Pack: {}", entry.pack));
                        crate::documentation::pack_manual_button(ui, entry.pack, entry.source_pdf, entry.reference);
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
                                    ui.separator();
                                    ui.label(
                                        "Remove the card from the reader before loading another one.",
                                    );
                                }
                                if let Some(status) = status.as_deref() {
                                    ui.separator();
                                    ui.label(status);
                                }

                                ui.separator();
                                if listing_visible {
                                    if ui.button("Hide listing").clicked() {
                                        listing_visible = false;
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
                                } else if ui.button("View listing").clicked() {
                                    listing_visible = true;
                                }
                            } else {
                                ui.label(
                                    "Select a program. You can load it directly or view its listing first.",
                                );
                            }
                        });
                });
            });

        if close_requested {
            keep_open = false;
        }
        self.program_library_open = keep_open;
        self.program_library_selected = selected;
        self.program_library_listing_visible = listing_visible;

        if let Some(index) = load_requested {
            self.program_library_status = match self.load_program_library_entry(ctx, index) {
                Ok(()) => {
                    self.program_library_open = false;
                    self.program_library_listing_visible = false;
                    let entry = &PROGRAM_LIBRARY[index];
                    Some(format!("Loaded {} - {}", entry.reference, entry.title))
                }
                Err(error) => Some(error),
            };
        }
    }

    #[cfg(target_arch = "wasm32")]
    fn show_program_library_compact(&mut self, ctx: &egui::Context) {
        let mut selected = self.program_library_selected;
        let mut listing_visible = self.program_library_listing_visible && selected.is_some();
        let mut close_requested = false;
        let mut load_requested = None;
        let status = self.program_library_status.clone();
        let reader_free = !self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::magnetic_card_inserted);
        let screen = ctx.screen_rect();

        egui::Window::new("hp67-program-library-mobile")
            .id(egui::Id::new("hp67-program-library-mobile"))
            .title_bar(false)
            .collapsible(false)
            .resizable(false)
            .fixed_rect(screen)
            .frame(
                egui::Frame::none()
                    .fill(Color32::from_rgb(17, 18, 16))
                    .inner_margin(egui::Margin::symmetric(12.0, 10.0)),
            )
            .show(ctx, |ui| {
                ui.spacing_mut().interact_size.y = WEB_TOUCH_TARGET;
                ui.spacing_mut().item_spacing.y = 6.0;

                ui.horizontal(|ui| {
                    if listing_visible {
                        if ui
                            .add_sized(
                                [104.0, WEB_TOUCH_TARGET],
                                egui::Button::new("← Programs"),
                            )
                            .clicked()
                        {
                            listing_visible = false;
                        }
                    } else {
                        ui.heading("Program Library");
                    }

                    ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                        if ui
                            .add_sized(
                                [72.0, WEB_TOUCH_TARGET],
                                egui::Button::new("Close"),
                            )
                            .clicked()
                        {
                            close_requested = true;
                        }
                    });
                });
                ui.separator();

                if listing_visible {
                    if let Some(index) = selected {
                        let entry = &PROGRAM_LIBRARY[index];
                        let action_space = WEB_TOUCH_TARGET + 12.0;
                        let body_height = (ui.available_height() - action_space).max(120.0);

                        egui::ScrollArea::both()
                            .id_source("hp67-program-library-mobile-listing")
                            .auto_shrink([false, false])
                            .max_height(body_height)
                            .show(ui, |ui| {
                                ui.heading(entry.title);
                                if !entry.reference.is_empty() {
                                    ui.label(format!("Reference: {}", entry.reference));
                                }
                                ui.label(format!("Pack: {}", entry.pack));
                        crate::documentation::pack_manual_button(ui, entry.pack, entry.source_pdf, entry.reference);
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
                                    ui.separator();
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
                            });

                        ui.separator();
                        let load_width = ui.available_width();
                        if ui
                            .add_enabled(
                                reader_free,
                                egui::Button::new("Load card").min_size(egui::vec2(
                                    load_width,
                                    WEB_TOUCH_TARGET,
                                )),
                            )
                            .clicked()
                        {
                            load_requested = Some(index);
                        }
                    } else {
                        listing_visible = false;
                    }
                } else {
                    ui.label("Choose a program card");
                    if let Some(status) = status.as_deref() {
                        ui.label(status);
                    }
                    ui.add_space(2.0);

                    let action_space = if selected.is_some() {
                        WEB_TOUCH_TARGET + 52.0
                    } else {
                        0.0
                    };
                    let list_height = (ui.available_height() - action_space).max(120.0);

                    egui::ScrollArea::vertical()
                        .id_source("hp67-program-library-mobile-packs")
                        .auto_shrink([false, false])
                        .max_height(list_height)
                        .show(ui, |ui| {
                            let order = crate::program_library::program_library_display_order();
                            let mut pack_start = 0usize;
                            while pack_start < order.len() {
                                let pack = PROGRAM_LIBRARY[order[pack_start]].pack;
                                let mut pack_end = pack_start + 1;
                                while pack_end < order.len()
                                    && PROGRAM_LIBRARY[order[pack_end]].pack == pack
                                {
                                    pack_end += 1;
                                }

                                egui::CollapsingHeader::new(format!(
                                    "{} ({})",
                                    pack,
                                    pack_end - pack_start
                                ))
                                .id_source(("hp67-program-pack-mobile", pack))
                                .default_open(pack_start == 0)
                                .show(ui, |ui| {
                                    for &index in &order[pack_start..pack_end] {
                                        let entry = &PROGRAM_LIBRARY[index];
                                        let label = if entry.reference.is_empty() {
                                            entry.title.to_owned()
                                        } else {
                                            format!("{} - {}", entry.reference, entry.title)
                                        };
                                        if ui
                                            .selectable_label(selected == Some(index), label)
                                            .clicked()
                                        {
                                            web_select_program(
                                                &mut selected,
                                                &mut listing_visible,
                                                index,
                                            );
                                        }
                                    }
                                });

                                pack_start = pack_end;
                            }
                        });

                    if let Some(index) = selected {
                        let entry = &PROGRAM_LIBRARY[index];
                        ui.separator();
                        let selected_label = if entry.reference.is_empty() {
                            entry.title.to_owned()
                        } else {
                            format!("Selected: {} - {}", entry.reference, entry.title)
                        };
                        ui.label(selected_label);
                        ui.horizontal(|ui| {
                            let button_width = ((ui.available_width() - 6.0) * 0.5).max(100.0);
                            if ui
                                .add_sized(
                                    [button_width, WEB_TOUCH_TARGET],
                                    egui::Button::new("View listing"),
                                )
                                .clicked()
                            {
                                listing_visible = true;
                            }
                            if ui
                                .add_enabled(
                                    reader_free,
                                    egui::Button::new("Load card").min_size(egui::vec2(
                                        button_width,
                                        WEB_TOUCH_TARGET,
                                    )),
                                )
                                .clicked()
                            {
                                load_requested = Some(index);
                            }
                        });
                    }
                }
            });

        if close_requested {
            self.program_library_open = false;
            self.program_library_listing_visible = false;
            return;
        }

        self.program_library_selected = selected;
        self.program_library_listing_visible = listing_visible;

        if let Some(index) = load_requested {
            self.program_library_status = match self.load_program_library_entry(ctx, index) {
                Ok(()) => {
                    self.program_library_open = false;
                    self.program_library_listing_visible = false;
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

    fn insert_new_blank_card(&mut self) -> bool {
        if !card_insertion_allowed(self.state.power_on) {
            return false;
        }

        let Some(machine) = self.live_machine.as_mut() else {
            return false;
        };
        if let Err(error) =
            machine.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
        {
            eprintln!("HP-67 blank card insertion failed: {error}");
            return false;
        }
        self.card_media = None;
        self.card_phase = ProgramCardPhase::WaitingAtReader;
        self.card_phase_started = None;
        self.card_read_progress = 0.0;
        self.card_library_entry = None;
        self.card_artwork_texture = None;
        self.card_insertion_end = CardInsertionEnd::End1;
        self.card_save_path = "hp67-card.hp67card".to_owned();
        self.card_save_status = None;
        true
    }

    fn show_calculator_surface(
        &mut self,
        ui: &mut egui::Ui,
        now: Instant,
        card_phase_progress: f32,
        minimum_touch_target: f32,
    ) {
        let host = ui.available_rect_before_wrap();
        let display = self.live_machine.as_ref().map_or(
            HardwareDisplayFrame::BLANK,
            Hp67LiveMachine::optical_display_frame,
        );
        let card_prompt = self
            .live_machine
            .as_ref()
            .is_some_and(Hp67LiveMachine::card_prompt_visible);
        let card_artwork = if let Some(entry) = self
            .card_library_entry
            .and_then(|index| PROGRAM_LIBRARY.get(index))
        {
            &entry.artwork
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
                card_prompt,
                rotated_180: self.card_insertion_end == CardInsertionEnd::End2,
                reader_enabled: self.state.power_on,
                reader_free_for_new_blank: reader_free_for_new_blank(self.card_phase),
                minimum_touch_target,
            },
            minimum_touch_target,
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
        if card_prompt && (panel.card_parked_left_clicked || panel.card_parked_left_double_clicked)
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
                self.last_reader_tick = None;
                if was_power_on && !self.state.power_on {
                    if let Some(machine) = self.live_machine.as_mut() {
                        match machine.take_magnetic_card_for_power_off() {
                            Ok(Some(card)) => {
                                self.card_media = Some(card);
                                self.card_phase = ProgramCardPhase::Idle;
                                self.card_phase_started = None;
                                self.card_read_progress = 0.0;
                            }
                            Ok(None) => {}
                            Err(error) => {
                                self.stop_on_fault(format!(
                                    "Power-off card recovery failed: {error}"
                                ));
                            }
                        }
                    }
                } else if !was_power_on && self.state.power_on {
                    let reset_result = if let Some(machine) = self.live_machine.as_mut() {
                        machine.reset_power_on()
                    } else {
                        Hp67LiveMachine::power_on_default().map(|machine| {
                            self.live_machine = Some(machine);
                        })
                    };
                    if let Err(error) = reset_result {
                        self.stop_on_fault(format!("Power-on reset failed: {error}"));
                    } else {
                        self.live_fault = None;
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
            self.stop_on_fault(format!("Program selector failed: {error}"));
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
    }
}

// Retain sub-microsecond time at the host boundary instead of rounding it away
// on every accelerated batch. The machine receives deterministic integer time.
fn take_reader_elapsed(anchor: &mut Option<Instant>, now: Instant) -> Duration {
    let previous = anchor.get_or_insert(now);
    let elapsed = now.saturating_duration_since(*previous);
    let whole_us = elapsed.as_micros().min(u128::from(u64::MAX)) as u64;
    let elapsed = Duration::from_micros(whole_us);
    *previous += elapsed;
    elapsed
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

// Only files in the current drop can contribute tracks to a new card.
fn decode_dropped_media(
    paths: &[std::path::PathBuf],
) -> Result<(Hp67MagneticCard, CardInsertionEnd), String> {
    let extension = |p: &Path| {
        p.extension()
            .and_then(|v| v.to_str())
            .unwrap_or_default()
            .to_ascii_lowercase()
    };
    let read = |p: &Path| fs::read(p).map_err(|e| format!("cannot read {}: {e}", p.display()));
    let hpp = |p: &Path| -> Result<TeenixHppImport, String> {
        let part = TeenixHppImport::from_bytes(&read(p)?)
            .map_err(|e| format!("{}: {e:?}", p.display()))?;
        if !part.length_matches {
            eprintln!("Teenix length mismatch: {}", p.display());
        }
        Ok(part)
    };
    match paths {
        [p] => match extension(p).as_str() {
            "hpp" => {
                let part = hpp(p)?;
                let end = CardInsertionEnd::for_track(part.card_track);
                Ok((part.into_card(), end))
            }
            "hp67raw" => Hp67MagneticCard::from_hp67raw_bytes(&read(p)?)
                .map(|c| (c, CardInsertionEnd::End1))
                .map_err(|e| format!("{}: {e:?}", p.display())),
            "hp67card" => Hp67MagneticCard::from_hp67card_bytes(&read(p)?)
                .map(|c| (c, CardInsertionEnd::End1))
                .map_err(|e| format!("{}: {e:?}", p.display())),
            _ => Err("Expected .hpp, .hp67raw or .hp67card".into()),
        },
        [a, b] if extension(a) == "hpp" && extension(b) == "hpp" => {
            join_hpp_parts(hpp(a)?, hpp(b)?)
        }
        _ => Err("Drop one card, or both matching .hpp parts together.".into()),
    }
}

fn join_hpp_parts(
    a: TeenixHppImport,
    b: TeenixHppImport,
) -> Result<(Hp67MagneticCard, CardInsertionEnd), String> {
    if a.bitmap_name.is_empty()
        || a.bitmap_name != b.bitmap_name
        || a.calculator_id != b.calculator_id
        || !matches!(
            (a.header_id, b.header_id),
            (1, 2) | (2, 1) | (3, 4) | (4, 3)
        )
    {
        return Err("Incompatible HPP pair. For independent sides use the converter with explicit --track1 and --track2.".into());
    }
    let mut card = a.into_card();
    card.set_track(b.card_track, b.track);
    Ok((card, CardInsertionEnd::End1))
}

fn scanned_card_index(card: &Hp67MagneticCard) -> Option<usize> {
    let bytes = card.to_hp67card_bytes();
    let mut hits = PROGRAM_LIBRARY.iter().enumerate().filter(|(_, entry)| {
        entry.artwork_path.is_some() && matches!(entry.parts, [part] if *part == bytes.as_slice())
    });
    let (index, _) = hits.next()?;
    hits.next().is_none().then_some(index)
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
        #[cfg(target_arch = "wasm32")]
        let web_compact = web_prefers_compact_layout(ctx.screen_rect().size());
        #[cfg(not(target_arch = "wasm32"))]
        let web_compact = false;
        egui::TopBottomPanel::top("hp67-menu").show(ctx, |ui| {
            if web_compact {
                ui.spacing_mut().interact_size.y = WEB_TOUCH_TARGET;
            }
            egui::menu::bar(ui, |ui| {
                ui.menu_button("Cards", |ui| {
                    if web_compact {
                        ui.spacing_mut().interact_size.y = WEB_TOUCH_TARGET;
                    }
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
                ui.menu_button("Speed", |ui| {
                    if ui
                        .checkbox(&mut self.unlimited_speed, "Unlimited")
                        .changed()
                    {
                        // Do not replay host time when switching pacing modes.
                        self.last_live_tick = None;
                        self.last_reader_tick = None;
                    }
                    ui.label(if self.unlimited_speed {
                        "Unlimited calculations; card handling stays at 1x"
                    } else {
                        "Real HP-67 speed (1x)"
                    });
                });
                ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                    ui.push_id("hp67-help-right", |ui| crate::documentation::help_menu(ui));
                });
            });
        });
        if let Some(error) = &self.live_fault {
            egui::TopBottomPanel::top("hp67-machine-fault").show(ctx, |ui| {
                ui.label(format!("Emulation stopped: {error}"));
                ui.label("Switch OFF and ON to retry.");
            });
        }
        if open_program_library {
            self.program_library_open = true;
            self.program_library_status = None;
            #[cfg(target_arch = "wasm32")]
            {
                self.program_library_listing_visible = false;
            }
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
                #[cfg(target_arch = "wasm32")]
                if web_prefers_scrollable_landscape(ctx.screen_rect().size()) {
                    egui::ScrollArea::vertical()
                        .id_source("hp67-mobile-landscape-calculator")
                        .auto_shrink([false, false])
                        .show(ui, |ui| {
                            let panel_width =
                                ui.available_width().min(WEB_LANDSCAPE_PANEL_MAX_WIDTH);
                            let panel_height = Hp67Panel::height_for_width(panel_width);
                            let left_space = ((ui.available_width() - panel_width) * 0.5).max(0.0);
                            ui.horizontal(|ui| {
                                ui.add_space(left_space);
                                ui.vertical(|ui| {
                                    ui.set_width(panel_width);
                                    ui.set_height(panel_height);
                                    self.show_calculator_surface(
                                        ui,
                                        now,
                                        card_phase_progress,
                                        WEB_TOUCH_TARGET,
                                    );
                                });
                            });
                        });
                    return;
                }

                let minimum_touch_target = if web_compact { WEB_TOUCH_TARGET } else { 0.0 };
                self.show_calculator_surface(ui, now, card_phase_progress, minimum_touch_target);
            });

        let elapsed = self
            .last_live_tick
            .replace(now)
            .map_or(Duration::ZERO, |previous| {
                now.saturating_duration_since(previous)
            });

        let mut live_error = None;
        let mut live_card_active = false;
        let mut yielded_for_reader = false;
        if self.state.power_on {
            if let Some(machine) = self.live_machine.as_mut() {
                let result = if self.unlimited_speed {
                    let started = Instant::now();
                    loop {
                        let reader_elapsed =
                            take_reader_elapsed(&mut self.last_reader_tick, Instant::now());
                        match machine
                            .advance_processing_batch(UNLIMITED_BATCH_WORDS, reader_elapsed)
                        {
                            Ok(false) => {
                                yielded_for_reader = true;
                                break Ok(());
                            }
                            Ok(true) if started.elapsed() < UNLIMITED_SLICE_BUDGET => {}
                            Ok(true) => break Ok(()),
                            Err(error) => break Err(error),
                        }
                    }
                } else {
                    self.last_reader_tick = Some(now);
                    machine.advance(elapsed)
                };
                if let Err(error) = result {
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
            self.stop_on_fault(error);
            ctx.request_repaint();
        }

        if self.card_phase.is_animating() || live_card_active {
            ctx.request_repaint_after(Duration::from_millis(16));
        }
        if self.state.power_on {
            // Keep the physical firmware machine advancing after boot idle as well
            // as during startup. The same cadence also samples held/released key
            // contacts without introducing host-side calculator semantics.
            if self.unlimited_speed && !yielded_for_reader {
                ctx.request_repaint();
            } else {
                ctx.request_repaint_after(Duration::from_micros(4_800));
            }
        }
        self.show_card_save_dialog(ctx);
        self.show_program_library(ctx);
        crate::documentation::show(ctx);
        #[cfg(not(target_arch = "wasm32"))]
        self.pdf_viewer.show(ctx);

        if ctx.input(|i| i.pointer.any_down()) {
            ctx.request_repaint();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn unlimited_reader_clock_preserves_submicroseconds_and_does_not_charge_time_twice() {
        let start = Instant::now();
        let mut anchor = None;
        assert_eq!(take_reader_elapsed(&mut anchor, start), Duration::ZERO);
        assert_eq!(
            take_reader_elapsed(&mut anchor, start + Duration::from_nanos(700)),
            Duration::ZERO
        );
        assert_eq!(
            take_reader_elapsed(&mut anchor, start + Duration::from_nanos(1400)),
            Duration::from_micros(1)
        );
        assert_eq!(
            take_reader_elapsed(&mut anchor, start + Duration::from_nanos(2100)),
            Duration::from_micros(1)
        );
        assert_eq!(
            take_reader_elapsed(&mut anchor, start + Duration::from_nanos(2100)),
            Duration::ZERO
        );
        assert_eq!(
            take_reader_elapsed(&mut anchor, start + Duration::from_micros(28_000)),
            Duration::from_micros(27_998)
        );
    }

    #[test]
    fn web_program_library_uses_compact_flow_for_phone_sized_viewports() {
        assert!(web_prefers_compact_layout(egui::vec2(390.0, 844.0)));
        assert!(web_prefers_compact_layout(egui::vec2(844.0, 390.0)));
        assert!(web_prefers_compact_layout(egui::vec2(600.0, 900.0)));
        assert!(!web_prefers_compact_layout(egui::vec2(1024.0, 768.0)));
    }

    #[test]
    fn selecting_a_web_program_does_not_force_the_listing_open() {
        let mut selected = Some(2usize);
        let mut listing_visible = true;
        web_select_program(&mut selected, &mut listing_visible, 7);
        assert_eq!(selected, Some(7));
        assert!(!listing_visible);
    }

    #[test]
    fn web_phone_landscape_uses_scrollable_full_size_calculator() {
        assert!(web_prefers_scrollable_landscape(egui::vec2(844.0, 390.0)));
        assert!(web_prefers_scrollable_landscape(egui::vec2(667.0, 375.0)));
        assert!(!web_prefers_scrollable_landscape(egui::vec2(390.0, 844.0)));
        assert!(!web_prefers_scrollable_landscape(egui::vec2(1024.0, 768.0)));
        assert_eq!(WEB_TOUCH_TARGET, 44.0);
        assert_eq!(WEB_LANDSCAPE_PANEL_MAX_WIDTH, 528.0);
    }

    #[test]
    fn embedded_source_artwork_keeps_600_dpi_and_covers_every_scanned_card() {
        // Canonical extractor: round(71.1 * 600 / 25.4) x round(11.4 * 600 / 25.4).
        for entry in PROGRAM_LIBRARY
            .iter()
            .filter(|entry| entry.artwork_path.is_some())
        {
            let bytes = entry
                .artwork_path
                .and_then(crate::program_library::embedded_card_artwork)
                .unwrap_or_else(|| panic!("{} lacks embedded source PNG", entry.reference));
            let reader = image::io::Reader::with_format(
                std::io::Cursor::new(bytes),
                image::ImageFormat::Png,
            );
            assert_eq!(
                reader.into_dimensions().unwrap(),
                (1680, 269),
                "{}",
                entry.reference
            );
        }
        let artillery = PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "GA1-05A")
            .unwrap();
        let image = decode_embedded_full_resolution_artwork(artillery)
            .unwrap()
            .unwrap();
        assert_eq!(image.dimensions(), (1680, 269));
        assert!(image.width() > CARD_ARTWORK_ATLAS_WIDTH * 3);
        assert!(image.height() > CARD_ARTWORK_ATLAS_ROW_HEIGHT * 3);
    }

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
    fn explicit_hpp_pair_restores_catalog_media_in_either_order() {
        let a = TeenixHppImport::from_bytes(include_bytes!(
            "../programs/HP67/HP-67 Games Pac 1/GA1-05A_1 Artillery.hpp"
        ))
        .unwrap();
        let b = TeenixHppImport::from_bytes(include_bytes!(
            "../programs/HP67/HP-67 Games Pac 1/GA1-05A_2 Artillery.hpp"
        ))
        .unwrap();
        let i = PROGRAM_LIBRARY
            .iter()
            .position(|e| e.reference == "GA1-05A")
            .unwrap();
        let expected = PROGRAM_LIBRARY[i].load_card().unwrap().card;
        for (x, y) in [(a.clone(), b.clone()), (b.clone(), a.clone())] {
            let (card, end) = join_hpp_parts(x, y).unwrap();
            assert_eq!(end, CardInsertionEnd::End1);
            assert_eq!(card, expected);
            assert_eq!(scanned_card_index(&card), Some(i));
        }
        assert!(join_hpp_parts(a.clone(), a.clone()).is_err());
        let mut other = b;
        other.bitmap_name.push('X');
        assert!(join_hpp_parts(a, other).is_err());
    }
}
