use super::*;
use super::super::act::ActInstructionState;

fn launch(bus: &mut Hp67ElectricalBackplane, word: u16, instruction: bool) {
    let bit = bus.word_bit();
    bus.commit_serial_launch(
        act_address_drive(0x123, bit),
        RomIsDrives::Aggregate(rom_word_drive(word, bit)),
        if bit >= 46 && instruction { Drive::High } else { Drive::Low }, None,
    );
}

fn receive(act: &mut ActSerialEndpoint, bus: &mut Hp67ElectricalBackplane, word: u16, instruction: bool) {
    act.begin_fetch_cycle(0x123);
    let tick = bus.tick().get();
    for bit in 0..56 {
        launch(bus, word, instruction);
        bus.advance_clock();
        bus.advance_clock();
        if bit >= 46 {
            act.sample_instruction_at_edge(bus).unwrap();
            if bit < 55 { assert!(act.pending_instruction().is_none()); }
        }
        bus.advance_clock();
        bus.advance_clock();
    }
    assert_eq!(bus.tick().get() - tick, 224);
}

#[test]
fn m15u_act_all_words_and_sync_qualifiers_start_from_received_latch() {
    for instruction in [false, true] {
        for word in 0..=0x3ff {
            let mut act = ActSerialEndpoint::new(0).with_received_execution();
            let mut bus = Hp67ElectricalBackplane::default();
            let state = ActArchitecturalState {
                instruction_state: if instruction { ActInstructionState::Normal } else { ActInstructionState::ThenGoto },
                ..Default::default()
            };
            assert_eq!(act.begin_received_execution(&state).unwrap(), None);
            receive(&mut act, &mut bus, word, instruction);
            assert_eq!(act.serial_execution(), None);
            let before = act;
            assert_eq!(act.begin_execution(word, &state), Err(ActSerialExecutionError::ReceivedInstructionRequired));
            assert_eq!(act, before);
            assert_eq!(act.begin_received_execution(&state).unwrap(), Some(word));
            assert!(act.pending_instruction().is_none());
            let expected = ActSerialExecution::new(word, state.instruction_state).unwrap();
            assert_eq!(act.serial_execution(), Some(expected));
            assert_eq!(act.serial_execution_state(), Some(ActSerialStateSnapshot::capture(&state)));
            for bit in 0..56 { act.advance_execution_for_bit(bit).unwrap(); }
            assert_eq!(act.serial_execution().unwrap().next_word_bit(), None);
        }
    }
}

#[test]
fn m15u_act_return_order_phase_and_sync_faults_preserve_both_latches() {
    let mut act = ActSerialEndpoint::new(0).with_received_execution();
    let mut bus = Hp67ElectricalBackplane::default();
    let before = act;
    assert!(act.sample_for_bit(47, LogicLevel::High).is_err());
    assert_eq!(act, before);
    for bit in 0..56 {
        launch(&mut bus, 0x270, true);
        if bit >= 46 {
            let before = act;
            assert!(act.sample_instruction_at_edge(&bus).is_err());
            assert_eq!(act, before);
        }
        bus.advance_clock();
        bus.advance_clock();
        if bit >= 46 {
            let before = act;
            if bit == 46 {
                let original = bus.driver_drive(Hp67Net::Isa, Hp67Driver::StructuralRomResponder);
                bus.drive(Hp67Net::Isa, Hp67Driver::StructuralRomResponder, Drive::High);
                bus.drive(Hp67Net::Isa, Hp67Driver::External, Drive::Low);
                assert!(act.sample_instruction_at_edge(&bus).is_err());
                assert_eq!(act, before);
                bus.drive(Hp67Net::Isa, Hp67Driver::StructuralRomResponder, original);
                bus.drive(Hp67Net::Isa, Hp67Driver::External, Drive::HighZ);
            }
            if bit == 47 {
                bus.drive(Hp67Net::Sync, Hp67Driver::External, Drive::Low);
                assert!(act.sample_instruction_at_edge(&bus).is_err());
                assert_eq!(act, before);
                bus.drive(Hp67Net::Sync, Hp67Driver::External, Drive::HighZ);
            }
            act.sample_instruction_at_edge(&bus).unwrap();
            let after = act;
            assert!(act.sample_instruction_at_edge(&bus).is_err());
            assert_eq!(act, after);
        }
        bus.advance_clock();
        bus.advance_clock();
    }
    assert_eq!(act.pending_instruction().unwrap().word, 0x270);
    let before = act;
    assert!(act.sample_for_bit(55, LogicLevel::Low).is_err());
    assert_eq!(act, before);
}

#[test]
fn m15u_act_receives_changed_wire_word_and_preserves_pending_on_bad_binding() {
    let mut act = ActSerialEndpoint::new(0).with_received_execution();
    let mut bus = Hp67ElectricalBackplane::default();
    for bit in 0..56 {
        launch(&mut bus, 0o1000, false);
        bus.advance_clock();
        bus.advance_clock();
        if bit == 46 { bus.drive(Hp67Net::Isa, Hp67Driver::StructuralRomResponder, Drive::High); }
        if bit >= 46 { act.sample_instruction_at_edge(&bus).unwrap(); }
        bus.advance_clock();
        bus.advance_clock();
    }
    assert_eq!(act.pending_instruction().unwrap().word, 0o1001);
    let before = act;
    assert!(act.begin_received_execution(&ActArchitecturalState::default()).is_err());
    assert_eq!(act, before);
    let state = ActArchitecturalState { instruction_state: ActInstructionState::ThenGoto, ..Default::default() };
    assert_eq!(act.begin_received_execution(&state).unwrap(), Some(0o1001));
    assert_eq!(act.serial_execution().unwrap().class(), ActSerialWordClass::ThenGotoData);
}

#[test]
fn m15u_act_pending_frame_cannot_be_overwritten_or_replace_active_execution() {
    let mut act = ActSerialEndpoint::new(0).with_received_execution();
    let mut bus = Hp67ElectricalBackplane::default();
    receive(&mut act, &mut bus, 0x270, true);
    let old = act.pending_instruction();
    act.begin_fetch_cycle(0x123);
    for bit in 0..56 {
        launch(&mut bus, 0x2f0, true);
        bus.advance_clock();
        bus.advance_clock();
        if (46..55).contains(&bit) { act.sample_instruction_at_edge(&bus).unwrap(); }
        if bit == 55 {
            let before = act;
            assert_eq!(act.sample_instruction_at_edge(&bus), Err(SerialFetchError::UnconsumedActInstruction));
            assert_eq!(act, before);
            assert_eq!(act.pending_instruction(), old);
            assert_eq!(act.begin_received_execution(&ActArchitecturalState::default()).unwrap(), Some(0x270));
            act.sample_instruction_at_edge(&bus).unwrap();
            let before = act;
            assert!(act.begin_received_execution(&ActArchitecturalState::default()).is_err());
            assert_eq!(act, before);
            for coordinate in 0..56 { act.advance_execution_for_bit(coordinate).unwrap(); }
            assert_eq!(act.begin_received_execution(&ActArchitecturalState::default()).unwrap(), Some(0x2f0));
        }
        bus.advance_clock();
        bus.advance_clock();
    }
}


#[test]
fn m15u_is_only_prefix_cannot_manufacture_a_qualified_instruction() {
    let mut act = ActSerialEndpoint::new(0).with_received_execution();
    let mut bus = Hp67ElectricalBackplane::default();
    for bit in 0..56 {
        launch(&mut bus, 0x270, true);
        bus.advance_clock();
        bus.advance_clock();
        if (46..55).contains(&bit) {
            act.sample_for_bit(bit, bus.level(Hp67Net::Isa)).unwrap();
        }
        if bit == 55 {
            let before = act;
            assert_eq!(act.sample_instruction_at_edge(&bus), Err(SerialFetchError::MissingSyncQualifier));
            assert_eq!(act, before);
            assert!(act.pending_instruction().is_none());
        }
        bus.advance_clock();
        bus.advance_clock();
    }
}
