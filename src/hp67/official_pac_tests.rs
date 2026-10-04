//! Behavioral acceptance from Standard Pac Moving Average, manual 01-03.
use super::diagnostic_tests::{
    boot_live_machine, expected_display, load_physical_program_card, wait_for_card_pass,
    wait_for_crd_prompt,
};
use super::m12_tests::press_live_key_to_dispatch;
use super::*;
use crate::program_library::{ProgramLibraryEntry, PROGRAM_LIBRARY};
use hp67emu::machines::hp67::{Hp67CardTrack, Hp67Key, Hp67MagneticCard};

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
    // HP-97 Standard Pac 01-03 Example 1, shared functional operations only.
    // The final row continues in memory; it does not claim a power-off/data-card round trip.
    let mut live = boot_live_machine().expect("firmware boot");
    let card = moving_average_entry()
        .load_card()
        .expect("valid library card")
        .card;
    load_physical_program_card(&mut live, card).expect("firmware two-end card load");
    println!("\nM14P OFFICIAL MOVING AVERAGE ACCEPTANCE");
    run_checkpoints(&mut live, MOVING_AVERAGE_CHECKPOINTS);
    println!(
        "OFFICIAL PAC ACCEPTANCE OK: {}/{} checkpoints passed",
        MOVING_AVERAGE_CHECKPOINTS.len(),
        MOVING_AVERAGE_CHECKPOINTS.len()
    );
}

const MOVING_AVERAGE_CHECKPOINTS: &[(&str, &[Hp67Key], &str)] = &[
    (
        "window",
        &[Hp67Key::Digit6, Hp67Key::FunctionF, Hp67Key::A],
        "6.00",
    ),
    (
        "125",
        &[
            Hp67Key::Digit1,
            Hp67Key::Digit2,
            Hp67Key::Digit5,
            Hp67Key::A,
        ],
        "1.00",
    ),
    (
        "183",
        &[
            Hp67Key::Digit1,
            Hp67Key::Digit8,
            Hp67Key::Digit3,
            Hp67Key::A,
        ],
        "2.00",
    ),
    (
        "207",
        &[
            Hp67Key::Digit2,
            Hp67Key::Digit0,
            Hp67Key::Digit7,
            Hp67Key::A,
        ],
        "3.00",
    ),
    ("partial mean", &[Hp67Key::D], "171.67"),
    (
        "222",
        &[
            Hp67Key::Digit2,
            Hp67Key::Digit2,
            Hp67Key::Digit2,
            Hp67Key::A,
        ],
        "4.00",
    ),
    (
        "198",
        &[
            Hp67Key::Digit1,
            Hp67Key::Digit9,
            Hp67Key::Digit8,
            Hp67Key::A,
        ],
        "5.00",
    ),
    (
        "240 / full mean",
        &[
            Hp67Key::Digit2,
            Hp67Key::Digit4,
            Hp67Key::Digit0,
            Hp67Key::A,
        ],
        "195.83",
    ),
    (
        "225 / rolling mean",
        &[
            Hp67Key::Digit2,
            Hp67Key::Digit2,
            Hp67Key::Digit5,
            Hp67Key::A,
        ],
        "212.50",
    ),
];

fn run_checkpoints(live: &mut Hp67LiveMachine, checkpoints: &[(&str, &[Hp67Key], &str)]) {
    for (index, &(description, keys, expected)) in checkpoints.iter().enumerate() {
        let mut cycles = 0;
        for (key_index, &key) in keys.iter().enumerate() {
            press_live_key_to_dispatch(live, key);
            let result = if key_index + 1 == keys.len() {
                Some(expected)
            } else {
                None
            };
            cycles += wait_for_result(live, result).unwrap_or_else(|error| {
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
}

#[test]
#[ignore = "official saved-data recovery through firmware card write/read; run explicitly in release"]
fn m14q_official_moving_average_data_recovery() {
    let mut writer = boot_live_machine().expect("writer firmware boot");
    load_physical_program_card(
        &mut writer,
        moving_average_entry()
            .load_card()
            .expect("program card")
            .card,
    )
    .expect("writer program load");
    println!("\nM14Q OFFICIAL MOVING AVERAGE DATA RECOVERY");
    run_checkpoints(&mut writer, &MOVING_AVERAGE_CHECKPOINTS[..8]);

    press_live_key_to_dispatch(&mut writer, Hp67Key::B);
    wait_for_crd_prompt(&mut writer).expect("B must request a blank data card");
    assert!(
        writer.card_write_mode(),
        "B must enter firmware data write mode"
    );
    writer
        .insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
        .expect("blank data card insertion");
    wait_for_card_pass(&mut writer).expect("firmware data write pass");
    let saved = writer
        .take_completed_magnetic_card()
        .expect("written physical card returned");
    wait_for_result(&mut writer, None).expect("single data pass must return to idle");
    assert_eq!(
        writer.machine.crc.queued_write_words(),
        0,
        "CRC write queue drained"
    );
    assert_eq!(
        saved
            .track(Hp67CardTrack::Track1)
            .word(0)
            .map(|word| (word >> 24) as u8),
        Some(1),
        "six-point window must use primary data header 1"
    );
    assert!(
        !saved.track(Hp67CardTrack::Track2).is_recorded(),
        "six-point window needs one data pass; opposite track must stay blank"
    );
    assert!(
        saved.track(Hp67CardTrack::Track1).dirty(),
        "firmware must materialize blank media"
    );
    println!("DATA WRITE OK: header 1; opposite track blank; CRC drained");

    // Only the lossless physical media crosses the fresh-machine boundary.
    let bytes = saved.to_hp67card_bytes();
    let restored = Hp67MagneticCard::from_hp67card_bytes(&bytes).expect("native persisted card");
    assert_eq!(
        restored.to_hp67card_bytes(),
        bytes,
        "lossless physical media persistence"
    );
    drop(writer);
    let mut reader = boot_live_machine().expect("fresh reader firmware boot");
    load_physical_program_card(
        &mut reader,
        moving_average_entry()
            .load_card()
            .expect("fresh program card")
            .card,
    )
    .expect("fresh two-end program load");
    load_physical_program_card(&mut reader, restored).expect("saved data read through firmware");
    run_checkpoints(
        &mut reader,
        &[
            ("recovered mean", &[Hp67Key::D], "195.83"),
            (
                "225 / recovered rolling",
                &[
                    Hp67Key::Digit2,
                    Hp67Key::Digit2,
                    Hp67Key::Digit5,
                    Hp67Key::A,
                ],
                "212.50",
            ),
            ("recovered mean query", &[Hp67Key::D], "212.50"),
        ],
    );
    println!("OFFICIAL PAC DATA RECOVERY OK: write / persist / fresh boot / program reload / data read / 3 checkpoints");
}

// New boundary fixtures based on the manual's general instructions, not printed HP examples.
#[test]
#[ignore = "nine/ten-point firmware data-card pass boundary and recovery; run explicitly in release"]
fn m14r_moving_average_data_card_pass_boundary() {
    use Hp67Key::*;
    let first_eight: &[(&str, &[Hp67Key], &str)] = &[
        ("1", &[Digit1, A], "1.00"),
        ("2", &[Digit2, A], "2.00"),
        ("3", &[Digit3, A], "3.00"),
        ("4", &[Digit4, A], "4.00"),
        ("5", &[Digit5, A], "5.00"),
        ("6", &[Digit6, A], "6.00"),
        ("7", &[Digit7, A], "7.00"),
        ("8", &[Digit8, A], "8.00"),
    ];
    println!("\nM14R MOVING AVERAGE DATA CARD PASS BOUNDARY");
    for two_passes in [false, true] {
        let mut writer = boot_live_machine().expect("writer boot");
        load_physical_program_card(
            &mut writer,
            moving_average_entry()
                .load_card()
                .expect("program card")
                .card,
        )
        .expect("writer program load");
        let (window_keys, window, mean, next_keys, rolling): (
            &[Hp67Key],
            &str,
            &str,
            &[Hp67Key],
            &str,
        ) = if two_passes {
            (
                &[Digit1, Digit0, FunctionF, A],
                "10.00",
                "5.50",
                &[Digit1, Digit1, A],
                "6.50",
            )
        } else {
            (
                &[Digit9, FunctionF, A],
                "9.00",
                "5.00",
                &[Digit1, Digit0, A],
                "6.00",
            )
        };
        println!(
            "WINDOW {window}, expected data passes={}",
            if two_passes { 2 } else { 1 }
        );
        run_checkpoints(&mut writer, &[("initialize", window_keys, window)]);
        run_checkpoints(&mut writer, first_eight);
        run_checkpoints(
            &mut writer,
            &[("9", &[Digit9, A], if two_passes { "9.00" } else { mean })],
        );
        if two_passes {
            run_checkpoints(
                &mut writer,
                &[("10 / full mean", &[Digit1, Digit0, A], mean)],
            );
        }
        run_checkpoints(&mut writer, &[("mean query", &[D], mean)]);

        press_live_key_to_dispatch(&mut writer, B);
        wait_for_crd_prompt(&mut writer).expect("B requests blank card");
        assert!(writer.card_write_mode(), "firmware write mode");
        writer
            .insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
            .expect("blank End1 insertion");
        wait_for_card_pass(&mut writer).expect("primary write pass");
        let mut saved = writer
            .take_completed_magnetic_card()
            .expect("same physical card returned");
        assert_eq!(
            saved
                .track(Hp67CardTrack::Track1)
                .word(0)
                .map(|word| (word >> 24) as u8),
            Some(1)
        );
        assert!(saved.track(Hp67CardTrack::Track1).dirty());
        assert!(!saved.track(Hp67CardTrack::Track2).is_recorded());
        let primary_words = *saved
            .track(Hp67CardTrack::Track1)
            .words()
            .expect("recorded primary");
        if two_passes {
            wait_for_crd_prompt(&mut writer).expect("ten-point firmware must request End2");
            assert!(writer.card_write_mode(), "second pass remains write mode");
            writer
                .insert_magnetic_card(saved, CardInsertionEnd::End2)
                .expect("same card End2 insertion");
            wait_for_card_pass(&mut writer).expect("secondary write pass");
            saved = writer
                .take_completed_magnetic_card()
                .expect("same card returned after End2");
            assert_eq!(
                saved.track(Hp67CardTrack::Track1).words(),
                Some(&primary_words),
                "End2 writing must preserve primary records"
            );
            assert_eq!(
                saved
                    .track(Hp67CardTrack::Track2)
                    .word(0)
                    .map(|word| (word >> 24) as u8),
                Some(2)
            );
            assert!(saved.track(Hp67CardTrack::Track2).dirty());
        }
        wait_for_result(&mut writer, None).expect("last write must return to RUN wait");
        assert!(
            !writer.card_prompt_visible(),
            "no extra firmware card request"
        );
        assert_eq!(writer.machine.crc.queued_write_words(), 0, "CRC drained");
        let bytes = saved.to_hp67card_bytes();
        let media =
            Hp67MagneticCard::from_hp67card_bytes(&bytes).expect("native persisted data card");
        assert_eq!(media.to_hp67card_bytes(), bytes);
        drop(writer);

        let mut reader = boot_live_machine().expect("fresh reader boot");
        load_physical_program_card(
            &mut reader,
            moving_average_entry()
                .load_card()
                .expect("fresh program card")
                .card,
        )
        .expect("fresh program load");
        let read_only = media.clone();
        // Explicit passes require real Crd for End2 and prove read mode/non-mutation.
        reader
            .insert_magnetic_card(media, CardInsertionEnd::End1)
            .expect("data End1 read");
        wait_for_card_pass(&mut reader).expect("primary read pass");
        assert!(!reader.card_write_mode(), "data read mode");
        let mut returned = reader
            .take_completed_magnetic_card()
            .expect("primary read returns card");
        assert_eq!(returned, read_only, "End1 read must preserve all media");
        if two_passes {
            wait_for_crd_prompt(&mut reader).expect("data header must request End2 read");
            reader
                .insert_magnetic_card(returned, CardInsertionEnd::End2)
                .expect("same card End2 read");
            wait_for_card_pass(&mut reader).expect("secondary read pass");
            assert!(!reader.card_write_mode(), "secondary read mode");
            returned = reader
                .take_completed_magnetic_card()
                .expect("secondary read returns card");
            assert_eq!(returned, read_only, "End2 read must preserve all media");
        }
        wait_for_result(&mut reader, None).expect("data read must settle");
        assert!(
            !reader.card_prompt_visible(),
            "no extra read pass requested"
        );
        run_checkpoints(
            &mut reader,
            &[
                ("recovered mean", &[D], mean),
                ("next / rolling mean", next_keys, rolling),
                ("rolling query", &[D], rolling),
            ],
        );
        println!("WINDOW {window} PASS BOUNDARY AND RECOVERY OK");
    }
    println!("DATA CARD PASS BOUNDARY OK: 2/2 cases (nine / ten points)");
}
