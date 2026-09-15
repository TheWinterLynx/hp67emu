//! Runs the first HP-67 power-on instructions from an external normalized ROM corpus.
//!
//! The program does not embed firmware. It loads the locally generated 5120-word
//! corpus, sends each fetch address and ROM response through the structural IS/ISA
//! electrical path, respects the one-word pipeline, executes only source-backed
//! ACT operations, and can optionally continue until the first unsupported word.

use std::{env, fs};

use hp67emu::{
    machines::hp67::{
        run_structural_fetch_cycle, ActFetchEndpoint, FetchPipelineLatch, Hp67ElectricalBackplane,
        Hp67RomWordSource, PowerOnActCore, PowerOnActError, RomFetchEndpoint,
    },
    research::rom_corpus::RomCorpus,
};

const EXPECTED_POPULATED_WORDS: usize = 5120;
const STARTUP_FETCHES: [(u16, u16); 3] = [(0x000, 0x000), (0x001, 0x3e3), (0x0f8, 0x11a)];

struct BankZeroCorpus<'a> {
    corpus: &'a RomCorpus,
}

impl Hp67RomWordSource for BankZeroCorpus<'_> {
    fn read_word(&self, address: u16) -> Option<u16> {
        self.corpus.get(0, usize::from(address)).ok().flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Arguments {
    corpus_path: String,
    probe_cycles: u64,
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut args = env::args().skip(1);
    let corpus_path = args
        .next()
        .unwrap_or_else(|| ".research/teenix-2026-hp67.tsv".to_owned());
    let mut probe_cycles = 0u64;

    while let Some(argument) = args.next() {
        match argument.as_str() {
            "--probe-cycles" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--probe-cycles requires an integer value".to_owned())?;
                probe_cycles = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --probe-cycles value: {value}"))?;
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }

    Ok(Arguments {
        corpus_path,
        probe_cycles,
    })
}

fn execute_cycle(
    cycle: u64,
    act: &mut PowerOnActCore,
    pipeline: &mut FetchPipelineLatch,
) -> Result<bool, String> {
    pipeline.begin_cycle();

    let Some(word) = pipeline.executing_word() else {
        println!("cycle {cycle}: EXEC <pipeline fill>");
        return Ok(true);
    };

    match act.execute_word(word) {
        Ok(execution) => {
            println!(
                "cycle {cycle}: EXEC pc=0x{:03x} word=0x{:03x} {:?} -> pc=0x{:03x}",
                execution.pc, execution.word, execution.operation, execution.next_pc
            );
            Ok(true)
        }
        Err(PowerOnActError::UnsupportedOpcode { pc, word }) => {
            println!(
                "PROBE STOP: unsupported ACT word at cycle {cycle}: pc=0x{pc:03x} word=0x{word:03x} (octal {word:04o})"
            );
            Ok(false)
        }
        Err(error) => Err(format!("cycle {cycle} ACT execution failed: {error:?}")),
    }
}

fn fetch_cycle<S: Hp67RomWordSource>(
    cycle: u64,
    backplane: &mut Hp67ElectricalBackplane,
    act: &PowerOnActCore,
    fetch_act: &mut ActFetchEndpoint,
    fetch_rom: &mut RomFetchEndpoint,
    pipeline: &mut FetchPipelineLatch,
    source: &S,
) -> Result<(u16, u16), String> {
    let address = act.pc();
    let fetched = run_structural_fetch_cycle(backplane, address, fetch_act, fetch_rom, source)
        .map_err(|error| format!("cycle {cycle} serial fetch failed: {error:?}"))?;
    pipeline.complete_cycle(fetched);

    println!(
        "cycle {cycle}: FETCH pc=0x{address:03x} -> word=0x{fetched:03x} (word_index={})",
        backplane.word_index()
    );
    Ok((address, fetched))
}

fn main() -> Result<(), String> {
    let arguments = parse_arguments()?;
    let input = fs::read_to_string(&arguments.corpus_path)
        .map_err(|error| format!("failed to read {}: {error}", arguments.corpus_path))?;
    let corpus = RomCorpus::from_normalized_tsv(&input)
        .map_err(|error| format!("failed to parse normalized ROM corpus: {error}"))?;

    if corpus.populated_words() != EXPECTED_POPULATED_WORDS {
        return Err(format!(
            "corpus has {} populated words; expected {EXPECTED_POPULATED_WORDS}",
            corpus.populated_words()
        ));
    }

    let source = BankZeroCorpus { corpus: &corpus };
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut fetch_act = ActFetchEndpoint::new(0);
    let mut fetch_rom = RomFetchEndpoint::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut act = PowerOnActCore::default();
    let mut verified_fetches = 0usize;
    let mut executed = Vec::new();

    println!("HP-67 real-microcode structural power-on smoke");
    println!(
        "corpus: {} ({} populated words)",
        arguments.corpus_path,
        corpus.populated_words()
    );
    println!("fetch path: ACT b16..b27 -> resolved IS -> ROM b46..b55 -> ACT");

    // Cycle 0 fills the pipeline. Cycles 1..3 execute the three directly
    // observed startup words while concurrently fetching the next word.
    for cycle in 0..4u64 {
        pipeline.begin_cycle();

        if let Some(word) = pipeline.executing_word() {
            let execution = act
                .execute_word(word)
                .map_err(|error| format!("cycle {cycle} ACT execution failed: {error:?}"))?;
            println!(
                "cycle {cycle}: EXEC pc=0x{:03x} word=0x{:03x} {:?} -> pc=0x{:03x}",
                execution.pc, execution.word, execution.operation, execution.next_pc
            );
            executed.push((execution.pc, execution.word));
        } else {
            println!("cycle {cycle}: EXEC <pipeline fill>");
        }

        let (address, fetched) = fetch_cycle(
            cycle,
            &mut backplane,
            &act,
            &mut fetch_act,
            &mut fetch_rom,
            &mut pipeline,
            &source,
        )?;

        if verified_fetches < STARTUP_FETCHES.len() {
            let (expected_address, expected_word) = STARTUP_FETCHES[verified_fetches];
            if address != expected_address || fetched != expected_word {
                return Err(format!(
                    "startup checkpoint {} mismatch: got 0x{address:03x}=0x{fetched:03x}, expected 0x{expected_address:03x}=0x{expected_word:03x}",
                    verified_fetches
                ));
            }
            verified_fetches += 1;
        }
    }

    if executed.as_slice() != STARTUP_FETCHES.as_slice() {
        return Err(format!(
            "executed startup sequence differs: got {executed:?}, expected {STARTUP_FETCHES:?}"
        ));
    }

    if act.pc() != 0x0f9 {
        return Err(format!(
            "startup ACT PC ended at 0x{:03x}; expected 0x0f9 after executing 0 -> c[w]",
            act.pc()
        ));
    }

    println!(
        "PASS: serial microcode power-on reached 0x0f8, executed 0x11a (0 -> c[w]), and advanced to 0x0f9."
    );

    if arguments.probe_cycles == 0 {
        return Ok(());
    }

    println!(
        "PROBE: continuing for up to {} additional machine cycle(s), stopping at the first unsupported ACT word.",
        arguments.probe_cycles
    );

    for cycle in 4..4 + arguments.probe_cycles {
        if !execute_cycle(cycle, &mut act, &mut pipeline)? {
            return Ok(());
        }
        fetch_cycle(
            cycle,
            &mut backplane,
            &act,
            &mut fetch_act,
            &mut fetch_rom,
            &mut pipeline,
            &source,
        )?;
    }

    println!(
        "PROBE LIMIT: completed {} additional machine cycle(s) without reaching an unsupported ACT word.",
        arguments.probe_cycles
    );
    Ok(())
}
