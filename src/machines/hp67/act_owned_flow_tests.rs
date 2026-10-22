use super::*;
use super::super::{ActInstructionState, Hp67ArchitecturalMachine};

fn prime(act: &mut ActSerialEndpoint, word: u16, instruction: bool) {
    let mut bus = Hp67ElectricalBackplane::default();
    act.begin_fetch_cycle(act.owned_flow_state().unwrap().pc);
    for bit in 0..56 {
        bus.commit_serial_launch(
            act_address_drive(act.address(), bit),
            RomIsDrives::Aggregate(rom_word_drive(word, bit)),
            if bit >= 46 && instruction { Drive::High } else { Drive::Low }, None,
        );
        bus.advance_clock();
        bus.advance_clock();
        if bit >= 46 { act.sample_instruction_at_edge(&bus).unwrap(); }
        bus.advance_clock();
        bus.advance_clock();
    }
    assert_eq!(bus.tick().get(), 224);
}

#[test]
fn m15v_flow_uses_owned_latches_when_all_mirror_flow_fields_are_poisoned() {
    let mut accepted = [0usize; 2];
    for pc in [0x68, 0x4ff, 0xfff] {
        for carry in [false, true] {
            for instruction_state in [ActInstructionState::Normal, ActInstructionState::ThenGoto] {
                for delayed_rom in [None, Some(0), Some(15)] {
                    for word in 0..=0x3ff {
                        let mut machine = Hp67ArchitecturalMachine::default();
                        machine.act.state.pc = pc;
                        machine.act.state.bank = 1;
                        machine.act.state.carry = carry;
                        machine.act.state.instruction_state = instruction_state;
                        machine.act.state.delayed_rom = delayed_rom;
                        machine.act.state.return_stack = [0xabc, 0x321];
                        machine.act.state.stack_pointer = u8::from(carry);
                        machine.act.state.key_buffer = Some(0o244);
                        machine.act.state.a[1] = 5;
                        machine.act.state.a[2] = 10;
                        let owned = ActSerialFlowState::capture(&machine.act.state);
                        let mut poisoned = machine.act.state.clone();
                        poisoned.pc ^= 0xfff;
                        poisoned.bank = 0;
                        poisoned.delayed_rom = Some(7);
                        poisoned.return_stack = [0x111, 0x222];
                        poisoned.stack_pointer = 255;
                        let image = ActSerialFlowResultImage::begin_with_flow(&poisoned, word, owned);
                        if machine.execute_word(word).is_err() { continue; }
                        assert_eq!(image.result(), ActSerialFlowState::capture(&machine.act.state));
                        accepted[usize::from(instruction_state == ActInstructionState::Normal)] += 1;
                    }
                }
            }
        }
    }
    assert!(accepted.into_iter().all(|count| count > 0));
}

#[test]
fn m15v_owned_flow_covers_calls_returns_dispatch_pages_bank_and_implied_goto() {
    for pc in [0x68, 0x4ff, 0xfff] {
        for carry in [false, true] {
            for instruction in [false, true] {
                for bank in [0, 1] {
                    for delayed_rom in [None, Some(0), Some(15)] {
                        for word in [0, 1, 3, 0o1020, 0o0020, 0o0220, 0o1060, 0o40, 0o64, 0x3ff] {
                            let mut machine = Hp67ArchitecturalMachine::default();
                            machine.act.state.pc = pc;
                            machine.act.state.bank = bank;
                            machine.act.state.delayed_rom = delayed_rom;
                            machine.act.state.carry = carry;
                            machine.act.state.instruction_state = if instruction { ActInstructionState::Normal } else { ActInstructionState::ThenGoto };
                            machine.act.state.return_stack = [0xabc, 0x321];
                            machine.act.state.stack_pointer = u8::from(carry);
                            machine.act.state.key_buffer = Some(0o244);
                            machine.act.state.a[1] = 5;
                            machine.act.state.a[2] = 10;
                            let before = machine.act.state.clone();
                            let initial = ActSerialFlowState::capture(&before);
                            let mut act = ActSerialEndpoint::new(pc).with_received_execution().with_owned_flow();
                            // Fixture seed only: production has fixed power-on latches.
                            act.owned_flow = Some(initial);
                            prime(&mut act, word, instruction);
                            act.begin_received_execution(&before).unwrap();
                            machine.execute_word(word).unwrap();
                            machine.prepare_hp67_fetch();
                            let expected = ActSerialFlowState::capture(&machine.act.state);
                            assert_eq!(act.owned_fetch_request(), Some((expected.pc, expected.bank)));
                            for bit in 0..55 {
                                act.advance_execution_for_bit(bit).unwrap();
                                assert_eq!(act.owned_flow_state(), Some(initial));
                            }
                            act.advance_execution_for_bit(55).unwrap();
                            assert_eq!(act.owned_flow_state(), Some(expected));
                            act.check_owned_flow_mirror(&machine.act.state).unwrap();
                        }
                    }
                }
            }
        }
    }
}

#[test]
fn m15v_failed_execution_coordinate_cannot_commit_owned_flow_early() {
    let mut act = ActSerialEndpoint::new(0).with_received_execution().with_owned_flow();
    prime(&mut act, 0, true);
    act.begin_received_execution(&ActArchitecturalState::default()).unwrap();
    let initial = act.owned_flow_state();
    for bit in 0..55 { act.advance_execution_for_bit(bit).unwrap(); }
    assert!(act.advance_execution_for_bit(54).is_err());
    assert_eq!(act.owned_flow_state(), initial);
    assert_eq!(act.owned_fetch_request(), Some((1, 0)));
    act.advance_execution_for_bit(55).unwrap();
    assert_eq!(act.owned_flow_state().unwrap().pc, 1);
}
