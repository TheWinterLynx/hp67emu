use super::*;
use crate::machines::hp67::rom_devices::HP67_ROM_PROFILES;
use crate::machines::hp67::{Hp67RamCommandReceiver, Hp67SerialCrcControl};

struct BankRom {
    bank: u8,
    word: u16,
}
impl Hp67RomWordSource for BankRom {
    fn read_word(&self, _: u16) -> Option<u16> {
        Some(self.word)
    }
    fn selected_rom_bank(&self, _: u16) -> u8 {
        self.bank
    }
}
fn selected_rom(address: u16, bank: u8, physical: bool) -> RomFetchEndpoint {
    let mut rom = if physical {
        RomFetchEndpoint::physical_roms()
    } else {
        RomFetchEndpoint::default()
    };
    for bit in 16..=27 {
        let level = if address & (1 << (bit - 16)) != 0 {
            LogicLevel::High
        } else {
            LogicLevel::Low
        };
        rom.sample_for_bit(bit, level, &BankRom { bank, word: 0 })
            .unwrap();
    }
    rom
}
fn sample_bus(bit: u8) -> Hp67ElectricalBackplane {
    let mut bus = Hp67ElectricalBackplane::default();
    for _ in 0..u64::from(bit) * 4 + 2 {
        bus.advance_clock();
    }
    bus
}

#[test]
fn m16f_every_window_rejects_every_wrong_sole_high_source_for_all_rom_profiles() {
    for profile in HP67_ROM_PROFILES {
        let rom = selected_rom(profile.first_address + 0x123, profile.bank, true);
        assert_eq!(rom.return_driver(), Some(profile.driver));
        for bit in 0..56 {
            let base = sample_bus(bit);
            assert_eq!(
                sample_transport_owned_isa_at_edge(&base, bit, rom.return_driver()),
                Ok(LogicLevel::Low)
            );
            // Independent literal windows, not the source validator's dispatch.
            let expected = match bit {
                0..=7 | 16..=27 => Some(Hp67Driver::Act1820_2530),
                46..=55 => Some(profile.driver),
                _ => None,
            };
            for driver in Hp67Driver::ALL {
                let mut bus = base.clone();
                bus.drive(Hp67Net::Isa, driver, Drive::High);
                let before = (rom, bus.tick(), bus.active_isa_driver_mask());
                let result = sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver());
                if expected == Some(driver) {
                    assert_eq!(result, Ok(LogicLevel::High));
                } else {
                    assert_eq!(
                        result,
                        Err(SerialFetchError::UnexpectedIsaSource {
                            word_bit: bit,
                            expected,
                            driver_mask: 1 << driver.index(),
                        })
                    );
                }
                assert_eq!((rom, bus.tick(), bus.active_isa_driver_mask()), before);
            }
        }
    }
}

#[test]
fn m16f_aggregate_reference_return_is_distinct_from_physical_rom_identity() {
    let aggregate = selected_rom(0x123, 0, false);
    let physical = selected_rom(0x123, 0, true);
    for bit in 46..=55 {
        for (rom, permitted) in [
            (aggregate, Hp67Driver::StructuralRomResponder),
            (physical, Hp67Driver::RomDisplay1818_0268),
        ] {
            for driver in [
                Hp67Driver::StructuralRomResponder,
                Hp67Driver::RomDisplay1818_0268,
            ] {
                let mut bus = sample_bus(bit);
                bus.drive(Hp67Net::Isa, driver, Drive::High);
                assert_eq!(
                    sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver()).is_ok(),
                    driver == permitted
                );
            }
        }
    }
}

#[test]
fn m16f_polarity_overlap_and_phase_faults_keep_the_existing_error_priority() {
    let rom = selected_rom(0x123, 0, true);
    for bit in 0..56 {
        for driver in Hp67Driver::ALL {
            let mut bus = sample_bus(bit);
            bus.drive(Hp67Net::Isa, driver, Drive::Low);
            assert!(matches!(
                sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver()),
                Err(SerialFetchError::InvalidIsaDrives { .. })
            ));
        }
        let mut bus = sample_bus(bit);
        bus.drive(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::High);
        bus.drive(Hp67Net::Isa, Hp67Driver::External, Drive::High);
        assert!(matches!(
            sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver()),
            Err(SerialFetchError::InvalidIsaDrives { .. })
        ));
        bus.advance_clock();
        assert!(matches!(
            sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver()),
            Err(SerialFetchError::InvalidSamplePhase { .. })
        ));
    }
}

#[test]
fn m16f_correct_owner_payload_changes_are_received_from_pins_not_rom_preview() {
    for profile in HP67_ROM_PROFILES {
        let rom = selected_rom(profile.first_address, profile.bank, true);
        let mut act = ActSerialEndpoint::new(profile.first_address);
        let mut command = Hp67RamCommandReceiver::default();
        let mut crc = Hp67SerialCrcControl::default();
        let mut bus = Hp67ElectricalBackplane::default();
        for bit in 0..56 {
            bus.drive(
                Hp67Net::Sync,
                Hp67Driver::Act1820_2530,
                if bit >= 46 { Drive::High } else { Drive::Low },
            );
            if bit >= 46 {
                bus.drive(Hp67Net::Isa, profile.driver, Drive::High);
            }
            bus.advance_clock();
            bus.advance_clock();
            assert_eq!(
                sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver()).unwrap(),
                if bit >= 46 {
                    LogicLevel::High
                } else {
                    LogicLevel::Low
                }
            );
            if bit >= 46 {
                act.sample_instruction_at_edge(&bus).unwrap();
                command.sample_at_edge(&bus).unwrap();
            }
            crc.sample_at_edge(&bus).unwrap();
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(rom.drive_for_bit(55), Drive::HighZ); // Planned ROM payload is zero.
        assert_eq!(act.fetched_word().unwrap(), 0x3ff); // Resolved source actually sent ones.
        assert_eq!(command.received_frame().unwrap().word, 0x3ff);
        assert_eq!(command.received_frame(), crc.received_frame());
        assert_eq!(bus.tick().get(), 224);
    }
}

#[test]
fn m16f_return_identity_follows_received_address_instead_of_host_request() {
    let mut rom = RomFetchEndpoint::physical_roms();
    let mut bus = Hp67ElectricalBackplane::default();
    let source = BankRom { bank: 0, word: 0 };
    for bit in 0..56 {
        // Address lines deliberately differ from a host request for 000.
        bus.commit_serial_launch(
            if (16..=27).contains(&bit) {
                Drive::High
            } else {
                Drive::HighZ
            },
            rom.is_drives_for_bit(bit),
            if bit >= 46 { Drive::High } else { Drive::Low },
            None,
        );
        bus.advance_clock();
        bus.advance_clock();
        let level = sample_transport_owned_isa_at_edge(&bus, bit, rom.return_driver()).unwrap();
        if (16..=27).contains(&bit) {
            rom.sample_for_bit(bit, level, &source).unwrap();
        }
        bus.advance_clock();
        bus.advance_clock();
    }
    assert_eq!(rom.received_address().unwrap(), 0xfff);
    assert_eq!(rom.return_driver(), Some(Hp67Driver::RomRam1818_0232));
    assert_eq!(bus.tick().get(), 224);
}

#[test]
fn m16f_prefix_rejects_foreign_high_before_display_or_data_consumes_it() {
    let mut bus = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(0);
    let rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut data = Hp67DataSerialWordPath::default();
    data.enable_electrical_level_bridge(None);
    let before = data;
    bus.drive(Hp67Net::Isa, Hp67Driver::External, Drive::High);
    let mut trace = StructuralBusTrace::default();
    let error = run_structural_data_tail_prefix(
        &mut bus,
        &ActArchitecturalState::default(),
        &mut act,
        &rom,
        &mut rom0,
        &mut data,
        Some(&mut trace),
    )
    .unwrap_err();
    assert!(matches!(
        error,
        StructuralWordError::Fetch(SerialFetchError::UnexpectedIsaSource { word_bit: 0, .. })
    ));
    assert_eq!(data, before);
    assert!(rom0.display_byte().is_err());
    assert_eq!(bus.tick().get(), 2);
    assert_eq!(trace.samples.len(), 2);
}

#[test]
fn m16f_autonomous_bank_roundtrip_selects_return_owner_from_live_rom_latches() {
    use crate::machines::hp67::hp67firmware::Hp67Firmware;
    let source = Hp67Firmware::default();
    let mut bus = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(0);
    let mut rom = RomFetchEndpoint::autonomous_roms();
    for (address, driver) in [
        (0x7f4, Hp67Driver::RomRam1818_0550),
        (0x7f5, Hp67Driver::RomRam1818_0231),
        (0x495, Hp67Driver::RomRam1818_0231),
        (0x11a, Hp67Driver::RomDisplay1818_0268),
    ] {
        run_structural_fetch_cycle(&mut bus, address, &mut act, &mut rom, &source).unwrap();
        assert_eq!(rom.return_driver(), Some(driver));
    }
    assert_eq!(bus.tick().get(), 4 * 224);
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16f_pin_capture_retains_foreign_is_evidence_and_exact_failure_tick() {
    let mut bus = Hp67ElectricalBackplane::default();
    bus.start_pin_capture("M16F foreign IS source".into())
        .unwrap();
    bus.drive(Hp67Net::Isa, Hp67Driver::External, Drive::High);
    let mut act = ActSerialEndpoint::new(0);
    let mut rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    assert!(run_structural_display_fetch_cycle(
        &mut bus,
        0,
        &ActArchitecturalState::default(),
        &mut act,
        &mut rom,
        &mut rom0,
        &BankRom { bank: 0, word: 0 }
    )
    .is_err());
    let capture = bus.take_pin_capture().unwrap();
    let last = capture.rows.last().unwrap();
    assert_eq!(last.at, 2);
    assert_eq!(last.levels[Hp67Net::Isa.index()], Some(LogicLevel::High));
    assert_eq!(
        last.owners[Hp67Net::Isa.index()].unwrap().high,
        1 << Hp67Driver::External.index()
    );
    let edges = capture
        .rows
        .windows(2)
        .filter(|w| {
            w[0].levels[Hp67Net::Phi1.index()] != w[1].levels[Hp67Net::Phi1.index()]
                || w[0].levels[Hp67Net::Phi2.index()] != w[1].levels[Hp67Net::Phi2.index()]
        })
        .count();
    assert_eq!(edges, 2);
}
