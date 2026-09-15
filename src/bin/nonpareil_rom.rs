//! Normalize one or more official Nonpareil `uasm` Woodstock object files.
//!
//! This developer tool contains no bundled firmware. It imports locally built
//! Nonpareil object files, merges non-conflicting bank/address records and writes
//! hp67emu's normalized sparse TSV corpus for independent comparison.

use std::{env, error::Error, fs, io, process};

use hp67emu::research::{
    nonpareil_obj::parse_nonpareil_woodstock_obj,
    rom_corpus::{RomCorpus, ROM_BANKS, WORDS_PER_BANK},
};

fn main() {
    if let Err(error) = run() {
        eprintln!("nonpareil_rom: {error}");
        process::exit(2);
    }
}

fn run() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().collect();
    if args.len() < 3 {
        eprintln!("Usage: nonpareil_rom <out.tsv> <input1.obj> [input2.obj ...]");
        process::exit(2);
    }

    let output_path = &args[1];
    let mut merged = RomCorpus::default();

    for input_path in &args[2..] {
        let text = fs::read_to_string(input_path)?;
        let incoming = parse_nonpareil_woodstock_obj(&text)?;
        println!(
            "{}: {} populated Woodstock word(s)",
            input_path,
            incoming.populated_words()
        );

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
                                "conflicting Nonpareil word at bank {bank}, PC 0x{pc:03x}: merged=0x{existing:03x}, {input_path}=0x{word:03x}"
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
        "normalized {} non-conflicting Nonpareil HP-67 word(s) -> {}",
        merged.populated_words(),
        output_path
    );
    Ok(())
}
