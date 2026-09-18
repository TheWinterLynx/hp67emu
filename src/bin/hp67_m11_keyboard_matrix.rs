//! M11 HP-67 keyboard/function matrix over real versioned firmware.
//!
//! This binary does not implement calculator semantics. It presents physical
//! HP-67 key contacts to ACT, observes firmware keycode capture and dispatch,
//! and then lets the real firmware execute. Exact assertions are used where the
//! physical display result is already unambiguous; shifted-function coverage is
//! exploratory and fails on any architectural/serial execution error.

use hp67emu::machines::hp67::{
    decode_rom0_display_byte, run_structural_display_fetch_cycle, ActInstructionState,
    ActSerialEndpoint, CathodeDriver1820_1749, FetchPipelineLatch, Hp67ArchitecturalMachine,
    Hp67ElectricalBackplane, Hp67Firmware, Hp67Key, Hp67Keyboard, Rom0DisplayEndpoint,
    RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS, HP67_KEY_PRESSED_STATUS_BIT,
};

const BOOT_WORD_LIMIT: u64 = 2_000;
const KEY_DISPATCH_WORD_LIMIT: u64 = 512;
const KEY_SETTLE_WORD_LIMIT: u64 = 4_096;
const SHIFT_EXERCISE_WORDS: usize = 768;

const DISPLAY_INIT_PC: u16 = 0o0161;
const MAIN_WAIT_PC: u16 = 0o0167;
const CARD_POLL_PC: u16 = 0o0206;
const KEYS_TO_A_OPCODE: u16 = 0o0120;
const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;
const UNSHIFTED_KEY_TABLE_FIRST: u16 = 0o1405;
const UNSHIFTED_KEY_TABLE_LAST: u16 = 0o1466;

const EXPECTED_BOOT_DISPLAY: [u8; 15] = [
    0x00, 0x00, 0x00, 0x3f, 0x80, 0x3f, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const EXPECTED_THREE_FIXED_TWO: [u8; 15] = [
    0x00, 0x00, 0x00, 0x4f, 0x80, 0x3f, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const EXPECTED_CLEAR_DISPLAY: [u8; 15] = EXPECTED_BOOT_DISPLAY;
const EXPECTED_ONE_POINT_TWO: [u8; 15] = [
    0x00, 0x00, 0x00, 0x06, 0x80, 0x5b, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const DIGIT_SEGMENTS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];

#[derive(Debug, Clone, Copy)]
struct KeySpec {
    key: Hp67Key,
    label: &'static str,
}

const ALL_KEYS: [KeySpec; 35] = [
    KeySpec { key: Hp67Key::A, label: "A" },
    KeySpec { key: Hp67Key::B, label: "B" },
    KeySpec { key: Hp67Key::C, label: "C" },
    KeySpec { key: Hp67Key::D, label: "D" },
    KeySpec { key: Hp67Key::E, label: "E" },
    KeySpec { key: Hp67Key::SigmaPlus, label: "SIGMA+" },
    KeySpec { key: Hp67Key::Gto, label: "GTO" },
    KeySpec { key: Hp67Key::Dsp, label: "DSP" },
    KeySpec { key: Hp67Key::Indirect, label: "IND" },
    KeySpec { key: Hp67Key::Sst, label: "SST" },
    KeySpec { key: Hp67Key::FunctionF, label: "f" },
    KeySpec { key: Hp67Key::FunctionG, label: "g" },
    KeySpec { key: Hp67Key::Sto, label: "STO" },
    KeySpec { key: Hp67Key::Rcl, label: "RCL" },
    KeySpec { key: Hp67Key::FunctionH, label: "h" },
    KeySpec { key: Hp67Key::Enter, label: "ENTER" },
    KeySpec { key: Hp67Key::ChangeSign, label: "CHS" },
    KeySpec { key: Hp67Key::Exponent, label: "EEX" },
    KeySpec { key: Hp67Key::ClearX, label: "CLX" },
    KeySpec { key: Hp67Key::Subtract, label: "-" },
    KeySpec { key: Hp67Key::Digit7, label: "7" },
    KeySpec { key: Hp67Key::Digit8, label: "8" },
    KeySpec { key: Hp67Key::Digit9, label: "9" },
    KeySpec { key: Hp67Key::Add, label: "+" },
    KeySpec { key: Hp67Key::Digit4, label: "4" },
    KeySpec { key: Hp67Key::Digit5, label: "5" },
    KeySpec { key: Hp67Key::Digit6, label: "6" },
    KeySpec { key: Hp67Key::Multiply, label: "*" },
    KeySpec { key: Hp67Key::Digit1, label: "1" },
    KeySpec { key: Hp67Key::Digit2, label: "2" },
    KeySpec { key: Hp67Key::Digit3, label: "3" },
    KeySpec { key: Hp67Key::Divide, label: "/" },
    KeySpec { key: Hp67Key::Digit0, label: "0" },
    KeySpec { key: Hp67Key::Decimal, label: "." },
    KeySpec { key: Hp67Key::RunStop, label: "R/S" },
];

#[derive(Debug, Clone, Copy)]
struct ExecutedWord {
    pc: u16,
    word: u16,
    prior_instruction_state: ActInstructionState,
}

#[derive(Debug, Clone, Copy)]
struct Dispatch {
    code: u8,
    target: u16,
}

struct Harness {
    source: Hp67Firmware,
    keyboard: Hp67Keyboard,
    machine: Hp67ArchitecturalMachine,
    backplane: Hp67ElectricalBackplane,
    act_serial: ActSerialEndpoint,
    fetch_rom: RomFetchEndpoint,
    display_rom0: Rom0DisplayEndpoint,
    display_cathode: CathodeDriver1820_1749,
    pipeline: FetchPipelineLatch,
    cycle: u64,
}

impl Default for Harness {
    fn default() -> Self {
        Self {
            source: Hp67Firmware::default(),
            keyboard: Hp67Keyboard::default(),
            machine: Hp67ArchitecturalMachine::default(),
            backplane: Hp67ElectricalBackplane::default(),
            act_serial: ActSerialEndpoint::new(0),
            fetch_rom: RomFetchEndpoint::default(),
            display_rom0: Rom0DisplayEndpoint::default(),
            display_cathode: CathodeDriver1820_1749::default(),
            pipeline: FetchPipelineLatch::default(),
            cycle: 0,
        }
    }
}

impl Harness {
    fn step(&mut self) -> Result<Option<ExecutedWord>, String> {
        let cycle = self.cycle;
        self.keyboard.sample_into_act(&mut self.machine.act.state);
        self.pipeline.begin_cycle();

        let executed = if let Some(word) = self.pipeline.executing_word() {
            let prior_instruction_state = self.machine.act.state.instruction_state;
            self.act_serial
                .begin_execution(word, &self.machine.act.state)
                .map_err(|error| {
                    format!("cycle {cycle} serial execution start failed: {error:?}")
                })?;
            let execution = self.machine.execute_word(word).map_err(|error| {
                format!("cycle {cycle} architectural execution failed: {error:?}")
            })?;
            Some(ExecutedWord {
                pc: execution.pc,
                word,
                prior_instruction_state,
            })
        } else {
            None
        };

        let requested_bank = self.machine.prepare_hp67_fetch();
        self.source.select_bank(requested_bank);
        let address = self.machine.pc();
        let result = run_structural_display_fetch_cycle(
            &mut self.backplane,
            address,
            &self.machine.act.state,
            &mut self.act_serial,
            &mut self.fetch_rom,
            &mut self.display_rom0,
            &self.source,
        )
        .map_err(|error| format!("cycle {cycle} shared display/fetch word failed: {error:?}"))?;
        self.display_cathode
            .apply_control_edges(result.str_event, result.rcd_falling)
            .map_err(|error| format!("cycle {cycle} cathode control failed: {error:?}"))?;
        self.pipeline.complete_cycle(result.fetched_word);

        if let Some(executed_word) = executed {
            let serial = self.act_serial.serial_execution().ok_or_else(|| {
                format!(
                    "cycle {cycle} lost serial execution for word 0x{:03x}",
                    executed_word.word
                )
            })?;
            if serial.word() != executed_word.word || !serial.is_complete() {
                return Err(format!(
                    "cycle {cycle} serial execution incomplete for 0x{:03x}: got 0x{:03x}, next_bit={:?}",
                    executed_word.word,
                    serial.word(),
                    serial.next_word_bit()
                ));
            }
        }

        self.cycle = self.cycle.saturating_add(1);
        Ok(executed)
    }

    fn boot_to_idle(&mut self) -> Result<[u8; 15], String> {
        let mut saw_display_init = false;
        let mut main_wait_visits = 0u64;
        let mut card_poll_visits = 0u64;

        for _ in 0..BOOT_WORD_LIMIT {
            if let Some(executed) = self.step()? {
                match executed.pc {
                    DISPLAY_INIT_PC => saw_display_init = true,
                    MAIN_WAIT_PC => main_wait_visits = main_wait_visits.saturating_add(1),
                    CARD_POLL_PC => card_poll_visits = card_poll_visits.saturating_add(1),
                    _ => {}
                }
                if saw_display_init
                    && main_wait_visits >= 2
                    && card_poll_visits >= 1
                    && self.machine.act.state.display_enable
                    && self.machine.act.state.key_buffer.is_none()
                    && !self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
                {
                    return self.capture_display();
                }
            }
        }

        Err(format!(
            "firmware did not reach power-on idle within {BOOT_WORD_LIMIT} words; pc={:04o}",
            self.machine.pc()
        ))
    }

    fn press_to_dispatch(&mut self, key: Hp67Key, label: &str) -> Result<Dispatch, String> {
        let code = key.scan_code();
        self.keyboard.press(key);
        let mut saw_keys_to_a = false;

        for _ in 0..KEY_DISPATCH_WORD_LIMIT {
            if let Some(executed) = self.step()? {
                let normal = executed.prior_instruction_state == ActInstructionState::Normal;
                if normal && executed.word == KEYS_TO_A_OPCODE {
                    let observed = (self.machine.act.state.a[2] << 4) | self.machine.act.state.a[1];
                    if observed != code {
                        return Err(format!(
                            "{label}: keys -> a produced 0x{observed:02x}, expected hardware code 0x{code:02x}"
                        ));
                    }
                    saw_keys_to_a = true;
                }
                if normal && executed.word == A_TO_ROM_ADDRESS_OPCODE {
                    if !saw_keys_to_a {
                        return Err(format!(
                            "{label}: a -> rom address executed before keys -> a"
                        ));
                    }
                    let target = self.machine.pc();
                    if !(UNSHIFTED_KEY_TABLE_FIRST..=UNSHIFTED_KEY_TABLE_LAST).contains(&target) {
                        return Err(format!(
                            "{label}: key dispatched to {target:04o}, outside {UNSHIFTED_KEY_TABLE_FIRST:04o}..={UNSHIFTED_KEY_TABLE_LAST:04o}"
                        ));
                    }
                    self.keyboard.release();
                    return Ok(Dispatch { code, target });
                }
            }
        }

        self.keyboard.release();
        Err(format!(
            "{label}: firmware did not complete keyboard dispatch within {KEY_DISPATCH_WORD_LIMIT} words; pc={:04o}; s15={}",
            self.machine.pc(),
            self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
        ))
    }

    fn wait_for_contact_release(&mut self, label: &str) -> Result<(), String> {
        for _ in 0..KEY_SETTLE_WORD_LIMIT {
            self.step()?;
            if !self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT] {
                return Ok(());
            }
        }
        Err(format!(
            "{label}: firmware did not clear S15 after contact release within {KEY_SETTLE_WORD_LIMIT} words"
        ))
    }

    fn press_and_settle(&mut self, key: Hp67Key, label: &str) -> Result<[u8; 15], String> {
        let dispatch = self.press_to_dispatch(key, label)?;
        for _ in 0..KEY_SETTLE_WORD_LIMIT {
            if let Some(executed) = self.step()? {
                if executed.pc == MAIN_WAIT_PC
                    && self.machine.act.state.display_enable
                    && !self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
                {
                    return self.capture_display();
                }
            }
        }

        Err(format!(
            "{label}: dispatch {:04o} did not settle to no-key main wait within {KEY_SETTLE_WORD_LIMIT} words; pc={:04o}",
            dispatch.target,
            self.machine.pc()
        ))
    }

    fn run_words(&mut self, count: usize) -> Result<(), String> {
        for _ in 0..count {
            self.step()?;
        }
        Ok(())
    }

    fn capture_display(&mut self) -> Result<[u8; 15], String> {
        self.source.select_bank(self.machine.bank());
        let address = self.machine.pc();
        let mut display_act = ActSerialEndpoint::new(address);
        let mut cathode = CathodeDriver1820_1749::default();
        let mut segments = [0u8; 15];

        for expected_slot in 1..=HP67_DISPLAY_SCAN_SLOTS {
            let result = run_structural_display_fetch_cycle(
                &mut self.backplane,
                address,
                &self.machine.act.state,
                &mut display_act,
                &mut self.fetch_rom,
                &mut self.display_rom0,
                &self.source,
            )
            .map_err(|error| {
                format!("display capture failed at slot {expected_slot}: {error:?}")
            })?;
            if result.str_event.scan_slot != expected_slot {
                return Err(format!(
                    "display phase mismatch: got {}, expected {expected_slot}",
                    result.str_event.scan_slot
                ));
            }
            segments[usize::from(expected_slot - 1)] =
                decode_rom0_display_byte(expected_slot, result.display_byte)
                    .map_err(|error| format!("ROM0 decode failed: {error:?}"))?
                    .bits();
            cathode
                .apply_control_edges(result.str_event, result.rcd_falling)
                .map_err(|error| format!("display cathode control failed: {error:?}"))?;
        }
        Ok(segments)
    }
}

fn format_segments(segments: &[u8; 15]) -> String {
    segments
        .iter()
        .map(|segment| format!("{segment:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn digit_key(digit: u8) -> Hp67Key {
    match digit {
        0 => Hp67Key::Digit0,
        1 => Hp67Key::Digit1,
        2 => Hp67Key::Digit2,
        3 => Hp67Key::Digit3,
        4 => Hp67Key::Digit4,
        5 => Hp67Key::Digit5,
        6 => Hp67Key::Digit6,
        7 => Hp67Key::Digit7,
        8 => Hp67Key::Digit8,
        9 => Hp67Key::Digit9,
        _ => unreachable!("decimal digit"),
    }
}

fn expected_single_digit(digit: u8) -> [u8; 15] {
    let mut expected = [0u8; 15];
    expected[3] = DIGIT_SEGMENTS[usize::from(digit)];
    expected[4] = 0x80;
    expected
}

fn boot_harness() -> Result<Harness, String> {
    let mut harness = Harness::default();
    let boot = harness.boot_to_idle()?;
    if boot != EXPECTED_BOOT_DISPLAY {
        return Err(format!(
            "boot display mismatch: got [{}], expected [{}]",
            format_segments(&boot),
            format_segments(&EXPECTED_BOOT_DISPLAY)
        ));
    }
    Ok(harness)
}

fn direct_keyboard_matrix() -> Result<(), String> {
    println!("\n=== DIRECT 35-KEY MATRIX ===");
    for spec in ALL_KEYS {
        let mut harness = boot_harness()?;
        let dispatch = harness.press_to_dispatch(spec.key, spec.label)?;
        harness.wait_for_contact_release(spec.label)?;
        println!(
            "DIRECT PASS: {:<7} code={:04o} -> table={:04o}",
            spec.label, dispatch.code, dispatch.target
        );
    }
    println!("DIRECT MATRIX PASS: all 35 physical contacts reached firmware dispatch.");
    Ok(())
}

fn digit_matrix() -> Result<(), String> {
    println!("\n=== DIGIT DISPLAY MATRIX ===");
    for digit in 0u8..=9 {
        let mut harness = boot_harness()?;
        let display = harness.press_and_settle(digit_key(digit), &digit.to_string())?;
        let expected = expected_single_digit(digit);
        if display != expected {
            return Err(format!(
                "digit {digit}: got [{}], expected [{}]",
                format_segments(&display),
                format_segments(&expected)
            ));
        }
        println!("DIGIT PASS: {digit} -> {}", format_segments(&display));
    }
    Ok(())
}

fn exact_sequence(
    name: &str,
    sequence: &[(Hp67Key, &str)],
    expected: [u8; 15],
) -> Result<(), String> {
    let mut harness = boot_harness()?;
    let (&(first_key, first_label), rest) = sequence
        .split_first()
        .ok_or_else(|| format!("{name}: empty key sequence"))?;
    let mut display = harness.press_and_settle(first_key, first_label)?;
    for &(key, label) in rest {
        display = harness.press_and_settle(key, label)?;
    }
    if display != expected {
        return Err(format!(
            "{name}: got [{}], expected [{}]",
            format_segments(&display),
            format_segments(&expected)
        ));
    }
    println!("{name} PASS: {}", format_segments(&display));
    Ok(())
}

fn basic_function_matrix() -> Result<(), String> {
    println!("\n=== BASIC FUNCTION MATRIX ===");

    exact_sequence(
        "DECIMAL 1.2",
        &[
            (Hp67Key::Digit1, "1"),
            (Hp67Key::Decimal, "."),
            (Hp67Key::Digit2, "2"),
        ],
        EXPECTED_ONE_POINT_TWO,
    )?;

    exact_sequence(
        "CLX",
        &[(Hp67Key::Digit8, "8"), (Hp67Key::ClearX, "CLX")],
        EXPECTED_CLEAR_DISPLAY,
    )?;

    exact_sequence(
        "ADD",
        &[
            (Hp67Key::Digit1, "1"),
            (Hp67Key::Enter, "ENTER"),
            (Hp67Key::Digit2, "2"),
            (Hp67Key::Add, "+"),
        ],
        EXPECTED_THREE_FIXED_TWO,
    )?;

    exact_sequence(
        "SUBTRACT",
        &[
            (Hp67Key::Digit5, "5"),
            (Hp67Key::Enter, "ENTER"),
            (Hp67Key::Digit2, "2"),
            (Hp67Key::Subtract, "-"),
        ],
        EXPECTED_THREE_FIXED_TWO,
    )?;

    exact_sequence(
        "MULTIPLY",
        &[
            (Hp67Key::Digit2, "2"),
            (Hp67Key::Enter, "ENTER"),
            (Hp67Key::Digit3, "3"),
            (Hp67Key::Multiply, "*"),
        ],
        EXPECTED_THREE_FIXED_TWO,
    )?;

    exact_sequence(
        "DIVIDE",
        &[
            (Hp67Key::Digit6, "6"),
            (Hp67Key::Enter, "ENTER"),
            (Hp67Key::Digit2, "2"),
            (Hp67Key::Divide, "/"),
        ],
        EXPECTED_THREE_FIXED_TWO,
    )?;

    let mut chs = boot_harness()?;
    chs.press_and_settle(Hp67Key::Digit1, "1")?;
    let chs_display = chs.press_and_settle(Hp67Key::ChangeSign, "CHS")?;
    println!("CHS FRONTIER: {}", format_segments(&chs_display));

    let mut eex = boot_harness()?;
    eex.press_and_settle(Hp67Key::Digit1, "1")?;
    eex.press_and_settle(Hp67Key::Exponent, "EEX")?;
    let eex_display = eex.press_and_settle(Hp67Key::Digit2, "2")?;
    println!("EEX FRONTIER: {}", format_segments(&eex_display));

    Ok(())
}

fn shifted_function_frontier() -> Result<(), String> {
    println!("\n=== SHIFTED FUNCTION FRONTIER ===");
    for prefix in [
        KeySpec {
            key: Hp67Key::FunctionF,
            label: "f",
        },
        KeySpec {
            key: Hp67Key::FunctionG,
            label: "g",
        },
        KeySpec {
            key: Hp67Key::FunctionH,
            label: "h",
        },
    ] {
        for target in ALL_KEYS {
            let mut harness = boot_harness()?;
            harness.press_and_settle(Hp67Key::Digit1, "1")?;
            harness.press_and_settle(prefix.key, prefix.label)?;
            let dispatch = harness.press_to_dispatch(target.key, target.label)?;
            harness.wait_for_contact_release(target.label)?;
            harness.run_words(SHIFT_EXERCISE_WORDS)?;
            println!(
                "SHIFT PASS: {} + {:<7} code={:04o} -> table={:04o} pc={:04o}",
                prefix.label,
                target.label,
                dispatch.code,
                dispatch.target,
                harness.machine.pc()
            );
        }
    }
    println!("SHIFT FRONTIER PASS: f/g/h paths executed without host-side semantics or execution errors.");
    Ok(())
}

fn main() -> Result<(), String> {
    if Hp67Firmware::populated_words() != 5120 {
        return Err(format!(
            "HP-67 firmware has {} populated words; expected 5120",
            Hp67Firmware::populated_words()
        ));
    }

    println!("HP-67 M11 physical keyboard/function matrix");
    println!("ROM SOURCE: versioned hp67firmware embedded in executable");
    println!("HOST CALCULATOR SEMANTICS: none");

    direct_keyboard_matrix()?;
    digit_matrix()?;
    basic_function_matrix()?;
    shifted_function_frontier()?;

    println!(
        "\nM11 FRONTIER PASS: 35 direct keys, exact digit/basic arithmetic paths, and f/g/h shifted dispatch paths traversed real firmware. CHS/EEX and shifted-function outputs are intentionally reported as frontiers for the next exact-lock pass."
    );
    Ok(())
}
