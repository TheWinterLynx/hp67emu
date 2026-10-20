use super::*;
use hp67emu::machines::hp67::Hp67RamCommand;

#[test]
fn m15p_live_ram_command_observer_matches_real_firmware_in_both_transports() {
    let mut ordinary = Hp67LiveMachine::power_on_default().unwrap();
    let mut recorded = Hp67LiveMachine::power_on_default().unwrap();
    let mut trace = StructuralBusTrace::default();
    let mut commands = 0;
    let mut implied = 0;
    for _ in 0..1200 {
        let tick = ordinary.backplane.tick().get();
        ordinary.step_firmware_cycle_with_execution().unwrap();
        recorded
            .step_firmware_cycle_with_trace(Some(&mut trace))
            .unwrap();
        let frame = ordinary.data_serial.received_ram_command().unwrap();
        assert_eq!(frame, recorded.data_serial.received_ram_command().unwrap());
        assert_eq!(frame.word, ordinary.act_serial.fetched_word().unwrap());
        assert_eq!(
            frame.instruction,
            ordinary.act_serial.fetched_instruction_state().unwrap() == ActInstructionState::Normal
        );
        if frame.command() != Hp67RamCommand::Idle {
            commands += 1;
        }
        if !frame.instruction {
            implied += 1;
            assert_eq!(frame.command(), Hp67RamCommand::Idle);
        }
        assert_eq!(ordinary.machine.act.state, recorded.machine.act.state);
        assert_eq!(ordinary.machine.ram, recorded.machine.ram);
        assert_eq!(ordinary.backplane.tick().get() - tick, 224);
        assert_eq!(ordinary.backplane.tick(), recorded.backplane.tick());
    }
    assert!(commands > 0);
    assert!(implied > 0);
    assert_eq!(ordinary.phase, LiveBootPhase::Idle);
    assert_eq!(trace.samples.len(), 1200 * 224);
    ordinary.reset_power_on().unwrap();
    assert_eq!(ordinary.data_serial.received_ram_command(), None);
    assert!(ordinary.data_serial.ram_command_receiver_enabled());
}

#[test]
fn m15p_synthetic_execution_cannot_feed_the_ram_command_observer() {
    let mut live = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    live.machine.act.state.c = [15; 14];
    live.pipeline.complete_cycle(0o1160);
    live.step_firmware_cycle_with_execution().unwrap();
    // Existing reference fixture injects execution while actual ROM fetch is 001.
    // The independent RAM receiver must see 3e3 on the wire, never injected 270.
    let frame = live.data_serial.received_ram_command().unwrap();
    assert_eq!(frame.word, 0x3e3);
    assert_eq!(frame.command(), Hp67RamCommand::Idle);
    assert!(live.pending_address_data_frame.is_some());
}
