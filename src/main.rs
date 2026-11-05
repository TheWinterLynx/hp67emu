#![cfg_attr(
    all(not(debug_assertions), not(test), target_os = "windows"),
    windows_subsystem = "windows"
)]

mod app;
mod hp67;
mod panel;
#[cfg(not(target_arch = "wasm32"))]
mod pin_trace_cli;
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
        let content = pin_trace_cli::read_text_limited(path, MAX_TRACE_BYTES)?;
        import_bus_trace(&content).map_err(|error| format!("{path}: {error}"))
    };
    let (expected_start, actual_start) = if args.len() == 6 {
        (
            args[4]
                .parse::<usize>()
                .map_err(|_| "invalid expected start")?,
            args[5]
                .parse::<usize>()
                .map_err(|_| "invalid actual start")?,
        )
    } else {
        (0, 0)
    };
    let result = compare_bus_traces(
        &read(&args[1])?,
        &read(&args[2])?,
        expected_start,
        actual_start,
    )?;
    let report = format!(
        "expected_file={}\nactual_file={}\n{}",
        args[1], args[2], result.report
    );
    pin_trace_cli::write_new(&args[3], &report)
        .map_err(|error| format!("comparison report failed: {error}"))?;
    Ok(result.matches)
}

#[cfg(not(target_arch = "wasm32"))]
fn decode_trace_file(args: &[String]) -> Result<(), String> {
    use hp67emu::machines::hp67::bus_trace::{
        decode_bus_report, import_bus_trace, MAX_TRACE_BYTES,
    };
    if args.len() != 3 && args.len() != 5 {
        return Err(
            "usage: hp67emu --decode-is <input.tsv> <report.txt> [start_row words]".to_owned(),
        );
    }
    let input = pin_trace_cli::read_text_limited(&args[1], MAX_TRACE_BYTES)?;
    let trace = import_bus_trace(&input)?;
    let (start, words) = if args.len() == 5 {
        (
            args[3].parse::<usize>().map_err(|_| "invalid start row")?,
            args[4].parse::<usize>().map_err(|_| "invalid word count")?,
        )
    } else {
        if trace.rows.len() % 224 != 0 {
            return Err("partial trace: select explicit complete-word range".to_owned());
        }
        (0, trace.rows.len() / 224)
    };
    let report = format!(
        "input_file={}\n{}",
        args[1],
        decode_bus_report(&trace, start, words)?
    );
    pin_trace_cli::write_new(&args[2], &report).map_err(|error| format!("decode report: {error}"))
}

#[cfg(not(target_arch = "wasm32"))]
fn main() -> eframe::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if pin_trace_cli::handles(args.first().map(String::as_str)) {
        match pin_trace_cli::run(&args) {
            Ok(0) => return Ok(()),
            Ok(code) => std::process::exit(code),
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    if args.first().map(String::as_str) == Some("--trace-is") {
        let result = (|| -> Result<(), String> {
            if args.len() != 3 {
                return Err("usage: hp67emu --trace-is <1..4096 words> <output.tsv>".to_owned());
            }
            let words = args[1]
                .parse::<usize>()
                .map_err(|_| "invalid trace word count".to_owned())?;
            let output = hp67::capture_live_bus_tsv(words)?;
            pin_trace_cli::write_new(&args[2], &output)
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
            Err(error) => {
                eprintln!("{error}");
                std::process::exit(1);
            }
        }
    }
    if args.first().map(String::as_str) == Some("--decode-is") {
        if let Err(error) = decode_trace_file(&args) {
            eprintln!("{error}");
            std::process::exit(1);
        }
        return Ok(());
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

#[cfg(all(test, not(target_arch = "wasm32")))]
mod core_audit_trace_files {
    use super::*;
    struct Temporary(std::path::PathBuf);
    impl Drop for Temporary {
        fn drop(&mut self) {
            let _ = std::fs::remove_dir_all(&self.0);
        }
    }
    #[test]
    fn core_audit_is_reports_preserve_existing_files_and_hard_linked_inputs() {
        let temp = Temporary(
            std::env::temp_dir().join(format!("hp67-core-is-write-{}", std::process::id())),
        );
        std::fs::create_dir(&temp.0).unwrap();
        let input = temp.0.join("input.tsv");
        let alias = temp.0.join("alias.tsv");
        let report = temp.0.join("report.txt");
        let original = hp67::capture_live_bus_tsv(16).unwrap();
        std::fs::write(&input, &original).unwrap();
        std::fs::hard_link(&input, &alias).unwrap();
        let input = input.to_str().unwrap().to_owned();
        let alias = alias.to_str().unwrap().to_owned();
        let report = report.to_str().unwrap().to_owned();
        for output in [&input, &alias] {
            assert!(
                decode_trace_file(&["--decode-is".into(), input.clone(), output.clone()]).is_err()
            );
            assert!(compare_trace_files(&[
                "--compare-is".into(),
                input.clone(),
                input.clone(),
                output.clone()
            ])
            .is_err());
            assert_eq!(std::fs::read_to_string(&input).unwrap(), original);
        }
        assert!(compare_trace_files(&[
            "--compare-is".into(),
            input.clone(),
            input.clone(),
            report.clone()
        ])
        .unwrap());
        let saved = std::fs::read(&report).unwrap();
        assert!(decode_trace_file(&["--decode-is".into(), input.clone(), report.clone()]).is_err());
        assert_eq!(std::fs::read(&report).unwrap(), saved);
        let decoded = temp.0.join("decoded.txt").to_str().unwrap().to_owned();
        decode_trace_file(&["--decode-is".into(), input, decoded]).unwrap();
    }
    #[test]
    fn core_audit_trace_reader_enforces_limit_on_open_stream() {
        let temp = Temporary(
            std::env::temp_dir().join(format!("hp67-core-is-limit-{}", std::process::id())),
        );
        std::fs::create_dir(&temp.0).unwrap();
        let file = temp.0.join("input");
        std::fs::write(&file, b"123456789").unwrap();
        let path = file.to_str().unwrap();
        assert!(pin_trace_cli::read_text_limited(path, 8).is_err());
        assert_eq!(
            pin_trace_cli::read_text_limited(path, 9).unwrap(),
            "123456789"
        );
    }
}
