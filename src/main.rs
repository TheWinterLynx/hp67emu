#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod app;
mod hp67;
mod panel;
mod ui;

use eframe::egui;

fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([528.0, 992.0])
            .with_min_inner_size([165.0, 310.0])
            .with_resizable(true),
        // eframe 0.27.2 NativeOptions::multisampling (u16), used by Glow's
        // GL config and WGPU sample_count. Retain four samples plus egui's AA.
        multisampling: 4,
        ..Default::default()
    }
}

fn main() -> eframe::Result<()> {
    eframe::run_native(
        "HP-67 Emulator",
        native_options(),
        Box::new(|cc| Box::new(app::Hp67App::new(cc))),
    )
}

#[cfg(test)]
mod capture;

#[test]
fn native_window_is_resizable_without_a_maximum_size() {
    let options = native_options();
    assert_eq!(options.viewport.resizable, Some(true));
    assert_eq!(
        options.viewport.min_inner_size,
        Some(egui::vec2(165.0, 310.0))
    );
    assert!(options.viewport.max_inner_size.is_none());
    assert_eq!(options.multisampling, 4);
}
