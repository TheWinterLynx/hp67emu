//! Runs the first HP-67 power-on instructions from an external normalized ROM corpus.
//!
//! The program does not embed firmware. It loads the locally generated 5120-word
//! corpus, sends each fetch address and ROM response through the structural IS/ISA
//! electrical path, respects the one-word pipeline, and executes only the minimal
//! ACT operations required by the directly observed startup trace.

use std::{env, fs};

use hp67emu::{
    machines::hp67::{
        run_structural_fetch_cycle, ActFetchEndpoint, FetchPipelineLatch, Hp67ElectricalBackplane,
        Hp67RomWordSource, PowerOnActCore, RomFetchEndpoint,
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

fn main() -> Result<(), String> {
    let corpus_path = env::args()
        .nth(1)
        .unwrap_or_else(|| ".research/teenix-2026-hp67.tsv".to_owned());
    let input = fs::read_to_string(&corpus_path)
        .map_err(|error| format!("failed to read {corpus_path}: {error}"))?;
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
    println!("corpus: {corpus_path} ({} populated words)", corpus.populated_words());
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

        let address = act.pc();
        let fetched = run_structural_fetch_cycle(
            &mut backplane,
            address,
            &mut fetch_act,
            &mut fetch_rom,
            &source,
        )
        .map_err(|error| format!("cycle {cycle} serial fetch failed: {error:?}"))?;
        pipeline.complete_cycle(fetched);

        println!(
            "cycle {cycle}: FETCH pc=0x{address:03x} -> word=0x{fetched:03x} (word_index={})",
            backplane.word_index()
        );

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
    Ok(())
}
