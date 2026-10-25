use super::*;
use crate::machines::hp67::{
    decode_crc_opcode,
    isa::{act_address_drive, rom_word_drive},
    rom_devices::RomIsDrives,
    CrcArchitecturalCore, CrcInstruction, CRC_FLAG_COUNT,
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
fn m16c_each_crc_command_qualifier_and_contact_reaches_resolved_f2_one_word_later() {
    for opcode in 0..=0x3ff {
        let Some(instruction) = decode_crc_opcode(opcode).unwrap() else {
            continue;
        };
        for contact in [false, true] {
            for qualified in [false, true] {
                let mut crc = Hp67SerialCrcControl::default();
                let mut mirror = CrcArchitecturalCore::default();
                for flag in 0..CRC_FLAG_COUNT {
                    mirror.set_external_flag(flag as u8, contact).unwrap();
                }
                crc.freeze_inputs(&mirror).unwrap();
                let mut bus = Hp67ElectricalBackplane::default();
                let mut flag = Hp67CrcFlagBridge::attach(&mut bus);
                word(&mut crc, &mut bus, opcode, qualified);
                flag.publish(&crc, &mut bus).unwrap();
                assert!(
                    !sample_crc_f2(&bus).unwrap(),
                    "fetch cannot execute its own command"
                );
                flag.begin_word(&mut bus).unwrap();
                word(&mut crc, &mut bus, 0, true);
                flag.publish(&crc, &mut bus).unwrap();
                let expected = qualified
                    && contact
                    && matches!(instruction, CrcInstruction::TestFlagAndClear { .. });
                assert_eq!(
                    sample_crc_f2(&bus).unwrap(),
                    expected,
                    "opcode={opcode:03x} qualified={qualified} contact={contact}"
                );
                let tick = bus.tick();
                flag.begin_word(&mut bus).unwrap();
                assert!(!sample_crc_f2(&bus).unwrap());
                assert_eq!(bus.tick(), tick);
                assert_eq!(bus.active_driver_count(Hp67Net::F2), 1);
            }
        }
    }
}

#[test]
fn m16c_f2_faults_include_agreeing_foreign_sources_and_do_not_get_overwritten() {
    for driver in Hp67Driver::ALL
        .into_iter()
        .filter(|d| *d != Hp67Driver::CardReaderController1820_1751)
    {
        for drive in [Drive::Low, Drive::High] {
            let mut bus = Hp67ElectricalBackplane::default();
            let mut flag = Hp67CrcFlagBridge::attach(&mut bus);
            bus.drive(Hp67Net::F2, driver, drive);
            let before = (flag, bus.level(Hp67Net::F2), bus.tick());
            assert!(sample_crc_f2(&bus).is_err());
            assert!(flag.begin_word(&mut bus).is_err());
            assert!(flag
                .publish(&Hp67SerialCrcControl::default(), &mut bus)
                .is_err());
            assert_eq!((flag, bus.level(Hp67Net::F2), bus.tick()), before);
            assert_eq!(bus.driver_drive(Hp67Net::F2, driver), drive);
            bus.drive(Hp67Net::F2, driver, Drive::HighZ);
            flag.begin_word(&mut bus).unwrap();
        }
    }
}

#[test]
fn m16c_f2_missing_replaced_changed_source_and_wrong_phase_preserve_source_state() {
    let mut bus = Hp67ElectricalBackplane::default();
    assert!(sample_crc_f2(&bus).is_err());
    let mut flag = Hp67CrcFlagBridge::attach(&mut bus);
    bus.drive(
        Hp67Net::F2,
        Hp67Driver::CardReaderController1820_1751,
        Drive::HighZ,
    );
    bus.drive(Hp67Net::F2, Hp67Driver::External, Drive::Low);
    assert!(sample_crc_f2(&bus).is_err());
    bus.drive(Hp67Net::F2, Hp67Driver::External, Drive::HighZ);
    bus.drive(
        Hp67Net::F2,
        Hp67Driver::CardReaderController1820_1751,
        Drive::Low,
    );
    assert_eq!(sample_crc_f2(&bus), Ok(true));
    assert_eq!(
        flag.check(&bus),
        Err(Hp67CrcFlagError::ChangedSource {
            expected: false,
            actual: true
        })
    );
    bus.drive(
        Hp67Net::F2,
        Hp67Driver::CardReaderController1820_1751,
        Drive::High,
    );
    for tick in 1..=4 {
        bus.advance_clock();
        let before = flag;
        assert!(flag
            .publish(&Hp67SerialCrcControl::default(), &mut bus)
            .is_err());
        assert_eq!(flag, before);
        assert_eq!(bus.tick().get(), tick);
        assert_eq!(bus.level(Hp67Net::F2), LogicLevel::High);
    }
}

#[test]
fn m16c_internal_crc_test_pulses_once_then_clears_internal_flag_without_contact_reset() {
    // READY (flag 0) is supplied by the buffer/transport handshake. The
    // selector-zero SET encoding is absent; do not invent one for a fixture.
    assert!((0..=0x3ff).all(|opcode| {
        decode_crc_opcode(opcode).unwrap() != Some(CrcInstruction::SetFlag { flag: 0 })
    }));
    for index in 1..CRC_FLAG_COUNT as u8 {
        let set = (0..=0x3ff)
            .find(|op| {
                decode_crc_opcode(*op).unwrap() == Some(CrcInstruction::SetFlag { flag: index })
            })
            .expect("each internal control flag 1..11 has a SET opcode");
        let test = (0..=0x3ff)
            .find(|op| {
                decode_crc_opcode(*op).unwrap()
                    == Some(CrcInstruction::TestFlagAndClear { flag: index })
            })
            .expect("each internal control flag 1..11 has a TEST/CLEAR opcode");
        let mut crc = Hp67SerialCrcControl::default();
        let mut bus = Hp67ElectricalBackplane::default();
        let mut flag = Hp67CrcFlagBridge::attach(&mut bus);
        for (opcode, expected) in [(set, false), (test, false), (test, true), (0, false)] {
            flag.begin_word(&mut bus).unwrap();
            word(&mut crc, &mut bus, opcode, true);
            flag.publish(&crc, &mut bus).unwrap();
            assert_eq!(
                sample_crc_f2(&bus).unwrap(),
                expected,
                "flag={index} opcode={opcode:03x}"
            );
        }
        assert!(!crc.flags()[usize::from(index)]);
    }
}
