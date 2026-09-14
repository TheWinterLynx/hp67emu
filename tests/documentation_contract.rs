//! Regression contract for source documentation.

use std::{
    fs,
    path::{Path, PathBuf},
};

const REQUIRED_SECTIONS: &[&str] = &[
    "## Purpose",
    "## Why it exists",
    "## Relationships",
    "## Responsibilities",
    "## Implementation",
];

#[test]
fn every_rust_source_has_a_companion_markdown_document() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let mut sources = Vec::new();
    collect_rust_files(&root.join("src"), &mut sources);
    collect_rust_files(&root.join("tests"), &mut sources);
    sources.sort();

    assert!(!sources.is_empty(), "documentation contract found no Rust files");

    let mut failures = Vec::new();
    for source in sources {
        let relative = source
            .strip_prefix(&root)
            .expect("source path must be inside repository");
        let normalized = relative.to_string_lossy().replace('\\', "/");
        let doc = root.join("docs/files").join(format!("{normalized}.md"));

        if !doc.is_file() {
            failures.push(format!("missing {}", doc.display()));
            continue;
        }

        let text = fs::read_to_string(&doc)
            .unwrap_or_else(|error| panic!("cannot read {}: {error}", doc.display()));
        for section in REQUIRED_SECTIONS {
            if !text.contains(section) {
                failures.push(format!("{} is missing `{section}`", doc.display()));
            }
        }
    }

    assert!(
        failures.is_empty(),
        "documentation contract failed:\n{}",
        failures.join("\n")
    );
}

#[test]
fn architecture_documents_are_present() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    for relative in [
        "docs/ARCHITECTURE.md",
        "docs/ROADMAP.md",
        "docs/HARDWARE_SOURCES.md",
        "docs/DOCUMENTATION_POLICY.md",
        "docs/ASSETS.md",
        "docs/CI.md",
        "TODO.md",
    ] {
        assert!(root.join(relative).is_file(), "missing required {relative}");
    }
}

fn collect_rust_files(directory: &Path, out: &mut Vec<PathBuf>) {
    let entries = fs::read_dir(directory)
        .unwrap_or_else(|error| panic!("cannot read {}: {error}", directory.display()));
    for entry in entries {
        let path = entry.expect("directory entry must be readable").path();
        if path.is_dir() {
            collect_rust_files(&path, out);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            out.push(path);
        }
    }
}
