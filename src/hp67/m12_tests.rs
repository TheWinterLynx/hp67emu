use super::{Hp67LiveMachine, LiveBootPhase};
use hp67emu::machines::hp67::{
    ActOperation, ActRegister, Hp67ArchitecturalOperation, Hp67Key, CRC_FLAG_PROGRAM_MODE,
    HP67_DISPLAY_SCAN_SLOTS,
};

#[test]
fn live_program_switch_drives_crc_external_flag() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();

    live.set_program_mode(true).unwrap();
    assert_eq!(
        live.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE),
        Some(true)
    );

    live.set_program_mode(false).unwrap();
    assert_eq!(
        live.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE),
        Some(false)
    );
}

const HP67_PROGRAM_PC_RAM: u8 = 0x3d;
const HP67_FIRST_PROGRAM_REGISTER: u8 = 0x2f;
const KEYS_TO_A_OPCODE: u16 = 0o0120;
const A_TO_ROM_ADDRESS_OPCODE: u16 = 0o0220;
const USER_INSTRUCTION_EXECUTE_PC: u16 = 0o6021;
const UNSHIFTED_KEY_TABLE_FIRST: u16 = 0o1405;
const UNSHIFTED_KEY_TABLE_LAST: u16 = 0o1466;
const H_SHIFTED_KEY_TABLE_FIRST: u16 = 0o0505;
const H_SHIFTED_KEY_TABLE_LAST: u16 = 0o0566;
const MODE_SWITCH_CYCLE_LIMIT: usize = 8_192;
const KEY_DISPATCH_CYCLE_LIMIT: usize = 512;
const FIRMWARE_SETTLE_CYCLE_LIMIT: usize = 4_096;
const PROGRAM_RUN_CYCLE_LIMIT: usize = 20_000;
const EXPECTED_PROGRAM_PREFIX: [u8; 10] =
    [0x01, 0x01, 0x0b, 0x01, 0x02, 0x01, 0x07, 0x03, 0x00, 0x00];
const EXPECTED_DELETED_STEP_003_PREFIX: [u8; 10] =
    [0x01, 0x01, 0x0b, 0x01, 0x07, 0x03, 0x00, 0x00, 0x00, 0x00];
const EXPECTED_EDITED_PROGRAM_PREFIX: [u8; 10] =
    [0x01, 0x01, 0x0b, 0x01, 0x03, 0x01, 0x07, 0x03, 0x00, 0x00];
const EXPECTED_3_00_FRAME: [u8; 15] = [0x00, 0x4f, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];
const EXPECTED_4_00_FRAME: [u8; 15] = [0x00, 0x66, 0x80, 0x3f, 0x3f, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0];

fn boot_live_to_idle() -> Hp67LiveMachine {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.phase = LiveBootPhase::Firmware;
    while !matches!(live.phase, LiveBootPhase::Idle) {
        live.step_firmware_cycle().unwrap();
    }
    live
}

fn first_page_program_pc(step: u8) -> ActRegister {
    assert!(step <= 7, "first-page M12 PC oracle covers steps 000..007");
    if step == 0 {
        return [0; 14];
    }
    let mut pc = [0; 14];
    pc[0] = 0x0f;
    pc[1] = 0x02;
    pc[2] = 7 - step;
    pc
}

fn assert_program_pc(live: &Hp67LiveMachine, step: u8, label: &str) {
    assert_eq!(
        live.machine.ram.read(HP67_PROGRAM_PC_RAM),
        Some(first_page_program_pc(step)),
        "{label}: user-program PC is not at step {step:03}"
    );
}

fn assert_program_prefix(live: &Hp67LiveMachine, expected: &[u8; 10], label: &str) {
    let register = live
        .machine
        .ram
        .read(HP67_FIRST_PROGRAM_REGISTER)
        .expect("HP-67 RAM 0x2F must hold program steps 001..007");
    assert_eq!(&register[0..10], expected, "{label}");
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

fn press_program_key_and_require_step_advance(
    live: &mut Hp67LiveMachine,
    key: Hp67Key,
    expected_step: u8,
) {
    assert!(
        (1..=7).contains(&expected_step),
        "this M12 oracle covers the first seven program steps"
    );

    let before_pc = live.machine.ram.read(HP67_PROGRAM_PC_RAM);
    let dispatch_target = press_live_key_to_dispatch(live, key);
    assert!(
        (UNSHIFTED_KEY_TABLE_FIRST..=UNSHIFTED_KEY_TABLE_LAST).contains(&dispatch_target),
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

#[derive(Debug, Clone, Copy, Default)]
struct SettleObservation {
    saw_single_step: bool,
    saw_user_instruction_execute: bool,
}

fn settle_live_dispatch_observing(
    live: &mut Hp67LiveMachine,
    key: Hp67Key,
    target: u16,
) -> SettleObservation {
    let wait_visits = live.main_wait_visits;
    let mut observation = SettleObservation::default();
    for _ in 0..FIRMWARE_SETTLE_CYCLE_LIMIT {
        if let Some(execution) = live.step_firmware_cycle_with_execution().unwrap() {
            observation.saw_user_instruction_execute |= execution.pc == USER_INSTRUCTION_EXECUTE_PC;
        }
        observation.saw_single_step |= live.machine.act.state.status[1];
        if live.main_wait_visits > wait_visits && !live.machine.act.state.status[15] {
            for _ in 0..HP67_DISPLAY_SCAN_SLOTS {
                if let Some(execution) = live.step_firmware_cycle_with_execution().unwrap() {
                    observation.saw_user_instruction_execute |=
                        execution.pc == USER_INSTRUCTION_EXECUTE_PC;
                }
                observation.saw_single_step |= live.machine.act.state.status[1];
            }
            return observation;
        }
    }
    panic!(
        "{key:?} dispatch {target:04o} did not return to the no-key firmware wait; pc={:04o}",
        live.machine.pc()
    );
}

fn settle_live_dispatch(live: &mut Hp67LiveMachine, key: Hp67Key, target: u16) {
    let _ = settle_live_dispatch_observing(live, key, target);
}

fn press_live_key_and_settle(live: &mut Hp67LiveMachine, key: Hp67Key, expected_target: u16) {
    let target = press_live_key_to_expected_dispatch(live, key, expected_target);
    settle_live_dispatch(live, key, target);
}

fn press_unshifted_key_and_observe(live: &mut Hp67LiveMachine, key: Hp67Key) -> SettleObservation {
    let target = press_live_key_to_dispatch(live, key);
    assert!(
        (UNSHIFTED_KEY_TABLE_FIRST..=UNSHIFTED_KEY_TABLE_LAST).contains(&target),
        "{key:?} dispatched outside the unshifted HP-67 key table: {target:04o}"
    );
    settle_live_dispatch_observing(live, key, target)
}

fn press_h_shifted_key_and_observe(live: &mut Hp67LiveMachine, key: Hp67Key) -> SettleObservation {
    press_live_key_and_settle(live, Hp67Key::FunctionH, UNSHIFTED_KEY_TABLE_FIRST);
    let target = press_live_key_to_dispatch(live, key);
    assert!(
        (H_SHIFTED_KEY_TABLE_FIRST..=H_SHIFTED_KEY_TABLE_LAST).contains(&target),
        "h + {key:?} dispatched outside the h-shifted HP-67 key table: {target:04o}"
    );
    settle_live_dispatch_observing(live, key, target)
}

fn enter_reference_program(live: &mut Hp67LiveMachine) {
    wait_for_firmware_mode(live, true, "RUN -> PRGM");
    assert!(
        live.machine.act.state.status[11],
        "firmware did not latch PROGRAM mode in S11"
    );

    press_program_key_and_require_step_advance(live, Hp67Key::Digit1, 1);
    press_program_key_and_require_step_advance(live, Hp67Key::Enter, 2);
    press_program_key_and_require_step_advance(live, Hp67Key::Digit2, 3);
    press_program_key_and_require_step_advance(live, Hp67Key::Add, 4);
    press_program_key_and_require_step_advance(live, Hp67Key::RunStop, 5);

    assert_program_prefix(
        live,
        &EXPECTED_PROGRAM_PREFIX,
        "PROGRAM steps 001..005 are not the expected 11 1B 12 37 00 byte sequence",
    );
}

fn return_to_run_step_zero(live: &mut Hp67LiveMachine) {
    wait_for_firmware_mode(live, false, "PRGM -> RUN");
    assert!(
        !live.machine.act.state.status[11],
        "firmware retained PROGRAM-mode latch S11 after returning to RUN"
    );
    press_live_key_and_settle(live, Hp67Key::FunctionH, UNSHIFTED_KEY_TABLE_FIRST);
    press_live_key_and_settle(live, Hp67Key::Gto, 0o0560);
    assert_program_pc(live, 0, "RUN-mode h RTN");
}

fn run_program_to_halt(live: &mut Hp67LiveMachine, expected_frame: &[u8; 15], label: &str) {
    let wait_visits = live.main_wait_visits;
    press_live_key_to_expected_dispatch(live, Hp67Key::RunStop, 0o1443);

    let mut saw_running = false;
    for _ in 0..PROGRAM_RUN_CYCLE_LIMIT {
        live.step_firmware_cycle().unwrap();
        saw_running |= live.machine.act.state.status[2];
        if saw_running
            && !live.machine.act.state.status[2]
            && live.main_wait_visits > wait_visits
            && !live.machine.act.state.status[15]
            && live.display_frame().segments() == expected_frame
        {
            assert_program_pc(live, 6, label);
            return;
        }
    }

    panic!(
        "{label} did not run and halt at the expected physical frame; pc={:04o} \
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

#[test]
fn live_program_mode_stores_and_executes_simple_program() {
    let mut live = boot_live_to_idle();
    enter_reference_program(&mut live);
    return_to_run_step_zero(&mut live);
    run_program_to_halt(
        &mut live,
        &EXPECTED_3_00_FRAME,
        "stored 11 1B 12 37 00 program",
    );
}

#[test]
fn live_program_mode_sst_bst_del_edit_real_program_memory() {
    let mut live = boot_live_to_idle();
    enter_reference_program(&mut live);
    assert_program_pc(&live, 5, "reference program entry");

    let sst = press_unshifted_key_and_observe(&mut live, Hp67Key::Sst);
    assert!(
        sst.saw_single_step,
        "PROGRAM SST never asserted firmware S1"
    );
    assert!(
        !sst.saw_user_instruction_execute,
        "PROGRAM SST incorrectly entered the RUN-mode user-instruction executor"
    );
    assert_program_pc(&live, 6, "PROGRAM SST");

    for expected_step in [5, 4, 3] {
        let bst = press_h_shifted_key_and_observe(&mut live, Hp67Key::Sst);
        assert!(
            !bst.saw_user_instruction_execute,
            "PROGRAM h BST executed a user instruction"
        );
        assert_program_pc(&live, expected_step, "PROGRAM h BST");
    }

    let del = press_h_shifted_key_and_observe(&mut live, Hp67Key::ClearX);
    assert!(
        !del.saw_user_instruction_execute,
        "PROGRAM h DEL executed a user instruction"
    );
    assert_program_pc(&live, 2, "PROGRAM h DEL");
    assert_program_prefix(
        &live,
        &EXPECTED_DELETED_STEP_003_PREFIX,
        "PROGRAM h DEL did not remove step 003 and shift subsequent instructions upward",
    );

    press_program_key_and_require_step_advance(&mut live, Hp67Key::Digit3, 3);
    assert_program_prefix(
        &live,
        &EXPECTED_EDITED_PROGRAM_PREFIX,
        "PROGRAM reinsertion did not produce 11 1B 13 37 00",
    );

    return_to_run_step_zero(&mut live);
    run_program_to_halt(
        &mut live,
        &EXPECTED_4_00_FRAME,
        "edited 11 1B 13 37 00 program",
    );
}

#[test]
fn live_run_mode_sst_executes_one_step_and_bst_only_backs_up() {
    let mut live = boot_live_to_idle();
    enter_reference_program(&mut live);
    return_to_run_step_zero(&mut live);

    for expected_pc in [2, 3, 4, 5] {
        let sst = press_unshifted_key_and_observe(&mut live, Hp67Key::Sst);
        assert!(sst.saw_single_step, "RUN SST never asserted firmware S1");
        assert!(
            sst.saw_user_instruction_execute,
            "RUN SST did not enter the real user-instruction executor"
        );
        assert!(
            !live.machine.act.state.status[1],
            "RUN SST left firmware single-step flag S1 set"
        );
        assert!(
            !live.machine.act.state.status[2],
            "RUN SST incorrectly left continuous-run flag S2 set"
        );
        assert_program_pc(&live, expected_pc, "RUN SST");
    }

    assert_eq!(
        live.display_frame().segments(),
        &EXPECTED_3_00_FRAME,
        "four RUN SST operations did not execute 1 ENTER 2 + to physical 3.00"
    );

    let before_bst_display = *live.display_frame().segments();
    let bst = press_h_shifted_key_and_observe(&mut live, Hp67Key::Sst);
    assert!(
        !bst.saw_single_step,
        "RUN h BST incorrectly asserted single-step flag S1"
    );
    assert!(
        !bst.saw_user_instruction_execute,
        "RUN h BST entered the user-instruction executor"
    );
    assert!(
        !live.machine.act.state.status[1] && !live.machine.act.state.status[2],
        "RUN h BST left S1 or S2 active"
    );
    assert_program_pc(&live, 4, "RUN h BST");
    assert_eq!(
        live.display_frame().segments(),
        &before_bst_display,
        "RUN h BST did not restore the original X display after release"
    );
}
