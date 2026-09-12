#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hp67;
mod key_depth;
#[path = "panel_fidelity.rs"]
mod panel;

use eframe::egui;

fn main() -> eframe::Result<()> {
    let options = eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([528.0, 992.0])
            .with_min_inner_size([330.0, 620.0]),
        ..Default::default()
    };

    eframe::run_native(
        "HP-67 Emulator",
        options,
        Box::new(|cc| Box::new(app::Hp67App::new(cc))),
    )
}
