use super::*;
use hp67emu::machines::hp67::{Hp67SerialCrcControl, CRC_FLAG_CARD_PRESENT};

#[test]
fn m15t_crc_live_matches_reference_and_traced_firmware_with_real_contacts() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut traced = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut trace = StructuralBusTrace::default();
    let mut controls = 0;
    let mut motor = false;
    for cycle in 0..2400 {
        if cycle == 1200 {
            for machine in [&mut live, &mut traced, &mut reference] {
                machine.set_card_present_contact(true).unwrap();
            }
        }
        let tick = live.backplane.tick().get();
        live.step_firmware_cycle_with_execution().unwrap();
        traced
            .step_firmware_cycle_with_trace(Some(&mut trace))
            .unwrap();
        reference.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.machine.crc, traced.machine.crc);
        assert_eq!(live.machine.act.state, traced.machine.act.state);
        assert_eq!(
            live.data_serial.crc_control(),
            traced.data_serial.crc_control()
        );
        let crc = live.data_serial.crc_control().unwrap();
        crc.check_mirror(&live.machine.crc).unwrap();
        controls += usize::from(crc.result().is_some());
        motor |= crc.flags()[CRC_FLAG_MOTOR_ON];
        assert_eq!(live.backplane.tick().get() - tick, 224);
    }
    assert!(controls > 0 && motor);
    assert_eq!(trace.samples.len(), 2400 * 224);
    assert_eq!(
        live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
        Some(true)
    );
    live.reset_power_on().unwrap();
    assert_eq!(
        live.data_serial.crc_control(),
        Some(&Hp67SerialCrcControl::default())
    );
    reference.reset_power_on().unwrap();
    assert!(reference.data_serial.crc_control().is_none());
}

#[test]
fn m15t_crc_corrupted_mirror_is_rejected_before_clock_or_instruction() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.machine
        .crc
        .commit_control_flag(CRC_FLAG_MOTOR_ON as u8, true)
        .unwrap();
    let tick = live.backplane.tick();
    let cycle = live.boot_cycle;
    let device = *live.data_serial.crc_control().unwrap();
    let error = live.step_firmware_cycle_with_execution().unwrap_err();
    assert!(error.contains("autonomous CRC latch 9 mismatch"), "{error}");
    assert_eq!(live.backplane.tick(), tick);
    assert_eq!(live.boot_cycle, cycle);
    assert_eq!(live.data_serial.crc_control(), Some(&device));
}

#[test]
fn m15t_crc_real_card_transport_hands_ready_to_independent_control() {
    use hp67emu::machines::hp67::{Hp67CardTrack, Hp67MagneticTrack, HP67_CARD_RECORDS_PER_TRACK};
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..1200 {
        live.step_firmware_cycle_with_execution().unwrap();
    }
    let track = Hp67MagneticTrack::from_words([0x0300_0000; HP67_CARD_RECORDS_PER_TRACK]).unwrap();
    let card = Hp67MagneticCard::default().with_track(Hp67CardTrack::Track1, track);
    live.insert_magnetic_card(card, CardInsertionEnd::End1)
        .unwrap();
    let mut ready = false;
    let mut consumed = false;
    for _ in 0..4096 {
        let execution = live.step_firmware_cycle_with_execution().unwrap();
        let crc = live.data_serial.crc_control().unwrap();
        crc.check_mirror(&live.machine.crc).unwrap();
        ready |= crc.flags()[CRC_FLAG_BUFFER_READY];
        if matches!(
            execution.map(|word| word.operation),
            Some(Hp67ArchitecturalOperation::CrcDataRead { .. })
        ) {
            consumed = true;
            break;
        }
    }
    assert!(ready && consumed);
}

#[test]
fn m16k_crc_transaction_real_firmware_mode_key_and_card_contacts_preserve_trace_and_oracle() {
    use hp67emu::machines::hp67::Hp67Net;
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut traced = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut controls = 0;
    for cycle in 0..3200 {
        for machine in [&mut live, &mut traced, &mut reference] {
            if cycle == 1200 {
                machine.set_program_mode(true).unwrap();
            }
            if cycle == 2200 {
                machine.set_program_mode(false).unwrap();
            }
            if cycle == 2600 {
                machine.set_card_present_contact(true).unwrap();
            }
            machine.set_key_contact(if (700..850).contains(&cycle) {
                Some(Hp67Key::Digit7)
            } else {
                None
            });
        }
        let tick = live.backplane.tick().get();
        let mut trace = StructuralBusTrace::default();
        assert_eq!(
            live.step_firmware_cycle_with_execution().unwrap(),
            traced
                .step_firmware_cycle_with_trace(Some(&mut trace))
                .unwrap()
        );
        reference.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.machine.ram, reference.machine.ram);
        assert_eq!(live.act_serial, traced.act_serial);
        assert_eq!(live.data_serial, traced.data_serial);
        assert_eq!(live.serial_ram, traced.serial_ram);
        assert_eq!(live.owned_crc_data, traced.owned_crc_data);
        assert_eq!(live.display_circuit, traced.display_circuit);
        for net in Hp67Net::ALL {
            assert_eq!(live.backplane.level(net), traced.backplane.level(net));
        }
        let crc = live.data_serial.crc_control().unwrap();
        crc.check_mirror(&live.machine.crc).unwrap();
        controls += usize::from(crc.result().is_some());
        assert_eq!(live.backplane.tick().get(), tick + 224);
        assert_eq!(trace.samples.len(), 224);
    }
    assert!(controls > 0);
    assert_eq!(
        live.machine.crc.external_flag(CRC_FLAG_CARD_PRESENT),
        Some(true)
    );
}
