use super::*;
use hp67emu::{emulation::{Drive, LogicLevel}, machines::hp67::{Hp67Driver, Hp67Net, CRC_FLAG_CARD_PRESENT, card_contact::sample_card_present}};

#[test]
fn m16h_live_mtrs_preflight_preserves_pending_ram_tail_and_recovers_without_losing_bits() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut control = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2400 {
        if live.data_serial.frame_in_progress() {
            let before = (
                live.act_serial,
                live.data_serial,
                live.serial_ram.clone(),
                live.card_present_contact,
            );
            let tick = live.backplane.tick();
            let drive = live
                .backplane
                .driver_drive(Hp67Net::Mtrs, Hp67Driver::External);
            live.backplane
                .drive(Hp67Net::Mtrs, Hp67Driver::CardReaderController1820_1751, drive);
            assert!(live
                .step_firmware_cycle_with_execution()
                .unwrap_err()
                .contains("CRC MTRS preflight"));
            assert_eq!(
                (
                    live.act_serial,
                    live.data_serial,
                    live.serial_ram.clone(),
                    live.card_present_contact
                ),
                before
            );
            assert_eq!(live.backplane.tick(), tick);
            live.backplane
                .drive(Hp67Net::Mtrs, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
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
fn m16h_foreign_mtrs_sources_block_words_and_insertion_without_losing_media() {
    for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::External) {
        for drive in [Drive::Low, Drive::High] {
            let mut live = Hp67LiveMachine::power_on_default().unwrap();
            live.set_key_contact(Some(Hp67Key::Digit7));
            live.backplane.drive(Hp67Net::Mtrs, driver, drive);
            let before = (live.act_serial, live.data_serial, live.serial_ram.clone(), live.machine.act.state.clone(), live.machine.crc.clone(), live.card_transport.clone(), live.boot_cycle, live.card_present_contact);
            assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("CRC MTRS preflight"));
            assert!(live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1).unwrap_err().contains("CRC MTRS preflight"));
            assert_eq!((live.act_serial, live.data_serial, live.serial_ram.clone(), live.machine.act.state.clone(), live.machine.crc.clone(), live.card_transport.clone(), live.boot_cycle, live.card_present_contact), before);
            assert_eq!(live.backplane.tick().get(), 0);
            assert_eq!(live.backplane.driver_drive(Hp67Net::Mtrs, driver), drive);
            live.backplane.drive(Hp67Net::Mtrs, driver, Drive::HighZ);
            live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1).unwrap();
            assert_eq!(sample_card_present(&live.backplane), Ok(true));
            live.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.backplane.tick().get(), 224);
        }
    }
}

#[test]
fn m16h_mtrs_observer_cannot_override_physical_contact_and_retry_preserves_state() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.machine.set_card_present(true).unwrap();
    let before = (live.act_serial, live.data_serial, live.serial_ram.clone(), live.boot_cycle);
    assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("MTRS/oracle contact mismatch"));
    assert_eq!((live.act_serial, live.data_serial, live.serial_ram.clone(), live.boot_cycle), before);
    assert_eq!(sample_card_present(&live.backplane), Ok(false));
    assert_eq!(live.backplane.tick().get(), 0);
    live.machine.set_card_present(false).unwrap();
    live.step_firmware_cycle_with_execution().unwrap();
    assert_eq!(live.backplane.tick().get(), 224);
}

#[test]
fn m16h_insertion_withdrawal_power_off_reset_and_reference_preserve_contact_ownership() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    for _ in 0..1200 {
        assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), reference.step_firmware_cycle_with_execution().unwrap());
    }
    for end in [CardInsertionEnd::End1, CardInsertionEnd::End2] {
        for machine in [&mut live, &mut reference] {
            machine.insert_magnetic_card(Hp67MagneticCard::default(), end).unwrap();
            assert!(!machine.card_transport.head_active(), "MTRS insertion cannot close HDS");
        }
        assert_eq!(sample_card_present(&live.backplane), Ok(true));
        assert_eq!(reference.backplane.level(Hp67Net::Mtrs), LogicLevel::Floating);
        assert_eq!(live.withdraw_unstarted_magnetic_card().unwrap(), reference.withdraw_unstarted_magnetic_card().unwrap());
        assert_eq!(sample_card_present(&live.backplane), Ok(false));
        assert_eq!(live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT), Some(false));
    }
    live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1).unwrap();
    let card_before = live.card_transport.clone();
    live.backplane.drive(Hp67Net::Mtrs, Hp67Driver::CardReaderController1820_1751, Drive::Low);
    assert!(live.withdraw_unstarted_magnetic_card().is_err());
    assert!(live.take_magnetic_card_for_power_off().is_err());
    assert_eq!(live.card_transport, card_before);
    assert_eq!(live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT), Some(true));
    live.backplane.drive(Hp67Net::Mtrs, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
    assert!(live.take_magnetic_card_for_power_off().unwrap().is_some());
    assert_eq!(sample_card_present(&live.backplane), Ok(false));
    live.reset_power_on().unwrap();
    reference.reset_power_on().unwrap();
    assert_eq!(sample_card_present(&live.backplane), Ok(false));
    assert_eq!(live.backplane.active_driver_count(Hp67Net::Mtrs), 1);
    assert_eq!(reference.backplane.level(Hp67Net::Mtrs), LogicLevel::Floating);
    assert!(reference.card_present_contact.is_none());
}

#[test]
fn m16h_card_present_firmware_reaches_motor_through_resolved_mtrs_with_reference_parity() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    for _ in 0..1200 {
        assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), reference.step_firmware_cycle_with_execution().unwrap());
    }
    for machine in [&mut live, &mut reference] { machine.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1).unwrap(); }
    for _ in 0..2400 {
        let tick = live.backplane.tick();
        assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), reference.step_firmware_cycle_with_execution().unwrap());
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.card_transport, reference.card_transport);
        assert_eq!(sample_card_present(&live.backplane), Ok(true));
        assert_eq!(live.backplane.tick().get(), tick.get() + 224);
        if live.card_motor_on() { return; }
    }
    panic!("real card-present firmware must command MOTOR after resolved MTRS reception");
}
