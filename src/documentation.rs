//! Presentation-only access to the packaged HP-67 manuals.
use eframe::egui;

pub struct Manual {
    pub title: &'static str,
    pub file: &'static str,
}

pub const MANUALS: &[Manual] = &[
    Manual {
        title: "Manual de usuario HP-67 (español)",
        file: "hp67-oh-es.pdf",
    },
    Manual {
        title: "HP-67 Business Decisions Pac I",
        file: "hp6797-pac-business-en.pdf",
    },
    Manual {
        title: "HP-67 Civil Engineering Pac I",
        file: "hp6797-pac-civileng-en.pdf",
    },
    Manual {
        title: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        file: "hp6797-pac-clinical-en.pdf",
    },
    Manual {
        title: "HP-67 Electrical Engineering Pac I",
        file: "hp6797-pac-eleceng-en.pdf",
    },
    Manual {
        title: "HP-67 Games Pac I",
        file: "hp6797-pac-games-en.pdf",
    },
    Manual {
        title: "HP-67 Mathematics Pac I",
        file: "hp6797-pac-math-en.pdf",
    },
    Manual {
        title: "HP-67 Mechanical Engineering Pac I",
        file: "hp6797-pac-mecheng-en.pdf",
    },
    Manual {
        title: "HP-67 Navigation Pac I",
        file: "hp6797-pac-navigation-en.pdf",
    },
    Manual {
        title: "HP-67 Standard Pac",
        file: "hp67-pac-standard-en.pdf",
    },
    Manual {
        title: "HP-67 Statistics Pac I",
        file: "hp6797-pac-stat-en.pdf",
    },
    Manual {
        title: "HP-67 Surveying Pac I",
        file: "hp6797-pac-surveying-en.pdf",
    },
];

#[cfg(any(target_arch = "wasm32", test))]
include!("../documentation/available.rs");
include!("../documentation/sections.rs");

#[cfg(not(target_arch = "wasm32"))]
pub fn program_sections() -> &'static [(&'static str, &'static str, usize)] {
    PROGRAM_SECTIONS
}

fn window_id() -> egui::Id {
    egui::Id::new("hp67-documentation-window")
}

fn usage_window_id() -> egui::Id {
    egui::Id::new("hp67-usage-window")
}

pub fn help_menu(ui: &mut egui::Ui) {
    ui.menu_button("Ayuda", |ui| {
        if ui.button("Uso del emulador…").clicked() {
            ui.ctx()
                .data_mut(|data| data.insert_temp(usage_window_id(), true));
            ui.close_menu();
        }
        if ui.button("Documentación…").clicked() {
            ui.ctx()
                .data_mut(|data| data.insert_temp(window_id(), true));
            ui.close_menu();
        }
    });
}

fn show_usage(ctx: &egui::Context) {
    let id = usage_window_id();
    let mut open = ctx.data(|data| data.get_temp::<bool>(id).unwrap_or(false));
    if !open {
        return;
    }
    egui::Window::new("Uso del emulador")
        .id(id)
        .open(&mut open)
        .default_width(360.0)
        .show(ctx, |ui| {
            ui.heading("Tarjetas");
            ui.label("Arrastra un archivo de tarjeta a la ventana. Si el formato HPP viene en dos archivos, arrastra ambos a la vez.");
            ui.separator();
            ui.heading("Velocidad");
            ui.label("Unlimited CPU acelera el firmware. El lector y las animaciones de tarjeta mantienen su ritmo habitual.");
            ui.label("Si el firmware agota el tiempo de espera al leer o escribir una tarjeta, desactiva Unlimited CPU para volver a la velocidad original (1×).");
        });
    ctx.data_mut(|data| data.insert_temp(id, open));
}

pub fn show(ctx: &egui::Context) {
    show_usage(ctx);
    let mut open = ctx.data(|data| data.get_temp::<bool>(window_id()).unwrap_or(false));
    if !open {
        return;
    }
    egui::Window::new("HP-67 · Documentación")
        .open(&mut open)
        .resizable(true)
        .default_width(520.0)
        .show(ctx, |ui| {
            if ui.button("Actualizar documentos disponibles").clicked() {
                ctx.data_mut(|data| {
                    for manual in MANUALS {
                        data.remove::<bool>(window_id().with(manual.file));
                    }
                });
            }
            ui.heading("Manual de la calculadora");
            manual_row(ui, &MANUALS[0]);
            ui.separator();
            ui.heading("Manuales de los PACs");
            egui::ScrollArea::vertical()
                .id_source("hp67-manuals-scroll")
                .max_height(430.0)
                .show(ui, |ui| {
                    for manual in &MANUALS[1..] {
                        manual_row(ui, manual);
                    }
                });
            ui.separator();
            #[cfg(not(target_arch = "wasm32"))]
            ui.label("Visor PDF integrado · Documentación incluida en el ejecutable");
            #[cfg(target_arch = "wasm32")]
            ui.label("Los manuales se abren en otra pestaña del navegador.");
            if let Some(status) =
                ctx.data(|data| data.get_temp::<String>(window_id().with("status")))
            {
                ui.colored_label(egui::Color32::LIGHT_RED, status);
            }
        });
    ctx.data_mut(|data| data.insert_temp(window_id(), open));
}

fn manual_for_pack(pack: &str) -> Option<&'static Manual> {
    MANUALS.iter().skip(1).find(|manual| manual.title == pack)
}

pub fn pack_manual_button(
    ui: &mut egui::Ui,
    pack: &str,
    source_pdf: Option<&str>,
    reference: &str,
) {
    let manual = manual_for_pack(pack).or_else(|| {
        let name = source_pdf?.rsplit('/').next()?;
        MANUALS.iter().find(|manual| manual.file == name)
    });
    if let Some(manual) = manual {
        let section = PROGRAM_SECTIONS
            .iter()
            .find(|(card, file, _)| *card == reference && *file == manual.file);
        let label = if section.is_some() {
            "Ver instrucciones del programa"
        } else {
            "Ver manual completo del PAC"
        };
        if ui
            .add_enabled(cached_available(ui.ctx(), manual), egui::Button::new(label))
            .clicked()
        {
            let page = section
                .map(|(_, _, page)| page.saturating_sub(1))
                .unwrap_or(0);
            #[cfg(not(target_arch = "wasm32"))]
            crate::pdf_viewer::request(
                ui.ctx(),
                manual.file,
                format!("{} · {}", manual.title, reference),
                page,
            );
            #[cfg(target_arch = "wasm32")]
            ui.ctx().open_url(egui::OpenUrl::new_tab(format!(
                "documentation/{}#page={}",
                manual.file,
                page + 1
            )));
        }
    }
}

fn cached_available(ctx: &egui::Context, manual: &Manual) -> bool {
    let id = window_id().with(manual.file);
    if let Some(present) = ctx.data(|data| data.get_temp::<bool>(id)) {
        return present;
    }
    let present = available(manual);
    ctx.data_mut(|data| data.insert_temp(id, present));
    present
}

fn manual_row(ui: &mut egui::Ui, manual: &Manual) {
    ui.horizontal_wrapped(|ui| {
        if ui
            .add_enabled(
                cached_available(ui.ctx(), manual),
                egui::Button::new(manual.title),
            )
            .clicked()
        {
            request_open(ui.ctx(), manual);
        }
        if !cached_available(ui.ctx(), manual) {
            ui.weak("No incluido en esta distribución");
        }
    });
}

fn request_open(ctx: &egui::Context, manual: &Manual) {
    match open_manual(ctx, manual) {
        Ok(()) => {
            ctx.data_mut(|data| {
                data.remove::<String>(window_id().with("status"));
            });
        }
        Err(error) => {
            ctx.data_mut(|data| {
                data.insert_temp(window_id().with("status"), error);
                data.insert_temp(window_id(), true);
            });
        }
    }
}

#[cfg(target_arch = "wasm32")]
fn available(manual: &Manual) -> bool {
    PACKAGED_PDFS.contains(&manual.file)
}

#[cfg(target_arch = "wasm32")]
fn open_manual(ctx: &egui::Context, manual: &Manual) -> Result<(), String> {
    if !available(manual) {
        return Err("Este manual no está incluido en esta distribución.".to_owned());
    }
    ctx.open_url(egui::OpenUrl::new_tab(format!(
        "documentation/{}",
        manual.file
    )));
    Ok(())
}

#[cfg(not(target_arch = "wasm32"))]
fn available(manual: &Manual) -> bool {
    crate::pdf_viewer::has_pdf(manual.file)
}

#[cfg(not(target_arch = "wasm32"))]
fn open_manual(ctx: &egui::Context, manual: &Manual) -> Result<(), String> {
    if !available(manual) {
        return Err("Este manual no está incluido en el ejecutable".to_owned());
    }
    crate::pdf_viewer::request(ctx, manual.file, manual.title.to_owned(), 0);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn canonical_manuals_are_unique_and_safe_relative_pdf_names() {
        let mut files = std::collections::HashSet::new();
        for manual in MANUALS {
            assert!(manual.file.ends_with(".pdf"));
            assert!(!manual.file.contains('/') && !manual.file.contains('\\'));
            assert!(files.insert(manual.file));
        }
        for file in PACKAGED_PDFS {
            assert!(files.contains(*file));
        }
        assert_eq!(MANUALS.len(), 12);
        assert!(MANUALS[1..]
            .windows(2)
            .all(|pair| pair[0].title.to_lowercase() <= pair[1].title.to_lowercase()));
        assert!(manual_for_pack("HP-67 Electrical Engineering Pac I").is_some());
        assert!(manual_for_pack("HP-67 Custom Diagnostic Pac").is_none());
    }

    #[test]
    fn program_sections_reference_known_manuals_and_real_page_numbers() {
        let mut references = std::collections::HashSet::new();
        for (reference, file, page) in PROGRAM_SECTIONS {
            assert!(references.insert(reference));
            assert!(*page > 0);
            assert!(MANUALS.iter().any(|manual| manual.file == *file));
        }
        let moving_average = PROGRAM_SECTIONS
            .iter()
            .find(|(reference, _, _)| *reference == "SD1-01A")
            .unwrap();
        // PDF page 14 is printed section 01-01; page 10 is the generic user-instruction guide.
        assert_eq!(moving_average.2, 14);
        let bilateral = PROGRAM_SECTIONS
            .iter()
            .find(|(reference, _, _)| *reference == "EE1-18A")
            .unwrap();
        assert_eq!(bilateral.2, 88);
    }
}
