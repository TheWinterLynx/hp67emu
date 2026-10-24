use super::*;
use hp67emu::{
    emulation::{Drive, LogicLevel},
    machines::hp67::{sample_crc_f2, Hp67Driver, Hp67Net},
};

#[test]
fn m16c_live_f2_matches_independent_crc_and_reference_firmware_through_contacts() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut asserted = 0;
    let mut controls = 0;
    for cycle in 0..2400 {
        if cycle == 1200 {
            live.machine.set_card_present(true).unwrap();
            reference.machine.set_card_present(true).unwrap();
        }
        let tick = live.backplane.tick().get();
        assert_eq!(
            live.step_firmware_cycle_with_execution().unwrap(),
            reference.step_firmware_cycle_with_execution().unwrap(),
            "cycle {cycle}"
        );
        let result = live.data_serial.crc_control().unwrap().result();
        let expected = result.is_some_and(|result| result.condition == Some(true));
        controls += usize::from(result.is_some());
        asserted += usize::from(expected);
        assert_eq!(sample_crc_f2(&live.backplane), Ok(expected));
        assert_eq!(live.backplane.tick().get() - tick, 224);
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.machine.ram, reference.machine.ram);
        assert_eq!(
            live.act_serial.owned_register_state(),
            Some(live.machine.act.state.clone())
        );
    }
    assert!(controls > 0);
    assert!(asserted > 0);
    live.reset_power_on().unwrap();
    assert_eq!(sample_crc_f2(&live.backplane), Ok(false));
    assert!(live.crc_flag.is_some());
    reference.reset_power_on().unwrap();
    assert!(reference.crc_flag.is_none());
    assert_eq!(reference.backplane.level(Hp67Net::F2), LogicLevel::Floating);
}

#[test]
fn m16c_live_foreign_f2_faults_stop_before_clock_inputs_or_authority_commit() {
    for driver in Hp67Driver::ALL
        .into_iter()
        .filter(|d| *d != Hp67Driver::CardReaderController1820_1751)
    {
        for drive in [Drive::Low, Drive::High] {
            let mut live = Hp67LiveMachine::power_on_default().unwrap();
            live.set_key_contact(Some(Hp67Key::Digit7));
            live.backplane.drive(Hp67Net::F2, driver, drive);
            let before = (
                live.act_serial,
                live.data_serial,
                live.serial_ram.clone(),
                live.owned_crc_data,
                live.machine.act.state.clone(),
                live.machine.crc.clone(),
                live.crc_flag,
                live.boot_cycle,
            );
            let mut trace = StructuralBusTrace::default();
            assert!(live
                .step_firmware_cycle_with_trace(Some(&mut trace))
                .unwrap_err()
                .contains("CRC F2 preflight"));
            assert_eq!(
                (
                    live.act_serial,
                    live.data_serial,
                    live.serial_ram.clone(),
                    live.owned_crc_data,
                    live.machine.act.state.clone(),
                    live.machine.crc.clone(),
                    live.crc_flag,
                    live.boot_cycle
                ),
                before
            );
            assert_eq!(live.backplane.tick().get(), 0);
            assert!(trace.samples.is_empty());
            live.backplane.drive(Hp67Net::F2, driver, Drive::HighZ);
            live.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.backplane.tick().get(), 224);
        }
    }
}

#[test]
fn m16c_live_f2_preflight_preserves_pending_ram_tail_and_recovers_without_losing_bits() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut control = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2400 {
        if live.data_serial.frame_in_progress() {
            let before = (
                live.act_serial,
                live.data_serial,
                live.serial_ram.clone(),
                live.crc_flag,
            );
            let tick = live.backplane.tick();
            let drive = live
                .backplane
                .driver_drive(Hp67Net::F2, Hp67Driver::CardReaderController1820_1751);
            live.backplane
                .drive(Hp67Net::F2, Hp67Driver::External, drive);
            assert!(live
                .step_firmware_cycle_with_execution()
                .unwrap_err()
                .contains("CRC F2 preflight"));
            assert_eq!(
                (
                    live.act_serial,
                    live.data_serial,
                    live.serial_ram.clone(),
                    live.crc_flag
                ),
                before
            );
            assert_eq!(live.backplane.tick(), tick);
            live.backplane
                .drive(Hp67Net::F2, Hp67Driver::External, Drive::HighZ);
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

#[cfg(feature = "pin-capture")]
#[test]
fn m16c_pin_capture_contains_crc_owned_f2_pulses_and_real_phi_words() {
    use hp67emu::machines::hp67::pin_trace::{analyze_pin_trace, import_pin_trace};
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.machine.set_card_present(true).unwrap();
    let mut seen = false;
    for _ in 0..BOOT_CYCLE_LIMIT {
        live.step_firmware_cycle_with_execution().unwrap();
        if sample_crc_f2(&live.backplane).unwrap() {
            seen = true;
            break;
        }
    }
    assert!(
        seen,
        "real CRC polling must assert F2 before capture starts"
    );
    live.backplane
        .start_pin_capture("M16C real firmware CRC F2; ordinal working bridge".into())
        .unwrap();
    for _ in 0..512 {
        live.step_firmware_cycle_with_execution().unwrap();
    }
    let trace = live.backplane.take_pin_capture().unwrap();
    let f2 = Hp67Net::F2.index();
    let owner = 1u16 << Hp67Driver::CardReaderController1820_1751.index();
    assert!(trace
        .rows
        .iter()
        .any(|row| row.levels[f2] == Some(LogicLevel::Low)));
    for row in &trace.rows {
        let mask = row.owners[f2].unwrap();
        assert_eq!(mask.high | mask.low, owner);
    }
    assert_eq!(
        analyze_pin_trace(&trace).unwrap().clock_edges.len(),
        512 * 224
    );
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}
