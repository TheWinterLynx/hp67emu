use super::*;
use crate::machines::hp67::{
    card_contact::Hp67CardPresentContact,
    isa::{act_address_drive, rom_word_drive},
    rom_devices::RomIsDrives,
    sample_crc_f2, CrcArchitecturalCore, Hp67CrcFlagBridge, Hp67SerialCrcControl,
    CRC_FLAG_CARD_PRESENT, CRC_FLAG_PROGRAM_MODE,
};

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
fn m16j_all_opcodes_qualifiers_and_both_contacts_match_crc_oracle_and_f2() {
    for program in [false, true] {
        for present in [false, true] {
            for instruction in [false, true] {
                for opcode in 0..=0x3ff {
                    let mut bus = Hp67ElectricalBackplane::default();
                    Hp67ProgramSwitch::attach(&mut bus, program);
                    let mut card = Hp67CardPresentContact::attach(&mut bus);
                    card.set_closed(&mut bus, present).unwrap();
                    let mut mirror = CrcArchitecturalCore::default();
                    mirror
                        .set_external_flag(CRC_FLAG_PROGRAM_MODE as u8, program)
                        .unwrap();
                    mirror
                        .set_external_flag(CRC_FLAG_CARD_PRESENT as u8, present)
                        .unwrap();
                    let mut crc = Hp67SerialCrcControl::default();
                    let mut f2 = Hp67CrcFlagBridge::attach(&mut bus);
                    crc.freeze_inputs_with_contacts(&mirror, &bus).unwrap();
                    word(&mut crc, &mut bus, opcode, instruction);
                    assert_eq!(crc.result(), None);
                    let expected = if instruction {
                        mirror.execute_opcode(opcode).unwrap()
                    } else {
                        None
                    };
                    word(&mut crc, &mut bus, 0, true);
                    crc.check_mirror(&mirror).unwrap();
                    assert_eq!(crc.result().and_then(|result| result.condition), expected, "opcode={opcode:03x} instruction={instruction} program={program} present={present}");
                    f2.publish(&crc, &mut bus).unwrap();
                    assert_eq!(sample_crc_f2(&bus).unwrap(), expected == Some(true));
                    assert_eq!(bus.tick().get(), 448);
                }
            }
        }
    }
}

#[test]
fn m16j_program_test_clear_retains_physical_switch_and_word_frozen_input() {
    let mut bus = Hp67ElectricalBackplane::default();
    Hp67CardPresentContact::attach(&mut bus);
    let mut selector = Hp67ProgramSwitch::attach(&mut bus, true);
    let mut mirror = CrcArchitecturalCore::default();
    mirror
        .set_external_flag(CRC_FLAG_PROGRAM_MODE as u8, true)
        .unwrap();
    let mut crc = Hp67SerialCrcControl::default();
    crc.freeze_inputs_with_contacts(&mirror, &bus).unwrap();
    word(&mut crc, &mut bus, 0o300, true);
    word(&mut crc, &mut bus, 0o300, true);
    assert_eq!(crc.result().unwrap().condition, Some(true));
    assert_eq!(selector.check(&bus), Ok(true));
    crc.freeze_inputs_with_contacts(&mirror, &bus).unwrap();
    selector.set_program(&mut bus, false).unwrap();
    word(&mut crc, &mut bus, 0o300, true);
    assert_eq!(crc.result().unwrap().condition, Some(true));
    mirror
        .set_external_flag(CRC_FLAG_PROGRAM_MODE as u8, false)
        .unwrap();
    crc.freeze_inputs_with_contacts(&mirror, &bus).unwrap();
    word(&mut crc, &mut bus, 0, true);
    assert_eq!(crc.result().unwrap().condition, Some(false));
    assert!(!selector.program_selected());
}

#[test]
fn m16j_all_foreign_program_sources_preserve_selector_and_crc_then_recover() {
    for program in [false, true] {
        for driver in Hp67Driver::ALL
            .into_iter()
            .filter(|d| *d != Hp67Driver::External)
        {
            for drive in [Drive::High, Drive::Low] {
                let mut bus = Hp67ElectricalBackplane::default();
                Hp67CardPresentContact::attach(&mut bus);
                let mut selector = Hp67ProgramSwitch::attach(&mut bus, program);
                let mut mirror = CrcArchitecturalCore::default();
                mirror
                    .set_external_flag(CRC_FLAG_PROGRAM_MODE as u8, program)
                    .unwrap();
                let mut crc = Hp67SerialCrcControl::default();
                crc.freeze_inputs_with_contacts(&mirror, &bus).unwrap();
                let before = (selector, crc, bus.tick());
                bus.drive(Hp67Net::PgmSw, driver, drive);
                assert!(selector.set_program(&mut bus, !program).is_err());
                assert!(crc.freeze_inputs_with_contacts(&mirror, &bus).is_err());
                assert_eq!((selector, crc, bus.tick()), before);
                assert_eq!(bus.driver_drive(Hp67Net::PgmSw, driver), drive);
                bus.drive(Hp67Net::PgmSw, driver, Drive::HighZ);
                crc.freeze_inputs_with_contacts(&mirror, &bus).unwrap();
                assert_eq!(selector.check(&bus), Ok(program));
            }
        }
    }
}

#[test]
fn m16j_contact_loss_replacement_mirror_and_late_freeze_reject_atomically() {
    let mut bus = Hp67ElectricalBackplane::default();
    Hp67CardPresentContact::attach(&mut bus);
    let mut selector = Hp67ProgramSwitch::attach(&mut bus, false);
    let mirror = CrcArchitecturalCore::default();
    let mut crc = Hp67SerialCrcControl::default();
    let before = crc;
    for net in [Hp67Net::Mtrs, Hp67Net::PgmSw] {
        bus.drive(net, Hp67Driver::External, Drive::HighZ);
        assert!(crc.freeze_inputs_with_contacts(&mirror, &bus).is_err());
        bus.drive(net, Hp67Driver::CardReaderController1820_1751, Drive::High);
        assert!(crc.freeze_inputs_with_contacts(&mirror, &bus).is_err());
        assert_eq!(crc, before);
        bus.drive(net, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
        bus.drive(net, Hp67Driver::External, Drive::High);
    }
    bus.drive(Hp67Net::PgmSw, Hp67Driver::External, Drive::Low);
    assert!(selector.set_program(&mut bus, false).is_err());
    assert_eq!(
        bus.driver_drive(Hp67Net::PgmSw, Hp67Driver::External),
        Drive::Low
    );
    assert!(crc.freeze_inputs_with_contacts(&mirror, &bus).is_err());
    bus.drive(Hp67Net::PgmSw, Hp67Driver::External, Drive::High);
    for flag in [CRC_FLAG_PROGRAM_MODE, CRC_FLAG_CARD_PRESENT] {
        let mut changed = mirror.clone();
        changed.set_external_flag(flag as u8, true).unwrap();
        assert!(crc.freeze_inputs_with_contacts(&changed, &bus).is_err());
        assert_eq!(crc, before);
    }
    bus.advance_clock();
    bus.advance_clock();
    crc.sample_at_edge(&bus).unwrap();
    let before = crc;
    assert!(crc.freeze_inputs_with_contacts(&mirror, &bus).is_err());
    assert_eq!(crc, before);
}

#[test]
fn m16j_eighteen_pin_capture_keeps_absent_program_switch_unknown() {
    use crate::machines::hp67::pin_trace::{import_pin_trace, PIN_NAMES};
    assert_eq!(Hp67Net::PgmSw.index(), 18);
    let mut text = String::from("# hp67-pins-v1 domain=ordinal source=emulator first_bit=0\n# provenance=legacy eighteen pins\nat\torder");
    for name in &PIN_NAMES[..18] {
        text.push('\t');
        text.push_str(name);
    }
    text.push('\n');
    for at in 0..2 {
        text.push_str(&format!("{at}\t0"));
        for index in 0..18 {
            text.push('\t');
            text.push_str(match index {
                0 if at == 1 => "0:0000:0001",
                0 | 1 => "1:0001:0000",
                2 => "0:0000:0000",
                _ => "Z:0000:0000",
            });
        }
        text.push('\n');
    }
    let trace = import_pin_trace(&text).unwrap();
    for row in &trace.rows {
        assert_eq!(row.levels[Hp67Net::PgmSw.index()], None);
        assert_eq!(row.owners[Hp67Net::PgmSw.index()], None);
    }
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16j_selector_capture_changes_only_on_position_change_without_advancing_time() {
    use crate::machines::hp67::pin_trace::import_pin_trace;
    let mut bus = Hp67ElectricalBackplane::default();
    let mut selector = Hp67ProgramSwitch::attach(&mut bus, false);
    bus.start_pin_capture("M16J physical selector logical publication, not PHI timing".into())
        .unwrap();
    selector.set_program(&mut bus, true).unwrap();
    let before = bus.clone().take_pin_capture().unwrap();
    selector.set_program(&mut bus, true).unwrap();
    assert_eq!(bus.clone().take_pin_capture().unwrap(), before);
    selector.set_program(&mut bus, false).unwrap();
    let trace = bus.take_pin_capture().unwrap();
    assert_eq!(trace.rows.len(), 3);
    assert_eq!(
        trace
            .rows
            .iter()
            .map(|row| row.levels[Hp67Net::PgmSw.index()])
            .collect::<Vec<_>>(),
        [
            Some(LogicLevel::High),
            Some(LogicLevel::Low),
            Some(LogicLevel::High)
        ]
    );
    assert_eq!(bus.tick().get(), 0);
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}
