use super::*;

#[test]
fn m15w_full_live_owned_register_file_matches_reference_through_keys_and_data() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut recorded = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut trace = StructuralBusTrace::default();
    let mut changes = 0;
    let mut tails = 0;
    let mut previous = live.act_serial.owned_flow_state().unwrap();
    for cycle in 0..2400 {
        let contact = match cycle {
            500 => Some(Some(Hp67Key::Rcl)),
            700 | 1400 => Some(None),
            1200 => Some(Some(Hp67Key::Digit0)),
            _ => None,
        };
        if let Some(contact) = contact {
            for machine in [&mut live, &mut recorded, &mut reference] {
                machine.set_key_contact(contact);
            }
        }
        tails += usize::from(live.data_serial.frame_in_progress());
        let tick = live.backplane.tick().get();
        let execution = live.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(
            execution,
            reference.step_firmware_cycle_with_execution().unwrap()
        );
        assert_eq!(
            execution,
            recorded
                .step_firmware_cycle_with_trace(Some(&mut trace))
                .unwrap()
        );
        let owned = live.act_serial.owned_flow_state().unwrap();
        assert_eq!(owned, ActSerialFlowState::capture(&live.machine.act.state));
        assert_eq!(live.fetch_rom.received_address(), Ok(owned.pc));
        assert_eq!(
            live.act_serial.owned_register_state().as_ref(),
            Some(&live.machine.act.state)
        );
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.act.state, recorded.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.act_serial, recorded.act_serial);
        assert_eq!(live.backplane.tick().get() - tick, 224);
        changes += usize::from(owned != previous);
        previous = owned;
    }
    assert!(changes > 10);
    assert!(tails > 0);
    assert_eq!(trace.samples.len(), 2400 * 224);
    live.reset_power_on().unwrap();
    assert_eq!(
        live.act_serial.owned_register_state(),
        Some(live.machine.act.state.clone())
    );
    assert_eq!(
        live.act_serial.owned_flow_state(),
        Some(ActSerialFlowState::capture(&live.machine.act.state))
    );
    assert!(live.act_serial.pending_instruction().is_none());
    assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), None);
    reference.reset_power_on().unwrap();
    assert!(reference.act_serial.owned_flow_state().is_none());
    assert!(reference.act_serial.owned_register_state().is_none());
}

#[test]
fn m15w_each_register_and_latch_mirror_is_rejected_before_word_or_tail() {
    for field in 0..21 {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        if field >= 19 {
            for _ in 0..2000 {
                if live.data_serial.frame_in_progress() {
                    break;
                }
                live.step_firmware_cycle_with_execution().unwrap();
            }
            assert!(live.data_serial.frame_in_progress());
        }
        let s = &mut live.machine.act.state;
        match field {
            0 => s.a[0] ^= 1,
            1 => s.b[0] ^= 1,
            2 | 19 => s.c[0] ^= 1,
            3 => s.y[0] ^= 1,
            4 => s.z[0] ^= 1,
            5 => s.t[0] ^= 1,
            6 => s.m1[0] ^= 1,
            7 => s.m2[0] ^= 1,
            8 => s.f ^= 1,
            9 => s.p ^= 1,
            10 => s.p_change[0] = 1,
            11 => s.decimal = !s.decimal,
            12 => s.carry = !s.carry,
            13 => s.previous_carry = !s.previous_carry,
            14 => s.status[0] = !s.status[0],
            15 => s.key_buffer = Some(7),
            16 => s.display_enable = !s.display_enable,
            17 => s.display_14_digit = !s.display_14_digit,
            18 => s.ram_address ^= 1,
            20 => s.status[3] = !s.status[3],
            _ => unreachable!(),
        }
        let tick = live.backplane.tick();
        let act = live.act_serial;
        let data = live.data_serial;
        let pipeline = live.pipeline;
        let error = live.step_firmware_cycle_with_execution().unwrap_err();
        assert!(
            error.contains("ACT owned register/mirror mismatch"),
            "field={field} {error}"
        );
        assert_eq!(live.backplane.tick(), tick);
        assert_eq!(live.act_serial, act);
        assert_eq!(live.data_serial, data);
        assert_eq!(live.pipeline, pipeline);
    }
}

#[test]
fn m15w_card_crc_ports_update_the_owned_act_without_importing_mirror_c() {
    use hp67emu::machines::hp67::{Hp67CardTrack, Hp67MagneticTrack, HP67_CARD_RECORDS_PER_TRACK};
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..1200 {
        live.step_firmware_cycle_with_execution().unwrap();
    }
    let track = Hp67MagneticTrack::from_words([0x0300_0000; HP67_CARD_RECORDS_PER_TRACK]).unwrap();
    let card = Hp67MagneticCard::default().with_track(Hp67CardTrack::Track1, track);
    live.insert_magnetic_card(card, CardInsertionEnd::End1)
        .unwrap();
    let mut conditions = 0;
    for _ in 0..4096 {
        let execution = live.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(
            live.act_serial.owned_register_state().as_ref(),
            Some(&live.machine.act.state)
        );
        if live
            .data_serial
            .crc_control()
            .unwrap()
            .result()
            .is_some_and(|result| result.condition == Some(true))
        {
            conditions += 1;
        }
        if matches!(
            execution.map(|word| word.operation),
            Some(Hp67ArchitecturalOperation::CrcDataRead { .. })
        ) {
            assert!(conditions > 0);
            return;
        }
    }
    panic!("firmware did not consume the card CRC record");
}
