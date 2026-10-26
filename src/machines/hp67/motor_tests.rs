use super::*;
use crate::machines::hp67::{isa::{act_address_drive, rom_word_drive}, rom_devices::RomIsDrives, CrcArchitecturalCore};

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
fn m16g_all_opcodes_and_qualifiers_preserve_delayed_motor_latch_and_polarity() {
    for initial in [false, true] {
        for instruction in [false, true] {
            for opcode in 0..=0x3ff {
                let mut crc = Hp67SerialCrcControl::default();
                let mut observer = CrcArchitecturalCore::default();
                let mut bus = Hp67ElectricalBackplane::default();
                let mut motor = Hp67MotorOutput::attach(&mut bus);
                if initial {
                    word(&mut crc, &mut bus, 0o260, true);
                    motor.publish(&crc, &mut bus).unwrap();
                    word(&mut crc, &mut bus, 0, true);
                    motor.publish(&crc, &mut bus).unwrap();
                    observer.execute_opcode(0o260).unwrap();
                }
                assert_eq!(sample_motor(&bus), Ok(initial));
                word(&mut crc, &mut bus, opcode, instruction);
                motor.publish(&crc, &mut bus).unwrap();
                assert_eq!(sample_motor(&bus), Ok(initial), "fetch must not execute itself");
                word(&mut crc, &mut bus, 0, true);
                motor.publish(&crc, &mut bus).unwrap();
                if instruction { observer.execute_opcode(opcode).unwrap(); }
                let expected = observer.flag(CRC_FLAG_MOTOR_ON).unwrap();
                assert_eq!(sample_motor(&bus), Ok(expected), "opcode={opcode:03x} qualified={instruction} initial={initial}");
                assert_eq!(bus.level(Hp67Net::Motor), if expected { LogicLevel::Low } else { LogicLevel::High });
                assert_eq!(bus.active_driver_count(Hp67Net::Motor), 1);
            }
        }
    }
}

#[test]
fn m16g_every_foreign_motor_source_is_rejected_without_overwriting_faults() {
    for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::CardReaderController1820_1751) {
        for drive in [Drive::Low, Drive::High] {
            let mut bus = Hp67ElectricalBackplane::default();
            let mut motor = Hp67MotorOutput::attach(&mut bus);
            bus.drive(Hp67Net::Motor, driver, drive);
            let before = (motor, bus.tick(), bus.level(Hp67Net::Motor));
            assert!(sample_motor(&bus).is_err());
            assert!(motor.check(&bus).is_err());
            assert!(motor.publish(&Hp67SerialCrcControl::default(), &mut bus).is_err());
            assert_eq!((motor, bus.tick(), bus.level(Hp67Net::Motor)), before);
            assert_eq!(bus.driver_drive(Hp67Net::Motor, driver), drive);
            bus.drive(Hp67Net::Motor, driver, Drive::HighZ);
            motor.publish(&Hp67SerialCrcControl::default(), &mut bus).unwrap();
            assert_eq!(sample_motor(&bus), Ok(false));
        }
    }
}

#[test]
fn m16g_missing_replaced_tampered_motor_and_wrong_phase_preserve_output() {
    let mut bus = Hp67ElectricalBackplane::default();
    assert!(sample_motor(&bus).is_err());
    let mut motor = Hp67MotorOutput::attach(&mut bus);
    bus.drive(Hp67Net::Motor, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
    assert!(motor.check(&bus).is_err());
    bus.drive(Hp67Net::Motor, Hp67Driver::External, Drive::High);
    assert!(sample_motor(&bus).is_err());
    bus.drive(Hp67Net::Motor, Hp67Driver::External, Drive::HighZ);
    bus.drive(Hp67Net::Motor, Hp67Driver::CardReaderController1820_1751, Drive::Low);
    assert_eq!(sample_motor(&bus), Ok(true)); // Receiver reads the pin.
    assert_eq!(motor.check(&bus), Err(Hp67MotorError::ChangedSource { expected: false, actual: true }));
    assert!(motor.publish(&Hp67SerialCrcControl::default(), &mut bus).is_err());
    bus.drive(Hp67Net::Motor, Hp67Driver::CardReaderController1820_1751, Drive::High);
    for _ in 0..8 {
        bus.advance_clock();
        let before = motor;
        assert!(matches!(motor.publish(&Hp67SerialCrcControl::default(), &mut bus), Err(Hp67MotorError::Phase { .. })));
        assert_eq!(motor, before);
        assert_eq!(sample_motor(&bus), Ok(false));
    }
}

#[test]
fn m16g_motor_is_retained_through_idle_words_and_stops_only_after_clear() {
    let mut bus = Hp67ElectricalBackplane::default();
    let mut crc = Hp67SerialCrcControl::default();
    let mut motor = Hp67MotorOutput::attach(&mut bus);
    for (opcode, expected) in [(0o260, false), (0, true), (0, true), (0, true), (0o360, true), (0, false), (0, false)] {
        word(&mut crc, &mut bus, opcode, true);
        motor.publish(&crc, &mut bus).unwrap();
        assert_eq!(motor.check(&bus), Ok(expected));
    }
    assert_eq!(bus.tick().get(), 7 * 224);
}

#[test]
fn m16g_legacy_capture_without_motor_preserves_unknown_instead_of_inventing_off() {
    use crate::machines::hp67::pin_trace::{import_pin_trace, PIN_NAMES};
    assert_eq!(Hp67Net::Motor.index(), 14);
    let mut text = String::from("# hp67-pins-v1 domain=ordinal source=emulator first_bit=0\n# provenance=legacy fourteen pins\nat\torder");
    for name in &PIN_NAMES[..14] { text.push('\t'); text.push_str(name); }
    text.push('\n');
    for at in 0..2 {
        text.push_str(&format!("{at}\t0"));
        for index in 0..14 {
            let cell = match index {
                0 if at == 1 => "0:0000:0001",
                0 | 1 => "1:0001:0000",
                2 => "0:0000:0000",
                _ => "Z:0000:0000",
            };
            text.push('\t'); text.push_str(cell);
        }
        text.push('\n');
    }
    let imported = import_pin_trace(&text).unwrap();
    for row in &imported.rows { assert_eq!(row.levels[Hp67Net::Motor.index()], None); assert_eq!(row.owners[Hp67Net::Motor.index()], None); }
    assert_eq!(import_pin_trace(&imported.to_tsv().unwrap()).unwrap(), imported);
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16g_capture_motor_on_off_edges_and_unchanged_publication_roundtrip() {
    use crate::machines::hp67::pin_trace::import_pin_trace;
    let mut bus = Hp67ElectricalBackplane::default();
    let mut motor = Hp67MotorOutput::attach(&mut bus);
    let mut crc = Hp67SerialCrcControl::default();
    bus.start_pin_capture("M16G logical CRC MOTOR edges; not measured timing".into()).unwrap();
    for opcode in [0o260, 0, 0o360, 0] {
        word(&mut crc, &mut bus, opcode, true);
        motor.publish(&crc, &mut bus).unwrap();
        let before = bus.clone().take_pin_capture().unwrap();
        motor.publish(&crc, &mut bus).unwrap();
        assert_eq!(bus.clone().take_pin_capture().unwrap(), before);
    }
    let capture = bus.take_pin_capture().unwrap();
    let changes: Vec<_> = capture.rows.windows(2).filter(|w| w[0].levels[Hp67Net::Motor.index()] != w[1].levels[Hp67Net::Motor.index()]).map(|w| (w[1].at, w[1].levels[Hp67Net::Motor.index()])).collect();
    assert_eq!(changes, [(448, Some(LogicLevel::Low)), (896, Some(LogicLevel::High))]);
    for row in &capture.rows {
        let owner = row.owners[Hp67Net::Motor.index()].unwrap();
        assert_eq!(owner.low | owner.high, 1 << Hp67Driver::CardReaderController1820_1751.index());
    }
    assert_eq!(import_pin_trace(&capture.to_tsv().unwrap()).unwrap(), capture);
    assert_eq!(bus.tick().get(), 896);
}
