//! Observed setup/hold separations and phase sensitivity, not chip requirements.
//! Censored/unknown boundaries are counted rather than assigned fake margins.
use super::{pin_trace::{analyze_pin_trace, Hp67PinTrace, PIN_NAMES, PinTimeDomain}, timing::Hp67ClockEdge, wiring::Hp67Net};
use crate::emulation::LogicLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct ObservedMargin {
    pub count: usize,
    pub min: Option<u64>,
    pub max: Option<u64>,
}
impl ObservedMargin {
    fn add(&mut self, value: u64) {
        self.count += 1; self.min = Some(self.min.map_or(value, |old| old.min(value))); self.max = Some(self.max.map_or(value, |old| old.max(value)));
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PhaseMargins {
    pub edge: Hp67ClockEdge,
    pub resolved: usize,
    pub unresolved: usize,
    pub coincident_changes: usize,
    pub setup: ObservedMargin,
    pub hold: ObservedMargin,
    pub setup_censored: usize,
    pub hold_censored: usize,
}
impl PhaseMargins {
    fn new(edge: Hp67ClockEdge) -> Self { Self { edge, resolved: 0, unresolved: 0, coincident_changes: 0, setup: Default::default(), hold: Default::default(), setup_censored: 0, hold_censored: 0 } }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinTimingSummary {
    pub net: Hp67Net,
    pub phases: [PhaseMargins; 4],
    pub phi1_low_intervals: usize,
    pub phi1_low_unresolved: usize,
    pub phi1_low_with_changes: usize,
    pub phi1_boundary_coincidences: usize,
    pub phase_value_disagreements: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinTimingAudit {
    pub pins: Vec<PinTimingSummary>,
    pub partial_trailing_phase: usize,
}
#[derive(Debug, Clone, Copy)]
struct Change { row: usize, binary: bool }
fn binary(level: Option<LogicLevel>) -> Option<bool> {
    match level { Some(LogicLevel::High) => Some(true), Some(LogicLevel::Low) => Some(false), _ => None }
}
const EDGES: [Hp67ClockEdge; 4] = [Hp67ClockEdge::Phi1Falling, Hp67ClockEdge::Phi1Rising, Hp67ClockEdge::Phi2Falling, Hp67ClockEdge::Phi2Rising];

pub fn audit_pin_timing(trace: &Hp67PinTrace, nets: &[Hp67Net]) -> Result<PinTimingAudit, String> {
    if nets.is_empty() { return Err("timing audit requires explicit observed pins".into()); }
    for (index, net) in nets.iter().enumerate() {
        if matches!(net, Hp67Net::Phi1 | Hp67Net::Phi2) { return Err("PHI pins are timing references, not setup/hold data inputs".into()); }
        if nets[..index].contains(net) { return Err("duplicate timing pin".into()); }
    }
    let topology = analyze_pin_trace(trace)?;
    let mut audit = PinTimingAudit { pins: Vec::new(), partial_trailing_phase: topology.trailing_phase };
    for &net in nets {
        let index = net.index();
        let changes: Vec<_> = trace.rows.windows(2).enumerate().filter_map(|(i, pair)| {
            (pair[0].levels[index] != pair[1].levels[index]).then_some(Change { row: i + 1, binary: binary(pair[0].levels[index]).is_some() && binary(pair[1].levels[index]).is_some() })
        }).collect();
        let mut pin = PinTimingSummary { net, phases: EDGES.map(PhaseMargins::new), phi1_low_intervals: 0, phi1_low_unresolved: 0, phi1_low_with_changes: 0, phi1_boundary_coincidences: 0, phase_value_disagreements: 0 };
        for clock in &topology.clock_edges {
            let phase = EDGES.iter().position(|edge| *edge == clock.edge).expect("known PHI edge");
            let row = &trace.rows[clock.row_index]; let summary = &mut pin.phases[phase];
            if binary(row.levels[index]).is_none() { summary.unresolved += 1; continue; }
            summary.resolved += 1;
            let after = changes.partition_point(|change| change.row <= clock.row_index);
            let previous = after.checked_sub(1).map(|i| changes[i]);
            let next = changes.get(after).copied();
            // One captured publication cannot order a pin edge against its PHI.
            if previous.is_some_and(|change| change.row == clock.row_index) { summary.coincident_changes += 1; continue; }
            match previous {
                Some(change) if change.binary => summary.setup.add(row.at - trace.rows[change.row].at),
                _ => summary.setup_censored += 1,
            }
            match next {
                Some(change) if change.binary => summary.hold.add(trace.rows[change.row].at - row.at),
                _ => summary.hold_censored += 1,
            }
        }
        for cell in topology.clock_edges.chunks(4) {
            if cell.len() < 2 { continue; }
            debug_assert_eq!(cell[0].edge, Hp67ClockEdge::Phi1Falling);
            debug_assert_eq!(cell[1].edge, Hp67ClockEdge::Phi1Rising);
            let first = cell[0].row_index; let last = cell[1].row_index;
            let begin = changes.partition_point(|change| change.row < first);
            let end = changes.partition_point(|change| change.row <= last);
            let interval = &changes[begin..end];
            pin.phi1_low_intervals += 1;
            if binary(trace.rows[first].levels[index]).is_none() || binary(trace.rows[last].levels[index]).is_none() || interval.iter().any(|change| !change.binary) { pin.phi1_low_unresolved += 1; }
            if interval.iter().any(|change| change.binary && change.row > first && change.row < last) { pin.phi1_low_with_changes += 1; }
            pin.phi1_boundary_coincidences += interval.iter().filter(|change| change.row == first || change.row == last).count();
            if let Some(reference) = binary(trace.rows[last].levels[index]) {
                if cell.iter().filter_map(|edge| binary(trace.rows[edge.row_index].levels[index])).any(|value| value != reference) { pin.phase_value_disagreements += 1; }
            }
        }
        audit.pins.push(pin);
    }
    Ok(audit)
}
impl PinTimingAudit {
    /// Incomplete/coincident observations are distinct from successful analysis.
    /// Neither status claims conformance to undocumented silicon requirements.
    pub fn exit_code(&self) -> i32 {
        if self.pins.iter().any(|pin| pin.phases.iter().any(|phase| phase.unresolved != 0 || phase.coincident_changes != 0)) { 3 } else { 0 }
    }
    pub fn report(&self, trace: &Hp67PinTrace) -> String {
        use std::fmt::Write;
        let units = match trace.domain { PinTimeDomain::Ordinal => "ordinal ticks; UNCALIBRATED", PinTimeDomain::Nanoseconds => "input nanoseconds; measurement accuracy not certified" };
        let mut out = format!("HP67 PHI WINDOW AUDIT v1\nsource={:?}\nprovenance={}\nunits={units}\nfirst_bit={} partial_trailing_phase={}\n", trace.source, trace.provenance, trace.first_bit, self.partial_trailing_phase);
        for pin in &self.pins {
            writeln!(out, "pin={} phi1_low_intervals={} phi1_low_unresolved={} phi1_low_with_changes={} phi1_boundary_coincidences={} phase_value_disagreements={}", PIN_NAMES[pin.net.index()], pin.phi1_low_intervals, pin.phi1_low_unresolved, pin.phi1_low_with_changes, pin.phi1_boundary_coincidences, pin.phase_value_disagreements).unwrap();
            for phase in &pin.phases { writeln!(out, "  {:?}: resolved={} unresolved={} coincident={} setup_n={} setup_min={:?} setup_max={:?} setup_censored={} hold_n={} hold_min={:?} hold_max={:?} hold_censored={}", phase.edge, phase.resolved, phase.unresolved, phase.coincident_changes, phase.setup.count, phase.setup.min, phase.setup.max, phase.setup_censored, phase.hold.count, phase.hold.min, phase.hold.max, phase.hold_censored).unwrap(); }
        }
        writeln!(out, "coverage_exit={}\nObserved separations only: no setup/hold requirement, propagation cause, exact receiver edge or timing profile is inferred. Censored bounds are not zero margins.", self.exit_code()).unwrap(); out
    }
}

#[cfg(test)]
#[path = "pin_timing_tests.rs"]
mod tests;
