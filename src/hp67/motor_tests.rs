use super::*;
use hp67emu::{emulation::{Drive, LogicLevel}, machines::hp67::{Hp67Driver, Hp67Net, motor::sample_motor}};

#[test]
fn m16g_live_motor_preflight_preserves_pending_ram_tail_and_recovers_without_losing_bits() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut control = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2400 {
        if live.data_serial.frame_in_progress() {
            let before = (
                live.act_serial,
                live.data_serial,
                live.serial_ram.clone(),
                live.motor_output,
            );
            let tick = live.backplane.tick();
            let drive = live
                .backplane
                .driver_drive(Hp67Net::Motor, Hp67Driver::CardReaderController1820_1751);
            live.backplane
                .drive(Hp67Net::Motor, Hp67Driver::External, drive);
            assert!(live
                .step_firmware_cycle_with_execution()
                .unwrap_err()
                .contains("CRC MOTOR preflight"));
            assert_eq!(
                (
                    live.act_serial,
                    live.data_serial,
                    live.serial_ram.clone(),
                    live.motor_output
                ),
                before
            );
            assert_eq!(live.backplane.tick(), tick);
            live.backplane
                .drive(Hp67Net::Motor, Hp67Driver::External, Drive::HighZ);
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
fn m16g_live_motor_faults_precede_inputs_oracle_and_clock_mutation() {
    for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::CardReaderController1820_1751) {
        for drive in [Drive::Low, Drive::High] {
            let mut live = Hp67LiveMachine::power_on_default().unwrap();
            live.set_key_contact(Some(Hp67Key::Digit7));
            live.backplane.drive(Hp67Net::Motor, driver, drive);
            let before = (live.act_serial, live.fetch_rom, live.display_rom0, live.data_serial, live.serial_ram.clone(), live.owned_crc_data, live.machine.act.state.clone(), live.machine.crc.clone(), live.boot_cycle);
            let error = live.step_firmware_cycle_with_execution().unwrap_err();
            assert!(error.contains("CRC MOTOR preflight"), "{error}");
            assert_eq!((live.act_serial, live.fetch_rom, live.display_rom0, live.data_serial, live.serial_ram.clone(), live.owned_crc_data, live.machine.act.state.clone(), live.machine.crc.clone(), live.boot_cycle), before);
            assert_eq!(live.backplane.tick().get(), 0);
            assert_eq!(live.backplane.driver_drive(Hp67Net::Motor, driver), drive);
            live.backplane.drive(Hp67Net::Motor, driver, Drive::HighZ);
            live.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.backplane.tick().get(), 224);
        }
    }
}

#[test]
fn m16g_real_firmware_motor_contact_start_reset_and_reference_composition() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    assert_eq!(sample_motor(&live.backplane), Ok(false));
    assert_eq!(reference.backplane.level(Hp67Net::Motor), LogicLevel::Floating);
    assert!(reference.motor_output.is_none());
    for _ in 0..1200 {
        assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), reference.step_firmware_cycle_with_execution().unwrap());
    }
    assert_eq!(sample_motor(&live.backplane), Ok(false));
    for machine in [&mut live, &mut reference] {
        machine.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1).unwrap();
    }
    let mut started = false;
    for _ in 0..2400 {
        let tick = live.backplane.tick();
        assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), reference.step_firmware_cycle_with_execution().unwrap());
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.card_transport, reference.card_transport);
        assert_eq!(live.backplane.tick().get(), tick.get() + 224);
        let pin = sample_motor(&live.backplane).unwrap();
        assert_eq!(pin, live.machine.crc.flag(CRC_FLAG_MOTOR_ON).unwrap());
        if pin { started = true; break; }
    }
    assert!(started, "real card-present firmware must start the motor");
    assert_eq!(live.backplane.level(Hp67Net::Motor), LogicLevel::Low);
    live.reset_power_on().unwrap();
    reference.reset_power_on().unwrap();
    assert_eq!(sample_motor(&live.backplane), Ok(false));
    assert_eq!(reference.backplane.level(Hp67Net::Motor), LogicLevel::Floating);
}
