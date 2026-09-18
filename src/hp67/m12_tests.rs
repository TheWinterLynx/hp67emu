use super::{Hp67LiveMachine, LiveBootPhase};
use hp67emu::machines::hp67::{
    ActOperation, ActRegister, Hp67ArchitecturalOperation, Hp67Key,
};

#[test]
fn live_program_switch_drives_crc_external_flag() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();

    live.set_program_mode(true).unwrap();
    assert_eq!(
        live.machine
            .crc
            .external_flag(hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE),
        Some(true)
    );

    live.set_program_mode(false).unwrap();
    assert_eq!(
        live.machine
            .crc
            .external_flag(hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE),
        Some(false)
    );
}

const HP67_PROGRAM_RAM_START: u8 = 0x10;
const HP67_PROGRAM_RAM_END: u8 = 0x2f;
const HP67_PROGRAM_PC_RAM: u8 = 0x3d;
const HP67_PROGRAM_RAM_WORDS: usize =
    (HP67_PROGRAM_RAM_END - HP67_PROGRAM_RAM_START + 1) as usize;
const KEYS_TO_A_OPCODE: u16 = 0o0120;
const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;
const MODE_SWITCH_CYCLE_LIMIT: usize = 8_192;
const KEY_DISPATCH_CYCLE_LIMIT: usize = 512;
const FIRMWARE_SETTLE_CYCLE_LIMIT: usize = 4_096;
const PROGRAM_RUN_CYCLE_LIMIT: usize = 20_000;

fn live_program_ram_snapshot(
    live: &Hp67LiveMachine,
) -> [Option<ActRegister>; HP67_PROGRAM_RAM_WORDS] {
    std::array::from_fn(|offset| {
        live.machine
            .ram
            .read(HP67_PROGRAM_RAM_START + offset as u8)
    })
}

fn wait_for_firmware_mode(live: &mut Hp67LiveMachine, program: bool, label: &str) {
    live.set_program_mode(program).unwrap();
    let wait_visits = live.main_wait_visits;
    for _ in 0..MODE_SWITCH_CYCLE_LIMIT {
        live.step_firmware_cycle().unwrap();
        if live.main_wait_visits > wait_visits
            && !live.machine.act.state.status[15]
            && live.machine.act.state.status[11] == program
        {
            return;
        }
    }
    panic!(
        "{label} switch transition did not settle in firmware; pc={:04o} s11={} s15={}",
        live.machine.pc(),
        live.machine.act.state.status[11],
        live.machine.act.state.status[15]
    );
}

fn press_live_key_to_dispatch(live: &mut Hp67LiveMachine, key: Hp67Key) -> u16 {
    let expected_code = key.scan_code();
    live.set_key_contact(Some(key));
    let mut saw_keys_to_a = false;

    for _ in 0..KEY_DISPATCH_CYCLE_LIMIT {
        let execution = live.step_firmware_cycle_with_execution().unwrap();
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
            assert_eq!(
                observed, expected_code,
                "{key:?} keys -> A produced {observed:04o}, expected physical code {expected_code:04o}"
            );
            saw_keys_to_a = true;
        }

        if matches!(
            execution.operation,
            Hp67ArchitecturalOperation::Act(ActOperation::Special {
                opcode: A_TO_ROM_ADDRESS_OPCODE
            })
        ) {
            assert!(
                saw_keys_to_a,
                "{key:?} executed A -> ROM address before keys -> A"
            );
            live.set_key_contact(None);
            return execution.next_pc;
        }
    }

    live.set_key_contact(None);
    panic!(
        "{key:?} firmware did not complete keys -> A / A -> ROM dispatch; pc={:04o} s11={} s15={}",
        live.machine.pc(),
        live.machine.act.state.status[11],
        live.machine.act.state.status[15]
    );
}

fn press_live_key_to_expected_dispatch(
    live: &mut Hp67LiveMachine,
    key: Hp67Key,
    expected_target: u16,
) -> u16 {
    let target = press_live_key_to_dispatch(live, key);
    assert_eq!(
        target, expected_target,
        "{key:?} firmware dispatch target mismatch"
    );
    target
}

fn press_program_key_and_require_ram_change(
    live: &mut Hp67LiveMachine,
    key: Hp67Key,
    expected_step: u8,
) {
    assert!(
        (1..=7).contains(&expected_step),
        "this M12 oracle covers the first seven program steps"
    );

    let before_program = live_program_ram_snapshot(live);
    let before_pc = live.machine.ram.read(HP67_PROGRAM_PC_RAM);
    let dispatch_target = press_live_key_to_dispatch(live, key);
    assert!(
        (0o1405..=0o1466).contains(&dispatch_target),
        "PROGRAM {key:?} dispatched outside the unshifted HP-67 key table: {dispatch_target:04o}"
    );

    let wait_visits = live.main_wait_visits;
    let mut settled = false;
    for _ in 0..FIRMWARE_SETTLE_CYCLE_LIMIT {
        live.step_firmware_cycle().unwrap();
        if live.machine.ram.read(HP67_PROGRAM_PC_RAM) != before_pc
            && live.main_wait_visits > wait_visits
            && !live.machine.act.state.status[15]
        {
            settled = true;
            break;
        }
    }

    let after_program = live_program_ram_snapshot(live);
    assert_ne!(
        after_program, before_program,
        "PROGRAM {key:?} advanced no program RAM after dispatch {dispatch_target:04o}"
    );

    let after_pc = live.machine.ram.read(HP67_PROGRAM_PC_RAM);
    assert_ne!(
        after_pc, before_pc,
        "PROGRAM {key:?} did not advance the user-program counter in RAM 0x3D after dispatch {dispatch_target:04o}; pc={:04o}",
        live.machine.pc()
    );
    assert!(
        settled,
        "PROGRAM {key:?} advanced state but did not return to the no-key firmware wait"
    );

    let pc_register = after_pc.expect("HP-67 program-counter register 0x3D must be installed");
    assert_eq!(
        &pc_register[0..3],
        &[0x0f, 0x02, 7 - expected_step],
        "PROGRAM {key:?} did not leave RAM 0x3D at expected user step {expected_step:03}"
    );
}

fn settle_live_dispatch(live: &mut Hp67LiveMachine, key: Hp67Key, target: u16) {
    let wait_visits = live.main_wait_visits;
    for _ in 0..FIRMWARE_SETTLE_CYCLE_LIMIT {
        live.step_firmware_cycle().unwrap();
        if live.main_wait_visits > wait_visits && !live.machine.act.state.status[15] {
            return;
        }
    }
    panic!(
        "{key:?} dispatch {target:04o} did not return to the no-key firmware wait; pc={:04o}",
        live.machine.pc()
    );
}

fn press_live_key_and_settle(live: &mut Hp67LiveMachine, key: Hp67Key, expected_target: u16) {
    let target = press_live_key_to_expected_dispatch(live, key, expected_target);
    settle_live_dispatch(live, key, target);
}

#[test]
fn live_program_mode_stores_and_executes_simple_program() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.phase = LiveBootPhase::Firmware;
    while !matches!(live.phase, LiveBootPhase::Idle) {
        live.step_firmware_cycle().unwrap();
    }

    wait_for_firmware_mode(&mut live, true, "RUN -> PRGM");
    assert!(
        live.machine.act.state.status[11],
        "firmware did not latch PROGRAM mode in S11"
    );

    press_program_key_and_require_ram_change(&mut live, Hp67Key::Digit1, 1);
    press_program_key_and_require_ram_change(&mut live, Hp67Key::Enter, 2);
    press_program_key_and_require_ram_change(&mut live, Hp67Key::Digit2, 3);
    press_program_key_and_require_ram_change(&mut live, Hp67Key::Add, 4);
    press_program_key_and_require_ram_change(&mut live, Hp67Key::RunStop, 5);

    let first_program_register = live
        .machine
        .ram
        .read(0x2f)
        .expect("HP-67 RAM 0x2F must hold program steps 001..007");
    assert_eq!(
        &first_program_register[0..10],
        &[0x01, 0x01, 0x0b, 0x01, 0x02, 0x01, 0x07, 0x03, 0x00, 0x00],
        "PROGRAM steps 001..005 are not the expected 11 1B 12 37 00 byte sequence"
    );

    wait_for_firmware_mode(&mut live, false, "PRGM -> RUN");
    assert!(
        !live.machine.act.state.status[11],
        "firmware retained PROGRAM-mode latch S11 after returning to RUN"
    );

    // HP-67 RUN-mode RTN with no program running clears the return stack and
    // sets the user-program counter to step 000. Use the real h -> RTN key
    // path and verify the physical counter register rather than normalizing X.
    press_live_key_and_settle(&mut live, Hp67Key::FunctionH, 0o1405);
    press_live_key_and_settle(&mut live, Hp67Key::Gto, 0o0560);
    assert_eq!(
        live.machine.ram.read(HP67_PROGRAM_PC_RAM),
        Some([0; 14]),
        "RUN-mode RTN did not clear RAM 0x3D to user-program step 000"
    );

    let wait_visits = live.main_wait_visits;
    press_live_key_to_expected_dispatch(&mut live, Hp67Key::RunStop, 0o1443);

    let mut saw_running = false;
    for _ in 0..PROGRAM_RUN_CYCLE_LIMIT {
        live.step_firmware_cycle().unwrap();
        saw_running |= live.machine.act.state.status[2];
        if saw_running
            && !live.machine.act.state.status[2]
            && live.main_wait_visits > wait_visits
            && !live.machine.act.state.status[15]
            && live.display_frame().segments()
                == &[0x00, 0x4f, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
        {
            assert_eq!(
                live.machine.ram.read(HP67_PROGRAM_PC_RAM),
                Some([0x0f, 0x02, 0x01, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]),
                "stored R/S halted without advancing the user-program counter to step 006"
            );
            return;
        }
    }

    panic!(
        "stored 11 1B 12 37 00 program did not run and halt at physical 3.00; pc={:04o} \
         saw_running={} s2={} s11={} s15={} wait_visits_before={} wait_visits_after={} \
         ram3d={:?} display={:02x?} A={:x?} B={:x?} C={:x?}",
        live.machine.pc(),
        saw_running,
        live.machine.act.state.status[2],
        live.machine.act.state.status[11],
        live.machine.act.state.status[15],
        wait_visits,
        live.main_wait_visits,
        live.machine.ram.read(HP67_PROGRAM_PC_RAM),
        live.display_frame().segments(),
        live.machine.act.state.a,
        live.machine.act.state.b,
        live.machine.act.state.c,
    );
}

