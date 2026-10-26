use super::*;
use hp67emu::{
    emulation::Drive,
    machines::hp67::{Hp67Driver, Hp67Net},
};

fn ram_storage(live: &Hp67LiveMachine) -> (u8, [hp67emu::machines::hp67::ActRegister; 64]) {
    let ram = live.serial_ram.as_ref().unwrap();
    (
        ram.address(),
        std::array::from_fn(|i| *ram.register(i as u8).unwrap()),
    )
}

#[test]
fn m16f_live_foreign_sole_is_source_stops_at_first_sample_before_receivers() {
    // These foreign sources survive the normal ACT/five-ROM publication.
    for driver in [
        Hp67Driver::External,
        Hp67Driver::CardReaderController1820_1751,
        Hp67Driver::CardSenseAmplifier1826_0322,
        Hp67Driver::CathodeDriver1820_1749,
        Hp67Driver::DisplayTransistorArray1858_0050,
        Hp67Driver::RamAggregate,
    ] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.backplane.drive(Hp67Net::Isa, driver, Drive::High);
        let mut trace = StructuralBusTrace::default();
        let error = live
            .step_firmware_cycle_with_trace(Some(&mut trace))
            .unwrap_err();
        assert!(error.contains("UnexpectedIsaSource"), "{error}");
        assert_eq!(live.backplane.tick().get(), 2);
        assert_eq!(live.boot_cycle, 0);
        assert!(live.act_serial.fetched_word().is_err());
        assert!(live.fetch_rom.received_address().is_err());
        assert!(live.display_rom0.display_byte().is_err());
        assert_eq!(trace.samples.len(), 2);
        assert_eq!(
            live.backplane.driver_drive(Hp67Net::Isa, driver),
            Drive::High
        );
        // Partial-cycle faults require reset, rather than pretending clocks rolled back.
        live.reset_power_on().unwrap();
        assert_eq!(live.backplane.tick().get(), 0);
        live.step_firmware_cycle_with_execution().unwrap();
        assert_eq!(live.backplane.tick().get(), 224);
    }
}

#[test]
fn m16f_live_foreign_is_does_not_consume_a_pending_ram_tail() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..2400 {
        if live.data_serial.frame_in_progress() {
            let before = (live.data_serial, ram_storage(&live), live.boot_cycle);
            let tick = live.backplane.tick();
            live.backplane
                .drive(Hp67Net::Isa, Hp67Driver::External, Drive::High);
            let error = live.step_firmware_cycle_with_execution().unwrap_err();
            assert!(error.contains("Isa"), "{error}");
            assert_eq!(
                (live.data_serial, ram_storage(&live), live.boot_cycle),
                before
            );
            assert_eq!(live.backplane.tick().get(), tick.get() + 2);
            return;
        }
        live.step_firmware_cycle_with_execution().unwrap();
    }
    panic!("real firmware must exercise a pending RAM tail");
}

#[test]
fn m16f_live_normal_and_traced_firmware_keep_all_word_edges_and_same_state() {
    let mut plain = Hp67LiveMachine::power_on_default().unwrap();
    let mut traced = Hp67LiveMachine::power_on_default().unwrap();
    for cycle in 0..2400 {
        let key = if (1000..1200).contains(&cycle) {
            Some(Hp67Key::Digit7)
        } else {
            None
        };
        plain.set_key_contact(key);
        traced.set_key_contact(key);
        let mut trace = StructuralBusTrace::default();
        let tick = plain.backplane.tick();
        assert_eq!(
            plain.step_firmware_cycle_with_execution().unwrap(),
            traced
                .step_firmware_cycle_with_trace(Some(&mut trace))
                .unwrap(),
            "cycle {cycle}"
        );
        assert_eq!(plain.machine.act.state, traced.machine.act.state);
        assert_eq!(plain.serial_ram, traced.serial_ram);
        assert_eq!(plain.machine.crc, traced.machine.crc);
        assert_eq!(plain.backplane.tick(), traced.backplane.tick());
        assert_eq!(plain.backplane.tick().get(), tick.get() + 224);
        assert_eq!(trace.samples.len(), 224);
    }
}
