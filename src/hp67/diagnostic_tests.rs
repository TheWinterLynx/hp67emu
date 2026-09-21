use super::*;

use hp67emu::machines::hp67::{Hp67CardTrack, Hp67Key, Hp67MagneticCard};

const CARD_PASS_CYCLE_LIMIT: usize = 65_536;
const CARD_PROMPT_CYCLE_LIMIT: usize = 16_384;
const CARD_SETTLE_CYCLE_LIMIT: usize = 16_384;

#[derive(Clone, Copy)]
struct DiagnosticCase {
    reference: &'static str,
    description: &'static str,
    expected: &'static str,
    media: &'static [u8],
    execution_cycle_limit: usize,
}

const DIAGNOSTICS: &[DiagnosticCase] = &[
    DiagnosticCase {
        reference: "CD-01",
        description: "GSB/GTO/RTN",
        expected: "7.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-01_Flow-GSB-GTO-RTN.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-02",
        description: "SF/CF/F?",
        expected: "6.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-02_Flags-SF-CF-Test.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-03",
        description: "conditionals",
        expected: "7.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-03_Conditionals.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-04",
        description: "indirect STO/RCL",
        expected: "42.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-04_Indirect-STO-RCL.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-05",
        description: "nested GSB",
        expected: "6.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-05_Nested-GSB.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-06",
        description: "second-half label search",
        expected: "67.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-06_Second-Half-Label-Search.hp67card"
        )),
        execution_cycle_limit: 500_000,
    },
    DiagnosticCase {
        reference: "CD-07",
        description: "ISZ loop",
        expected: "3.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-07_ISZ-Loop.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-08",
        description: "DSZ loop",
        expected: "3.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-08_DSZ-Loop.hp67card"
        )),
        execution_cycle_limit: 250_000,
    },
    DiagnosticCase {
        reference: "CD-09",
        description: "2000-iteration DSZ burn-in",
        expected: "2000.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-09_Long-DSZ-Burn-In.hp67card"
        )),
        execution_cycle_limit: 8_000_000,
    },
    DiagnosticCase {
        reference: "CD-10",
        description: "500 nested-GSB iterations",
        expected: "500.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-10_Nested-GSB-Burn-In.hp67card"
        )),
        execution_cycle_limit: 5_000_000,
    },
    DiagnosticCase {
        reference: "CD-11",
        description: "100 function-identity iterations",
        expected: "1.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-11_Function-Identity-Burn-In.hp67card"
        )),
        execution_cycle_limit: 5_000_000,
    },
    DiagnosticCase {
        reference: "CD-12",
        description: "cross-half nested-GSB burn-in",
        expected: "500.00",
        media: include_bytes!(concat!(
            env!("CARGO_MANIFEST_DIR"),
            "/programs/HP67/Custom Diagnostic Pacs/CD-12_Cross-Half-GSB-Burn-In.hp67card"
        )),
        execution_cycle_limit: 5_000_000,
    },
];

fn boot_live_machine() -> Result<Hp67LiveMachine, String> {
    let mut live = Hp67LiveMachine::power_on_default()?;
    live.phase = LiveBootPhase::Firmware;
    for _ in 0..BOOT_CYCLE_LIMIT {
        if matches!(live.phase, LiveBootPhase::Idle) {
            return Ok(live);
        }
        live.step_firmware_cycle()?;
    }
    Err(format!(
        "firmware did not reach idle within {BOOT_CYCLE_LIMIT} cycles"
    ))
}

fn wait_for_card_pass(live: &mut Hp67LiveMachine) -> Result<(), String> {
    for cycle in 0..CARD_PASS_CYCLE_LIMIT {
        live.step_firmware_cycle()
            .map_err(|error| format!("card pass failed at cycle {cycle}: {error}"))?;
        if live.card_transport_complete() {
            return Ok(());
        }
    }
    Err(format!(
        "card did not complete within {CARD_PASS_CYCLE_LIMIT} cycles"
    ))
}

fn wait_for_crd_prompt(live: &mut Hp67LiveMachine) -> Result<(), String> {
    for cycle in 0..CARD_PROMPT_CYCLE_LIMIT {
        live.step_firmware_cycle()
            .map_err(|error| format!("waiting for Crd failed at cycle {cycle}: {error}"))?;
        if live.card_prompt_visible() {
            return Ok(());
        }
    }
    Err(format!(
        "firmware did not request the opposite end within {CARD_PROMPT_CYCLE_LIMIT} cycles"
    ))
}

fn settle_after_card(live: &mut Hp67LiveMachine) -> Result<(), String> {
    let wait_before = live.main_wait_visits;
    for cycle in 0..CARD_SETTLE_CYCLE_LIMIT {
        live.step_firmware_cycle()
            .map_err(|error| format!("card settle failed at cycle {cycle}: {error}"))?;
        if live.main_wait_visits > wait_before
            && !live.card_motor_on()
            && !live.machine.act.state.status[15]
        {
            return Ok(());
        }
    }
    Err(format!(
        "firmware did not return to no-key RUN wait within {CARD_SETTLE_CYCLE_LIMIT} cycles"
    ))
}

fn load_native_program_card(
    live: &mut Hp67LiveMachine,
    media: &'static [u8],
) -> Result<(), String> {
    let mut card = Hp67MagneticCard::from_hp67card_bytes(media)
        .map_err(|error| format!("invalid native .hp67card: {error:?}"))?;

    let track1 = card.track(Hp67CardTrack::Track1).is_recorded();
    let track2 = card.track(Hp67CardTrack::Track2).is_recorded();
    if !track1 && !track2 {
        return Err("native card contains no recorded track".to_owned());
    }

    if track1 {
        live.insert_magnetic_card(card, CardInsertionEnd::End1)?;
        wait_for_card_pass(live)?;
        card = live
            .take_completed_magnetic_card()
            .ok_or_else(|| "Track 1 completed but the physical card was not returned".to_owned())?;
    }

    if track2 {
        if track1 {
            wait_for_crd_prompt(live)?;
        }
        live.insert_magnetic_card(card, CardInsertionEnd::End2)?;
        wait_for_card_pass(live)?;
        card = live
            .take_completed_magnetic_card()
            .ok_or_else(|| "Track 2 completed but the physical card was not returned".to_owned())?;
    }

    let _ = card;
    settle_after_card(live)
}

fn press_a_through_firmware(live: &mut Hp67LiveMachine) -> Result<u16, String> {
    const KEYS_TO_A_OPCODE: u16 = 0o0120;
    const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;

    let expected_code = Hp67Key::A.scan_code();
    live.set_key_contact(Some(Hp67Key::A));
    let mut saw_keys_to_a = false;

    for cycle in 0..512 {
        let execution = match live.step_firmware_cycle_with_execution() {
            Ok(execution) => execution,
            Err(error) => {
                live.set_key_contact(None);
                return Err(format!("A-key dispatch failed at cycle {cycle}: {error}"));
            }
        };
        let Some(execution) = execution else {
            continue;
        };

        if matches!(
            execution.operation,
            Hp67ArchitecturalOperation::Act(ActOperation::Special {
                opcode: KEYS_TO_A_OPCODE
            })
        ) {
            let observed = (live.machine.act.state.a[2] << 4) | live.machine.act.state.a[1];
            if observed != expected_code {
                live.set_key_contact(None);
                return Err(format!(
                    "A-key scan code mismatch: expected 0x{expected_code:02x}, got 0x{observed:02x}"
                ));
            }
            saw_keys_to_a = true;
        }

        if matches!(
            execution.operation,
            Hp67ArchitecturalOperation::Act(ActOperation::Special {
                opcode: A_TO_ROM_ADDRESS_OPCODE
            })
        ) {
            live.set_key_contact(None);
            if !saw_keys_to_a {
                return Err("A-key reached ROM dispatch before keys->A".to_owned());
            }
            return Ok(execution.next_pc);
        }
    }

    live.set_key_contact(None);
    Err("A-key did not reach firmware dispatch within 512 cycles".to_owned())
}

fn expected_display(text: &str) -> Result<[u8; HP67_DISPLAY_SCAN_SLOTS], String> {
    const DIGITS: [u8; 10] = [
        0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f,
    ];

    let mut segments = [0u8; HP67_DISPLAY_SCAN_SLOTS];
    let mut slot = 1usize;
    for character in text.chars() {
        if slot >= HP67_DISPLAY_SCAN_SLOTS {
            return Err(format!("expected display '{text}' exceeds physical display width"));
        }
        segments[slot] = match character {
            '0'..='9' => DIGITS[character.to_digit(10).unwrap() as usize],
            '.' => Hp67SegmentMask::DP.bits(),
            other => {
                return Err(format!(
                    "unsupported expected-display character '{other}' in '{text}'"
                ));
            }
        };
        slot += 1;
    }
    Ok(segments)
}

fn run_diagnostic(case: DiagnosticCase) -> Result<(usize, u16), String> {
    let mut live = boot_live_machine()?;
    load_native_program_card(&mut live, case.media)?;

    let expected = expected_display(case.expected)?;
    let dispatch = press_a_through_firmware(&mut live)?;
    let wait_before = live.main_wait_visits;

    for cycle in 1..=case.execution_cycle_limit {
        live.step_firmware_cycle()
            .map_err(|error| format!("execution failed at cycle {cycle}: {error}"))?;

        if live.main_wait_visits > wait_before
            && !live.machine.act.state.status[15]
            && !live.machine.act.state.status[2]
            && live.display_frame().segments() == &expected
        {
            return Ok((cycle, dispatch));
        }
    }

    Err(format!(
        "timeout after {} cycles; expected {} display={:02x?} pc={:04o} running={} key_pending={}",
        case.execution_cycle_limit,
        case.expected,
        live.display_frame().segments(),
        live.machine.pc(),
        live.machine.act.state.status[2],
        live.machine.act.state.status[15],
    ))
}

#[test]
#[ignore = "long-running full firmware/card diagnostic suite; run explicitly in --release"]
fn live_custom_diagnostic_pac_suite_reports_ok_ko() {
    println!();
    println!("HP-67 CUSTOM DIAGNOSTIC PAC SUITE");
    println!("=================================");
    println!(
        "{:<6} {:<36} {:<10} {:>10}  {}",
        "REF", "TEST", "EXPECTED", "CYCLES", "RESULT"
    );

    let mut failures = Vec::new();
    let mut passed = 0usize;

    for &case in DIAGNOSTICS {
        match run_diagnostic(case) {
            Ok((cycles, dispatch)) => {
                passed += 1;
                println!(
                    "{:<6} {:<36} {:<10} {:>10}  OK  dispatch={dispatch:04o}",
                    case.reference, case.description, case.expected, cycles
                );
            }
            Err(error) => {
                println!(
                    "{:<6} {:<36} {:<10} {:>10}  KO  {}",
                    case.reference, case.description, case.expected, "-", error
                );
                failures.push(format!("{}: {error}", case.reference));
            }
        }
    }

    println!("---------------------------------");
    if failures.is_empty() {
        println!("DIAGNOSTIC SUITE OK: {passed}/{} passed", DIAGNOSTICS.len());
    } else {
        println!(
            "DIAGNOSTIC SUITE KO: {passed}/{} passed, {} failed",
            DIAGNOSTICS.len(),
            failures.len()
        );
        for failure in &failures {
            println!("  {failure}");
        }
        panic!("HP-67 diagnostic suite reported KO");
    }
}
