use super::*;
use crate::machines::hp67::{
    crc_owned_data::Hp67CrcOwnedData,
    isa::{act_address_drive, rom_word_drive},
    rom_devices::RomIsDrives,
    CardInsertionEnd, Hp67CardTransport, Hp67MagneticCard, Hp67MagneticTrack, Hp67SerialCrcControl,
    CRC_FLAG_F7_STATUS, HP67_NOMINAL_CARD_RECORD_US,
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
fn m16i_crc_samples_wps_only_on_hds_closure_and_rearms_after_opening() {
    for permitted in [false, true] {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut source = Hp67ReaderContacts::attach(&mut bus);
        let mut crc = Hp67CrcReaderGate::default();
        for (head, wps, expected) in [
            (false, permitted, false),
            (true, permitted, permitted),
            (true, !permitted, permitted),
            (false, !permitted, false),
            (true, !permitted, !permitted),
        ] {
            let inputs = Hp67ReaderContactInputs {
                head_active: head,
                write_permitted: wps,
            };
            source.publish(&mut bus, inputs).unwrap();
            assert_eq!(source.check(&bus), Ok(inputs));
            crc.sample_contacts(&bus).unwrap();
            assert_eq!(crc.head_active(), head);
            assert_eq!(crc.write_permitted(), expected);
            assert_eq!(bus.tick().get(), 0);
        }
    }
}

#[test]
fn m16i_held_ground_wps_must_open_and_reclose_between_accepted_passes() {
    let mut bus = Hp67ElectricalBackplane::default();
    let mut source = Hp67ReaderContacts::attach(&mut bus);
    let mut crc = Hp67CrcReaderGate::default();
    for (head, wps, expected) in [
        (true, true, true),
        (false, true, false),
        (true, true, false),
        (false, false, false),
        (false, true, false),
        (true, true, true),
    ] {
        source
            .publish(
                &mut bus,
                Hp67ReaderContactInputs {
                    head_active: head,
                    write_permitted: wps,
                },
            )
            .unwrap();
        crc.sample_contacts(&bus).unwrap();
        assert_eq!(crc.write_permitted(), expected);
    }
}

#[test]
fn m16i_each_foreign_contact_source_rejects_atomically_and_recovers() {
    for initial in [false, true] {
        for net in [Hp67Net::Hds, Hp67Net::Wps] {
            for driver in Hp67Driver::ALL
                .into_iter()
                .filter(|d| *d != Hp67Driver::External)
            {
                for drive in [Drive::Low, Drive::High] {
                    let mut bus = Hp67ElectricalBackplane::default();
                    let mut source = Hp67ReaderContacts::attach(&mut bus);
                    let state = Hp67ReaderContactInputs {
                        head_active: initial,
                        write_permitted: initial,
                    };
                    source.publish(&mut bus, state).unwrap();
                    let mut receiver = Hp67CrcReaderGate::default();
                    receiver.sample_contacts(&bus).unwrap();
                    bus.drive(net, driver, drive);
                    let before = (
                        source,
                        receiver,
                        bus.tick(),
                        bus.level(Hp67Net::Hds),
                        bus.level(Hp67Net::Wps),
                    );
                    assert!(source.publish(&mut bus, Default::default()).is_err());
                    assert!(receiver.sample_contacts(&bus).is_err());
                    assert_eq!(
                        (
                            source,
                            receiver,
                            bus.tick(),
                            bus.level(Hp67Net::Hds),
                            bus.level(Hp67Net::Wps)
                        ),
                        before
                    );
                    assert_eq!(bus.driver_drive(net, driver), drive);
                    bus.drive(net, driver, Drive::HighZ);
                    assert_eq!(source.check(&bus), Ok(state));
                    receiver.sample_contacts(&bus).unwrap();
                    assert_eq!(receiver.head_active(), initial);
                }
            }
        }
    }
}

#[test]
fn m16i_missing_replaced_and_tampered_contact_sources_are_not_overwritten() {
    for net in [Hp67Net::Hds, Hp67Net::Wps] {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut source = Hp67ReaderContacts::attach(&mut bus);
        let mut receiver = Hp67CrcReaderGate::default();
        bus.drive(net, Hp67Driver::External, Drive::HighZ);
        assert!(receiver.sample_contacts(&bus).is_err());
        bus.drive(net, Hp67Driver::CardReaderController1820_1751, Drive::High);
        assert!(source.publish(&mut bus, Default::default()).is_err());
        assert!(receiver.sample_contacts(&bus).is_err());
        assert_eq!(receiver, Hp67CrcReaderGate::default());
        bus.drive(net, Hp67Driver::CardReaderController1820_1751, Drive::HighZ);
        bus.drive(net, Hp67Driver::External, Drive::Low);
        assert!(matches!(
            source.check(&bus),
            Err(Hp67ReaderContactError::ChangedSource { .. })
        ));
        assert!(source.publish(&mut bus, Default::default()).is_err());
        assert_eq!(bus.driver_drive(net, Hp67Driver::External), Drive::Low);
    }
}

#[test]
fn m16i_resolved_wps_controls_write_inhibit_and_ground_bypass_independently_of_card_metadata() {
    for protected in [false, true] {
        for permitted in [false, true] {
            let mut bus = Hp67ElectricalBackplane::default();
            let mut source = Hp67ReaderContacts::attach(&mut bus);
            source
                .publish(
                    &mut bus,
                    Hp67ReaderContactInputs {
                        head_active: true,
                        write_permitted: permitted,
                    },
                )
                .unwrap();
            let mut receiver = Hp67CrcReaderGate::default();
            receiver.sample_contacts(&bus).unwrap();
            let mut transport = Hp67CardTransport::default();
            transport
                .insert_card(
                    Hp67MagneticCard::new(
                        Hp67MagneticTrack::default().with_write_protected(protected),
                        Hp67MagneticTrack::default(),
                    ),
                    CardInsertionEnd::End1,
                )
                .unwrap();
            transport.set_head_active(receiver.head_active());
            let mut control = Hp67SerialCrcControl::default();
            word(&mut control, &mut bus, 0o660, true);
            word(&mut control, &mut bus, 0, true);
            let mut data = Hp67CrcOwnedData::default();
            let before = data.state();
            let (_, after) = data.stage_data_word(Some(0x01234567), true).unwrap();
            data.commit_staged(before, after).unwrap();
            assert_eq!(
                transport
                    .advance_us_owned_with_reader(0, true, &mut data, &mut control, &receiver)
                    .unwrap(),
                0
            );
            word(&mut control, &mut bus, 0o100, true);
            word(&mut control, &mut bus, 0, true);
            assert_eq!(
                transport
                    .advance_us_owned_with_reader(0, true, &mut data, &mut control, &receiver)
                    .unwrap(),
                0
            );
            assert!(transport.record_stream_active());
            let written = transport
                .advance_us_owned_with_reader(
                    HP67_NOMINAL_CARD_RECORD_US,
                    true,
                    &mut data,
                    &mut control,
                    &receiver,
                )
                .unwrap();
            assert_eq!(written, usize::from(permitted));
            assert_eq!(transport.next_record(), usize::from(permitted));
            assert_eq!(transport.active_track().unwrap().dirty(), permitted);
            assert_eq!(
                transport.active_track().unwrap().write_protected(),
                protected
            );
            assert_eq!(control.flags()[CRC_FLAG_F7_STATUS], !permitted);
            assert_eq!(data.queued_write_words(), usize::from(!permitted));
            if permitted {
                assert_eq!(transport.active_track().unwrap().word(0), Some(0x01234567));
            }
        }
    }
}

#[test]
fn m16i_head_mismatch_rejects_before_crc_buffer_sync_or_media_mutation() {
    let mut bus = Hp67ElectricalBackplane::default();
    let mut source = Hp67ReaderContacts::attach(&mut bus);
    source
        .publish(
            &mut bus,
            Hp67ReaderContactInputs {
                head_active: true,
                write_permitted: true,
            },
        )
        .unwrap();
    let mut receiver = Hp67CrcReaderGate::default();
    receiver.sample_contacts(&bus).unwrap();
    let mut transport = Hp67CardTransport::default();
    transport
        .insert_card(Hp67MagneticCard::default(), CardInsertionEnd::End1)
        .unwrap();
    let mut control = Hp67SerialCrcControl::default();
    control.apply_buffer_status(true, true);
    let mut data = Hp67CrcOwnedData::default();
    let before = (transport.clone(), control, data);
    assert!(transport
        .advance_us_owned_with_reader(100_000, true, &mut data, &mut control, &receiver)
        .is_err());
    assert_eq!((transport, control, data), before);
}

#[test]
fn m16i_legacy_sixteen_pin_capture_preserves_unobserved_head_and_corner_contacts() {
    use crate::machines::hp67::pin_trace::{import_pin_trace, PIN_NAMES};
    assert_eq!(Hp67Net::Hds.index(), 16);
    assert_eq!(Hp67Net::Wps.index(), 17);
    let mut text = String::from("# hp67-pins-v1 domain=ordinal source=emulator first_bit=0\n# provenance=legacy sixteen pins\nat\torder");
    for name in &PIN_NAMES[..16] {
        text.push('\t');
        text.push_str(name);
    }
    text.push('\n');
    for at in 0..2 {
        text.push_str(&format!("{at}\t0"));
        for index in 0..16 {
            let cell = match index {
                0 if at == 1 => "0:0000:0001",
                0 | 1 => "1:0001:0000",
                2 => "0:0000:0000",
                _ => "Z:0000:0000",
            };
            text.push('\t');
            text.push_str(cell);
        }
        text.push('\n');
    }
    let trace = import_pin_trace(&text).unwrap();
    for row in &trace.rows {
        for net in [Hp67Net::Hds, Hp67Net::Wps] {
            assert_eq!(row.levels[net.index()], None);
            assert_eq!(row.owners[net.index()], None);
        }
    }
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16i_joint_contact_publication_has_no_intermediate_capture_or_extra_phi_edges() {
    use crate::machines::hp67::pin_trace::import_pin_trace;
    let mut bus = Hp67ElectricalBackplane::default();
    let mut source = Hp67ReaderContacts::attach(&mut bus);
    bus.start_pin_capture("M16I logical pair publication; not physical edge alignment".into())
        .unwrap();
    source
        .publish(
            &mut bus,
            Hp67ReaderContactInputs {
                head_active: true,
                write_permitted: true,
            },
        )
        .unwrap();
    let before = bus.clone().take_pin_capture().unwrap();
    source
        .publish(
            &mut bus,
            Hp67ReaderContactInputs {
                head_active: true,
                write_permitted: true,
            },
        )
        .unwrap();
    assert_eq!(bus.clone().take_pin_capture().unwrap(), before);
    source.publish(&mut bus, Default::default()).unwrap();
    let trace = bus.take_pin_capture().unwrap();
    assert_eq!(trace.rows.len(), 3);
    for row in &trace.rows {
        assert_eq!(
            row.levels[Hp67Net::Hds.index()],
            row.levels[Hp67Net::Wps.index()]
        );
        assert_eq!(row.at, 0);
    }
    assert_eq!(
        trace.rows.iter().map(|row| row.order).collect::<Vec<_>>(),
        [0, 1, 2]
    );
    assert_eq!(bus.tick().get(), 0);
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}
