use super::*;
use hp67emu::machines::hp67::{
    pin_trace::{analyze_pin_trace, import_pin_trace},
    wiring::{Hp67Driver, Hp67Net},
};
#[test]
fn m15z_live_full_pin_recording_preserves_firmware_devices_and_all_phi_edges() {
    let mut recorded = Hp67LiveMachine::power_on_default().unwrap();
    let mut plain = Hp67LiveMachine::power_on_default().unwrap();
    recorded
        .backplane
        .start_pin_capture("real firmware fixture; ordinal model observations".into())
        .unwrap();
    for cycle in 0..512 {
        let contact = match cycle {
            200 => Some(Some(Hp67Key::Digit7)),
            400 => Some(None),
            _ => None,
        };
        if let Some(contact) = contact {
            recorded.set_key_contact(contact);
            plain.set_key_contact(contact);
        }
        assert_eq!(
            recorded.step_firmware_cycle_with_execution().unwrap(),
            plain.step_firmware_cycle_with_execution().unwrap()
        );
        assert_eq!(recorded.machine.act.state, plain.machine.act.state);
        assert_eq!(recorded.machine.ram, plain.machine.ram);
        assert_eq!(recorded.machine.crc, plain.machine.crc);
        assert_eq!(recorded.act_serial, plain.act_serial);
        assert_eq!(recorded.data_serial, plain.data_serial);
        assert_eq!(recorded.serial_ram, plain.serial_ram);
        assert_eq!(recorded.owned_crc_data, plain.owned_crc_data);
        assert_eq!(recorded.display_circuit, plain.display_circuit);
        for net in Hp67Net::ALL {
            assert_eq!(recorded.backplane.level(net), plain.backplane.level(net));
        }
    }
    let trace = recorded.backplane.take_pin_capture().unwrap();
    assert!(!trace.overflowed);
    let analysis = analyze_pin_trace(&trace).unwrap();
    assert_eq!(analysis.cells.len(), 512 * 56);
    assert_eq!(analysis.trailing_phase, 0);
    assert_eq!(analysis.str_cells[7], 512);
    assert_eq!(analysis.rcd_cells[7], 512 / 15);
    assert_eq!(recorded.backplane.tick().get(), 512 * 224);
    assert_eq!(analysis.words[&0].address_mask, 0xfff);
    assert_eq!(analysis.words[&0].opcode_mask, 0x3ff);
    assert!(trace
        .rows
        .iter()
        .any(|row| row.owners[Hp67Net::Data.index()]
            .is_some_and(|owner| (owner.high | owner.low)
                & (1 << Hp67Driver::RamAggregate.index())
                != 0)));
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}
#[test]
fn m15z_failed_live_word_keeps_its_electrical_fault_prefix_in_capture() {
    use hp67emu::emulation::{Drive, LogicLevel};
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.backplane
        .start_pin_capture("injected STR fault; not physical evidence".into())
        .unwrap();
    live.backplane
        .drive(Hp67Net::Str, Hp67Driver::External, Drive::Low);
    assert!(live.step_firmware_cycle_with_execution().is_err());
    let trace = live.backplane.take_pin_capture().unwrap();
    assert!(trace
        .rows
        .iter()
        .any(|row| row.levels[Hp67Net::Str.index()] == Some(LogicLevel::Contention)));
    let analysis = analyze_pin_trace(&trace).unwrap();
    assert!(analysis.cells.len() < 56);
    assert_ne!(analysis.trailing_phase, 0);
    live.reset_power_on().unwrap();
    assert!(live.backplane.take_pin_capture().is_none());
}

#[test]
fn m16l_integrity_accepts_real_firmware_selector_and_inserted_card_captures() {
    use hp67emu::machines::hp67::{
        pin_integrity::audit_pin_integrity, Hp67CardTrack, Hp67MagneticTrack,
        HP67_CARD_RECORDS_PER_TRACK,
    };
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..1200 {
        live.step_firmware_cycle_with_execution().unwrap();
    }
    for program in [true, false] {
        live.set_program_mode(program).unwrap();
        live.backplane
            .start_pin_capture("real firmware selector event; ordinal bridge".into())
            .unwrap();
        for _ in 0..256 {
            live.step_firmware_cycle_with_execution().unwrap();
        }
        let trace = live.backplane.take_pin_capture().unwrap();
        let audit = audit_pin_integrity(&trace).unwrap();
        assert_eq!(audit.exit_code(), 0, "{}", audit.report(&trace));
        assert_eq!(audit.cells, 256 * 56);
    }
    let track = Hp67MagneticTrack::from_words([0x0300_0000; HP67_CARD_RECORDS_PER_TRACK]).unwrap();
    let card = Hp67MagneticCard::default().with_track(Hp67CardTrack::Track1, track);
    live.insert_magnetic_card(card, CardInsertionEnd::End1)
        .unwrap();
    live.backplane
        .start_pin_capture("real firmware inserted card; ordinal bridge".into())
        .unwrap();
    for _ in 0..256 {
        live.step_firmware_cycle_with_execution().unwrap();
    }
    let trace = live.backplane.take_pin_capture().unwrap();
    let audit = audit_pin_integrity(&trace).unwrap();
    assert_eq!(audit.exit_code(), 0, "{}", audit.report(&trace));
    assert_eq!(audit.cells, 256 * 56);
}
