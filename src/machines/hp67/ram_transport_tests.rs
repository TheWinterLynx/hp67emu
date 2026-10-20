use super::*;

struct Program;
impl Hp67RomWordSource for Program {
    fn read_word(&self, address: u16) -> Option<u16> {
        [0o1160, 0o1360, 0o0070, 0o1260, 0, 0]
            .get(usize::from(address))
            .copied()
    }
}

#[test]
fn m15q_shared_transport_ram_read_ignores_reference_payload_and_preserves_clocked_tail() {
    for traced in [false, true] {
        for mask in [0, 15] {
            let mut bus = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default().with_ram_command_receiver();
            let mut ram = Hp67SerialRam::default();
            let mut state = ActArchitecturalState::default();
            let mut trace = StructuralBusTrace::default();
            let mut executing = None;
            let value = std::array::from_fn(|digit| (digit as u8 * 3 + 1) & 15);
            let mut address = [15; 14];
            address[0] = 15;
            address[1] = 2;
            for cycle in 0..6 {
                let tick = bus.tick().get();
                if data.frame_in_progress() {
                    let completed = run_structural_data_tail_prefix_with_ram(
                        &mut bus,
                        &state,
                        &mut act,
                        &rom,
                        &mut rom0,
                        &mut data,
                        if traced { Some(&mut trace) } else { None },
                        Some(&mut ram),
                    )
                    .unwrap();
                    let expected = match cycle {
                        2 => address,
                        3 | 4 => value,
                        _ => panic!("unexpected DATA tail"),
                    };
                    assert_eq!(completed, Some(expected));
                    if cycle == 3 {
                        assert_eq!(ram.register(0x2f), Some(&value));
                    }
                }
                let (payload, direction) = match executing {
                    Some(0o1160) => (
                        Some(address),
                        Some(Hp67DataTransferDirection::ActToPeripheral),
                    ),
                    Some(0o1360) => (
                        Some(value),
                        Some(Hp67DataTransferDirection::ActToPeripheral),
                    ),
                    // Deliberately wrong source contents. Only the frame mask is
                    // supplied; actual bits must originate in independent RAM.
                    Some(0o0070) => (
                        Some([mask; 14]),
                        Some(Hp67DataTransferDirection::PeripheralToAct),
                    ),
                    _ => (None, None),
                };
                state.c = payload.unwrap_or([0; 14]);
                if let Some(word) = executing {
                    act.begin_execution(word, &state).unwrap();
                }
                data.enable_electrical_level_bridge(direction);
                let result = if traced {
                    run_structural_display_fetch_data_phase_cycle_traced_with_ram(
                        &mut bus,
                        cycle,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &Program,
                        &mut data,
                        payload,
                        &mut trace,
                        Some(&mut ram),
                    )
                    .unwrap()
                } else {
                    run_structural_display_fetch_data_phase_cycle_with_ram(
                        &mut bus,
                        cycle,
                        &state,
                        &mut act,
                        &mut rom,
                        &mut rom0,
                        &Program,
                        &mut data,
                        payload,
                        Some(&mut ram),
                    )
                    .unwrap()
                };
                assert_eq!(result.completed_data, None);
                assert_eq!(ram.received_command(), data.received_ram_command());
                assert_eq!(bus.tick().get() - tick, 224);
                if cycle == 4 {
                    assert_eq!(ram.register(0x2f), Some(&[0; 14]));
                }
                executing = Some(result.word.fetched_word);
            }
            assert_eq!(ram.address(), 0x2f);
            assert_eq!(trace.samples.len(), if traced { 6 * 224 } else { 0 });
        }
    }
}
