use super::*;
use hp67emu::emulation::Drive;
use hp67emu::machines::hp67::{Hp67Driver, Hp67Net};

#[test]
fn m16b_live_clock_fault_precedes_inputs_oracle_and_owned_state_mutation() {
    for net in [Hp67Net::Phi1, Hp67Net::Phi2] {
        for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::Act1820_2530) {
            let mut live = Hp67LiveMachine::power_on_default().unwrap();
            live.set_key_contact(Some(Hp67Key::Digit7));
            live.backplane.drive(net, driver, Drive::High);
            let before = (live.act_serial, live.fetch_rom, live.display_rom0, live.data_serial,
                live.serial_ram.clone(), live.owned_crc_data.clone(), live.machine.act.state.clone(),
                live.machine.crc.clone(), live.boot_cycle);
            let mut trace = StructuralBusTrace::default();
            let error = live.step_firmware_cycle_with_trace(Some(&mut trace)).unwrap_err();
            assert!(error.contains("invalid PHI image"));
            assert!(error.contains(&format!("{net:?}")));
            assert_eq!((live.act_serial, live.fetch_rom, live.display_rom0, live.data_serial,
                live.serial_ram.clone(), live.owned_crc_data.clone(), live.machine.act.state.clone(),
                live.machine.crc.clone(), live.boot_cycle), before);
            assert_eq!(live.backplane.tick().get(), 0);
            assert!(trace.samples.is_empty());
            live.backplane.drive(net, driver, Drive::HighZ);
            live.step_firmware_cycle_with_execution().unwrap();
            assert_eq!(live.backplane.tick().get(), 224);
        }
    }
}

#[test]
fn m16b_live_clock_fault_preserves_pending_ram_tail_and_retry_matches_unfaulted_firmware() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    let mut control = Hp67LiveMachine::power_on_default().unwrap();
    for cycle in 0..2400 {
        assert_eq!(live.data_serial.frame_in_progress(), control.data_serial.frame_in_progress());
        if live.data_serial.frame_in_progress() {
            let before = (live.act_serial, live.data_serial, live.serial_ram.clone(),
                live.machine.act.state.clone(), live.machine.crc.clone(), live.boot_cycle);
            let tick = live.backplane.tick();
            live.backplane.drive(Hp67Net::Phi1, Hp67Driver::External, Drive::High);
            assert!(live.step_firmware_cycle_with_execution().unwrap_err().contains("invalid PHI image"));
            assert_eq!(live.backplane.tick(), tick);
            assert_eq!((live.act_serial, live.data_serial, live.serial_ram.clone(),
                live.machine.act.state.clone(), live.machine.crc.clone(), live.boot_cycle), before);
            live.backplane.drive(Hp67Net::Phi1, Hp67Driver::External, Drive::HighZ);
            for _ in 0..32 {
                assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), control.step_firmware_cycle_with_execution().unwrap());
                assert_eq!(live.machine.act.state, control.machine.act.state);
                assert_eq!(live.serial_ram, control.serial_ram);
                assert_eq!(live.machine.crc, control.machine.crc);
                assert_eq!(live.backplane.tick(), control.backplane.tick());
            }
            return;
        }
        assert_eq!(live.step_firmware_cycle_with_execution().unwrap(), control.step_firmware_cycle_with_execution().unwrap(), "cycle {cycle}");
    }
    panic!("real firmware must exercise a pending RAM DATA tail");
}
