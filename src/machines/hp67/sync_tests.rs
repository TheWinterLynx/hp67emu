use super::super::{
    isa::{act_address_drive, rom_word_drive},
    rom_devices::RomIsDrives,
    ActSerialEndpoint, Hp67RamCommandReceiver, Hp67SerialCrcControl,
};
use super::*;

fn pins(
    bus: &Hp67ElectricalBackplane,
) -> (
    [LogicLevel; Hp67Net::COUNT],
    [[Drive; Hp67Driver::COUNT]; Hp67Net::COUNT],
) {
    (
        Hp67Net::ALL.map(|net| bus.level(net)),
        Hp67Net::ALL.map(|net| Hp67Driver::ALL.map(|driver| bus.driver_drive(net, driver))),
    )
}

#[test]
fn m16e_pristine_entry_is_not_a_valid_receiver_source_or_a_later_repair() {
    let mut bus = Hp67ElectricalBackplane::default();
    assert!(validate_sync_transport_entry(&bus).is_ok());
    assert!(validate_sync_source(&bus).is_err());
    bus.advance_clock();
    assert!(validate_sync_transport_entry(&bus).is_err());
    for drive in [Drive::Low, Drive::High] {
        bus.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, drive);
        assert!(validate_sync_transport_entry(&bus).is_ok());
        assert!(validate_sync_source(&bus).is_ok());
    }
}

#[test]
fn m16e_every_foreign_sync_source_is_rejected_at_all_phases_and_both_levels() {
    for phase in 0..4 {
        for act_drive in [Drive::Low, Drive::High] {
            for driver in Hp67Driver::ALL
                .into_iter()
                .filter(|d| *d != Hp67Driver::Act1820_2530)
            {
                for foreign in [Drive::Low, Drive::High] {
                    for replace in [false, true] {
                        let mut bus = Hp67ElectricalBackplane::default();
                        bus.drive(
                            Hp67Net::Sync,
                            Hp67Driver::Act1820_2530,
                            if replace { Drive::HighZ } else { act_drive },
                        );
                        for _ in 0..phase {
                            bus.advance_clock();
                        }
                        bus.drive(Hp67Net::Sync, driver, foreign);
                        let before = pins(&bus);
                        let error = validate_sync_source(&bus).unwrap_err();
                        assert_eq!(error.tick, phase);
                        assert_eq!(
                            error.active_mask & (1 << driver.index()),
                            1 << driver.index()
                        );
                        assert!(validate_sync_transport_entry(&bus).is_err());
                        assert_eq!(pins(&bus), before);
                        assert_eq!(bus.tick().get(), phase);
                    }
                }
            }
        }
    }
}

#[test]
fn m16e_act_ram_crc_reject_faults_at_each_return_bit_then_retry_same_sample() {
    for instruction in [false, true] {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0);
        let mut ram = Hp67RamCommandReceiver::default();
        let mut crc = Hp67SerialCrcControl::default();
        let mut storage = Hp67SerialRam::default();
        storage.begin_cycle(&bus).unwrap();
        for bit in 0..56 {
            if bit == 2 {
                storage.bind_body(&bus).unwrap();
            }
            let sync = if bit >= 46 && instruction {
                Drive::High
            } else {
                Drive::Low
            };
            bus.commit_serial_launch(
                act_address_drive(0, bit),
                RomIsDrives::Aggregate(rom_word_drive(0x230, bit)),
                sync,
                None,
            );
            bus.advance_clock();
            bus.advance_clock();
            if bit >= 46 {
                for driver in Hp67Driver::ALL
                    .into_iter()
                    .filter(|d| *d != Hp67Driver::Act1820_2530)
                {
                    for drive in [Drive::Low, Drive::High] {
                        for replace in [false, true] {
                            let mut fault = bus.clone();
                            if replace {
                                fault.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::HighZ);
                            }
                            fault.drive(Hp67Net::Sync, driver, drive);
                            let before_pins = pins(&fault);
                            let before = (act, ram, crc, storage.clone());
                            assert!(act.sample_instruction_at_edge(&fault).is_err());
                            assert!(ram.sample_at_edge(&fault).is_err());
                            assert!(crc.sample_at_edge(&fault).is_err());
                            assert!(storage.sample_at_edge(&fault).is_err());
                            assert_eq!((act, ram, crc, storage.clone()), before);
                            assert_eq!(pins(&fault), before_pins);
                        }
                    }
                }
                let mut missing = bus.clone();
                missing.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::HighZ);
                let before = (act, ram, crc, storage.clone());
                assert!(act.sample_instruction_at_edge(&missing).is_err());
                assert!(ram.sample_at_edge(&missing).is_err());
                assert!(crc.sample_at_edge(&missing).is_err());
                assert!(storage.sample_at_edge(&missing).is_err());
                assert_eq!((act, ram, crc, storage.clone()), before);
                act.sample_instruction_at_edge(&bus).unwrap();
                ram.sample_at_edge(&bus).unwrap();
            }
            crc.sample_at_edge(&bus).unwrap();
            storage.sample_at_edge(&bus).unwrap();
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(act.fetched_word().unwrap(), 0x230);
        assert_eq!(ram.received_frame(), crc.received_frame());
        let received = ram.received_frame().unwrap();
        assert_eq!((received.word, received.instruction), (0x230, instruction));
        assert_eq!(bus.tick().get(), 224);
    }
}

#[test]
fn m16e_staged_sync_fault_is_visible_only_after_commit_and_recovery_is_read_only() {
    let mut bus = Hp67ElectricalBackplane::default();
    bus.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::Low);
    let owner = bus.claim_driver_owner(Hp67Driver::External).unwrap();
    bus.stage_drive(&owner, Hp67Net::Sync, Drive::Low);
    assert!(validate_sync_source(&bus).is_ok());
    bus.commit_staged().unwrap();
    assert_eq!(bus.level(Hp67Net::Sync), LogicLevel::Low);
    let before = pins(&bus);
    assert!(validate_sync_source(&bus).is_err());
    assert_eq!(pins(&bus), before);
    bus.stage_drive(&owner, Hp67Net::Sync, Drive::HighZ);
    bus.commit_staged().unwrap();
    assert!(validate_sync_source(&bus).is_ok());
    assert_eq!(bus.tick().get(), 0);
}

use super::super::*;

struct Rom;
impl Hp67RomWordSource for Rom {
    fn read_word(&self, _: u16) -> Option<u16> {
        Some(0x155)
    }
}

#[test]
fn m16e_all_word_compositions_reject_sync_fault_before_endpoint_or_pin_mutation() {
    for lane in 0..6 {
        for drive in [Drive::Low, Drive::High] {
            let net = Hp67Net::Sync;
            let mut bus = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0x123);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            let mut ram = Hp67SerialRam::default();
            let state = ActArchitecturalState::default();
            let mut trace = StructuralBusTrace::default();
            bus.drive(net, Hp67Driver::External, drive);
            let before = (act, rom, rom0, data, ram.clone(), pins(&bus));
            let result = match lane {
                0 => run_structural_fetch_cycle(&mut bus, 0x007, &mut act, &mut rom, &Rom)
                    .map_err(StructuralWordError::Fetch),
                1 => run_structural_display_fetch_cycle(
                    &mut bus, 0x007, &state, &mut act, &mut rom, &mut rom0, &Rom,
                )
                .map(|r| r.fetched_word),
                2 => run_structural_display_fetch_cycle_traced(
                    &mut bus, 0x007, &state, &mut act, &mut rom, &mut rom0, &Rom, &mut trace,
                )
                .map(|r| r.fetched_word),
                3 => run_structural_display_fetch_data_phase_cycle_with_ram(
                    &mut bus,
                    0x007,
                    &state,
                    &mut act,
                    &mut rom,
                    &mut rom0,
                    &Rom,
                    &mut data,
                    Some([7; 14]),
                    Some(&mut ram),
                )
                .map(|r| r.word.fetched_word),
                4 => run_structural_display_fetch_data_phase_cycle_traced_with_ram(
                    &mut bus,
                    0x007,
                    &state,
                    &mut act,
                    &mut rom,
                    &mut rom0,
                    &Rom,
                    &mut data,
                    Some([7; 14]),
                    &mut trace,
                    Some(&mut ram),
                )
                .map(|r| r.word.fetched_word),
                _ => run_structural_data_tail_prefix_with_ram(
                    &mut bus,
                    &state,
                    &mut act,
                    &rom,
                    &mut rom0,
                    &mut data,
                    Some(&mut trace),
                    Some(&mut ram),
                )
                .map(|_| 0),
            };
            assert!(matches!(
                result,
                Err(StructuralWordError::Fetch(
                    SerialFetchError::InvalidSyncSource(_)
                ))
            ));
            assert_eq!((act, rom, rom0, data, ram, pins(&bus)), before);
            assert_eq!(bus.tick().get(), 0);
            assert!(trace.samples.is_empty());
        }
    }
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16e_rejected_transport_does_not_record_a_phantom_source_or_clock() {
    let mut bus = Hp67ElectricalBackplane::default();
    bus.start_pin_capture("M16E SYNC fault".into()).unwrap();
    bus.drive(Hp67Net::Sync, Hp67Driver::External, Drive::Low);
    let before = bus.clone().take_pin_capture().unwrap().to_tsv().unwrap();
    let mut act = ActSerialEndpoint::new(0);
    let mut rom = RomFetchEndpoint::default();
    assert!(run_structural_fetch_cycle(&mut bus, 0, &mut act, &mut rom, &Rom).is_err());
    assert_eq!(bus.take_pin_capture().unwrap().to_tsv().unwrap(), before);
    assert_eq!(bus.tick().get(), 0);
}
