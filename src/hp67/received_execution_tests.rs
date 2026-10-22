use super::*;

#[test]
fn m15u_live_received_execution_matches_reference_and_trace_with_tail_and_goto() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut recorded = Hp67LiveMachine::power_on_default().unwrap();
    let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut trace = StructuralBusTrace::default();
    let mut implied = 0;
    let mut tails = 0;
    for cycle in 0..2200 {
        let contact = match cycle { 500 => Some(Some(Hp67Key::Rcl)), 700 | 1400 => Some(None), 1200 => Some(Some(Hp67Key::Digit0)), _ => None };
        if let Some(contact) = contact {
            for machine in [&mut live, &mut recorded, &mut reference] { machine.set_key_contact(contact); }
        }
        let pending = live.act_serial.pending_instruction();
        let tick = live.backplane.tick().get();
        tails += usize::from(live.data_serial.frame_in_progress());
        let execution = live.step_firmware_cycle_with_execution().unwrap();
        let observed = recorded.step_firmware_cycle_with_trace(Some(&mut trace)).unwrap();
        let expected = reference.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(execution, expected);
        assert_eq!(execution, observed);
        assert_eq!(execution.map(|execution| execution.word), pending.map(|frame| frame.word));
        implied += usize::from(pending.is_some_and(|frame| !frame.instruction));
        assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.machine.act.state, recorded.machine.act.state);
        assert_eq!(live.machine.crc, reference.machine.crc);
        assert_eq!(live.act_serial, recorded.act_serial);
        assert_eq!(live.backplane.tick().get() - tick, 224);
    }
    assert!(implied > 0 && tails > 0);
    assert_eq!(trace.samples.len(), 2200 * 224);
    live.reset_power_on().unwrap();
    assert!(live.act_serial.received_execution_enabled());
    assert!(live.act_serial.pending_instruction().is_none());
    assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), None);
    reference.reset_power_on().unwrap();
    assert!(!reference.act_serial.received_execution_enabled());
}

#[test]
fn m15u_poisoned_pipeline_cannot_supply_an_instruction_or_consume_a_tail() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2000 {
        if live.data_serial.frame_in_progress() { break; }
        live.step_firmware_cycle_with_execution().unwrap();
    }
    assert!(live.data_serial.frame_in_progress());
    let frame = live.act_serial.pending_instruction().unwrap();
    live.pipeline.complete_cycle(frame.word ^ 1);
    let tick = live.backplane.tick();
    let act = live.act_serial;
    let data = live.data_serial;
    let state = live.machine.act.state.clone();
    let error = live.step_firmware_cycle_with_execution().unwrap_err();
    assert!(error.contains("ACT received/pipeline mismatch"), "{error}");
    assert_eq!(live.backplane.tick(), tick);
    assert_eq!(live.act_serial, act);
    assert_eq!(live.data_serial, data);
    assert_eq!(live.machine.act.state, state);
}

#[test]
fn m15u_poisoned_sync_mirror_cannot_reclassify_the_received_word() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.step_firmware_cycle_with_execution().unwrap();
    let frame = live.act_serial.pending_instruction().unwrap();
    live.machine.act.state.instruction_state = if frame.instruction { ActInstructionState::ThenGoto } else { ActInstructionState::Normal };
    let tick = live.backplane.tick();
    let act = live.act_serial;
    let error = live.step_firmware_cycle_with_execution().unwrap_err();
    assert!(error.contains("ACT received SYNC/mirror mismatch"), "{error}");
    assert_eq!(live.backplane.tick(), tick);
    assert_eq!(live.act_serial, act);
}
