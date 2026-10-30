use super::*;
use hp67emu::{emulation::{Drive, LogicLevel}, machines::hp67::{Hp67Driver, Hp67Net, CRC_FLAG_PROGRAM_MODE, program_switch::sample_program_mode}};

#[test]
fn m16j_live_selector_preflight_preserves_pending_ram_tail_and_recovers_without_losing_bits() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut control = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2400 {
        if live.data_serial.frame_in_progress() {
            let before = (
                live.act_serial,
                live.data_serial,
                live.serial_ram.clone(),
                live.program_switch,
            );
            let tick = live.backplane.tick();
            let drive = live
                .backplane
                .driver_drive(Hp67Net::PgmSw, Hp67Driver::External);
            live.backplane.drive(
                Hp67Net::PgmSw,
                Hp67Driver::CardReaderController1820_1751,
                drive,
            );
            assert!(live
                .step_firmware_cycle_with_execution()
                .unwrap_err()
                .contains("PGMSW preflight"));
            assert_eq!(
                (
                    live.act_serial,
                    live.data_serial,
                    live.serial_ram.clone(),
                    live.program_switch
                ),
                before
            );
            assert_eq!(live.backplane.tick(), tick);
            live.backplane.drive(
                Hp67Net::PgmSw,
                Hp67Driver::CardReaderController1820_1751,
                Drive::HighZ,
            );
            for _ in 0..32 {
                assert_eq!(
                    live.step_firmware_cycle_with_execution().unwrap(),
                    control.step_firmware_cycle_with_execution().unwrap()
                );
                assert_eq!(live.machine.act.state, control.machine.act.state);
                assert_eq!(live.serial_ram, control.serial_ram);
                assert_eq!(live.machine.crc, control.machine.crc);
            }
            return;
        }
        assert_eq!(
            live.step_firmware_cycle_with_execution().unwrap(),
            control.step_firmware_cycle_with_execution().unwrap()
        );
    }
    panic!("real firmware must exercise a pending RAM DATA tail");
}

#[test]
fn m16j_foreign_selector_sources_block_words_and_position_updates_before_mutation() {
    for program in [false, true] {
        for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::External) {
            for drive in [Drive::High, Drive::Low] {
                let mut live = Hp67LiveMachine::power_on_default().unwrap();
                live.set_program_mode(program).unwrap();
                live.backplane.drive(Hp67Net::PgmSw, driver, drive);
                let before = (live.act_serial, live.data_serial, live.serial_ram.clone(), live.machine.act.state.clone(), live.machine.crc.clone(), live.program_switch, live.boot_cycle);
                assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("PGMSW preflight"));
                assert!(live.set_program_mode(!program).unwrap_err().contains("PGMSW preflight"));
                assert_eq!((live.act_serial, live.data_serial, live.serial_ram.clone(), live.machine.act.state.clone(), live.machine.crc.clone(), live.program_switch, live.boot_cycle), before);
                assert_eq!(live.backplane.tick().get(), 0);
                assert_eq!(live.backplane.driver_drive(Hp67Net::PgmSw, driver), drive);
                live.backplane.drive(Hp67Net::PgmSw, driver, Drive::HighZ);
                live.step_firmware_cycle_with_execution().unwrap();
                assert_eq!(live.backplane.tick().get(), 224);
            }
        }
    }
}

#[test]
fn m16j_program_observer_cannot_replace_selector_and_reset_recovers_physical_position() {
    for program in [false, true] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.set_program_mode(program).unwrap();
        live.machine.set_program_mode(!program).unwrap();
        let before = (live.act_serial, live.data_serial, live.boot_cycle);
        assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("PGMSW/oracle contact mismatch"));
        assert_eq!((live.act_serial, live.data_serial, live.boot_cycle), before);
        assert_eq!(sample_program_mode(&live.backplane), Ok(program));
        assert_eq!(live.backplane.tick().get(), 0);
        // A reset clears electrical faults, never moves the real selector.
        live.backplane.drive(Hp67Net::PgmSw, Hp67Driver::External, if program { Drive::High } else { Drive::Low });
        live.backplane.drive(Hp67Net::PgmSw, Hp67Driver::CardReaderController1820_1751, Drive::Low);
        live.reset_power_on().unwrap();
        assert_eq!(sample_program_mode(&live.backplane), Ok(program));
        assert_eq!(live.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(program));
        assert_eq!(live.backplane.active_driver_count(Hp67Net::PgmSw), 1);
        assert_eq!(live.backplane.tick().get(), 0);
        live.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(live.backplane.tick().get(), 224);
    }
}

fn compare_word(live: &mut Hp67LiveMachine, reference: &mut Hp67LiveMachine) {
    let tick = live.backplane.tick();
    assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), reference.step_firmware_cycle_with_execution().unwrap());
    assert_eq!(live.machine.act.state, reference.machine.act.state);
    assert_eq!(live.machine.crc, reference.machine.crc);
    assert_eq!(live.machine.ram, reference.machine.ram);
    assert_eq!(live.backplane.tick().get(), tick.get() + 224);
}

fn settle_mode(live: &mut Hp67LiveMachine, reference: &mut Hp67LiveMachine, program: bool) {
    let visits = live.main_wait_visits;
    for _ in 0..8192 {
        compare_word(live, reference);
        if live.main_wait_visits > visits && !live.machine.act.state.status[15] && live.machine.act.state.status[11] == program { return; }
    }
    panic!("real firmware did not consume selector position program={program}");
}

#[test]
fn m16j_real_firmware_mode_transitions_and_power_cycles_preserve_physical_selector() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    for _ in 0..1200 { compare_word(&mut live, &mut reference); }
    for program in [true, false, true] {
        let act_before = live.machine.act.state.clone();
        for machine in [&mut live, &mut reference] { machine.set_program_mode(program).unwrap(); }
        assert_eq!(live.machine.act.state, act_before, "moving the switch cannot perform firmware work");
        assert_eq!(sample_program_mode(&live.backplane), Ok(program));
        settle_mode(&mut live, &mut reference, program);
        for machine in [&mut live, &mut reference] { machine.reset_power_on().unwrap(); }
        assert_eq!(sample_program_mode(&live.backplane), Ok(program));
        assert_eq!(live.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(program));
        assert_eq!(reference.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(program));
        assert_eq!(live.phase, LiveBootPhase::ResetHold);
        assert_eq!(live.boot_cycle, 0);
        assert!(!live.machine.act.state.status[11], "electronic status clears despite retained mechanical selection");
        settle_mode(&mut live, &mut reference, program);
    }
}

#[test]
fn m16j_reference_selector_remains_explicit_and_released_across_reset() {
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    reference.set_program_mode(true).unwrap();
    reference.reset_power_on().unwrap();
    assert!(reference.program_switch.is_none());
    assert_eq!(reference.backplane.level(Hp67Net::PgmSw), LogicLevel::Floating);
    assert_eq!(reference.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(true));
    reference.set_program_mode(false).unwrap();
    reference.reset_power_on().unwrap();
    assert_eq!(reference.machine.crc.external_flag(CRC_FLAG_PROGRAM_MODE), Some(false));
}
