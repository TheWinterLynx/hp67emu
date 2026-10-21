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
            live.step_firmware_cycle_with_trace(if traced { Some(&mut trace) } else { None })
                .unwrap();
            reference.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.machine.act.state, reference.machine.act.state);
            assert_eq!(live.machine.ram, reference.machine.ram);
            assert_eq!(live.machine.crc, reference.machine.crc);
            assert_eq!(live.backplane.tick().get() - tick, 224);
            let ram = live.serial_ram.as_ref().unwrap();
            for address in 0..64 {
                assert_eq!(
                    ram.register(address).copied(),
                    live.machine.ram.read(address)
                );
            }
            transfers += usize::from(
                live.pending_ram_data_transfer.is_some()
                    || live.pending_address_data_frame.is_some(),
            );
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
        if cycle == 500 {
            live.set_key_contact(Some(Hp67Key::Rcl));
        }
        if cycle == 700 {
            live.set_key_contact(None);
        }
        if cycle == 1200 {
            live.set_key_contact(Some(Hp67Key::Digit0));
        }
        if cycle == 1400 {
            live.set_key_contact(None);
        }
        let plan = live
            .pipeline
            .prefetched_word()
            .and_then(|word| act_data_transfer_plan(word, &live.machine.act.state));
        if let Some(plan) = plan.filter(|plan| {
            plan.direction == Hp67DataTransferDirection::PeripheralToAct
                && plan.address < 64
                && live.pending_ram_data_transfer.is_none()
        }) {
            let actual = *live
                .serial_ram
                .as_ref()
                .unwrap()
                .register(plan.address)
                .unwrap();
            let poisoned = actual.map(|digit| digit ^ 15);
            assert!(live.machine.ram.write(plan.address, poisoned));
            live.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.pending_ram_data_transfer.unwrap().expected, poisoned);
            let before = live.machine.act.state.c;
            let error = live.step_firmware_cycle_with_execution().unwrap_err();
            assert!(error.contains("RAM DATA authority mismatch"), "{error}");
            assert!(error.contains(&format!("actual={actual:?}")), "{error}");
            assert_eq!(live.machine.act.state.c, before);
            assert_eq!(
                live.serial_ram.as_ref().unwrap().register(plan.address),
                Some(&actual)
            );
            return;
        }
        live.step_firmware_cycle_with_execution().unwrap();
    }
    panic!("real firmware did not exercise an installed RAM read");
}

#[test]
fn m15r_live_reads_use_ram_driver_and_release_reference_rom_driver() {
    use hp67emu::{
        emulation::Drive,
        machines::hp67::{Hp67Driver, Hp67Net},
    };
    for traced in [false, true] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        let mut trace = StructuralBusTrace::default();
        let mut reads = 0;
        for cycle in 0..2000 {
            if cycle == 500 {
                live.set_key_contact(Some(Hp67Key::Rcl));
            }
            if cycle == 700 {
                live.set_key_contact(None);
            }
            if cycle == 1200 {
                live.set_key_contact(Some(Hp67Key::Digit0));
            }
            if cycle == 1400 {
                live.set_key_contact(None);
            }
            let tick = live.backplane.tick().get();
            live.step_firmware_cycle_with_trace(if traced { Some(&mut trace) } else { None })
                .unwrap();
            assert_eq!(live.backplane.tick().get() - tick, 224);
            assert_eq!(
                live.backplane
                    .driver_drive(Hp67Net::Data, Hp67Driver::StructuralRomResponder),
                Drive::HighZ
            );
            if live.pending_ram_data_transfer.is_some_and(|transfer| {
                transfer.direction == Hp67DataTransferDirection::PeripheralToAct
            }) {
                reads += 1;
                assert_ne!(
                    live.backplane
                        .driver_drive(Hp67Net::Data, Hp67Driver::RamAggregate),
                    Drive::HighZ
                );
                assert_eq!(
                    live.backplane
                        .driver_drive(Hp67Net::Data, Hp67Driver::Act1820_2530),
                    Drive::HighZ
                );
                assert_eq!(live.backplane.active_data_driver_count(), 1);
            }
        }
        assert!(reads > 0);
        assert_eq!(trace.samples.len(), if traced { 2000 * 224 } else { 0 });
    }
}

#[test]
fn m15s_receive_only_read_tail_fault_preserves_act_and_ram() {
    use hp67emu::{emulation::Drive, machines::hp67::{Hp67Driver, Hp67Net}};
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for cycle in 0..2000 {
        if cycle == 500 { live.set_key_contact(Some(Hp67Key::Rcl)); }
        if cycle == 700 { live.set_key_contact(None); }
        if cycle == 1200 { live.set_key_contact(Some(Hp67Key::Digit0)); }
        if cycle == 1400 { live.set_key_contact(None); }
        live.step_firmware_cycle_with_execution().unwrap();
        if live.pending_ram_data_transfer.is_some_and(|transfer| transfer.direction == Hp67DataTransferDirection::PeripheralToAct) {
            let before = live.machine.act.state.c;
            let ram = live.machine.ram.clone();
            let cycle = live.boot_cycle;
            let executed = live.machine.executed_words();
            live.backplane.drive(Hp67Net::Data, Hp67Driver::External, Drive::High);
            let error = live.step_firmware_cycle_with_execution().unwrap_err();
            assert!(error.contains("clocked RAM DATA prefix failed"), "{error}");
            assert_eq!(live.machine.act.state.c, before);
            assert_eq!(live.machine.ram, ram);
            assert_eq!(live.boot_cycle, cycle);
            assert_eq!(live.machine.executed_words(), executed);
            assert!(live.pending_ram_data_transfer.is_some());
            assert!(live.data_serial.frame_in_progress());
            for address in 0..64 {
                assert_eq!(live.serial_ram.as_ref().unwrap().register(address).copied(), ram.read(address));
            }
            return;
        }
    }
    panic!("firmware did not produce an installed RAM read");
}
