//! Local command-line tool for normalizing and comparing HP-67 ROM corpora.
//!
//! This binary intentionally contains no bundled firmware. It consumes files
//! supplied by the developer and emits a stable hp67emu TSV representation that
//! can be compared at bank/page/address granularity.

use std::{env, error::Error, fs, path::Path, process};

use hp67emu::research::rom_corpus::{
    compare, parse_address_opcode_pairs, parse_x11_hp67_c, NumberRadix, RomCorpus, RomDifference,
    ROM_BANKS, ROM_PAGES,
};

const MAX_PRINTED_DIFFERENCES: usize = 128;

fn main() {
    match run() {
        Ok(0) => {}
        Ok(code) => process::exit(code),
        Err(error) => {
            eprintln!("rom_compare: {error}");
            process::exit(2);
        }
    }
}

fn run() -> Result<i32, Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        return Ok(2);
    }

    match args[1].as_str() {
        "extract-x11" if args.len() == 4 => {
            let source = fs::read_to_string(&args[2])?;
            let corpus = parse_x11_hp67_c(&source)?;
            fs::write(&args[3], corpus.to_normalized_tsv())?;
            println!(
                "extracted {} HP-67 ROM words from x11-calc -> {}",
                corpus.populated_words(),
                args[3]
            );
            Ok(0)
        }
        "import-pairs" if args.len() == 6 => {
            let input = fs::read_to_string(&args[2])?;
            let bank: usize = args[4].parse()?;
            let radix = NumberRadix::parse(&args[5])?;
            let corpus = parse_address_opcode_pairs(&input, bank, radix)?;
            fs::write(&args[3], corpus.to_normalized_tsv())?;
            println!(
                "normalized {} ROM words into bank {} -> {}",
                corpus.populated_words(),
                bank,
                args[3]
            );
            Ok(0)
        }
        "compare" if args.len() == 4 => compare_files(&args[2], &args[3]),
        "inspect" if args.len() == 3 => {
            inspect_binary(Path::new(&args[2]))?;
            Ok(0)
        }
        _ => {
            print_usage();
            Ok(2)
        }
    }
}

fn compare_files(left_path: &str, right_path: &str) -> Result<i32, Box<dyn Error>> {
    let left = RomCorpus::from_normalized_tsv(&fs::read_to_string(left_path)?)?;
    let right = RomCorpus::from_normalized_tsv(&fs::read_to_string(right_path)?)?;
    let result = compare(&left, &right);

    println!("left : {left_path} ({} words)", left.populated_words());
    println!("right: {right_path} ({} words)", right.populated_words());
    println!();
    println!("bank page | left right | match mismatch left-only right-only");
    for bank in 0..ROM_BANKS {
        for page in 0..ROM_PAGES {
            let summary = result.pages[bank][page];
            println!(
                "  {bank}    {page}  | {:4}  {:4} | {:4}    {:4}       {:4}       {:4}",
                summary.left_words,
                summary.right_words,
                summary.matches,
                summary.mismatches,
                summary.left_only,
                summary.right_only
            );
        }
    }

    if result.is_identical() {
        println!("\nIDENTICAL: {} populated words match exactly.", result.matching_words());
        return Ok(0);
    }

    println!("\nDIFFERENCES: {}", result.differences.len());
    for difference in result.differences.iter().take(MAX_PRINTED_DIFFERENCES) {
        print_difference(*difference);
    }
    if result.differences.len() > MAX_PRINTED_DIFFERENCES {
        println!(
            "... {} additional differences omitted",
            result.differences.len() - MAX_PRINTED_DIFFERENCES
        );
    }
    Ok(1)
}

fn print_difference(difference: RomDifference) {
    let location = difference.location();
    let prefix = format!(
        "bank {} page {} pc 0x{:03x} (@{:04o})",
        location.bank, location.page, location.pc, location.pc
    );
    match difference {
        RomDifference::MissingLeft { right, .. } => {
            println!("{prefix}: left=missing right=0x{right:03x} ({right:04o})");
        }
        RomDifference::MissingRight { left, .. } => {
            println!("{prefix}: left=0x{left:03x} ({left:04o}) right=missing");
        }
        RomDifference::Mismatch { left, right, .. } => {
            println!(
                "{prefix}: left=0x{left:03x} ({left:04o}) right=0x{right:03x} ({right:04o})"
            );
        }
    }
}

fn inspect_binary(path: &Path) -> Result<(), Box<dyn Error>> {
    let bytes = fs::read(path)?;
    println!("file: {}", path.display());
    println!("size: {} bytes", bytes.len());
    println!("even byte count: {}", bytes.len() % 2 == 0);
    if bytes.len() % 2 == 0 {
        println!("16-bit word count if interpreted as raw u16: {}", bytes.len() / 2);
    }
    let preview_len = bytes.len().min(32);
    let preview = bytes[..preview_len]
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect::<Vec<_>>()
        .join(" ");
    println!("first {preview_len} bytes: {preview}");
    println!("No binary format is inferred automatically; .pfl layout remains unproven.");
    Ok(())
}

fn print_usage() {
    eprintln!(
        "Usage:\n  \
rom_compare extract-x11 <x11-calc-67.c> <out.tsv>\n  \
rom_compare import-pairs <input.txt> <out.tsv> <bank> <auto|8|10|16>\n  \
rom_compare compare <left.tsv> <right.tsv>\n  \
rom_compare inspect <binary-file>"
    );
}
