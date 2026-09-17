//! Proves real HP-67 arithmetic through physical keyboard contacts and embedded firmware.
//!
//! Sequence: `1 ENTER 2 +`. Every key is presented as a physical contact, held
//! until real firmware executes `keys -> a` and dispatches through the unshifted
//! key table, then released. No host-side calculator operation or display value
//! is injected. The final assertion is the physical ROM0/cathode `3.00` pattern.

use hp67emu::machines::hp67::{
    decode_rom0_display_byte, run_structural_display_fetch_cycle, ActInstructionState,
    ActSerialEndpoint, CathodeDriver1820_1749, EmbeddedHp67Rom, FetchPipelineLatch,
    Hp67ArchitecturalMachine, Hp67ElectricalBackplane, Hp67Key, Hp67Keyboard, Rom0DisplayEndpoint,
    RomFetchEndpoint, HP67_DISPLAY_SCAN_SLOTS, HP67_KEY_PRESSED_STATUS_BIT,
};

const BOOT_WORD_LIMIT: u64 = 2_000;
const KEY_DISPATCH_WORD_LIMIT: u64 = 512;
const KEY_SETTLE_WORD_LIMIT: u64 = 4_096;

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
const EXPECTED_DIGIT_ONE_DISPLAY: [u8; 15] = [
    0x00, 0x00, 0x00, 0x06, 0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];
const EXPECTED_SUM_DISPLAY: [u8; 15] = [
    0x00, 0x00, 0x00, 0x4f, 0x80, 0x3f, 0x3f, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00,
];

#[derive(Debug, Clone, Copy)]
struct ExecutedWord {
    pc: u16,
    word: u16,
    prior_instruction_state: ActInstructionState,
}

struct Harness {
    source: EmbeddedHp67Rom,
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
            source: EmbeddedHp67Rom::default(),
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
                .map_err(|error| format!("cycle {cycle} serial execution start failed: {error:?}"))?;
            let execution = self
                .machine
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
            "embedded firmware did not reach power-on idle within {BOOT_WORD_LIMIT} words; pc={:04o}",
            self.machine.pc()
        ))
    }

    fn press_and_settle(&mut self, key: Hp67Key, label: &str) -> Result<[u8; 15], String> {
        let code = key.scan_code();
        self.keyboard.press(key);
        let mut saw_keys_to_a = false;
        let mut dispatched = false;

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
                    if !(UNSHIFTED_KEY_TABLE_FIRST..=UNSHIFTED_KEY_TABLE_LAST)
                        .contains(&self.machine.pc())
                    {
                        return Err(format!(
                            "{label}: unshifted key dispatched to {:04o}, outside {:04o}..={:04o}",
                            self.machine.pc(),
                            UNSHIFTED_KEY_TABLE_FIRST,
                            UNSHIFTED_KEY_TABLE_LAST
                        ));
                    }
                    println!(
                        "KEY DISPATCH: {label:<5} code={code:04o} -> table={:04o}",
                        self.machine.pc()
                    );
                    dispatched = true;
                    break;
                }
            }
        }

        if !dispatched {
            return Err(format!(
                "{label}: firmware did not complete keyboard dispatch within {KEY_DISPATCH_WORD_LIMIT} words; pc={:04o}; s15={}",
                self.machine.pc(),
                self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
            ));
        }

        self.keyboard.release();
        for _ in 0..KEY_SETTLE_WORD_LIMIT {
            if let Some(executed) = self.step()? {
                if executed.pc == MAIN_WAIT_PC
                    && self.machine.act.state.display_enable
                    && !self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
                {
                    let display = self.capture_display()?;
                    println!("DISPLAY AFTER {label:<5}: {}", format_segments(&display));
                    return Ok(display);
                }
            }
        }

        Err(format!(
            "{label}: firmware did not settle back to no-key main wait within {KEY_SETTLE_WORD_LIMIT} words; pc={:04o}; s15={}",
            self.machine.pc(),
            self.machine.act.state.status[HP67_KEY_PRESSED_STATUS_BIT]
        ))
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

fn main() -> Result<(), String> {
    if EmbeddedHp67Rom::populated_words() != 5120 {
        return Err(format!(
            "embedded HP-67 firmware has {} populated words; expected 5120",
            EmbeddedHp67Rom::populated_words()
        ));
    }

    println!("HP-67 embedded-firmware physical arithmetic smoke");
    println!("ROM SOURCE: embedded in executable; no runtime firmware file");

    let mut harness = Harness::default();
    let boot = harness.boot_to_idle()?;
    if boot != EXPECTED_BOOT_DISPLAY {
        return Err(format!(
            "M7 embedded boot display mismatch: got [{}], expected [{}]",
            format_segments(&boot),
            format_segments(&EXPECTED_BOOT_DISPLAY)
        ));
    }
    println!("M7 PASS: embedded real firmware reached physical 0.00 idle");

    let one = harness.press_and_settle(Hp67Key::Digit1, "1")?;
    if one != EXPECTED_DIGIT_ONE_DISPLAY {
        return Err(format!(
            "M9 Digit1 display mismatch: got [{}], expected physical 1. pattern [{}]",
            format_segments(&one),
            format_segments(&EXPECTED_DIGIT_ONE_DISPLAY)
        ));
    }
    println!("M9 PASS: physical Digit1 produced physical 1. display");

    harness.press_and_settle(Hp67Key::Enter, "ENTER")?;
    harness.press_and_settle(Hp67Key::Digit2, "2")?;
    let sum = harness.press_and_settle(Hp67Key::Add, "+")?;

    if sum != EXPECTED_SUM_DISPLAY {
        return Err(format!(
            "M10 arithmetic display mismatch: got [{}], expected physical 3.00 pattern [{}]",
            format_segments(&sum),
            format_segments(&EXPECTED_SUM_DISPLAY)
        ));
    }

    println!(
        "M10 ARITHMETIC PASS: physical 1 -> ENTER -> 2 -> + traversed real embedded firmware and produced the ROM0/1820-1749 physical 3.00 display without host calculator semantics."
    );
    Ok(())
}
