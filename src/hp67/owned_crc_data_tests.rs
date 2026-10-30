use super::*;
use hp67emu::machines::hp67::{Hp67CardTrack, Hp67MagneticTrack, HP67_CARD_RECORDS_PER_TRACK};

fn compare_word(
    live: &mut Hp67LiveMachine,
    reference: &mut Hp67LiveMachine,
) -> Option<Hp67ArchitecturalExecution> {
    let tick = live.backplane.tick().get();
    let actual = live.step_firmware_cycle_with_execution().unwrap();
    assert_eq!(
        actual,
        reference.step_firmware_cycle_with_execution().unwrap()
    );
    assert_eq!(live.machine.act.state, reference.machine.act.state);
    assert_eq!(live.machine.crc, reference.machine.crc);
    assert_eq!(live.card_transport, reference.card_transport);
    live.owned_crc_data
        .as_ref()
        .unwrap()
        .check_mirror(&live.machine.crc)
        .unwrap();
    assert_eq!(
        hp67emu::machines::hp67::program_switch::sample_program_mode(&live.backplane).unwrap(),
        live.machine
            .crc
            .external_flag(hp67emu::machines::hp67::CRC_FLAG_PROGRAM_MODE)
            .unwrap()
    );
    let reader =
        hp67emu::machines::hp67::reader_contacts::sample_reader_contacts(&live.backplane).unwrap();
    assert_eq!(reader.head_active, live.card_transport.head_active());
    assert_eq!(
        live.crc_reader_gate.as_ref().unwrap().head_active(),
        reader.head_active
    );
    if reader.head_active {
        assert_eq!(
            live.crc_reader_gate.as_ref().unwrap().write_permitted(),
            !live
                .card_transport
                .active_track()
                .unwrap()
                .write_protected()
        );
    }
    assert_eq!(
        hp67emu::machines::hp67::card_contact::sample_card_present(&live.backplane).unwrap(),
        live.machine
            .crc
            .external_flag(hp67emu::machines::hp67::CRC_FLAG_CARD_PRESENT)
            .unwrap()
    );
    assert_eq!(
        hp67emu::machines::hp67::motor::sample_motor(&live.backplane).unwrap(),
        live.machine.crc.flag(CRC_FLAG_MOTOR_ON).unwrap()
    );
    assert_eq!(live.backplane.tick().get() - tick, 224);
    actual
}
fn boot_pair() -> (Hp67LiveMachine, Hp67LiveMachine) {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    for _ in 0..1200 {
        compare_word(&mut live, &mut reference);
    }
    (live, reference)
}
#[test]
fn m15x_live_crc_storage_matches_reference_through_keys_trace_and_reset() {
    let (mut live, mut reference) = boot_pair();
    let mut traced = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..1200 {
        traced.step_firmware_cycle_with_execution().unwrap();
    }
    let mut trace = StructuralBusTrace::default();
    for cycle in 0..1000 {
        let contact = match cycle {
            0 => Some(Some(Hp67Key::Rcl)),
            200 | 700 => Some(None),
            500 => Some(Some(Hp67Key::Digit0)),
            _ => None,
        };
        if let Some(contact) = contact {
            for machine in [&mut live, &mut reference, &mut traced] {
                machine.set_key_contact(contact);
            }
        }
        let actual = compare_word(&mut live, &mut reference);
        assert_eq!(
            actual,
            traced
                .step_firmware_cycle_with_trace(Some(&mut trace))
                .unwrap()
        );
        assert_eq!(live.owned_crc_data, traced.owned_crc_data);
        assert_eq!(live.machine.crc, traced.machine.crc);
    }
    assert_eq!(trace.samples.len(), 1000 * 224);
    live.reset_power_on().unwrap();
    assert_eq!(live.owned_crc_data, Some(Default::default()));
    live.step_firmware_cycle_with_execution().unwrap();
    reference.reset_power_on().unwrap();
    assert!(reference.owned_crc_data.is_none());
}
#[test]
fn m15x_poisoned_fifo_mirror_is_rejected_before_clocks_or_data_tail() {
    for tail in [false, true] {
        for write in [false, true] {
            let mut live = Hp67LiveMachine::power_on_default().unwrap();
            if tail {
                for _ in 0..2000 {
                    if live.data_serial.frame_in_progress() {
                        break;
                    }
                    live.step_firmware_cycle_with_execution().unwrap();
                }
                assert!(live.data_serial.frame_in_progress());
            }
            let ready = live.machine.crc.flag(CRC_FLAG_BUFFER_READY).unwrap();
            if write {
                live.machine
                    .crc
                    .commit_control_flag(CRC_FLAG_WRITE_MODE as u8, true)
                    .unwrap();
                live.machine.crc.queue_write_word(0x1234567).unwrap();
                live.machine
                    .crc
                    .commit_control_flag(CRC_FLAG_WRITE_MODE as u8, false)
                    .unwrap();
            } else {
                live.machine.crc.present_read_word(0x1234567).unwrap();
            }
            live.machine
                .crc
                .commit_control_flag(CRC_FLAG_BUFFER_READY as u8, ready)
                .unwrap();
            let tick = live.backplane.tick();
            let act = live.act_serial;
            let data = live.data_serial;
            let pipeline = live.pipeline;
            let crc = live.owned_crc_data;
            let error = live.step_firmware_cycle_with_execution().unwrap_err();
            assert!(error.contains("CRC owned DATA/mirror mismatch"), "{error}");
            assert_eq!(live.backplane.tick(), tick);
            assert_eq!(live.act_serial, act);
            assert_eq!(live.data_serial, data);
            assert_eq!(live.pipeline, pipeline);
            assert_eq!(live.owned_crc_data, crc);
        }
    }
}
#[test]
fn m15x_real_firmware_owned_crc_writes_and_reads_all_34_records() {
    let (mut live, mut reference) = boot_pair();
    for machine in [&mut live, &mut reference] {
        machine.set_program_mode(true).unwrap();
    }
    let waits = live.main_wait_visits;
    for _ in 0..2048 {
        compare_word(&mut live, &mut reference);
        if live.main_wait_visits > waits && live.machine.act.state.status[11] {
            break;
        }
    }
    assert!(live.main_wait_visits > waits && live.machine.act.state.status[11]);
    let blank =
        Hp67MagneticCard::default().with_track(Hp67CardTrack::Track1, Hp67MagneticTrack::default());
    for machine in [&mut live, &mut reference] {
        machine
            .insert_magnetic_card(blank.clone(), CardInsertionEnd::End1)
            .unwrap();
    }
    for _ in 0..2048 {
        compare_word(&mut live, &mut reference);
        if live.card_record_stream_active() {
            break;
        }
    }
    assert!(live.card_record_stream_active());
    let mut written = Vec::new();
    for _ in 0..32768 {
        if let Some(Hp67ArchitecturalExecution {
            operation: Hp67ArchitecturalOperation::CrcDataWrite { card_word, .. },
            ..
        }) = compare_word(&mut live, &mut reference)
        {
            written.push(card_word);
        }
        if live.card_transport_complete() && live.machine.crc.queued_write_words() == 0 {
            break;
        }
    }
    assert_eq!(written.len(), HP67_CARD_RECORDS_PER_TRACK);
    let card = live.take_completed_magnetic_card().unwrap();
    assert_eq!(Some(card.clone()), reference.take_completed_magnetic_card());
    let (mut reader, mut read_reference) = boot_pair();
    for machine in [&mut reader, &mut read_reference] {
        machine
            .insert_magnetic_card(card.clone(), CardInsertionEnd::End1)
            .unwrap();
    }
    for _ in 0..2048 {
        compare_word(&mut reader, &mut read_reference);
        if reader.card_record_stream_active() {
            break;
        }
    }
    assert!(reader.card_record_stream_active());
    let mut read = Vec::new();
    for _ in 0..32768 {
        if let Some(Hp67ArchitecturalExecution {
            operation: Hp67ArchitecturalOperation::CrcDataRead { card_word, .. },
            ..
        }) = compare_word(&mut reader, &mut read_reference)
        {
            read.push(card_word);
        }
        if reader.card_transport_complete() && read.len() == HP67_CARD_RECORDS_PER_TRACK {
            break;
        }
    }
    assert_eq!(read, written);
    assert_eq!(reader.take_completed_magnetic_card(), Some(card));
}
