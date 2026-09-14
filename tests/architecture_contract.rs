//! Regression contract that keeps reusable emulation/reference code independent of UI.

use std::{fs, path::Path};

#[test]
fn reusable_core_has_no_gui_or_image_dependencies() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    // Match dependency-shaped tokens rather than bare words so explanatory
    // comments may legitimately mention UI technologies without tripping the
    // architecture gate.
    let forbidden = [
        "use eframe",
        "use egui",
        "eframe::",
        "egui::",
        "image::",
        "crate::app",
        "crate::panel",
        "crate::ui",
    ];

    for directory in [
        root.join("src/emulation"),
        root.join("src/machines"),
        root.join("src/reference"),
    ] {
        visit_rs(&directory, &mut |path, text| {
            for token in forbidden {
                assert!(
                    !text.contains(token),
                    "{} contains forbidden core dependency `{token}`",
                    path.display()
                );
            }
        });
    }
}

fn visit_rs(directory: &Path, action: &mut dyn FnMut(&Path, &str)) {
    for entry in fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()))
    {
        let path = entry.expect("directory entry must be readable").path();
        if path.is_dir() {
            visit_rs(&path, action);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            let text = fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("cannot read {}: {error}", path.display()));
            action(&path, &text);
        }
    }
}
