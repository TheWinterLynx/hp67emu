//! Runs HP-67 power-on microcode through the structural serial fetch path.
//!
//! Firmware stays external. Addresses and returned words still cross the
//! resolved IS/ISA model bit by bit, while the independent architectural
//! HP-67 bring-up machine resolves ACT versus CRC ownership at each instruction
//! boundary. This lets one local run cover thousands of real firmware words.

use std::{cell::Cell, env, fs};

use hp67emu::{
    machines::hp67::{
        run_structural_fetch_cycle, ActError, ActFetchEndpoint, FetchPipelineLatch,
        Hp67ArchitecturalError, Hp67ArchitecturalMachine, Hp67ElectricalBackplane,
        Hp67RomWordSource, RomFetchEndpoint,
    },
    research::rom_corpus::{RomCorpus, ROM_PAGES, WORDS_PER_PAGE},
};

const EXPECTED_POPULATED_WORDS: usize = 5120;
const STARTUP_FETCHES: [(u16, u16); 3] = [(0x000, 0x000), (0x001, 0x3e3), (0x0f8, 0x11a)];

struct CorpusRom<'a> {
    corpus: &'a RomCorpus,
    requested_bank: Cell<u8>,
    page_bank_mask: [u8; ROM_PAGES],
}

impl<'a> CorpusRom<'a> {
    fn new(corpus: &'a RomCorpus) -> Self {
        let mut page_bank_mask = [0u8; ROM_PAGES];
        for (page, mask) in page_bank_mask.iter_mut().enumerate() {
            let page_base = page * WORDS_PER_PAGE;
            for bank in 0..2usize {
                if (page_base..page_base + WORDS_PER_PAGE)
                    .any(|pc| corpus.get(bank, pc).ok().flatten().is_some())
                {
                    *mask |= 1u8 << bank;
                }
            }
        }
        Self {
            corpus,
            requested_bank: Cell::new(0),
            page_bank_mask,
        }
    }

    fn select_bank(&self, bank: u8) {
        self.requested_bank.set(bank & 1);
    }
}

impl Hp67RomWordSource for CorpusRom<'_> {
    fn read_word(&self, address: u16) -> Option<u16> {
        let requested = usize::from(self.requested_bank.get());
        let page = usize::from(address) / WORDS_PER_PAGE;
        let effective = if self.page_bank_mask[page] & (1u8 << requested) != 0 {
            requested
        } else {
            0
        };
        self.corpus.get(effective, usize::from(address)).ok().flatten()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Arguments {
    corpus_path: String,
    probe_cycles: u64,
    trace_limit: u64,
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut args = env::args().skip(1);
    let corpus_path = args
        .next()
        .unwrap_or_else(|| ".research/teenix-2026-hp67.tsv".to_owned());
    let mut probe_cycles = 0u64;
    let mut trace_limit = 64u64;

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
            "--trace-limit" => {
                let value = args
                    .next()
                    .ok_or_else(|| "--trace-limit requires an integer value".to_owned())?;
                trace_limit = value
                    .parse::<u64>()
                    .map_err(|_| format!("invalid --trace-limit value: {value}"))?;
            }
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }

    Ok(Arguments {
        corpus_path,
        probe_cycles,
        trace_limit,
    })
}

fn execute_cycle(
    cycle: u64,
    machine: &mut Hp67ArchitecturalMachine,
    pipeline: &mut FetchPipelineLatch,
    verbose: bool,
) -> Result<bool, String> {
    pipeline.begin_cycle();

    let Some(word) = pipeline.executing_word() else {
        if verbose {
            println!("cycle {cycle}: EXEC <pipeline fill>");
        }
        return Ok(true);
    };

    match machine.execute_word(word) {
        Ok(execution) => {
            if verbose {
                println!(
                    "cycle {cycle}: EXEC pc=0x{:03x} word=0x{:03x} {:?} -> pc=0x{:03x}",
                    execution.pc, execution.word, execution.operation, execution.next_pc
                );
            }
            Ok(true)
        }
        Err(Hp67ArchitecturalError::Act(ActError::UnknownSpecial { pc, word })) => {
            println!(
                "PROBE STOP: unknown ACT special at cycle {cycle}: pc=0x{pc:03x} word=0x{word:03x} (octal {word:04o})"
            );
            Ok(false)
        }
        Err(Hp67ArchitecturalError::Act(ActError::UnsupportedRomSelfTest { pc })) => {
            println!("PROBE STOP: ROM self-test reached at cycle {cycle}: pc=0x{pc:03x}");
            Ok(false)
        }
        Err(Hp67ArchitecturalError::CrcDataPortNotModeled { pc, address, write }) => {
            let direction = if write { "write" } else { "read" };
            println!(
                "PROBE STOP: CRC DATA-port {direction} reached at cycle {cycle}: pc=0x{pc:03x} address=0x{address:02x}"
            );
            Ok(false)
        }
        Err(error) => Err(format!("cycle {cycle} architectural execution failed: {error:?}")),
    }
}

fn fetch_cycle(
    cycle: u64,
    backplane: &mut Hp67ElectricalBackplane,
    machine: &mut Hp67ArchitecturalMachine,
    fetch_act: &mut ActFetchEndpoint,
    fetch_rom: &mut RomFetchEndpoint,
    pipeline: &mut FetchPipelineLatch,
    source: &CorpusRom<'_>,
    verbose: bool,
) -> Result<(u16, u16), String> {
    let requested_bank = machine.prepare_hp67_fetch();
    source.select_bank(requested_bank);
    let address = machine.pc();
    let fetched = run_structural_fetch_cycle(backplane, address, fetch_act, fetch_rom, source)
        .map_err(|error| format!("cycle {cycle} serial fetch failed: {error:?}"))?;
    pipeline.complete_cycle(fetched);

    if verbose {
        println!(
            "cycle {cycle}: FETCH bank={} pc=0x{address:03x} -> word=0x{fetched:03x} (word_index={})",
            machine.bank(),
            backplane.word_index()
        );
    }
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

    let source = CorpusRom::new(&corpus);
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut fetch_act = ActFetchEndpoint::new(0);
    let mut fetch_rom = RomFetchEndpoint::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut verified_fetches = 0usize;
    let mut executed = Vec::new();

    println!("HP-67 real-microcode structural power-on smoke");
    println!(
        "corpus: {} ({} populated words)",
        arguments.corpus_path,
        corpus.populated_words()
    );
    println!("fetch path: ACT b16..b27 -> resolved IS -> ROM b46..b55 -> ACT");
    println!("machine path: independent ACT + CRC control architectural composition");

    for cycle in 0..4u64 {
        pipeline.begin_cycle();

        if let Some(word) = pipeline.executing_word() {
            let execution = machine
                .execute_word(word)
                .map_err(|error| format!("cycle {cycle} execution failed: {error:?}"))?;
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
            &mut machine,
            &mut fetch_act,
            &mut fetch_rom,
            &mut pipeline,
            &source,
            true,
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

    if machine.pc() != 0x0f9 {
        return Err(format!(
            "startup ACT PC ended at 0x{:03x}; expected 0x0f9 after executing 0 -> c[w]",
            machine.pc()
        ));
    }

    println!(
        "PASS: serial microcode power-on reached 0x0f8, executed 0x11a, and advanced to 0x0f9."
    );

    if arguments.probe_cycles == 0 {
        return Ok(());
    }

    println!(
        "PROBE: running up to {} additional machine cycle(s); detailed trace limited to {} cycle(s).",
        arguments.probe_cycles, arguments.trace_limit
    );

    let mut completed_probe_cycles = 0u64;
    for offset in 0..arguments.probe_cycles {
        let cycle = 4 + offset;
        let verbose = offset < arguments.trace_limit;
        if !execute_cycle(cycle, &mut machine, &mut pipeline, verbose)? {
            println!(
                "PROBE SUMMARY: completed {completed_probe_cycles} additional cycles; executed_words={}; pc=0x{:03x}; bank={}",
                machine.executed_words(),
                machine.pc(),
                machine.bank()
            );
            return Ok(());
        }
        fetch_cycle(
            cycle,
            &mut backplane,
            &mut machine,
            &mut fetch_act,
            &mut fetch_rom,
            &mut pipeline,
            &source,
            verbose,
        )?;
        completed_probe_cycles += 1;
    }

    println!(
        "PROBE LIMIT: completed {} additional machine cycle(s) without an unsupported architectural/hardware boundary; executed_words={}; pc=0x{:03x}; bank={}.",
        completed_probe_cycles,
        machine.executed_words(),
        machine.pc(),
        machine.bank()
    );
    Ok(())
}
