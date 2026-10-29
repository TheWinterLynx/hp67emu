use super::*;
use crate::machines::hp67::{CrcArchitecturalCore, Hp67SerialCrcControl, Hp67CrcFlagBridge, sample_crc_f2, CRC_FLAG_CARD_PRESENT, CRC_FLAG_COUNT, isa::{act_address_drive, rom_word_drive}, rom_devices::RomIsDrives};

fn word(
    crc: &mut Hp67SerialCrcControl,
    bus: &mut Hp67ElectricalBackplane,
    opcode: u16,
    instruction: bool,
) {
    let tick = bus.tick().get();
    for _ in 0..56 {
        let bit = bus.word_bit();
        bus.commit_serial_launch(
            act_address_drive(0x123, bit),
            RomIsDrives::Aggregate(rom_word_drive(opcode, bit)),
            if bit >= 46 && instruction {
                Drive::High
            } else {
                Drive::Low
            },
            None,
        );
        bus.advance_clock();
        bus.advance_clock();
        crc.sample_at_edge(bus).unwrap();
        bus.advance_clock();
        bus.advance_clock();
    }
    assert_eq!(bus.tick().get() - tick, 224);
}

#[test]
fn m16h_all_opcodes_qualifiers_and_contacts_match_independent_crc_oracle() {
    for present in [false, true] {
        for instruction in [false, true] {
            for opcode in 0..=0x3ff {
                let mut bus = Hp67ElectricalBackplane::default();
                let mut contact = Hp67CardPresentContact::attach(&mut bus);
                contact.set_closed(&mut bus, present).unwrap();
                let mut mirror = CrcArchitecturalCore::default();
                for index in 0..CRC_FLAG_COUNT {
                    mirror.set_external_flag(index as u8, index % 2 == 1).unwrap();
                }
                mirror.set_external_flag(CRC_FLAG_CARD_PRESENT as u8, present).unwrap();
                let mut crc = Hp67SerialCrcControl::default();
                let mut f2 = Hp67CrcFlagBridge::attach(&mut bus);
                crc.freeze_inputs_with_mtrs(&mirror, &bus).unwrap();
                word(&mut crc, &mut bus, opcode, instruction);
                assert_eq!(crc.result(), None);
                f2.begin_word(&mut bus).unwrap();
                let expected = if instruction { mirror.execute_opcode(opcode).unwrap() } else { None };
                word(&mut crc, &mut bus, 0, true);
                assert_eq!(crc.result().and_then(|result| result.condition), expected, "opcode={opcode:03x} qualified={instruction} MTRS={present}");
                crc.check_mirror(&mirror).unwrap();
                f2.publish(&crc, &mut bus).unwrap();
                assert_eq!(sample_crc_f2(&bus).unwrap(), expected == Some(true));
                assert_eq!(contact.check(&bus), Ok(present));
                assert_eq!(bus.tick().get(), 448);
            }
        }
    }
}

#[test]
fn m16h_contact_survives_test_clear_and_frozen_image_changes_only_next_word() {
    let mut bus = Hp67ElectricalBackplane::default();
    let mut contact = Hp67CardPresentContact::attach(&mut bus);
    let mut mirror = CrcArchitecturalCore::default();
    let mut crc = Hp67SerialCrcControl::default();
    // Set internal flag 10 through its actual delayed command.
    word(&mut crc, &mut bus, 0o460, true);
    word(&mut crc, &mut bus, 0o560, true);
    mirror.execute_opcode(0o460).unwrap();
    contact.set_closed(&mut bus, true).unwrap();
    mirror.set_external_flag(CRC_FLAG_CARD_PRESENT as u8, true).unwrap();
    crc.freeze_inputs_with_mtrs(&mirror, &bus).unwrap();
    assert_eq!(mirror.execute_opcode(0o560).unwrap(), Some(true));
    word(&mut crc, &mut bus, 0o560, true);
    crc.check_mirror(&mirror).unwrap();
    assert!(!crc.flags()[CRC_FLAG_CARD_PRESENT]);
    // Open the conductor after freezing: this word must still see closed.
    crc.freeze_inputs_with_mtrs(&mirror, &bus).unwrap();
    contact.set_closed(&mut bus, false).unwrap();
    word(&mut crc, &mut bus, 0o560, true);
    assert_eq!(crc.result().unwrap().condition, Some(true));
    mirror.set_external_flag(CRC_FLAG_CARD_PRESENT as u8, false).unwrap();
    crc.freeze_inputs_with_mtrs(&mirror, &bus).unwrap();
    word(&mut crc, &mut bus, 0, true);
    assert_eq!(crc.result().unwrap().condition, Some(false));
    assert_eq!(contact.check(&bus), Ok(false));
    // No internal latch, buffer, or contact is imported by the freeze operation.
    crc.check_mirror(&mirror).unwrap();
}

#[test]
fn m16h_all_foreign_sources_reject_before_crc_or_contact_mutation_and_recover() {
    for closed in [false, true] {
        for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::External) {
            for drive in [Drive::Low, Drive::High] {
                let mut bus = Hp67ElectricalBackplane::default();
                let mut contact = Hp67CardPresentContact::attach(&mut bus);
                contact.set_closed(&mut bus, closed).unwrap();
                let mut mirror = CrcArchitecturalCore::default();
                mirror.set_external_flag(CRC_FLAG_CARD_PRESENT as u8, closed).unwrap();
                let mut crc = Hp67SerialCrcControl::default();
                crc.freeze_inputs_with_mtrs(&mirror, &bus).unwrap();
                let before = (crc, contact, bus.tick());
                bus.drive(Hp67Net::Mtrs, driver, drive);
                assert!(sample_card_present(&bus).is_err());
                assert!(contact.set_closed(&mut bus, !closed).is_err());
                assert!(crc.freeze_inputs_with_mtrs(&mirror, &bus).is_err());
                assert_eq!((crc, contact, bus.tick()), before);
                assert_eq!(bus.driver_drive(Hp67Net::Mtrs, driver), drive);
                bus.drive(Hp67Net::Mtrs, driver, Drive::HighZ);
                assert_eq!(contact.check(&bus), Ok(closed));
                crc.freeze_inputs_with_mtrs(&mirror, &bus).unwrap();
            }
        }
    }
}

#[test]
fn m16h_missing_replaced_changed_source_mirror_and_late_freeze_preserve_state() {
    let mut bus = Hp67ElectricalBackplane::default();
    let mut contact = Hp67CardPresentContact::attach(&mut bus);
    let mut crc = Hp67SerialCrcControl::default();
    let mirror = CrcArchitecturalCore::default();
    for drive in [Drive::HighZ, Drive::Low] {
        bus.drive(Hp67Net::Mtrs, Hp67Driver::External, drive);
        assert!(contact.set_closed(&mut bus, true).is_err());
        if drive == Drive::HighZ {
            bus.drive(Hp67Net::Mtrs, Hp67Driver::CardReaderController1820_1751, Drive::High);
        }
        let before = crc;
        assert!(crc.freeze_inputs_with_mtrs(&mirror, &bus).is_err());
        assert_eq!(crc, before);
        bus.drive(Hp67Net::Mtrs, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
    }
    bus.drive(Hp67Net::Mtrs, Hp67Driver::External, Drive::High);
    let mut changed = mirror.clone();
    changed.set_external_flag(CRC_FLAG_CARD_PRESENT as u8, true).unwrap();
    let before = crc;
    assert!(crc.freeze_inputs_with_mtrs(&changed, &bus).is_err());
    changed = mirror.clone();
    changed.commit_control_flag(9, true).unwrap();
    assert!(crc.freeze_inputs_with_mtrs(&changed, &bus).is_err());
    assert_eq!(crc, before);
    bus.advance_clock();
    bus.advance_clock();
    crc.sample_at_edge(&bus).unwrap();
    let before = crc;
    assert!(crc.freeze_inputs_with_mtrs(&mirror, &bus).is_err());
    assert_eq!(crc, before);
}

#[test]
fn m16h_fifteen_pin_capture_keeps_absent_mtrs_unknown() {
    use crate::machines::hp67::pin_trace::{import_pin_trace, PIN_NAMES};
    assert_eq!(Hp67Net::Mtrs.index(), 15);
    let mut text = String::from("# hp67-pins-v1 domain=ordinal source=emulator first_bit=0\n# provenance=legacy fifteen pins\nat\torder");
    for name in &PIN_NAMES[..15] { text.push('\t'); text.push_str(name); }
    text.push('\n');
    text.push_str("0\t0");
    for index in 0..15 { text.push('\t'); text.push_str(if index <= 1 { "1:0001:0000" } else if index == 2 { "0:0000:0000" } else { "Z:0000:0000" }); }
    text.push('\n');
    let capture = import_pin_trace(&text).unwrap();
    assert_eq!(capture.rows[0].levels[Hp67Net::Mtrs.index()], None);
    assert_eq!(capture.rows[0].owners[Hp67Net::Mtrs.index()], None);
    assert_eq!(import_pin_trace(&capture.to_tsv().unwrap()).unwrap(), capture);
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16h_capture_contact_events_preserve_time_and_changed_only_source_rows() {
    use crate::machines::hp67::pin_trace::import_pin_trace;
    let mut bus = Hp67ElectricalBackplane::default();
    let mut contact = Hp67CardPresentContact::attach(&mut bus);
    let mut crc = Hp67SerialCrcControl::default();
    bus.start_pin_capture("M16H normalized MTRS, not physical contact timing".into()).unwrap();
    for closed in [true, false] {
        word(&mut crc, &mut bus, 0, true);
        let tick = bus.tick();
        contact.set_closed(&mut bus, closed).unwrap();
        assert_eq!(bus.tick(), tick);
        let captured = bus.clone().take_pin_capture().unwrap();
        contact.set_closed(&mut bus, closed).unwrap();
        assert_eq!(bus.clone().take_pin_capture().unwrap(), captured);
    }
    let capture = bus.take_pin_capture().unwrap();
    let changes: Vec<_> = capture.rows.windows(2).filter(|rows| rows[0].levels[Hp67Net::Mtrs.index()] != rows[1].levels[Hp67Net::Mtrs.index()]).map(|rows| (rows[1].at, rows[1].levels[Hp67Net::Mtrs.index()])).collect();
    assert_eq!(changes, [(224, Some(LogicLevel::Low)), (448, Some(LogicLevel::High))]);
    assert_eq!(import_pin_trace(&capture.to_tsv().unwrap()).unwrap(), capture);
    assert_eq!(bus.tick().get(), 448);
}
