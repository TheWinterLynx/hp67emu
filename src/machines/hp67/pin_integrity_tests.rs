use super::*;
use crate::{
    emulation::Drive,
    machines::hp67::{
        isa::{act_address_drive, rom_word_drive},
        pin_trace::{import_pin_trace, PinSource, PinTimeDomain},
        rom_devices::RomIsDrives,
        Hp67ElectricalBackplane,
    },
};
fn snapshot(bus: &Hp67ElectricalBackplane, order: u32) -> PinRow {
    PinRow {
        at: bus.tick().get(),
        order,
        levels: std::array::from_fn(|i| Some(bus.level(Hp67Net::ALL[i]))),
        owners: std::array::from_fn(|i| {
            let mut owner = PinOwners { high: 0, low: 0 };
            for driver in Hp67Driver::ALL {
                match bus.driver_drive(Hp67Net::ALL[i], driver) {
                    Drive::High => owner.high |= mask(driver),
                    Drive::Low => owner.low |= mask(driver),
                    Drive::HighZ => {}
                }
            }
            Some(owner)
        }),
    }
}
fn fixture(words: usize) -> Hp67PinTrace {
    let mut bus = Hp67ElectricalBackplane::default();
    for net in Hp67Net::ALL {
        if let Some(driver) = sole_source(net) {
            bus.drive(net, driver, Drive::High);
        }
    }
    let mut trace =
        Hp67PinTrace::emulator(0, "synthetic digital bridge, not measured silicon".into()).unwrap();
    trace.rows.push(snapshot(&bus, 0));
    for _ in 0..words {
        for bit in 0..56 {
            let mut rom = [Drive::HighZ; 5];
            rom[0] = rom_word_drive(0x3ff, bit);
            bus.commit_serial_launch(
                if bit < 8 {
                    Drive::High
                } else {
                    act_address_drive(0x123, bit)
                },
                RomIsDrives::Chips(rom),
                if bit >= 46 { Drive::High } else { Drive::Low },
                None,
            );
            trace.rows.push(snapshot(&bus, 1));
            for _ in 0..4 {
                bus.advance_clock();
                trace.rows.push(snapshot(&bus, 0));
            }
        }
    }
    trace
}
fn sample_mut(trace: &mut Hp67PinTrace, bit: u8) -> &mut PinRow {
    trace
        .rows
        .iter_mut()
        .find(|r| r.at == u64::from(bit) * 4 + 2)
        .unwrap()
}
#[test]
fn m16l_integrity_checks_production_contracts_and_reports_unassessed_pins_without_claiming_timing()
{
    let trace = fixture(2);
    let before = trace.clone();
    let audit = audit_pin_integrity(&trace).unwrap();
    assert_eq!(audit.exit_code(), 0);
    assert_eq!(audit.cells, 112);
    assert_eq!(audit.pins.iter().filter(|p| p.assessed).count(), 13);
    for net in [
        Hp67Net::F1,
        Hp67Net::Kc1,
        Hp67Net::Kc2,
        Hp67Net::Kc3,
        Hp67Net::Kc4,
        Hp67Net::Kc5,
    ] {
        assert!(!audit.pins[net.index()].assessed);
    }
    let report = audit.report(&trace);
    assert!(report.contains("SOURCE-BLOCKED"));
    assert!(report.contains("F1/KC acquisition is UNASSESSED"));
    assert!(report.contains("ROM bank identity is NOT certified"));
    assert_eq!(trace, before);
    assert_eq!(
        audit_pin_integrity(&import_pin_trace(&trace.to_tsv().unwrap()).unwrap()).unwrap(),
        audit
    );
}
#[test]
fn m16l_integrity_rejects_all_agreeing_foreign_and_replacement_sources_on_sole_owner_pins() {
    for net in Hp67Net::ALL {
        let Some(expected) = sole_source(net) else {
            continue;
        };
        for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != expected) {
            for replace in [false, true] {
                let mut trace = fixture(1);
                let row = sample_mut(&mut trace, 16);
                let owners = row.owners[net.index()].as_mut().unwrap();
                // Match the actual level: a level-only audit cannot detect this.
                let active = if row.levels[net.index()] == Some(LogicLevel::High) {
                    &mut owners.high
                } else {
                    &mut owners.low
                };
                if replace {
                    *active = mask(driver);
                } else {
                    *active |= mask(driver);
                }
                let audit = audit_pin_integrity(&trace).unwrap();
                assert_eq!(
                    audit.exit_code(),
                    2,
                    "{net:?} {driver:?} replacement={replace}"
                );
                assert!(audit.pins[net.index()].violations > 0);
            }
        }
    }
}
#[test]
fn m16l_integrity_is_roles_passive_release_page_and_aggregate_reference_are_distinct() {
    for (bit, driver) in [
        (0, Hp67Driver::RomDisplay1818_0268),
        (8, Hp67Driver::Act1820_2530),
        (16, Hp67Driver::External),
        (46, Hp67Driver::RomRam1818_0550),
        (55, Hp67Driver::StructuralRomResponder),
    ] {
        let mut trace = fixture(1);
        let row = sample_mut(&mut trace, bit);
        row.levels[Hp67Net::Isa.index()] = Some(LogicLevel::High);
        row.owners[Hp67Net::Isa.index()] = Some(PinOwners {
            high: mask(driver),
            low: 0,
        });
        assert_eq!(audit_pin_integrity(&trace).unwrap().exit_code(), 2);
    }
    let mut trace = fixture(1);
    let row = sample_mut(&mut trace, 46);
    row.levels[Hp67Net::Isa.index()] = Some(LogicLevel::Low);
    row.owners[Hp67Net::Isa.index()] = Some(PinOwners { high: 0, low: 0 });
    assert_eq!(audit_pin_integrity(&trace).unwrap().exit_code(), 0);
    row_reset_to_active_low(&mut trace);
    assert_eq!(audit_pin_integrity(&trace).unwrap().exit_code(), 2);
}
fn row_reset_to_active_low(trace: &mut Hp67PinTrace) {
    sample_mut(trace, 46).owners[Hp67Net::Isa.index()] = Some(PinOwners {
        high: 0,
        low: mask(Hp67Driver::RomDisplay1818_0268),
    });
}
#[test]
fn m16l_integrity_data_rejects_agreeing_overlap_and_foreign_sources_without_guessing_direction() {
    for drivers in [
        mask(Hp67Driver::External),
        mask(Hp67Driver::StructuralRomResponder),
        mask(Hp67Driver::Act1820_2530) | mask(Hp67Driver::RamAggregate),
    ] {
        let mut trace = fixture(1);
        let row = sample_mut(&mut trace, 2);
        row.levels[Hp67Net::Data.index()] = Some(LogicLevel::High);
        row.owners[Hp67Net::Data.index()] = Some(PinOwners {
            high: drivers,
            low: 0,
        });
        assert_eq!(audit_pin_integrity(&trace).unwrap().exit_code(), 2);
    }
    for driver in [Hp67Driver::Act1820_2530, Hp67Driver::RamAggregate] {
        let mut trace = fixture(1);
        let row = sample_mut(&mut trace, 2);
        row.levels[Hp67Net::Data.index()] = Some(LogicLevel::Low);
        row.owners[Hp67Net::Data.index()] = Some(PinOwners {
            high: 0,
            low: mask(driver),
        });
        assert_eq!(audit_pin_integrity(&trace).unwrap().exit_code(), 0);
    }
}
#[test]
fn m16l_integrity_absent_contacts_owners_address_and_partial_capture_cannot_pass() {
    for net in [
        Hp67Net::Motor,
        Hp67Net::Mtrs,
        Hp67Net::Hds,
        Hp67Net::Wps,
        Hp67Net::PgmSw,
        Hp67Net::Isa,
    ] {
        let mut trace = fixture(1);
        for row in &mut trace.rows {
            row.levels[net.index()] = None;
            row.owners[net.index()] = None;
        }
        let audit = audit_pin_integrity(&trace).unwrap();
        assert_eq!(audit.exit_code(), 3);
        assert!(audit.pins[net.index()].missing_levels > 0);
    }
    let mut trace = fixture(1);
    trace.rows.pop();
    assert_eq!(audit_pin_integrity(&trace).unwrap().exit_code(), 3);
    let mut trace = fixture(1);
    for (i, row) in trace.rows.iter_mut().enumerate() {
        row.at = i as u64 * 1000;
        row.order = 0;
        row.owners = [None; Hp67Net::COUNT];
    }
    trace.domain = PinTimeDomain::Nanoseconds;
    trace.source = PinSource::Measured;
    let audit = audit_pin_integrity(&trace).unwrap();
    assert_eq!(audit.exit_code(), 3);
    assert!(audit
        .pins
        .iter()
        .filter(|p| p.assessed)
        .all(|p| p.missing_owners > 0));
}
#[test]
fn m16l_integrity_validates_direct_images_and_bounds_diagnostics_without_losing_fault_counts() {
    let mut malformed = fixture(1);
    sample_mut(&mut malformed, 0).owners[Hp67Net::F2.index()]
        .as_mut()
        .unwrap()
        .low = mask(Hp67Driver::CardReaderController1820_1751);
    assert!(audit_pin_integrity(&malformed).is_err());
    let mut trace = fixture(2);
    for row in &mut trace.rows {
        row.owners[Hp67Net::F2.index()] = Some(PinOwners {
            high: mask(Hp67Driver::External),
            low: 0,
        });
    }
    let audit = audit_pin_integrity(&trace).unwrap();
    assert_eq!(audit.exit_code(), 2);
    assert_eq!(audit.pins[Hp67Net::F2.index()].violations, 112);
    assert_eq!(audit.findings.len(), MAX_INTEGRITY_FINDINGS);
    assert!(audit.report(&trace).contains("violations_total=112"));
}
