#![cfg_attr(
    all(not(debug_assertions), not(test), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod hp67;
mod panel;
mod program_library;
mod ui;

#[cfg(not(target_arch = "wasm32"))]
use eframe::egui;

#[cfg(not(target_arch = "wasm32"))]
fn native_options() -> eframe::NativeOptions {
    eframe::NativeOptions {
        viewport: egui::ViewportBuilder::default()
            .with_inner_size([528.0, 965.0])
            .with_min_inner_size([174.0, 318.0])
            .with_resizable(true),
        multisampling: 4,
        renderer: eframe::Renderer::Wgpu,
        ..Default::default()
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    eframe::run_native(
        "HP-67 Emulator",
        native_options(),
        Box::new(|cc| Box::new(app::Hp67App::new(cc))),
    )
}

#[cfg(target_arch = "wasm32")]
fn main() {
    wasm_bindgen_futures::spawn_local(async {
        eframe::WebRunner::new()
            .start(
                "the_canvas_id",
                eframe::WebOptions::default(),
                Box::new(|cc| Box::new(app::Hp67App::new(cc))),
            )
            .await
            .expect("failed to start HP-67 web emulator");
    });
}

#[cfg(all(test, not(target_arch = "wasm32")))]
#[test]
fn native_window_is_resizable_without_a_maximum_size() {
    let options = native_options();
    assert_eq!(options.viewport.resizable, Some(true));
    assert_eq!(
        options.viewport.min_inner_size,
        Some(egui::vec2(174.0, 318.0))
    );
    assert!(options.viewport.max_inner_size.is_none());
    assert_eq!(options.multisampling, 4);
}
