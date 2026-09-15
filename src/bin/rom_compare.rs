//! Local command-line tool for normalizing and comparing HP-67 ROM corpora.
//!
//! This binary intentionally contains no bundled firmware. It consumes files
//! supplied by the developer and emits a stable hp67emu TSV representation that
//! can be compared at bank/page/address granularity.

use std::{env, error::Error, fs, io, path::Path, process};

use hp67emu::research::rom_corpus::{
    compare, parse_address_opcode_pairs, parse_x11_hp67_c, NumberRadix, RomCorpus, RomDifference,
    ROM_BANKS, ROM_PAGES, WORDS_PER_BANK,
};
use hp67emu::research::teenix::{decode_container, has_newe_signature};

const MAX_PRINTED_DIFFERENCES: usize = 128;
const DEFAULT_TEENIX_PREVIEW_LINES: usize = 60;

/// Three words observed by Tony Nixon on a physical HP-67 immediately after
/// power-on while monitoring SYNC and IS. These are evidence points, not an
/// embedded firmware image.
const PHYSICAL_STARTUP_WORDS: &[(usize, usize, u16)] = &[
    (0, 0x000, 0x000),
    (0, 0x001, 0x3e3),
    (0, 0x0f8, 0x11a),
];

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
        "merge" if args.len() >= 5 => merge_files(&args[2], &args[3..]),
        "compare" if args.len() == 4 => compare_files(&args[2], &args[3]),
        "verify-startup" if args.len() == 3 => verify_startup(&args[2]),
        "decode-teenix" if args.len() == 4 => decode_teenix_file(&args[2], &args[3]),
        "preview-teenix" if args.len() == 3 || args.len() == 4 => {
            let line_count = if args.len() == 4 {
                args[3].parse::<usize>()?
            } else {
                DEFAULT_TEENIX_PREVIEW_LINES
            };
            preview_teenix_file(&args[2], line_count)
        }
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

fn decode_teenix_file(input_path: &str, output_path: &str) -> Result<i32, Box<dyn Error>> {
    let bytes = fs::read(input_path)?;
    let container = decode_container(&bytes)?;
    fs::write(output_path, container.payload())?;
    println!("decoded Teenix NeWe container: {input_path}");
    println!("declared payload bytes: {}", container.declared_payload_len);
    println!("decoded payload written to: {output_path}");
    Ok(0)
}

fn preview_teenix_file(path: &str, line_count: usize) -> Result<i32, Box<dyn Error>> {
    let bytes = fs::read(path)?;
    let container = decode_container(&bytes)?;
    let text = container.payload_text()?;

    println!("Teenix NeWe container: {path}");
    println!("declared payload bytes: {}", container.declared_payload_len);
    println!("payload lines: {}", text.lines().count());
    println!("--- first {line_count} lines ---");
    for (index, line) in text.lines().take(line_count).enumerate() {
        println!("{:05}: {line}", index + 1);
    }
    Ok(0)
}

fn merge_files(output_path: &str, input_paths: &[String]) -> Result<i32, Box<dyn Error>> {
    let mut merged = RomCorpus::default();
    for input_path in input_paths {
        let incoming = RomCorpus::from_normalized_tsv(&fs::read_to_string(input_path)?)?;
        for bank in 0..ROM_BANKS {
            for pc in 0..WORDS_PER_BANK {
                let Some(word) = incoming.get(bank, pc)? else {
                    continue;
                };
                match merged.get(bank, pc)? {
                    None => merged.set(bank, pc, word)?,
                    Some(existing) if existing == word => {}
                    Some(existing) => {
                        return Err(io::Error::new(
                            io::ErrorKind::InvalidData,
                            format!(
                                "conflicting ROM word at bank {bank}, PC 0x{pc:03x}: existing 0x{existing:03x}, {} has 0x{word:03x}",
                                input_path
                            ),
                        )
                        .into());
                    }
                }
            }
        }
    }

    fs::write(output_path, merged.to_normalized_tsv())?;
    println!(
        "merged {} non-conflicting ROM words from {} corpora -> {}",
        merged.populated_words(),
        input_paths.len(),
        output_path
    );
    Ok(0)
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

fn verify_startup(path: &str) -> Result<i32, Box<dyn Error>> {
    let corpus = RomCorpus::from_normalized_tsv(&fs::read_to_string(path)?)?;
    let mut failures = 0usize;

    println!("physical HP-67 startup evidence check: {path}");
    for &(bank, pc, expected) in PHYSICAL_STARTUP_WORDS {
        match corpus.get(bank, pc)? {
            Some(actual) if actual == expected => {
                println!(
                    "PASS bank {bank} pc 0x{pc:03x} (@{pc:04o}) = 0x{actual:03x} ({actual:04o})"
                );
            }
            Some(actual) => {
                failures += 1;
                println!(
                    "FAIL bank {bank} pc 0x{pc:03x} (@{pc:04o}): expected 0x{expected:03x} ({expected:04o}), got 0x{actual:03x} ({actual:04o})"
                );
            }
            None => {
                failures += 1;
                println!(
                    "FAIL bank {bank} pc 0x{pc:03x} (@{pc:04o}): expected 0x{expected:03x} ({expected:04o}), location missing"
                );
            }
        }
    }

    if failures == 0 {
        println!("\nMATCH: all physical startup evidence points agree.");
        Ok(0)
    } else {
        println!("\nMISMATCH: {failures} physical startup evidence point(s) disagree or are missing.");
        Ok(1)
    }
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

    if has_newe_signature(&bytes) {
        println!("Teenix NeWe XOR-0x55 signature: YES");
        match decode_container(&bytes) {
            Ok(container) => {
                println!("Teenix container validation: PASS");
                println!("declared payload bytes: {}", container.declared_payload_len);
                if let Ok(text) = container.payload_text() {
                    println!("decoded payload lines: {}", text.lines().count());
                    if let Some(first_line) = text.lines().next() {
                        println!("decoded first payload line: {first_line}");
                    }
                }
            }
            Err(error) => println!("Teenix container validation: FAIL ({error})"),
        }
    } else {
        println!("Teenix NeWe XOR-0x55 signature: no");
    }

    if !bytes.is_empty() {
        let mut counts = [0usize; 256];
        for &byte in &bytes {
            counts[byte as usize] += 1;
        }
        let ascii_printable = bytes
            .iter()
            .filter(|&&byte| {
                byte == b'\n' || byte == b'\r' || byte == b'\t' || (0x20..=0x7e).contains(&byte)
            })
            .count();
        let seven_bit = bytes.iter().filter(|&&byte| byte < 0x80).count();
        let unique = counts.iter().filter(|&&count| count != 0).count();
        let len = bytes.len() as f64;
        let entropy = counts
            .iter()
            .filter(|&&count| count != 0)
            .map(|&count| {
                let p = count as f64 / len;
                -p * p.log2()
            })
            .sum::<f64>();

        let mut common = counts
            .iter()
            .enumerate()
            .filter(|(_, count)| **count != 0)
            .map(|(byte, &count)| (count, byte))
            .collect::<Vec<_>>();
        common.sort_unstable_by(|left, right| right.cmp(left));
        let common = common
            .into_iter()
            .take(8)
            .map(|(count, byte)| format!("0x{byte:02x}:{count}"))
            .collect::<Vec<_>>()
            .join(" ");

        println!("7-bit bytes: {:.2}%", seven_bit as f64 * 100.0 / len);
        println!("printable/text bytes: {:.2}%", ascii_printable as f64 * 100.0 / len);
        println!("unique byte values: {unique}/256");
        println!("Shannon entropy: {entropy:.3} bits/byte");
        println!("most common bytes: {common}");
    }

    Ok(())
}

fn print_usage() {
    eprintln!(
        "Usage:\n  \
rom_compare extract-x11 <x11-calc-67.c> <out.tsv>\n  \
rom_compare import-pairs <input.txt> <out.tsv> <bank> <auto|8|10|16>\n  \
rom_compare merge <out.tsv> <input1.tsv> <input2.tsv> [more.tsv ...]\n  \
rom_compare compare <left.tsv> <right.tsv>\n  \
rom_compare verify-startup <corpus.tsv>\n  \
rom_compare decode-teenix <input.pfl> <out.txt>\n  \
rom_compare preview-teenix <input.pfl> [line-count]\n  \
rom_compare inspect <binary-file>"
    );
}
