//! Runs HP-67 power-on microcode through the structural serial fetch path.
//!
//! Firmware stays external. Addresses and returned words still cross the
//! resolved IS/ISA model bit by bit, while the independent architectural
//! HP-67 bring-up machine resolves ACT versus CRC ownership at each instruction
//! boundary. This lets one local run cover thousands of real firmware words and
//! detect convergence into the documented no-key idle loop.

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

// Nonpareil's reviewed HP-67 disassembly labels these source-backed firmware
// landmarks in octal. Keep them as addresses only; no firmware payload is
// embedded here.
const DISPLAY_INIT_PC: u16 = 0o0161; // 0x071: "hi i'm woodstock", then display off/toggle.
const MAIN_WAIT_PC: u16 = 0o0167; // 0x077: documented main wait-for-key loop entry.
const CARD_POLL_PC: u16 = 0o0206; // 0x086: card-present poll inside that idle loop.
const PHYSICAL_DELAYED_ROM_PC: u16 = 0x067;
const PHYSICAL_DELAYED_ROM_TARGET_PC: u16 = 0x0fc6;

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
    stop_at_idle: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ProbeControl {
    Continue,
    BoundaryStop,
    IdleReached,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
struct BootMilestones {
    saw_display_init: bool,
    main_wait_visits: u64,
    card_poll_visits: u64,
    saw_physical_delayed_rom_source: bool,
    saw_physical_delayed_rom_target: bool,
    idle_cycle: Option<u64>,
}

impl BootMilestones {
    fn observe(&mut self, cycle: u64, machine: &Hp67ArchitecturalMachine, execution_pc: u16) {
        match execution_pc {
            DISPLAY_INIT_PC => self.saw_display_init = true,
            MAIN_WAIT_PC => self.main_wait_visits = self.main_wait_visits.saturating_add(1),
            CARD_POLL_PC => self.card_poll_visits = self.card_poll_visits.saturating_add(1),
            PHYSICAL_DELAYED_ROM_PC => self.saw_physical_delayed_rom_source = true,
            PHYSICAL_DELAYED_ROM_TARGET_PC => self.saw_physical_delayed_rom_target = true,
            _ => {}
        }

        if self.idle_cycle.is_none() && self.idle_ready(machine) {
            self.idle_cycle = Some(cycle);
        }
    }

    fn idle_ready(&self, machine: &Hp67ArchitecturalMachine) -> bool {
        self.saw_display_init
            && self.main_wait_visits >= 2
            && self.card_poll_visits >= 1
            && machine.act.state.display_enable
            && machine.act.state.key_buffer.is_none()
    }
}

fn format_act_register(register: &[u8; 14]) -> String {
    let mut output = String::with_capacity(register.len());
    for digit in register.iter().rev() {
        output.push(
            char::from_digit(u32::from(*digit & 0x0f), 16)
                .expect("nibble is always a hexadecimal digit")
                .to_ascii_uppercase(),
        );
    }
    output
}

fn parse_arguments() -> Result<Arguments, String> {
    let mut args = env::args().skip(1);
    let corpus_path = args
        .next()
        .unwrap_or_else(|| ".research/teenix-2026-hp67.tsv".to_owned());
    let mut probe_cycles = 0u64;
    let mut trace_limit = 64u64;
    let mut stop_at_idle = false;

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
            "--stop-at-idle" => stop_at_idle = true,
            _ => return Err(format!("unknown argument: {argument}")),
        }
    }

    Ok(Arguments {
        corpus_path,
        probe_cycles,
        trace_limit,
        stop_at_idle,
    })
}

fn execute_cycle(
    cycle: u64,
    machine: &mut Hp67ArchitecturalMachine,
    pipeline: &mut FetchPipelineLatch,
    milestones: &mut BootMilestones,
    stop_at_idle: bool,
    verbose: bool,
) -> Result<ProbeControl, String> {
    pipeline.begin_cycle();

    let Some(word) = pipeline.executing_word() else {
        if verbose {
            println!("cycle {cycle}: EXEC <pipeline fill>");
        }
        return Ok(ProbeControl::Continue);
    };

    match machine.execute_word(word) {
        Ok(execution) => {
            if verbose {
                println!(
                    "cycle {cycle}: EXEC pc=0x{:03x} word=0x{:03x} {:?} -> pc=0x{:03x}",
                    execution.pc, execution.word, execution.operation, execution.next_pc
                );
            }
            milestones.observe(cycle, machine, execution.pc);
            if stop_at_idle && milestones.idle_ready(machine) {
                Ok(ProbeControl::IdleReached)
            } else {
                Ok(ProbeControl::Continue)
            }
        }
        Err(Hp67ArchitecturalError::Act(ActError::UnknownSpecial { pc, word })) => {
            println!(
                "PROBE STOP: unknown ACT special at cycle {cycle}: pc=0x{pc:03x} word=0x{word:03x} (octal {word:04o})"
            );
            Ok(ProbeControl::BoundaryStop)
        }
        Err(Hp67ArchitecturalError::Act(ActError::UnsupportedRomSelfTest { pc })) => {
            println!("PROBE STOP: ROM self-test reached at cycle {cycle}: pc=0x{pc:03x}");
            Ok(ProbeControl::BoundaryStop)
        }
        Err(Hp67ArchitecturalError::CrcDataPortNotModeled { pc, address, write }) => {
            let direction = if write { "write" } else { "read" };
            println!(
                "PROBE STOP: CRC DATA-port {direction} reached at cycle {cycle}: pc=0x{pc:03x} address=0x{address:02x}"
            );
            Ok(ProbeControl::BoundaryStop)
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

fn print_boot_summary(milestones: &BootMilestones, machine: &Hp67ArchitecturalMachine) {
    println!(
        "BOOT SUMMARY: display_init_seen={}; main_wait_visits={}; card_poll_visits={}; display_enable={}; no_key={}; physical_0x067_to_0xfc6={}; idle_cycle={}",
        milestones.saw_display_init,
        milestones.main_wait_visits,
        milestones.card_poll_visits,
        machine.act.state.display_enable,
        machine.act.state.key_buffer.is_none(),
        milestones.saw_physical_delayed_rom_source && milestones.saw_physical_delayed_rom_target,
        milestones
            .idle_cycle
            .map_or_else(|| "none".to_owned(), |cycle| cycle.to_string())
    );
    println!(
        "DISPLAY ARCH STATE: A={}; B={}; C={}; display_14_digit={}; display_enable={}",
        format_act_register(&machine.act.state.a),
        format_act_register(&machine.act.state.b),
        format_act_register(&machine.act.state.c),
        machine.act.state.display_14_digit,
        machine.act.state.display_enable
    );
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
    let mut milestones = BootMilestones::default();
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
            milestones.observe(cycle, &machine, execution.pc);
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
        print_boot_summary(&milestones, &machine);
        return Ok(());
    }

    println!(
        "PROBE: running up to {} additional machine cycle(s); detailed trace limited to {} cycle(s).",
        arguments.probe_cycles, arguments.trace_limit
    );
    if arguments.stop_at_idle {
        println!(
            "BOOT TARGET: stop after two visits to main wait L0167 with a card-poll pass, display enabled and no key."
        );
    }

    let mut completed_probe_cycles = 0u64;
    for offset in 0..arguments.probe_cycles {
        let cycle = 4 + offset;
        let verbose = offset < arguments.trace_limit;
        match execute_cycle(
            cycle,
            &mut machine,
            &mut pipeline,
            &mut milestones,
            arguments.stop_at_idle,
            verbose,
        )? {
            ProbeControl::Continue => {}
            ProbeControl::BoundaryStop => {
                println!(
                    "PROBE SUMMARY: completed {completed_probe_cycles} additional cycles; executed_words={}; pc=0x{:03x}; bank={}",
                    machine.executed_words(),
                    machine.pc(),
                    machine.bank()
                );
                print_boot_summary(&milestones, &machine);
                return Ok(());
            }
            ProbeControl::IdleReached => {
                println!(
                    "BOOT IDLE PASS: real firmware completed initialization and cycled through the documented no-key wait loop with display_enable=true."
                );
                println!(
                    "BOOT IDLE: cycle={cycle}; executed_words={}; pc=0x{:03x}; bank={}",
                    machine.executed_words(),
                    machine.pc(),
                    machine.bank()
                );
                print_boot_summary(&milestones, &machine);
                return Ok(());
            }
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
    print_boot_summary(&milestones, &machine);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn boot_idle_requires_a_complete_no_key_loop_after_display_init() {
        let mut machine = Hp67ArchitecturalMachine::default();
        let mut milestones = BootMilestones::default();

        milestones.observe(10, &machine, DISPLAY_INIT_PC);
        machine.act.state.display_enable = true;
        milestones.observe(20, &machine, MAIN_WAIT_PC);
        assert!(!milestones.idle_ready(&machine));

        milestones.observe(30, &machine, CARD_POLL_PC);
        assert!(!milestones.idle_ready(&machine));

        milestones.observe(40, &machine, MAIN_WAIT_PC);
        assert!(milestones.idle_ready(&machine));
        assert_eq!(milestones.idle_cycle, Some(40));
    }

    #[test]
    fn physical_delayed_rom_landmarks_are_recorded_independently() {
        let machine = Hp67ArchitecturalMachine::default();
        let mut milestones = BootMilestones::default();
        milestones.observe(1, &machine, PHYSICAL_DELAYED_ROM_PC);
        assert!(milestones.saw_physical_delayed_rom_source);
        assert!(!milestones.saw_physical_delayed_rom_target);
        milestones.observe(2, &machine, PHYSICAL_DELAYED_ROM_TARGET_PC);
        assert!(milestones.saw_physical_delayed_rom_target);
    }

    #[test]
    fn display_register_dump_is_most_significant_digit_first() {
        let mut register = [0u8; 14];
        for (index, digit) in register.iter_mut().enumerate() {
            *digit = index as u8;
        }
        assert_eq!(format_act_register(&register), "DCBA9876543210");
    }
}
