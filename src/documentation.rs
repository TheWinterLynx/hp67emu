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
        title: "HP-67 Standard Pac",
        file: "hp67-pac-standard-en.pdf",
    },
    Manual {
        title: "HP-67 Games Pac I",
        file: "hp6797-pac-games-en.pdf",
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
        title: "HP-67 Electrical Engineering Pac I",
        file: "hp6797-pac-eleceng-en.pdf",
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
        title: "HP-67 Statistics Pac I",
        file: "hp6797-pac-stat-en.pdf",
    },
    Manual {
        title: "HP-67 Clinical Laboratory and Nuclear Medicine Pac",
        file: "hp6797-pac-clinical-en.pdf",
    },
    Manual {
        title: "HP-67 Navigation Pac I",
        file: "hp6797-pac-navigation-en.pdf",
    },
    Manual {
        title: "HP-67 Surveying Pac I",
        file: "hp6797-pac-surveying-en.pdf",
    },
];

#[cfg(any(target_arch = "wasm32", test))]
include!("../documentation/available.rs");

fn window_id() -> egui::Id {
    egui::Id::new("hp67-documentation-window")
}

pub fn help_menu(ui: &mut egui::Ui) {
    ui.menu_button("Ayuda", |ui| {
        if ui.button("Documentación…").clicked() {
            ui.ctx()
                .data_mut(|data| data.insert_temp(window_id(), true));
            ui.close_menu();
        }
    });
}

pub fn show(ctx: &egui::Context) {
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
            ui.label("Los documentos se abren en tu lector de PDF habitual.");
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

pub fn pack_manual_button(ui: &mut egui::Ui, pack: &str, source_pdf: Option<&str>) {
    let manual = manual_for_pack(pack).or_else(|| {
        let name = source_pdf?.rsplit('/').next()?;
        MANUALS.iter().find(|manual| manual.file == name)
    });
    if let Some(manual) = manual {
        if ui
            .add_enabled(
                cached_available(ui.ctx(), manual),
                egui::Button::new("Abrir manual del PAC"),
            )
            .clicked()
        {
            request_open(ui.ctx(), manual);
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
fn path_in(root: &std::path::Path, manual: &Manual) -> Option<std::path::PathBuf> {
    use std::io::Read;
    let path = root.join("documentation").join(manual.file);
    let mut signature = [0u8; 5];
    std::fs::File::open(&path)
        .ok()?
        .read_exact(&mut signature)
        .ok()?;
    (signature == *b"%PDF-").then_some(path)
}

#[cfg(not(target_arch = "wasm32"))]
fn manual_path(manual: &Manual) -> Option<std::path::PathBuf> {
    // A portable distribution carries documentation beside its executable.
    // Development runs can use the same folder in the source checkout.
    std::env::current_exe()
        .ok()
        .and_then(|exe| exe.parent().and_then(|root| path_in(root, manual)))
        .or_else(|| path_in(std::path::Path::new(env!("CARGO_MANIFEST_DIR")), manual))
}

#[cfg(not(target_arch = "wasm32"))]
fn available(manual: &Manual) -> bool {
    manual_path(manual).is_some()
}

#[cfg(not(target_arch = "wasm32"))]
fn open_manual(_ctx: &egui::Context, manual: &Manual) -> Result<(), String> {
    let path = manual_path(manual)
        .ok_or_else(|| "Este manual no está incluido en esta distribución.".to_owned())?;
    #[cfg(target_os = "windows")]
    let mut command = std::process::Command::new("explorer.exe");
    #[cfg(target_os = "macos")]
    let mut command = std::process::Command::new("open");
    #[cfg(not(any(target_os = "windows", target_os = "macos")))]
    let mut command = std::process::Command::new("xdg-open");
    // Direct argv: never interpret document paths through a command shell.
    command
        .arg(path)
        .spawn()
        .map(|_| ())
        .map_err(|error| format!("No se pudo abrir el lector de PDF: {error}"))
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
        assert!(manual_for_pack("HP-67 Electrical Engineering Pac I").is_some());
        assert!(manual_for_pack("HP-67 Custom Diagnostic Pac").is_none());
    }

    #[cfg(not(target_arch = "wasm32"))]
    #[test]
    fn missing_or_non_pdf_documents_are_not_offered_as_available() {
        let root = std::env::temp_dir().join(format!("hp67-manual-test-{}", std::process::id()));
        std::fs::create_dir_all(root.join("documentation")).unwrap();
        let path = root.join("documentation").join(MANUALS[0].file);
        assert!(path_in(&root, &MANUALS[0]).is_none());
        std::fs::write(&path, b"not a PDF").unwrap();
        assert!(path_in(&root, &MANUALS[0]).is_none());
        std::fs::write(&path, b"%PDF-1.7\n").unwrap();
        assert_eq!(path_in(&root, &MANUALS[0]), Some(path.clone()));
        std::fs::remove_file(path).unwrap();
        std::fs::remove_dir_all(root).unwrap();
    }
}
