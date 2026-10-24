use super::*;
use crate::machines::hp67::pin_trace::{import_pin_trace, PinSource};

fn fixture() -> Hp67PinTrace {
    import_pin_trace("# hp67-pins-v1 domain=ns source=synthetic first_bit=0\n# provenance=synthetic setup/hold fixture; NOT hardware timing\nat\torder\tPHI1\tPHI2\tIS\tDATA\tSYNC\tSTR\tRCD\n0\t0\t1\t1\t0\t0\t0\t1\t1\n5\t0\t1\t1\t1\t0\t0\t1\t1\n10\t0\t0\t1\t1\t0\t0\t1\t1\n12\t0\t1\t1\t1\t0\t0\t1\t1\n15\t0\t1\t0\t1\t0\t0\t1\t1\n17\t0\t1\t0\t0\t0\t0\t1\t1\n20\t0\t1\t1\t0\t0\t0\t1\t1\n25\t0\t0\t1\t0\t0\t0\t1\t1\n27\t0\t1\t1\t0\t0\t0\t1\t1\n30\t0\t1\t0\t0\t0\t0\t1\t1\n32\t0\t1\t0\t1\t0\t0\t1\t1\n35\t0\t1\t1\t1\t0\t0\t1\t1\n").unwrap()
}
#[test]
fn m16a_all_phi_edges_report_exact_synthetic_observed_separations() {
    let trace = fixture();
    let audit = audit_pin_timing(&trace, &[Hp67Net::Isa]).unwrap();
    let pin = &audit.pins[0];
    let expected = [(5, 8, 7, 7), (7, 10, 5, 5), (10, 13, 2, 2), (3, 3, 12, 12)];
    for (phase, (smin, smax, hmin, hmax)) in pin.phases.iter().zip(expected) {
        assert_eq!(phase.resolved, 2);
        assert_eq!(phase.unresolved, 0);
        assert_eq!(phase.coincident_changes, 0);
        assert_eq!((phase.setup.min, phase.setup.max), (Some(smin), Some(smax)));
        assert_eq!((phase.hold.min, phase.hold.max), (Some(hmin), Some(hmax)));
    }
    assert_eq!(pin.phases[3].hold_censored, 1);
    assert_eq!(pin.phi1_low_intervals, 2);
    assert_eq!(pin.phi1_low_with_changes, 0);
    assert_eq!(pin.phase_value_disagreements, 2);
    assert_eq!(audit.exit_code(), 0);
    let report = audit.report(&trace);
    assert!(report.contains("source=Synthetic"));
    assert!(report.contains("no setup/hold requirement"));
}
#[test]
fn m16a_censored_unknown_and_coincident_bounds_never_become_zero_requirements() {
    let mut trace = fixture();
    let constant = audit_pin_timing(&trace, &[Hp67Net::Sync]).unwrap();
    for phase in &constant.pins[0].phases {
        assert_eq!(phase.setup.count, 0);
        assert_eq!(phase.setup.min, None);
        assert_eq!(phase.hold.count, 0);
        assert_eq!(phase.setup_censored, 2);
        assert_eq!(phase.hold_censored, 2);
    }
    for row in &mut trace.rows {
        row.levels[Hp67Net::Data.index()] = None;
    }
    let missing = audit_pin_timing(&trace, &[Hp67Net::Data]).unwrap();
    assert_eq!(missing.exit_code(), 3);
    assert!(missing.pins[0]
        .phases
        .iter()
        .all(|phase| phase.unresolved == 2 && phase.setup.min.is_none()));
    let mut trace = fixture();
    let row = trace.rows.iter_mut().find(|row| row.at == 12).unwrap();
    row.levels[Hp67Net::Isa.index()] = Some(LogicLevel::Low);
    let ambiguous = audit_pin_timing(&trace, &[Hp67Net::Isa]).unwrap();
    assert_eq!(ambiguous.pins[0].phases[1].coincident_changes, 1);
    assert_eq!(ambiguous.pins[0].phi1_boundary_coincidences, 1);
    assert_eq!(ambiguous.exit_code(), 3);
}
#[test]
fn m16a_phi1_window_changes_and_ordinal_delta_order_stay_explicit() {
    let mut trace = fixture();
    let mut inside = trace.rows.iter().find(|row| row.at == 10).unwrap().clone();
    inside.at = 11;
    inside.levels[Hp67Net::Isa.index()] = Some(LogicLevel::Low);
    trace.rows.insert(3, inside);
    for row in trace
        .rows
        .iter_mut()
        .filter(|row| row.at >= 12 && row.at <= 17)
    {
        row.levels[Hp67Net::Isa.index()] = Some(LogicLevel::Low);
    }
    let audit = audit_pin_timing(&trace, &[Hp67Net::Isa]).unwrap();
    assert_eq!(audit.pins[0].phi1_low_with_changes, 1);
    assert_eq!(audit.pins[0].phases[1].setup.min, Some(1));
    trace.domain = PinTimeDomain::Ordinal;
    trace.source = PinSource::Emulator;
    let row = trace.rows.iter_mut().find(|row| row.at == 11).unwrap();
    row.at = 10;
    row.order = 1;
    let audit = audit_pin_timing(&trace, &[Hp67Net::Isa]).unwrap();
    assert_eq!(audit.pins[0].phases[0].hold.min, Some(0));
    assert!(audit.report(&trace).contains("UNCALIBRATED"));
    assert_eq!(audit.pins[0].phases[0].coincident_changes, 0);
}
#[test]
fn m16a_clock_anchor_validation_and_observation_indices_cover_all_four_edges() {
    let mut trace = fixture();
    trace.first_bit = 55;
    let topology = analyze_pin_trace(&trace).unwrap();
    assert_eq!(topology.clock_edges.len(), 8);
    for (index, edge) in topology.clock_edges.iter().enumerate() {
        assert_eq!(edge.edge, EDGES[index % 4]);
        assert_eq!(edge.bit, if index < 4 { 55 } else { 0 });
        assert_eq!(edge.word, if index >= 4 { 1 } else { 0 });
    }
    assert!(audit_pin_timing(&trace, &[]).is_err());
    assert!(audit_pin_timing(&trace, &[Hp67Net::Isa, Hp67Net::Isa]).is_err());
    assert!(audit_pin_timing(&trace, &[Hp67Net::Phi1]).is_err());
    let mut invalid_domain = trace.clone();
    invalid_domain.rows[2].order = 1;
    assert!(audit_pin_timing(&invalid_domain, &[Hp67Net::Isa]).is_err());
    trace.rows[2].levels[Hp67Net::Phi2.index()] = Some(LogicLevel::Low);
    assert!(audit_pin_timing(&trace, &[Hp67Net::Isa]).is_err());
}
