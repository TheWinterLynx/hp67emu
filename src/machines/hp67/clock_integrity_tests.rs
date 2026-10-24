use super::*;
use crate::emulation::{Drive, LogicLevel};

fn pins(bus: &Hp67ElectricalBackplane) -> ([LogicLevel; Hp67Net::COUNT], [[Drive; Hp67Driver::COUNT]; Hp67Net::COUNT]) {
    (Hp67Net::ALL.map(|net| bus.level(net)), Hp67Net::ALL.map(|net| Hp67Driver::ALL.map(|driver| bus.driver_drive(net, driver))))
}

struct Rom;
impl Hp67RomWordSource for Rom {
    fn read_word(&self, _: u16) -> Option<u16> { Some(0x155) }
}

#[test]
fn m16b_all_clock_phases_reject_every_foreign_driver_without_advancing() {
    for phase in 0..4 {
        for net in [Hp67Net::Phi1, Hp67Net::Phi2] {
            for driver in Hp67Driver::ALL.into_iter().filter(|d| *d != Hp67Driver::Act1820_2530) {
                for drive in [Drive::Low, Drive::High] {
                    let mut bus = Hp67ElectricalBackplane::default();
                    for _ in 0..phase { bus.try_advance_clock_edge().unwrap(); }
                    let expected = bus.level(net);
                    bus.drive(net, driver, drive);
                    let before = pins(&bus);
                    let error = bus.try_advance_clock_edge().unwrap_err();
                    assert_eq!(error.tick, phase);
                    assert_eq!(error.net, net);
                    assert_eq!(error.expected, expected);
                    assert_ne!(error.active_mask & (1 << driver.index()), 0);
                    assert_eq!(pins(&bus), before);
                    assert_eq!(bus.tick().get(), phase);
                    bus.drive(net, driver, Drive::HighZ);
                    bus.try_advance_clock_edge().unwrap();
                    assert_eq!(bus.tick().get(), phase + 1);
                }
            }
        }
    }
}

#[test]
fn m16b_missing_replaced_and_wrong_phase_act_clocks_are_not_repaired_by_stepping() {
    for phase in 0..4 {
        for net in [Hp67Net::Phi1, Hp67Net::Phi2] {
            for replacement in [false, true] {
                let mut bus = Hp67ElectricalBackplane::default();
                for _ in 0..phase { bus.advance_clock(); }
                let original = bus.driver_drive(net, Hp67Driver::Act1820_2530);
                bus.drive(net, Hp67Driver::Act1820_2530, Drive::HighZ);
                if replacement { bus.drive(net, Hp67Driver::External, original); }
                let before = pins(&bus);
                assert!(bus.try_advance_clock_edge().is_err());
                assert_eq!(bus.tick().get(), phase);
                assert_eq!(pins(&bus), before);
                if replacement { bus.drive(net, Hp67Driver::External, Drive::HighZ); }
                bus.drive(net, Hp67Driver::Act1820_2530, if original == Drive::High { Drive::Low } else { Drive::High });
                let error = bus.validate_clock().unwrap_err();
                assert_ne!(error.actual, error.expected);
                bus.drive(net, Hp67Driver::Act1820_2530, original);
                assert!(bus.validate_clock().is_ok());
            }
        }
    }
}

#[test]
fn m16b_staged_clock_fault_appears_only_after_commit_and_other_nets_stay_independent() {
    let mut bus = Hp67ElectricalBackplane::default();
    let owner = bus.claim_driver_owner(Hp67Driver::External).unwrap();
    {
        let (_, mut stage) = bus.begin_evaluation().unwrap();
        stage.stage_drive(&owner, Hp67Net::Phi2, Drive::High);
    }
    assert!(bus.validate_clock().is_ok());
    bus.commit_staged().unwrap(); // agreeing drives are electrically High, not contention.
    assert_eq!(bus.level(Hp67Net::Phi2), LogicLevel::High);
    assert!(bus.validate_clock().is_err());
    bus.stage_drive(&owner, Hp67Net::Phi2, Drive::HighZ);
    bus.commit_staged().unwrap();
    bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::Low);
    assert!(bus.validate_clock().is_ok());
    for _ in 0..224 { bus.try_advance_clock_edge().unwrap(); }
    assert_eq!(bus.word_index(), 1);
    assert_eq!(bus.level(Hp67Net::Data), LogicLevel::Low);
}

#[test]
fn m16b_all_word_compositions_reject_clock_fault_before_endpoint_or_pin_mutation() {
    for lane in 0..6 {
        for net in [Hp67Net::Phi1, Hp67Net::Phi2] {
            let mut bus = Hp67ElectricalBackplane::default();
            let mut act = ActSerialEndpoint::new(0x123);
            let mut rom = RomFetchEndpoint::default();
            let mut rom0 = Rom0DisplayEndpoint::default();
            let mut data = Hp67DataSerialWordPath::default();
            let mut ram = Hp67SerialRam::default();
            let state = ActArchitecturalState::default();
            let mut trace = StructuralBusTrace::default();
            bus.drive(net, Hp67Driver::External, Drive::High);
            let before = (act, rom, rom0, data, ram.clone(), pins(&bus));
            let result = match lane {
                0 => run_structural_fetch_cycle(&mut bus, 0x007, &mut act, &mut rom, &Rom).map_err(StructuralWordError::Fetch),
                1 => run_structural_display_fetch_cycle(&mut bus, 0x007, &state, &mut act, &mut rom, &mut rom0, &Rom).map(|r| r.fetched_word),
                2 => run_structural_display_fetch_cycle_traced(&mut bus, 0x007, &state, &mut act, &mut rom, &mut rom0, &Rom, &mut trace).map(|r| r.fetched_word),
                3 => run_structural_display_fetch_data_phase_cycle_with_ram(&mut bus, 0x007, &state, &mut act, &mut rom, &mut rom0, &Rom, &mut data, Some([7;14]), Some(&mut ram)).map(|r| r.word.fetched_word),
                4 => run_structural_display_fetch_data_phase_cycle_traced_with_ram(&mut bus, 0x007, &state, &mut act, &mut rom, &mut rom0, &Rom, &mut data, Some([7;14]), &mut trace, Some(&mut ram)).map(|r| r.word.fetched_word),
                _ => run_structural_data_tail_prefix_with_ram(&mut bus, &state, &mut act, &rom, &mut rom0, &mut data, Some(&mut trace), Some(&mut ram)).map(|_| 0),
            };
            assert!(matches!(result, Err(StructuralWordError::Fetch(SerialFetchError::InvalidClock(_)))));
            assert_eq!((act, rom, rom0, data, ram, pins(&bus)), before);
            assert_eq!(bus.tick().get(), 0);
            assert!(trace.samples.is_empty());
        }
    }
}

#[test]
fn m16b_receivers_reject_agreeing_foreign_clocks_before_consuming_inputs() {
    for bit in [0, 46] {
        let mut bus = Hp67ElectricalBackplane::default();
        let mut ram = Hp67SerialRam::default();
        ram.begin_cycle(&bus).unwrap();
        for _ in 0..(bit * 4 + 2) { bus.advance_clock(); }
        bus.drive(Hp67Net::Phi2, Hp67Driver::External, Drive::High);
        bus.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::High);
        let mut act = ActSerialEndpoint::new(0);
        let mut rom0 = Rom0DisplayEndpoint::default();
        let mut command = Hp67RamCommandReceiver::default();
        let mut crc = Hp67SerialCrcControl::default();
        let mut data = Hp67DataSerialWordPath::default();
        let before = (act, rom0, command, crc, data, ram.clone());
        assert!(act.sample_instruction_at_edge(&bus).is_err());
        assert!(rom0.sample_data_at_edge(&bus).is_err());
        assert!(command.sample_at_edge(&bus).is_err());
        assert!(crc.sample_at_edge(&bus).is_err());
        assert!(data.sample_transport_word_bit_at_edge(bit, &bus).is_err());
        assert!(ram.sample_at_edge(&bus).is_err());
        assert_eq!((act, rom0, command, crc, data, ram), before);
    }
}

#[test]
fn m16b_checked_and_legacy_clock_traversals_have_identical_committed_images() {
    let mut checked = Hp67ElectricalBackplane::default();
    let mut baseline = checked.clone();
    for _ in 0..224 * 15 {
        assert_eq!(checked.try_advance_clock_edge().unwrap(), baseline.advance_clock_edge());
        assert_eq!(checked.word_bit(), baseline.word_bit());
        assert_eq!(checked.word_index(), baseline.word_index());
        assert_eq!(pins(&checked), pins(&baseline));
    }
}

#[cfg(feature = "pin-capture")]
#[test]
fn m16b_failed_checked_edge_records_no_phantom_transition_or_publication() {
    let mut bus = Hp67ElectricalBackplane::default();
    bus.start_pin_capture("M16B clock fault fixture".into()).unwrap();
    for _ in 0..2 { bus.try_advance_clock_edge().unwrap(); }
    bus.drive(Hp67Net::Phi1, Hp67Driver::External, Drive::High);
    let before = bus.clone().take_pin_capture().unwrap().to_tsv().unwrap();
    assert!(bus.try_advance_clock_edge().is_err());
    let after = bus.take_pin_capture().unwrap().to_tsv().unwrap();
    assert_eq!(after, before);
    assert_eq!(bus.tick().get(), 2);
}
