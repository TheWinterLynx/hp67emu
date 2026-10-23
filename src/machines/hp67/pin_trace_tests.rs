use super::analysis::PinWord;
use super::*;
use crate::machines::hp67::{
    isa::{act_address_drive, rom_word_drive},
    rom_devices::RomIsDrives,
};

fn fixture(words: usize) -> Hp67PinTrace {
    let mut bus = Hp67ElectricalBackplane::default();
    bus.start_pin_capture("synthetic electrical fixture; not hardware evidence".into())
        .unwrap();
    for word in 0..words {
        for bit in 0..56 {
            let is = if bit < 8 {
                if 8 & (1 << bit) != 0 {
                    Drive::High
                } else {
                    Drive::HighZ
                }
            } else {
                act_address_drive(0x123, bit)
            };
            let data = 0x00fedcba987654u64 & (1 << ((bit + 54) % 56)) != 0;
            bus.commit_serial_launch(
                is,
                RomIsDrives::Aggregate(rom_word_drive(0x2a5, bit)),
                if bit >= 46 && word % 2 == 0 {
                    Drive::High
                } else {
                    Drive::Low
                },
                Some((if data { Drive::High } else { Drive::Low }, Drive::HighZ)),
            );
            bus.advance_clock();
            bus.advance_clock();
            if matches!(bit, 0 | 7 | 8) {
                bus.commit_display_controls(
                    if bit == 7 { Drive::Low } else { Drive::High },
                    if bit == 7 && word % 15 == 14 {
                        Drive::Low
                    } else {
                        Drive::High
                    },
                );
            }
            bus.advance_clock();
            bus.advance_clock();
        }
    }
    bus.take_pin_capture().unwrap()
}
fn nanoseconds(trace: &Hp67PinTrace) -> Hp67PinTrace {
    let mut trace = trace.clone();
    trace.domain = PinTimeDomain::Nanoseconds;
    trace.source = PinSource::Synthetic;
    trace.provenance = "synthetic nonuniform intervals 2/3/5/7 ns; not measured".into();
    for row in &mut trace.rows {
        row.at = if row.at == 0 {
            u64::from(row.order)
        } else {
            100 + ((row.at - 1) / 4) * 17
                + [0, 2, 5, 10][((row.at - 1) % 4) as usize]
                + u64::from(row.order)
        };
        row.order = 0;
        row.owners = [None; Hp67Net::COUNT];
    }
    trace
}
#[test]
fn m15z_full_pin_capture_roundtrips_atomic_bundles_and_ownership_changes() {
    let trace = fixture(16);
    let imported = import_pin_trace(&trace.to_tsv().unwrap()).unwrap();
    assert_eq!(trace, imported);
    let mut bus = Hp67ElectricalBackplane::default();
    bus.start_pin_capture("owner fixture".into()).unwrap();
    bus.commit_display_controls(Drive::High, Drive::High);
    bus.drive(Hp67Net::Str, Hp67Driver::External, Drive::High);
    bus.drive(Hp67Net::Str, Hp67Driver::External, Drive::Low);
    bus.drive(Hp67Net::Str, Hp67Driver::External, Drive::HighZ);
    let trace = bus.take_pin_capture().unwrap();
    assert_eq!(trace.rows.len(), 5);
    assert_eq!(
        trace.rows[1].levels[Hp67Net::Str.index()],
        Some(LogicLevel::High)
    );
    assert_eq!(
        trace.rows[1].levels[Hp67Net::Rcd.index()],
        Some(LogicLevel::High)
    );
    assert_eq!(
        trace.rows[2].levels[Hp67Net::Str.index()],
        Some(LogicLevel::High)
    );
    assert_eq!(
        trace.rows[2].owners[Hp67Net::Str.index()]
            .unwrap()
            .high
            .count_ones(),
        2
    );
    assert_eq!(
        trace.rows[3].levels[Hp67Net::Str.index()],
        Some(LogicLevel::Contention)
    );
    for (index, row) in trace.rows.iter().enumerate() {
        assert_eq!(row.at, 0);
        assert_eq!(row.order as usize, index);
    }
    assert_eq!(import_pin_trace(&trace.to_tsv().unwrap()).unwrap(), trace);
}
#[test]
fn m15z_physical_domain_measures_nonuniform_phi_intervals_without_calibration_claim() {
    let trace = nanoseconds(&fixture(2));
    let trace = import_pin_trace(&trace.to_tsv().unwrap()).unwrap();
    let analysis = analyze_pin_trace(&trace).unwrap();
    for (intervals, expected) in analysis.phase_intervals.iter().zip([2, 3, 5, 7]) {
        assert!(!intervals.is_empty());
        assert!(intervals.iter().all(|value| *value == expected));
    }
    assert_eq!(analysis.cells.len(), 112);
    assert_eq!(analysis.trailing_phase, 0);
    let report = analysis.report(&trace);
    assert!(report.contains("source=Synthetic"));
    assert!(report.contains("No timing profile is installed"));
    assert!(report.contains("median=7"));
    let ordinal = fixture(2);
    assert!(analyze_pin_trace(&ordinal)
        .unwrap()
        .report(&ordinal)
        .contains("UNCALIBRATED"));
}
#[test]
fn m15z_clock_overlap_missing_simultaneous_and_unresolved_edges_are_errors() {
    let base = fixture(2);
    for variant in 0..4 {
        let mut trace = base.clone();
        let row = trace.rows.iter_mut().find(|row| row.at == 1).unwrap();
        match variant {
            0 => row.levels[Hp67Net::Phi2.index()] = Some(LogicLevel::Low),
            1 => {
                row.levels[Hp67Net::Phi1.index()] = Some(LogicLevel::High);
                row.levels[Hp67Net::Phi2.index()] = Some(LogicLevel::Low);
            }
            2 => row.levels[Hp67Net::Phi1.index()] = None,
            _ => {
                row.levels[Hp67Net::Phi1.index()] = Some(LogicLevel::High);
            }
        }
        assert!(analyze_pin_trace(&trace).is_err(), "variant={variant}");
    }
    let mut trace = base.clone();
    let row = trace.rows.iter_mut().find(|row| row.at == 2).unwrap();
    row.levels[Hp67Net::Phi2.index()] = Some(LogicLevel::Low);
    assert!(analyze_pin_trace(&trace).is_err());
}
#[test]
fn m15z_strict_import_rejects_ambiguous_domains_columns_masks_and_coordinates() {
    let text = fixture(1).to_tsv().unwrap();
    for malformed in [
        text.replace("domain=ordinal", "domain=ns"),
        text.replace("source=emulator", "source=measured"),
        text.replace("first_bit=0", "first_bit=56"),
        text.replace("# provenance=", "# wrong="),
        text.replace("\tPHI2\t", "\tPHI1\t"),
        text.replacen("1:0001:0000", "0:0001:0000", 1),
        text.replacen("1:0001:0000", "1:8000:0000", 1),
        text.replacen("0\t1\t", "0\t0\t", 1),
    ] {
        assert!(import_pin_trace(&malformed).is_err());
    }
    let physical = nanoseconds(&fixture(1)).to_tsv().unwrap();
    assert!(import_pin_trace(&physical.replacen("100\t0\t", "100\t1\t", 1)).is_err());
    assert!(import_pin_trace(&physical.replacen("100\t0\t", "1.0\t0\t", 1)).is_err());
    let mut model = fixture(1);
    model.domain = PinTimeDomain::Nanoseconds;
    model.source = PinSource::Measured;
    assert!(import_pin_trace(&model.to_tsv().unwrap()).is_err());
}
#[test]
fn m15z_explicit_word_comparison_reports_mismatch_coverage_and_owners() {
    let trace = fixture(3);
    let physical = nanoseconds(&trace);
    let equal =
        compare_pin_traces(&trace, &physical, 0, 2, 1, &[Hp67Net::Isa, Hp67Net::Sync]).unwrap();
    assert_eq!(equal.exit_code(), 0);
    assert_eq!(equal.owner_comparisons, 0);
    assert!(equal.report().contains("durations NOT compared"));
    assert!(compare_pin_traces(&trace, &trace, 0, 0, 0, &[Hp67Net::Isa]).is_err());
    assert!(compare_pin_traces(&trace, &trace, 0, 0, 513, &[Hp67Net::Isa]).is_err());
    assert!(compare_pin_traces(&trace, &trace, 3, 0, 1, &[Hp67Net::Isa]).is_err());
    let mut changed = trace.clone();
    let row = changed
        .rows
        .iter_mut()
        .find(|row| row.at == 16 * 4 + 2)
        .unwrap();
    row.levels[Hp67Net::Isa.index()] = Some(LogicLevel::High);
    row.owners[Hp67Net::Isa.index()] = Some(PinOwners {
        high: 1 << Hp67Driver::External.index(),
        low: 0,
    });
    // Address 0x123 has bit zero set: change only owner first, then logic.
    let owner = compare_pin_traces(&trace, &changed, 0, 0, 1, &[Hp67Net::Isa]).unwrap();
    assert!(owner.owner_differences > 0);
    assert_eq!(owner.exit_code(), 2);
    let row = changed
        .rows
        .iter_mut()
        .find(|row| row.at == 16 * 4 + 2)
        .unwrap();
    row.levels[Hp67Net::Isa.index()] = Some(LogicLevel::Low);
    row.owners[Hp67Net::Isa.index()] = Some(PinOwners { high: 0, low: 0 });
    assert!(
        compare_pin_traces(&trace, &changed, 0, 0, 1, &[Hp67Net::Isa])
            .unwrap()
            .differences
            > 0
    );
    let row = changed
        .rows
        .iter_mut()
        .find(|row| row.at == 16 * 4 + 2)
        .unwrap();
    row.levels[Hp67Net::Isa.index()] = None;
    row.owners[Hp67Net::Isa.index()] = None;
    let partial = compare_pin_traces(&trace, &changed, 0, 0, 1, &[Hp67Net::Isa]).unwrap();
    assert_eq!(partial.exit_code(), 3);
    assert_eq!(partial.unresolved, 1);
}
#[test]
fn m15z_word_decode_keeps_data_wrap_masks_qualifiers_and_display_edges() {
    let analysis = analyze_pin_trace(&fixture(16)).unwrap();
    let first: &PinWord = &analysis.words[&0];
    assert_eq!((first.address, first.address_mask), (0x123, 0xfff));
    assert_eq!((first.opcode, first.opcode_mask), (0x2a5, 0x3ff));
    assert_eq!(first.qualifier, Some(true));
    assert_eq!(analysis.words[&1].qualifier, Some(false));
    assert!(!analysis.words[&1].qualifier_conflict);
    assert_eq!(first.data, 0x00fedcba987654);
    assert_eq!(first.data_mask, (1u64 << 56) - 1);
    assert_eq!(analysis.words[&15].data_mask, (1u64 << 54) - 1);
    assert_eq!(analysis.str_cells[7], 16);
    assert_eq!(analysis.rcd_cells[7], 1);
    assert_eq!(analysis.str_cells.iter().sum::<usize>(), 16);
}
#[test]
fn m15z_capture_bound_and_rejected_start_cannot_export_truncated_success() {
    let mut trace = fixture(1);
    let mut row = trace.rows[0].clone();
    trace.rows.resize(MAX_PIN_ROWS, row.clone());
    row.at = 1000;
    trace.record(row);
    assert!(trace.overflowed);
    assert_eq!(trace.rows.len(), MAX_PIN_ROWS);
    assert!(trace.to_tsv().is_err());
    assert!(analyze_pin_trace(&trace).is_err());
    let mut bus = Hp67ElectricalBackplane::default();
    bus.advance_clock();
    assert!(bus.start_pin_capture("fixture".into()).is_err());
    assert!(bus.take_pin_capture().is_none());
    let mut bus = Hp67ElectricalBackplane::default();
    bus.start_pin_capture("fixture".into()).unwrap();
    assert!(bus.start_pin_capture("replacement".into()).is_err());
    assert_eq!(bus.take_pin_capture().unwrap().provenance, "fixture");
}
