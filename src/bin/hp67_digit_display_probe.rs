//! Probes the HP-67 frontier from a proven physical Digit1 dispatch to display output.
//!
//! This is deliberately a probe, not an M9 pass criterion. It proves M7 and M8
//! again, releases the physical key after firmware dispatches to the unshifted
//! key-72 table entry, then lets real firmware return to its wait loop and dumps
//! the resulting ROM0/cathode display plus ACT A/B/C state. No display or digit
//! semantics are injected by the host.

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
const BOOT_WORD_LIMIT: u64 = 2_000;
const KEY_WORD_LIMIT: u64 = 512;
const POST_KEY_WORD_LIMIT: u64 = 4_096;

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

#[derive(Debug, Clone, Copy)]
struct ExecutedWord {
    pc: u16,
    word: u16,
    prior_instruction_state: ActInstructionState,
}

fn format_register(register: &[u8; 14]) -> String {
    register
        .iter()
        .rev()
        .map(|digit| {
            char::from_digit(u32::from(*digit & 0x0f), 16)
                .expect("nibble must be hexadecimal")
                .to_ascii_uppercase()
        })
        .collect()
}

fn format_segments(segments: &[u8; 15]) -> String {
    segments
        .iter()
        .map(|segment| format!("{segment:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

#[allow(clippy::too_many_arguments)]
fn step_word(
    cycle: u64,
    keyboard: &Hp67Keyboard,
    machine: &mut Hp67ArchitecturalMachine,
    backplane: &mut Hp67ElectricalBackplane,
    act_serial: &mut ActSerialEndpoint,
    fetch_rom: &mut RomFetchEndpoint,
    display_rom0: &mut Rom0DisplayEndpoint,
    display_cathode: &mut CathodeDriver1820_1749,
    pipeline: &mut FetchPipelineLatch,
    source: &CorpusRom<'_>,
) -> Result<Option<ExecutedWord>, String> {
    keyboard.sample_into_act(&mut machine.act.state);
    pipeline.begin_cycle();

    let executed = if let Some(word) = pipeline.executing_word() {
        let prior_instruction_state = machine.act.state.instruction_state;
        act_serial
            .begin_execution(word, &machine.act.state)
            .map_err(|error| format!("cycle {cycle} serial execution start failed: {error:?}"))?;
        let execution = machine
            .execute_word(word)
            .map_err(|error| format!("cycle {cycle} architectural execution failed: {error:?}"))?;
        Some(ExecutedWord {
            pc: execution.pc,
            word,
            prior_instruction_state,
        })
    } else {
        None
    };

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

    if let Some(executed_word) = executed {
        let serial = act_serial.serial_execution().ok_or_else(|| {
            format!(
                "cycle {cycle} lost serial execution for word 0x{:03x}",
                executed_word.word
            )
        })?;
        if serial.word() != executed_word.word || !serial.is_complete() {
            return Err(format!(
                "cycle {cycle} serial execution incomplete for word 0x{:03x}: got 0x{:03x}, next_bit={:?}",
                executed_word.word,
                serial.word(),
                serial.next_word_bit()
            ));
        }
    }

    Ok(executed)
}

fn capture_display(
    machine: &Hp67ArchitecturalMachine,
    backplane: &mut Hp67ElectricalBackplane,
    fetch_rom: &mut RomFetchEndpoint,
    display_rom0: &mut Rom0DisplayEndpoint,
    source: &CorpusRom<'_>,
) -> Result<[u8; 15], String> {
    source.select_bank(machine.bank());
    let address = machine.pc();
    let mut display_act = ActSerialEndpoint::new(address);
    let mut cathode = CathodeDriver1820_1749::default();
    let mut segments = [0u8; 15];

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
            format!("display capture failed at scan slot {expected_slot}: {error:?}")
        })?;
        if result.str_event.scan_slot != expected_slot {
            return Err(format!(
                "display capture phase mismatch: got slot {}, expected {expected_slot}",
                result.str_event.scan_slot
            ));
        }
        segments[usize::from(expected_slot - 1)] =
            decode_rom0_display_byte(expected_slot, result.display_byte)
                .map_err(|error| format!("ROM0 display decode failed: {error:?}"))?
                .bits();
        cathode
            .apply_control_edges(result.str_event, result.rcd_falling)
            .map_err(|error| format!("display cathode control failed: {error:?}"))?;
    }

    if cathode.scan_slot() != 1 {
        return Err(format!(
            "display cathode did not wrap to slot 1: got {}",
            cathode.scan_slot()
        ));
    }
    Ok(segments)
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
    let mut keyboard = Hp67Keyboard::default();
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act_serial = ActSerialEndpoint::new(0);
    let mut fetch_rom = RomFetchEndpoint::default();
    let mut display_rom0 = Rom0DisplayEndpoint::default();
    let mut display_cathode = CathodeDriver1820_1749::default();
    let mut pipeline = FetchPipelineLatch::default();

    println!("HP-67 real-firmware Digit1 display frontier probe");

    let mut saw_display_init = false;
    let mut main_wait_visits = 0u64;
    let mut card_poll_visits = 0u64;
    let mut idle_cycle = None;

    for cycle in 0..BOOT_WORD_LIMIT {
        if let Some(executed) = step_word(
            cycle,
            &keyboard,
            &mut machine,
            &mut backplane,
            &mut act_serial,
            &mut fetch_rom,
            &mut display_rom0,
            &mut display_cathode,
            &mut pipeline,
            &source,
        )? {
            match executed.pc {
                DISPLAY_INIT_PC => saw_display_init = true,
                MAIN_WAIT_PC => main_wait_visits = main_wait_visits.saturating_add(1),
                CARD_POLL_PC => card_poll_visits = card_poll_visits.saturating_add(1),
                _ => {}
            }
            if saw_display_init
                && main_wait_visits >= 2
                && card_poll_visits >= 1
                && machine.act.state.display_enable
                && machine.act.state.key_buffer.is_none()
                && !machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
            {
                idle_cycle = Some(cycle);
                break;
            }
        }
    }

    let idle_cycle = idle_cycle.ok_or_else(|| {
        format!("firmware did not reach idle within {BOOT_WORD_LIMIT} machine words")
    })?;
    let boot_display = capture_display(
        &machine,
        &mut backplane,
        &mut fetch_rom,
        &mut display_rom0,
        &source,
    )?;
    if boot_display != EXPECTED_BOOT_DISPLAY_SEGMENTS {
        return Err(format!(
            "M7 boot display mismatch: got [{}]",
            format_segments(&boot_display)
        ));
    }
    println!(
        "M7 PASS: idle cycle={idle_cycle}; display=[{}]",
        format_segments(&boot_display)
    );

    keyboard.press(Hp67Key::Digit1);
    if keyboard.code() != Some(DIGIT_ONE_KEYCODE) {
        return Err("Digit1 keycode is not source-backed 0142".to_owned());
    }

    let mut saw_keys_to_a = false;
    let mut dispatch_cycle = None;
    for offset in 1..=KEY_WORD_LIMIT {
        let cycle = idle_cycle + offset;
        if let Some(executed) = step_word(
            cycle,
            &keyboard,
            &mut machine,
            &mut backplane,
            &mut act_serial,
            &mut fetch_rom,
            &mut display_rom0,
            &mut display_cathode,
            &mut pipeline,
            &source,
        )? {
            let normal = executed.prior_instruction_state == ActInstructionState::Normal;
            if normal && executed.word == KEYS_TO_A_OPCODE {
                if machine.act.state.a[2] != 0x6 || machine.act.state.a[1] != 0x2 {
                    return Err(format!(
                        "keys -> a produced A[2:1]={:X}{:X}, expected 62",
                        machine.act.state.a[2], machine.act.state.a[1]
                    ));
                }
                saw_keys_to_a = true;
            }
            if normal && executed.word == A_TO_ROM_ADDRESS_OPCODE {
                if !saw_keys_to_a || machine.pc() != DIGIT_ONE_UNSHIFTED_TABLE_PC {
                    return Err(format!(
                        "Digit1 dispatch mismatch: saw_keys_to_a={saw_keys_to_a}; pc={:04o}",
                        machine.pc()
                    ));
                }
                dispatch_cycle = Some(cycle);
                break;
            }
        }
    }

    let dispatch_cycle = dispatch_cycle.ok_or_else(|| {
        format!("firmware did not complete Digit1 dispatch within {KEY_WORD_LIMIT} words")
    })?;
    println!(
        "M8 PASS: cycle={dispatch_cycle}; keycode={DIGIT_ONE_KEYCODE:04o}; dispatch={DIGIT_ONE_UNSHIFTED_TABLE_PC:04o}"
    );

    keyboard.release();
    println!("KEY CONTACT UP: code remains latched; S15 clearing remains firmware-owned");

    let mut saw_post_key_display_init = false;
    for offset in 1..=POST_KEY_WORD_LIMIT {
        let cycle = dispatch_cycle + offset;
        if let Some(executed) = step_word(
            cycle,
            &keyboard,
            &mut machine,
            &mut backplane,
            &mut act_serial,
            &mut fetch_rom,
            &mut display_rom0,
            &mut display_cathode,
            &mut pipeline,
            &source,
        )? {
            if executed.pc == DISPLAY_INIT_PC {
                saw_post_key_display_init = true;
            }
            if saw_post_key_display_init
                && executed.pc == MAIN_WAIT_PC
                && machine.act.state.display_enable
                && !machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
            {
                let post_key_display = capture_display(
                    &machine,
                    &mut backplane,
                    &mut fetch_rom,
                    &mut display_rom0,
                    &source,
                )?;
                println!("POST-KEY IDLE: cycle={cycle}; pc={:04o}", machine.pc());
                println!(
                    "POST-KEY DISPLAY SEGMENTS: {}",
                    format_segments(&post_key_display)
                );
                println!(
                    "POST-KEY ACT: A={}; B={}; C={}; display_enable={}; s15={}; keycode={:?}",
                    format_register(&machine.act.state.a),
                    format_register(&machine.act.state.b),
                    format_register(&machine.act.state.c),
                    machine.act.state.display_enable,
                    machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT],
                    machine.act.state.key_buffer
                );
                if post_key_display == boot_display {
                    return Err(
                        "firmware returned to idle after Digit1 but physical display remained at power-on 0.00"
                            .to_owned(),
                    );
                }
                println!(
                    "M9 FRONTIER REACHED: real firmware changed the physical display after Digit1; inspect the source-derived segment pattern before promoting this probe to an exact M9 assertion."
                );
                return Ok(());
            }
        }
    }

    Err(format!(
        "Digit1 dispatched successfully, but firmware did not return through display init to no-key main wait within {POST_KEY_WORD_LIMIT} words; pc={:04o}; s15={} ",
        machine.pc(), machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
    ))
}
