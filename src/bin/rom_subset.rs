//! Verify that every populated word in one normalized ROM corpus is present
//! and identical in another corpus.
//!
//! This is primarily useful when comparing the physically populated 5120-word
//! HP-67 Teenix image against a software corpus such as x11-calc that exposes a
//! full 8192-entry two-bank address space. Locations populated only by the
//! reference/right corpus are reported but do not make the subset check fail.

use std::{env, error::Error, fs, process};

use hp67emu::research::rom_corpus::{compare, RomCorpus, RomDifference};

const MAX_PRINTED_FAILURES: usize = 128;

fn main() {
    match run() {
        Ok(0) => {}
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("rom_subset: {error}");
            process::exit(2);
        }
    }
}

fn run() -> Result<i32, Box<dyn Error>> {
    let args = env::args().collect::<Vec<_>>();
    if args.len() != 3 {
        eprintln!("Usage: rom_subset <subset.tsv> <reference.tsv>");
        return Ok(2);
    }

    let subset = RomCorpus::from_normalized_tsv(&fs::read_to_string(&args[1])?)?;
    let reference = RomCorpus::from_normalized_tsv(&fs::read_to_string(&args[2])?)?;
    let comparison = compare(&subset, &reference);

    let mut mismatches = 0usize;
    let mut missing_reference = 0usize;
    let mut reference_only = 0usize;
    let mut failures = Vec::new();

    for difference in &comparison.differences {
        match difference {
            RomDifference::Mismatch { .. } => {
                mismatches += 1;
                failures.push(*difference);
            }
            RomDifference::MissingRight { .. } => {
                missing_reference += 1;
                failures.push(*difference);
            }
            RomDifference::MissingLeft { .. } => reference_only += 1,
        }
    }

    println!("subset   : {} ({} populated words)", args[1], subset.populated_words());
    println!("reference: {} ({} populated words)", args[2], reference.populated_words());
    println!("matching subset words : {}", comparison.matching_words());
    println!("value mismatches      : {mismatches}");
    println!("missing in reference  : {missing_reference}");
    println!("reference-only words  : {reference_only} (informational)");

    for difference in failures.iter().take(MAX_PRINTED_FAILURES) {
        print_failure(*difference);
    }
    if failures.len() > MAX_PRINTED_FAILURES {
        println!(
            "... {} additional subset failure(s) omitted",
            failures.len() - MAX_PRINTED_FAILURES
        );
    }

    if failures.is_empty() {
        println!(
            "\nSUBSET MATCH: all {} populated subset words exist and match exactly.",
            subset.populated_words()
        );
        Ok(0)
    } else {
        println!("\nSUBSET MISMATCH: {} populated subset location(s) disagree or are missing.", failures.len());
        Ok(1)
    }
}

fn print_failure(difference: RomDifference) {
    let location = difference.location();
    let prefix = format!(
        "bank {} page {} pc 0x{:03x} (@{:04o})",
        location.bank, location.page, location.pc, location.pc
    );
    match difference {
        RomDifference::Mismatch { left, right, .. } => println!(
            "{prefix}: subset=0x{left:03x} ({left:04o}) reference=0x{right:03x} ({right:04o})"
        ),
        RomDifference::MissingRight { left, .. } => println!(
            "{prefix}: subset=0x{left:03x} ({left:04o}) reference=missing"
        ),
        RomDifference::MissingLeft { .. } => {
            unreachable!("reference-only differences are informational, not failures")
        }
    }
}
