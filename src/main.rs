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
fn compare_trace_files(args: &[String]) -> Result<bool, String> {
    use hp67emu::machines::hp67::bus_trace::{
        compare_bus_traces, import_bus_trace, MAX_TRACE_BYTES,
    };
    if args.len() != 4 && args.len() != 6 {
        return Err("usage: hp67emu --compare-is <expected.tsv> <actual.tsv> <report.txt> [expected_start actual_start]".to_owned());
    }
    let read = |path: &str| -> Result<_, String> {
        let size = std::fs::metadata(path).map_err(|error| format!("{path}: {error}"))?.len();
        if size > MAX_TRACE_BYTES as u64 { return Err(format!("{path}: trace exceeds byte limit")); }
        let content = std::fs::read_to_string(path).map_err(|error| format!("{path}: {error}"))?;
        import_bus_trace(&content).map_err(|error| format!("{path}: {error}"))
    };
    if let Ok(output) = std::fs::canonicalize(&args[3]) {
        for path in [&args[1], &args[2]] {
            if std::fs::canonicalize(path).ok().as_ref() == Some(&output) {
                return Err("report must not overwrite either input trace".to_owned());
            }
        }
    }
    let (expected_start, actual_start) = if args.len() == 6 {
        (args[4].parse::<usize>().map_err(|_| "invalid expected start")?,
         args[5].parse::<usize>().map_err(|_| "invalid actual start")?)
    } else { (0, 0) };
    let result = compare_bus_traces(&read(&args[1])?, &read(&args[2])?, expected_start, actual_start)?;
    let report = format!("expected_file={}\nactual_file={}\n{}", args[1], args[2], result.report);
    std::fs::write(&args[3], report).map_err(|error| format!("comparison report failed: {error}"))?;
    Ok(result.matches)
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.first().map(String::as_str) == Some("--trace-is") {
        let result = (|| -> Result<(), String> {
            if args.len() != 3 {
                return Err("usage: hp67emu --trace-is <1..4096 words> <output.tsv>".to_owned());
            }
            let words = args[1]
                .parse::<usize>()
                .map_err(|_| "invalid trace word count".to_owned())?;
            let output = hp67::capture_live_bus_tsv(words)?;
            std::fs::write(&args[2], output)
                .map_err(|error| format!("trace export failed: {error}"))?;
            Ok(())
        })();
        if let Err(error) = result {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
    }
    if args.first().map(String::as_str) == Some("--compare-is") {
        match compare_trace_files(&args) {
            Ok(true) => return Ok(()),
            Ok(false) => std::process::exit(2),
            Err(error) => { eprintln!("{error}"); std::process::exit(1); }
        }
    }
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
