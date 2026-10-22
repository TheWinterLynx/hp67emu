use super::*;

#[test]
fn m15v_live_owned_flow_matches_reference_and_actual_rom_address() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut recorded = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut trace = StructuralBusTrace::default();
    let mut changes = 0;
    let mut previous = live.act_serial.owned_flow_state().unwrap();
    for cycle in 0..2200 {
        let contact = match cycle { 500 => Some(Some(Hp67Key::Rcl)), 700 | 1400 => Some(None), 1200 => Some(Some(Hp67Key::Digit0)), _ => None };
        if let Some(contact) = contact {
            for machine in [&mut live, &mut recorded, &mut reference] { machine.set_key_contact(contact); }
        }
        let tick = live.backplane.tick().get();
        let execution = live.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(execution, reference.step_firmware_cycle_with_execution().unwrap());
        assert_eq!(execution, recorded.step_firmware_cycle_with_trace(Some(&mut trace)).unwrap());
        let owned = live.act_serial.owned_flow_state().unwrap();
        assert_eq!(owned, ActSerialFlowState::capture(&live.machine.act.state));
        assert_eq!(live.fetch_rom.received_address(), Ok(owned.pc));
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.act.state, recorded.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.act_serial, recorded.act_serial);
        assert_eq!(live.backplane.tick().get() - tick, 224);
        changes += usize::from(owned != previous);
        previous = owned;
    }
    assert!(changes > 10);
    assert_eq!(trace.samples.len(), 2200 * 224);
    live.reset_power_on().unwrap();
    assert_eq!(live.act_serial.owned_flow_state(), Some(ActSerialFlowState::capture(&live.machine.act.state)));
    assert!(live.act_serial.pending_instruction().is_none());
    assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), None);
    reference.reset_power_on().unwrap();
    assert!(reference.act_serial.owned_flow_state().is_none());
}

#[test]
fn m15v_every_poisoned_flow_mirror_field_fails_before_clocks_and_data_tail() {
    for field in 0..5 {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        for _ in 0..2000 {
            if live.data_serial.frame_in_progress() { break; }
            live.step_firmware_cycle_with_execution().unwrap();
        }
        assert!(live.data_serial.frame_in_progress());
        match field {
            0 => live.machine.act.state.pc ^= 1,
            1 => live.machine.act.state.bank ^= 1,
            2 => live.machine.act.state.delayed_rom = if live.machine.act.state.delayed_rom.is_some() { None } else { Some(15) },
            3 => live.machine.act.state.return_stack[0] ^= 1,
            4 => live.machine.act.state.stack_pointer ^= 1,
            _ => unreachable!(),
        }
        let tick = live.backplane.tick();
        let act = live.act_serial;
        let data = live.data_serial;
        let pipeline = live.pipeline;
        let state = live.machine.act.state.clone();
        let error = live.step_firmware_cycle_with_execution().unwrap_err();
        assert!(error.contains("ACT owned flow/mirror mismatch"), "{error}");
        assert_eq!(live.backplane.tick(), tick);
        assert_eq!(live.act_serial, act);
        assert_eq!(live.data_serial, data);
        assert_eq!(live.pipeline, pipeline);
        assert_eq!(live.machine.act.state, state);
    }
}

#[test]
fn m15v_startup_address_serializer_uses_owned_pc_even_with_poisoned_mirror() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.machine.act.state.pc = 0x450;
    live.machine.act.state.bank = 1;
    // Isolate the serializer below the public live mirror guard.
    live.transport_fetch_word(0, None, None).unwrap();
    assert_eq!(live.fetch_rom.received_address(), Ok(0));
    assert_eq!(live.act_serial.address(), 0);
    assert_eq!(live.act_serial.owned_flow_state().unwrap().pc, 0);
    assert_eq!(live.machine.act.state.pc, 0x450);
    assert_eq!(live.backplane.tick().get(), 224);
}
