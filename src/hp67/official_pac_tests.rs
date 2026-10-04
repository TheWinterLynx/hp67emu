//! Behavioral acceptance from Standard Pac Moving Average, manual 01-03.
use super::diagnostic_tests::{boot_live_machine, expected_display, load_physical_program_card};
use super::m12_tests::press_live_key_to_dispatch;
use super::*;
use crate::program_library::{ProgramLibraryEntry, PROGRAM_LIBRARY};
use hp67emu::machines::hp67::{Hp67CardTrack, Hp67Key};

fn moving_average_entry() -> &'static ProgramLibraryEntry {
    PROGRAM_LIBRARY
        .iter()
        .find(|entry| entry.reference == "SD1-01A")
        .expect("checked-in Moving Average card")
}

#[test]
fn m14p_moving_average_requires_both_recorded_tracks() {
    let card = moving_average_entry()
        .load_card()
        .expect("valid library card")
        .card;
    assert!(card.track(Hp67CardTrack::Track1).is_recorded());
    assert!(card.track(Hp67CardTrack::Track2).is_recorded());
}

fn wait_for_result(live: &mut Hp67LiveMachine, expected: Option<&str>) -> Result<usize, String> {
    let segments = expected.map(expected_display).transpose()?;
    let wait_before = live.main_wait_visits;
    for cycle in 1..=1_000_000 {
        live.step_firmware_cycle()?;
        if live.main_wait_visits > wait_before
            && !live.machine.act.state.status[15]
            && !live.machine.act.state.status[2]
            && segments
                .as_ref()
                .map_or(true, |frame| live.display_frame().segments() == frame)
        {
            return Ok(cycle);
        }
    }
    Err(format!(
        "expected {expected:?}; pc={:04o} running={} key_pending={} segments={:02x?}",
        live.machine.pc(),
        live.machine.act.state.status[2],
        live.machine.act.state.status[15],
        live.display_frame().segments()
    ))
}

#[test]
#[ignore = "official Pac acceptance through physical card and firmware; run explicitly in release"]
fn m14p_official_moving_average_acceptance() {
    use Hp67Key::*;
    // HP-97 Standard Pac 01-03 Example 1, shared functional operations only.
    // The final row continues in memory; it does not claim a power-off/data-card round trip.
    let checkpoints: &[(&str, &[Hp67Key], &str)] = &[
        ("window", &[Digit6, FunctionF, A], "6.00"),
        ("125", &[Digit1, Digit2, Digit5, A], "1.00"),
        ("183", &[Digit1, Digit8, Digit3, A], "2.00"),
        ("207", &[Digit2, Digit0, Digit7, A], "3.00"),
        ("partial mean", &[D], "171.67"),
        ("222", &[Digit2, Digit2, Digit2, A], "4.00"),
        ("198", &[Digit1, Digit9, Digit8, A], "5.00"),
        ("240 / full mean", &[Digit2, Digit4, Digit0, A], "195.83"),
        ("225 / rolling mean", &[Digit2, Digit2, Digit5, A], "212.50"),
    ];
    let mut live = boot_live_machine().expect("firmware boot");
    let card = moving_average_entry()
        .load_card()
        .expect("valid library card")
        .card;
    load_physical_program_card(&mut live, card).expect("firmware two-end card load");
    println!("\nM14P OFFICIAL MOVING AVERAGE ACCEPTANCE");
    for (index, &(description, keys, expected)) in checkpoints.iter().enumerate() {
        let mut cycles = 0;
        for (key_index, &key) in keys.iter().enumerate() {
            press_live_key_to_dispatch(&mut live, key);
            let result = if key_index + 1 == keys.len() {
                Some(expected)
            } else {
                None
            };
            cycles += wait_for_result(&mut live, result).unwrap_or_else(|error| {
                panic!(
                    "checkpoint {} {description}, key {key:?}: {error}",
                    index + 1
                )
            });
        }
        println!(
            "{:02} {description:<24} {expected:>8} {cycles:>10} OK",
            index + 1
        );
    }
    println!(
        "OFFICIAL PAC ACCEPTANCE OK: {}/{} checkpoints passed",
        checkpoints.len(),
        checkpoints.len()
    );
}
