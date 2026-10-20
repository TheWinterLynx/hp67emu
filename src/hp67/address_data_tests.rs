use super::*;
use hp67emu::{
    emulation::{Drive, LogicLevel},
    machines::hp67::{Hp67Driver, Hp67Net},
};

#[test]
fn m15o_address_write_read_use_frozen_data_and_clocked_tails() {
    for traced in [false, true] {
        for address in [0x00, 0x20, 0x3f] {
            let mut live = Hp67LiveMachine::power_on_reference_fixture().unwrap();
            let mut trace = StructuralBusTrace::default();
            let mut address_c = [15; 14];
            address_c[0] = address & 15;
            address_c[1] = address >> 4;
            let payload = std::array::from_fn(|digit| (digit as u8 * 3 + 1) & 15);
            let before = live.machine.ram.clone();
            live.machine.act.state.c = address_c;
            for (index, word) in [0o1160, 0o1360, 0o0070, 0].into_iter().enumerate() {
                if index == 1 {
                    // The old address tail must retain C even after the next
                    // source register changes, before binding the RAM write.
                    live.machine.act.state.c = payload;
                } else if index == 2 {
                    live.machine.act.state.c = [2; 14];
                }
                let tick = live.backplane.tick().get();
                live.pipeline.complete_cycle(word);
                live.step_firmware_cycle_with_trace(if traced { Some(&mut trace) } else { None })
                    .unwrap();
                assert_eq!(live.backplane.tick().get() - tick, 224);
                assert_eq!(live.machine.act.state.ram_address, address);
                if index == 0 {
                    assert_eq!(live.pending_address_data_frame, Some(address_c));
                    assert!(live.pending_ram_data_transfer.is_none());
                    assert_eq!(live.machine.ram, before);
                    assert_eq!(
                        live.backplane
                            .driver_drive(Hp67Net::Data, Hp67Driver::Act1820_2530),
                        Drive::High
                    );
                    assert_eq!(
                        live.backplane
                            .driver_drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
                        Drive::HighZ
                    );
                    assert_eq!(live.backplane.active_data_driver_count(), 1);
                } else {
                    assert!(live.pending_address_data_frame.is_none());
                }
            }
            assert_eq!(live.machine.act.state.c, payload);
            assert_eq!(live.machine.ram.read(address), Some(payload));
            assert!(live.pending_ram_data_transfer.is_none());
            assert_eq!(live.backplane.level(Hp67Net::Data), LogicLevel::Floating);
            assert_eq!(trace.samples.len(), if traced { 4 * 224 } else { 0 });
            for (index, sample) in trace.samples.iter().enumerate() {
                assert_eq!(sample.tick, index as u64 + 1);
                assert_eq!(sample.transport_bit, ((index / 4) % 56) as u8);
            }
        }
    }
}

#[test]
fn m15o_peripheral_address_transmits_without_ram_write_and_reset_clears_pending() {
    let mut live = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let ram = live.machine.ram.clone();
    let mut payload = [15; 14];
    payload[0] = 11;
    payload[1] = 9;
    live.machine.act.state.c = payload;
    live.pipeline.complete_cycle(0o1160);
    live.step_firmware_cycle_with_execution().unwrap();
    assert_eq!(live.pending_address_data_frame, Some(payload));
    assert_eq!(live.machine.act.state.ram_address, 0x9b);
    assert_eq!(live.machine.ram, ram);
    live.pipeline.complete_cycle(0);
    live.step_firmware_cycle_with_execution().unwrap();
    assert!(live.pending_address_data_frame.is_none());
    assert_eq!(live.machine.ram, ram);
    live.pipeline.complete_cycle(0o1160);
    live.step_firmware_cycle_with_execution().unwrap();
    live.reset_power_on().unwrap();
    assert!(live.pending_address_data_frame.is_none());
    assert!(!live.data_serial.frame_in_progress());
}

#[test]
fn m15o_implied_goto_address_bits_do_not_start_data_frame() {
    let mut live = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    live.machine.act.state.instruction_state = ActInstructionState::ThenGoto;
    live.machine.act.state.c = [15; 14];
    live.pipeline.complete_cycle(0o1160);
    live.step_firmware_cycle_with_execution().unwrap();
    assert!(live.pending_address_data_frame.is_none());
    assert!(!live.data_serial.frame_in_progress());
    assert_eq!(live.backplane.active_data_driver_count(), 0);
}

#[test]
fn m15o_address_tail_fault_blocks_next_instruction_and_preserves_destination() {
    let mut live = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    live.machine.act.state.c = [15; 14];
    live.pipeline.complete_cycle(0o1160);
    live.step_firmware_cycle_with_execution().unwrap();
    let ram = live.machine.ram.clone();
    let state = live.machine.act.state.clone();
    let cycle = live.boot_cycle;
    live.backplane
        .drive(Hp67Net::Data, Hp67Driver::External, Drive::Low);
    live.pipeline.complete_cycle(0o1360);
    assert!(live
        .step_firmware_cycle_with_execution()
        .unwrap_err()
        .contains("prefix failed"));
    assert_eq!(live.pending_address_data_frame, Some([15; 14]));
    assert_eq!(live.machine.act.state, state);
    assert_eq!(live.machine.ram, ram);
    assert_eq!(live.boot_cycle, cycle);
}

#[test]
fn m15o_real_firmware_address_frames_match_in_both_transports() {
    let mut ordinary = Hp67LiveMachine::power_on_default().unwrap();
    let mut recorded = Hp67LiveMachine::power_on_default().unwrap();
    let mut trace = StructuralBusTrace::default();
    let mut address_frames = 0;
    for _ in 0..1200 {
        let tick = ordinary.backplane.tick().get();
        ordinary.step_firmware_cycle_with_execution().unwrap();
        recorded
            .step_firmware_cycle_with_trace(Some(&mut trace))
            .unwrap();
        assert_eq!(ordinary.backplane.tick().get() - tick, 224);
        assert_eq!(ordinary.backplane.tick(), recorded.backplane.tick());
        assert_eq!(ordinary.machine.act.state, recorded.machine.act.state);
        assert_eq!(ordinary.machine.ram, recorded.machine.ram);
        assert_eq!(
            ordinary.pending_address_data_frame,
            recorded.pending_address_data_frame
        );
        address_frames += usize::from(ordinary.pending_address_data_frame.is_some());
    }
    assert!(address_frames > 0);
    assert_eq!(ordinary.phase, LiveBootPhase::Idle);
    assert_eq!(trace.samples.len(), 1200 * 224);
}
