//! Proves the first physical HP-67 keyboard path through real firmware.
//!
//! This smoke deliberately stops at firmware dispatch. It does not require a
//! calculator-level result or synthesize display/register contents. The path is:
//! physical Digit1 contact -> S15/key code -> real `keys -> a` -> firmware remap
//! -> real `a -> rom address` -> unshifted key-72 table entry at octal 1440.

use std::{cell::Cell, env, fs};

use hp67emu::{
    machines::hp67::{
        decode_rom0_display_byte, run_structural_display_fetch_cycle, ActInstructionState,
        ActSerialEndpoint, CathodeDriver1820_1749, FetchPipelineLatch, Hp67ArchitecturalMachine,
        Hp67ElectricalBackplane, Hp67Key, Hp67Keyboard, Hp67RomWordSource, Rom0DisplayEndpoint,
        RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS, HP67_KEY_PRESSED_STATUS_BIT,
    },
    research::rom_corpus::{RomCorpus, ROM_PAGES, WORDS_PER_PAGE},
};

const EXPECTED_POPULATED_WORDS: usize = 5120;
const BOOT_CYCLE_LIMIT: u64 = 2_000;
const KEY_PROBE_WORD_LIMIT: u64 = 512;

const DISPLAY_INIT_PC: u16 = 0o0161;
const MAIN_WAIT_PC: u16 = 0o0167;
const CARD_POLL_PC: u16 = 0o0206;
const KEYS_TO_A_OPCODE: u16 = 0o0120;
const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;
const DIGIT_ONE_KEYCODE: u8 = 0o142;
const DIGIT_ONE_UNSHIFTED_TABLE_PC: u16 = 0o1440;

const EXPECTED_BOOT_DISPLAY_SEGMENTS: [u8; 15] = [
    0x00, 0x00, 0x00, 0x3f, 0x80, 0x3f, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

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
        self.corpus
            .get(effective, usize::from(address))
            .ok()
            .flatten()
    }
}

#[derive(Debug, Default)]
struct BootMilestones {
    saw_display_init: bool,
    main_wait_visits: u64,
    card_poll_visits: u64,
}

impl BootMilestones {
    fn observe(&mut self, pc: u16) {
        match pc {
            DISPLAY_INIT_PC => self.saw_display_init = true,
            MAIN_WAIT_PC => self.main_wait_visits = self.main_wait_visits.saturating_add(1),
            CARD_POLL_PC => self.card_poll_visits = self.card_poll_visits.saturating_add(1),
            _ => {}
        }
    }

    fn idle_ready(&self, machine: &Hp67ArchitecturalMachine) -> bool {
        self.saw_display_init
            && self.main_wait_visits >= 2
            && self.card_poll_visits >= 1
            && machine.act.state.display_enable
            && machine.act.state.key_buffer.is_none()
            && !machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
    }
}

fn fetch_cycle(
    cycle: u64,
    backplane: &mut Hp67ElectricalBackplane,
    machine: &mut Hp67ArchitecturalMachine,
    act_serial: &mut ActSerialEndpoint,
    fetch_rom: &mut RomFetchEndpoint,
    display_rom0: &mut Rom0DisplayEndpoint,
    display_cathode: &mut CathodeDriver1820_1749,
    pipeline: &mut FetchPipelineLatch,
    source: &CorpusRom<'_>,
) -> Result<(), String> {
    let requested_bank = machine.prepare_hp67_fetch();
    source.select_bank(requested_bank);
    let address = machine.pc();
    let result = run_structural_display_fetch_cycle(
        backplane,
        address,
        &machine.act.state,
        act_serial,
        fetch_rom,
        display_rom0,
        source,
    )
    .map_err(|error| format!("cycle {cycle} shared display/fetch word failed: {error:?}"))?;

    display_cathode
        .apply_control_edges(result.str_event, result.rcd_falling)
        .map_err(|error| format!("cycle {cycle} cathode control failed: {error:?}"))?;
    pipeline.complete_cycle(result.fetched_word);
    Ok(())
}

fn verify_serial_execution_complete(
    cycle: u64,
    pipeline: &FetchPipelineLatch,
    act_serial: &ActSerialEndpoint,
) -> Result<(), String> {
    let Some(word) = pipeline.executing_word() else {
        return Ok(());
    };
    let execution = act_serial
        .serial_execution()
        .ok_or_else(|| format!("cycle {cycle} lost serial execution for word 0x{word:03x}"))?;
    if execution.word() != word || !execution.is_complete() {
        return Err(format!(
            "cycle {cycle} serial execution incomplete: expected 0x{word:03x}, got 0x{:03x}, next_bit={:?}",
            execution.word(),
            execution.next_word_bit()
        ));
    }
    if act_serial.serial_execution_state().is_none() {
        return Err(format!(
            "cycle {cycle} lost pre-instruction serial state for word 0x{word:03x}"
        ));
    }
    Ok(())
}

fn execute_current_word(
    cycle: u64,
    machine: &mut Hp67ArchitecturalMachine,
    act_serial: &mut ActSerialEndpoint,
    pipeline: &mut FetchPipelineLatch,
) -> Result<Option<(u16, u16, ActInstructionState)>, String> {
    pipeline.begin_cycle();
    let Some(word) = pipeline.executing_word() else {
        return Ok(None);
    };

    let instruction_state = machine.act.state.instruction_state;
    act_serial
        .begin_execution(word, &machine.act.state)
        .map_err(|error| format!("cycle {cycle} serial execution start failed: {error:?}"))?;
    let execution = machine
        .execute_word(word)
        .map_err(|error| format!("cycle {cycle} architectural execution failed: {error:?}"))?;
    Ok(Some((execution.pc, word, instruction_state)))
}

fn verify_boot_display<S: Hp67RomWordSource>(
    machine: &Hp67ArchitecturalMachine,
    backplane: &mut Hp67ElectricalBackplane,
    fetch_rom: &mut RomFetchEndpoint,
    display_rom0: &mut Rom0DisplayEndpoint,
    source: &S,
) -> Result<(), String> {
    let address = machine.pc();
    let mut display_act = ActSerialEndpoint::new(address);
    let mut cathode = CathodeDriver1820_1749::default();
    let mut segments = Vec::with_capacity(usize::from(HP67_DISPLAY_SCAN_SLOTS));

    for expected_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
        let result = run_structural_display_fetch_cycle(
            backplane,
            address,
            &machine.act.state,
            &mut display_act,
            fetch_rom,
            display_rom0,
            source,
        )
        .map_err(|error| {
            format!("boot display shared word failed at slot {expected_slot}: {error:?}")
        })?;
        if result.str_event.scan_slot != expected_slot {
            return Err(format!(
                "boot display phase mismatch: got slot {}, expected {expected_slot}",
                result.str_event.scan_slot
            ));
        }
        let decoded = decode_rom0_display_byte(expected_slot, result.display_byte)
            .map_err(|error| format!("boot display ROM0 decode failed: {error:?}"))?;
        segments.push(decoded.bits());
        cathode
            .apply_control_edges(result.str_event, result.rcd_falling)
            .map_err(|error| format!("boot display cathode control failed: {error:?}"))?;
    }

    if segments.as_slice() != EXPECTED_BOOT_DISPLAY_SEGMENTS {
        return Err(format!(
            "boot display segments differ: got {:02X?}, expected {:02X?}",
            segments, EXPECTED_BOOT_DISPLAY_SEGMENTS
        ));
    }
    Ok(())
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

    let source = CorpusRom::new(&corpus);
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act_serial = ActSerialEndpoint::new(0);
    let mut fetch_rom = RomFetchEndpoint::default();
    let mut display_rom0 = Rom0DisplayEndpoint::default();
    let mut display_cathode = CathodeDriver1820_1749::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut milestones = BootMilestones::default();

    println!("HP-67 real-firmware physical-key dispatch smoke");

    let mut idle_cycle = None;
    for cycle in 0..BOOT_CYCLE_LIMIT {
        if let Some((pc, _, _)) =
            execute_current_word(cycle, &mut machine, &mut act_serial, &mut pipeline)?
        {
            milestones.observe(pc);
            if milestones.idle_ready(&machine) {
                idle_cycle = Some(cycle);
            }
        }

        fetch_cycle(
            cycle,
            &mut backplane,
            &mut machine,
            &mut act_serial,
            &mut fetch_rom,
            &mut display_rom0,
            &mut display_cathode,
            &mut pipeline,
            &source,
        )?;
        verify_serial_execution_complete(cycle, &pipeline, &act_serial)?;

        if idle_cycle == Some(cycle) {
            source.select_bank(machine.bank());
            verify_boot_display(
                &machine,
                &mut backplane,
                &mut fetch_rom,
                &mut display_rom0,
                &source,
            )?;
            break;
        }
    }

    let idle_cycle = idle_cycle.ok_or_else(|| {
        format!("real firmware did not reach the no-key idle checkpoint within {BOOT_CYCLE_LIMIT} cycles")
    })?;
    println!("BOOT IDLE PASS: cycle={idle_cycle}; words={}", machine.executed_words());

    let mut keyboard = Hp67Keyboard::default();
    keyboard.press(Hp67Key::Digit1);
    if keyboard.code() != Some(DIGIT_ONE_KEYCODE) {
        return Err(format!(
            "Digit1 hardware code mismatch: got {:?}, expected {DIGIT_ONE_KEYCODE:04o} octal",
            keyboard.code()
        ));
    }
    println!(
        "KEY CONTACT DOWN: digit 1, hardware code {DIGIT_ONE_KEYCODE:04o} octal"
    );

    let mut saw_keys_to_a = false;
    for offset in 1..=KEY_PROBE_WORD_LIMIT {
        let cycle = idle_cycle + offset;
        keyboard.sample_into_act(&mut machine.act.state);

        let executed = execute_current_word(cycle, &mut machine, &mut act_serial, &mut pipeline)?;
        let mut dispatch_complete = false;

        if let Some((pc, word, prior_state)) = executed {
            let normal = prior_state == ActInstructionState::Normal;
            if normal && word == KEYS_TO_A_OPCODE {
                if machine.act.state.a[2] != (DIGIT_ONE_KEYCODE >> 4)
                    || machine.act.state.a[1] != (DIGIT_ONE_KEYCODE & 0x0f)
                {
                    return Err(format!(
                        "firmware executed keys -> a at pc={pc:04o}, but A[2:1]={:X}{:X} instead of {:02X}",
                        machine.act.state.a[2],
                        machine.act.state.a[1],
                        DIGIT_ONE_KEYCODE
                    ));
                }
                saw_keys_to_a = true;
                println!(
                    "KEYS -> A PASS: pc={pc:04o}; A[2:1]={:X}{:X}",
                    machine.act.state.a[2], machine.act.state.a[1]
                );
            }

            if normal && word == A_TO_ROM_ADDRESS_OPCODE {
                if !saw_keys_to_a {
                    return Err(format!(
                        "firmware executed a -> rom address at pc={pc:04o} before observing keys -> a"
                    ));
                }
                if machine.pc() != DIGIT_ONE_UNSHIFTED_TABLE_PC {
                    return Err(format!(
                        "digit-1 firmware remap dispatched to {:04o}, expected source-backed unshifted key-72 entry {:04o}",
                        machine.pc(), DIGIT_ONE_UNSHIFTED_TABLE_PC
                    ));
                }
                dispatch_complete = true;
            }
        }

        fetch_cycle(
            cycle,
            &mut backplane,
            &mut machine,
            &mut act_serial,
            &mut fetch_rom,
            &mut display_rom0,
            &mut display_cathode,
            &mut pipeline,
            &source,
        )?;
        verify_serial_execution_complete(cycle, &pipeline, &act_serial)?;

        if dispatch_complete {
            keyboard.release();
            println!(
                "M8 KEYBOARD PASS: physical Digit1 contact asserted S15, real firmware consumed keycode {:04o} with keys -> a, remapped it, and a -> rom address dispatched to {:04o} (unshifted key 72: 1).",
                DIGIT_ONE_KEYCODE, DIGIT_ONE_UNSHIFTED_TABLE_PC
            );
            return Ok(());
        }
    }

    Err(format!(
        "Digit1 contact reached the ACT but real firmware did not complete keys -> a -> unshifted table dispatch within {KEY_PROBE_WORD_LIMIT} words; saw_keys_to_a={saw_keys_to_a}; pc={:04o}; s15={}",
        machine.pc(), machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
    ))
}
