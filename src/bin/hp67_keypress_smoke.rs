//! Drives one physical HP-67 key contact through the same live machine used by the UI.
//!
//! The smoke deliberately does not inject calculator semantics. It boots the
//! external firmware to its no-key idle checkpoint, closes the physical `1`
//! contact, advances real firmware until the hardware display changes, releases
//! the contact, and then lets firmware settle the display.

use std::time::Duration;

use hp67emu::{
    hp67::{HardwareDisplayFrame, Hp67LiveMachine, KeyAction},
    machines::hp67::{HP67_OBSERVED_POWER_ON_SYNC_DELAY_US, HP67_OBSERVED_WORD_TIME_US},
};

const BOOT_WORD_LIMIT: u64 = 2_000;
const KEY_RESPONSE_WORD_LIMIT: u64 = 512;
const SETTLE_WORDS: u64 = 256;

fn format_segments(frame: HardwareDisplayFrame) -> String {
    frame
        .segments()
        .iter()
        .map(|byte| format!("{byte:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

fn advance_one_word(machine: &mut Hp67LiveMachine) -> Result<(), String> {
    machine.advance(Duration::from_micros(HP67_OBSERVED_WORD_TIME_US))
}

fn main() -> Result<(), String> {
    let mut machine = Hp67LiveMachine::power_on_default()?;
    machine.reset_power_on()?;

    machine.advance(Duration::from_micros(HP67_OBSERVED_POWER_ON_SYNC_DELAY_US))?;

    let mut boot_words = 0u64;
    while machine.is_booting() && boot_words < BOOT_WORD_LIMIT {
        advance_one_word(&mut machine)?;
        boot_words += 1;
    }
    if machine.is_booting() {
        return Err(format!(
            "HP-67 live machine did not reach firmware idle within {BOOT_WORD_LIMIT} words"
        ));
    }

    let idle_display = machine.display_frame();
    println!("HP-67 live physical-key smoke");
    println!("BOOT IDLE: words={boot_words}");
    println!("BOOT DISPLAY SEGMENTS: {}", format_segments(idle_display));

    machine.set_key_contact(Some(KeyAction::Digit(1)));
    println!("KEY CONTACT DOWN: digit 1, ACT scan code 0142 octal");

    let mut changed_after = None;
    let mut first_changed_display = idle_display;
    for offset in 1..=KEY_RESPONSE_WORD_LIMIT {
        advance_one_word(&mut machine)?;
        let frame = machine.display_frame();
        if frame != idle_display {
            changed_after = Some(offset);
            first_changed_display = frame;
            break;
        }
    }

    let Some(response_words) = changed_after else {
        machine.set_key_contact(None);
        return Err(format!(
            "digit-1 contact reached the live machine but firmware produced no display change within {KEY_RESPONSE_WORD_LIMIT} words"
        ));
    };

    println!("KEY RESPONSE: display first changed after {response_words} firmware word(s)");
    println!(
        "FIRST CHANGED DISPLAY SEGMENTS: {}",
        format_segments(first_changed_display)
    );

    machine.set_key_contact(None);
    println!("KEY CONTACT UP: digit 1 released");

    for _ in 0..SETTLE_WORDS {
        advance_one_word(&mut machine)?;
    }

    let settled_display = machine.display_frame();
    println!(
        "SETTLED DISPLAY SEGMENTS: {}",
        format_segments(settled_display)
    );

    if settled_display == idle_display {
        return Err(
            "firmware acknowledged digit 1 transiently but returned to the unchanged boot display"
                .to_owned(),
        );
    }

    println!(
        "KEYPRESS PASS: physical digit-1 contact -> ACT key input -> real firmware -> hardware display changed without calculator-level key semantics."
    );
    Ok(())
}
