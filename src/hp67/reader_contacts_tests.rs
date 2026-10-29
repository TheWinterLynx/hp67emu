use super::*;
use hp67emu::{emulation::{Drive, LogicLevel}, machines::hp67::{Hp67Driver, Hp67Net, Hp67MagneticTrack, Hp67CardTrack, CRC_FLAG_F7_STATUS, reader_contacts::sample_reader_contacts}};

#[test]
fn m16i_live_reader_preflight_preserves_pending_ram_tail_and_recovers_without_losing_bits() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut control = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2400 {
        if live.data_serial.frame_in_progress() {
            let before = (
                live.act_serial,
                live.data_serial,
                live.serial_ram.clone(),
                live.reader_contacts,
            );
            let tick = live.backplane.tick();
            let drive = live
                .backplane
                .driver_drive(Hp67Net::Hds, Hp67Driver::External);
            live.backplane.drive(
                Hp67Net::Hds,
                Hp67Driver::CardReaderController1820_1751,
                drive,
            );
            assert!(live
                .step_firmware_cycle_with_execution()
                .unwrap_err()
                .contains("HDS/WPS preflight"));
            assert_eq!(
                (
                    live.act_serial,
                    live.data_serial,
                    live.serial_ram.clone(),
                    live.reader_contacts
                ),
                before
            );
            assert_eq!(live.backplane.tick(), tick);
            live.backplane.drive(
                Hp67Net::Hds,
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
fn m16i_all_foreign_head_corner_sources_block_word_and_card_events_before_mutation() {
    for net in [Hp67Net::Hds, Hp67Net::Wps] {
        for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::External) {
            for drive in [Drive::High, Drive::Low] {
                let mut live = Hp67LiveMachine::power_on_default().unwrap();
                live.backplane.drive(net, driver, drive);
                let before = (live.act_serial, live.data_serial, live.serial_ram.clone(), live.machine.crc.clone(), live.card_transport.clone(), live.reader_contacts, live.crc_reader_gate, live.boot_cycle);
                assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("HDS/WPS preflight"));
                assert!(live.insert_magnetic_card(Hp67MagneticCard::default(), CardInsertionEnd::End1).unwrap_err().contains("HDS/WPS preflight"));
                assert_eq!((live.act_serial, live.data_serial, live.serial_ram.clone(), live.machine.crc.clone(), live.card_transport.clone(), live.reader_contacts, live.crc_reader_gate, live.boot_cycle), before);
                assert_eq!(live.backplane.tick().get(), 0);
                assert_eq!(live.backplane.driver_drive(net, driver), drive);
                live.backplane.drive(net, driver, Drive::HighZ);
                live.step_firmware_cycle_with_execution().unwrap();
                assert_eq!(live.backplane.tick().get(), 224);
            }
        }
    }
}

#[test]
fn m16i_head_contacts_select_correct_card_end_rearm_and_preserve_media_on_faults() {
    let card = Hp67MagneticCard::new(Hp67MagneticTrack::default().with_write_protected(true), Hp67MagneticTrack::default());
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for (end, permitted) in [(CardInsertionEnd::End1, false), (CardInsertionEnd::End2, true)] {
        live.insert_magnetic_card(card.clone(), end).unwrap();
        let before_head = sample_reader_contacts(&live.backplane).unwrap();
        assert!(!before_head.head_active && !before_head.write_permitted);
        let transport_before = live.card_transport.clone();
        live.backplane.drive(Hp67Net::Wps, Hp67Driver::CardReaderController1820_1751, Drive::High);
        assert!(live.set_reader_head_contact(true).is_err());
        assert!(live.take_magnetic_card_for_power_off().is_err());
        assert!(live.withdraw_unstarted_magnetic_card().is_err());
        assert_eq!(live.card_transport, transport_before);
        assert!(!live.crc_reader_gate.unwrap().head_active());
        live.backplane.drive(Hp67Net::Wps, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
        live.set_reader_head_contact(true).unwrap();
        assert!(live.card_transport.head_active());
        let receiver = live.crc_reader_gate.unwrap();
        assert!(receiver.head_active());
        assert_eq!(receiver.write_permitted(), permitted);
        assert_eq!(live.backplane.level(Hp67Net::Hds), LogicLevel::Low);
        assert_eq!(live.backplane.level(Hp67Net::Wps), if permitted { LogicLevel::Low } else { LogicLevel::High });
        assert_eq!(live.take_magnetic_card_for_power_off().unwrap(), Some(card.clone()));
        assert_eq!(sample_reader_contacts(&live.backplane).unwrap(), Default::default());
        assert!(!live.crc_reader_gate.unwrap().write_permitted());
    }
    live.reset_power_on().unwrap();
    assert_eq!(sample_reader_contacts(&live.backplane).unwrap(), Default::default());
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    reference.reset_power_on().unwrap();
    assert!(reference.reader_contacts.is_none() && reference.crc_reader_gate.is_none());
    for net in [Hp67Net::Hds, Hp67Net::Wps] { assert_eq!(reference.backplane.level(net), LogicLevel::Floating); }
}

#[test]
fn m16i_head_observer_substitution_rejects_before_clock_or_crc_mutation() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.card_transport.set_head_active(true);
    let before = (live.act_serial, live.data_serial, live.machine.crc.clone(), live.crc_reader_gate, live.boot_cycle);
    assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("HDS receiver/transport mismatch"));
    assert_eq!((live.act_serial, live.data_serial, live.machine.crc.clone(), live.crc_reader_gate, live.boot_cycle), before);
    assert_eq!(live.backplane.tick().get(), 0);
    live.card_transport.set_head_active(false);
    live.step_firmware_cycle_with_execution().unwrap();
    assert_eq!(live.backplane.tick().get(), 224);
}

fn compare_word(live: &mut Hp67LiveMachine, reference: &mut Hp67LiveMachine) -> Option<Hp67ArchitecturalExecution> {
    let tick = live.backplane.tick();
    let actual = live.step_firmware_cycle_with_execution().unwrap();
    assert_eq!(actual, reference.step_firmware_cycle_with_execution().unwrap());
    assert_eq!(live.machine.act.state, reference.machine.act.state);
    assert_eq!(live.machine.crc, reference.machine.crc);
    assert_eq!(live.card_transport, reference.card_transport);
    assert_eq!(sample_reader_contacts(&live.backplane).unwrap().head_active, live.card_transport.head_active());
    assert_eq!(live.backplane.tick().get(), tick.get() + 224);
    actual
}

#[test]
fn m16i_real_firmware_each_protected_end_reaches_f7_without_writing_through_resolved_contacts() {
    for end in [CardInsertionEnd::End1, CardInsertionEnd::End2] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
        for _ in 0..1200 { compare_word(&mut live, &mut reference); }
        for machine in [&mut live, &mut reference] { machine.set_program_mode(true).unwrap(); }
        let waits = live.main_wait_visits;
        for _ in 0..2048 {
            compare_word(&mut live, &mut reference);
            if live.main_wait_visits > waits && live.machine.act.state.status[11] { break; }
        }
        assert!(live.main_wait_visits > waits && live.machine.act.state.status[11]);
        let selected = end.track();
        let card = Hp67MagneticCard::default().with_track(selected, Hp67MagneticTrack::default().with_write_protected(true));
        for machine in [&mut live, &mut reference] { machine.insert_magnetic_card(card.clone(), end).unwrap(); }
        let mut reported = false;
        for _ in 0..8192 {
            let execution = compare_word(&mut live, &mut reference);
            if let Some(Hp67ArchitecturalExecution { operation: Hp67ArchitecturalOperation::CrcControl { instruction: CrcInstruction::TestFlagAndClear { flag }, condition: Some(true) }, .. }) = execution {
                if usize::from(flag) == CRC_FLAG_F7_STATUS {
                    assert!(live.crc_reader_gate.unwrap().head_active());
                    assert!(!live.crc_reader_gate.unwrap().write_permitted());
                    assert_eq!(live.card_transport.next_record(), 0);
                    assert_eq!(live.card_transport.card(), Some(&card));
                    assert!(!live.card_transport.card().unwrap().track(selected).dirty());
                    reported = true;
                    break;
                }
            }
            assert!(!matches!(execution, Some(Hp67ArchitecturalExecution { operation: Hp67ArchitecturalOperation::CrcDataWrite { .. }, .. })), "protected pass cannot reach a CRC data write");
        }
        assert!(reported, "selected end must reach real-firmware F7 protection report");
        assert_eq!(live.card_transport.card().unwrap().track(if selected == Hp67CardTrack::Track1 { Hp67CardTrack::Track2 } else { Hp67CardTrack::Track1 }), card.track(if selected == Hp67CardTrack::Track1 { Hp67CardTrack::Track2 } else { Hp67CardTrack::Track1 }));
    }
}
