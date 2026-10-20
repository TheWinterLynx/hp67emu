use super::*;

#[test]
fn m15q_live_autonomous_ram_matches_reference_boot_and_all_registers() {
    for traced in [false, true] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
        let mut trace = StructuralBusTrace::default();
        let mut transfers = 0;
        for _ in 0..2000 {
            let tick = live.backplane.tick().get();
            live.step_firmware_cycle_with_trace(if traced { Some(&mut trace) } else { None }).unwrap();
            reference.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.machine.act.state, reference.machine.act.state);
            assert_eq!(live.machine.ram, reference.machine.ram);
            assert_eq!(live.machine.crc, reference.machine.crc);
            assert_eq!(live.backplane.tick().get() - tick, 224);
            let ram = live.serial_ram.as_ref().unwrap();
            for address in 0..64 {
                assert_eq!(ram.register(address).copied(), live.machine.ram.read(address));
            }
            transfers += usize::from(live.pending_ram_data_transfer.is_some() || live.pending_address_data_frame.is_some());
        }
        assert!(transfers > 0);
        assert_eq!(live.phase, LiveBootPhase::Idle);
        assert_eq!(trace.samples.len(), if traced { 2000 * 224 } else { 0 });
        live.reset_power_on().unwrap();
        assert_eq!(live.serial_ram, Some(Hp67SerialRam::default()));
        assert!(!live.data_serial.frame_in_progress());
    }
}

#[test]
fn m15q_corrupted_oracle_ram_cannot_supply_read_bits_to_live_act() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for cycle in 0..2000 {
        if cycle == 500 { live.set_key_contact(Some(Hp67Key::Digit1)); }
        if cycle == 800 { live.set_key_contact(None); }
        let plan = live.pipeline.prefetched_word()
            .and_then(|word| act_data_transfer_plan(word, &live.machine.act.state));
        if let Some(plan) = plan.filter(|plan| plan.direction == Hp67DataTransferDirection::PeripheralToAct
            && plan.address < 64 && live.pending_ram_data_transfer.is_none()) {
            let actual = *live.serial_ram.as_ref().unwrap().register(plan.address).unwrap();
            let poisoned = actual.map(|digit| digit ^ 15);
            assert!(live.machine.ram.write(plan.address, poisoned));
            live.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.pending_ram_data_transfer.unwrap().expected, poisoned);
            let before = live.machine.act.state.c;
            let error = live.step_firmware_cycle_with_execution().unwrap_err();
            assert!(error.contains("RAM DATA authority mismatch"), "{error}");
            assert!(error.contains(&format!("actual={actual:?}")), "{error}");
            assert_eq!(live.machine.act.state.c, before);
            assert_eq!(live.serial_ram.as_ref().unwrap().register(plan.address), Some(&actual));
            return;
        }
        live.step_firmware_cycle_with_execution().unwrap();
    }
    panic!("real firmware did not exercise an installed RAM read");
}
