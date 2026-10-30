//! Offline ownership audit of the current production digital bridge.
//! This checks implemented source contracts, never undocumented silicon timing.
use super::{
    pin_trace::{analyze_pin_trace, Hp67PinTrace, PinOwners, PinRow, PIN_NAMES},
    rom_devices::HP67_ROM_PROFILES,
    timing::Hp67ClockEdge,
    wiring::{Hp67Driver, Hp67Net},
};
use crate::emulation::LogicLevel;

pub const MAX_INTEGRITY_FINDINGS: usize = 64;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinIntegrityCoverage {
    pub net: Hp67Net,
    pub assessed: bool,
    pub checked: usize,
    pub missing_levels: usize,
    pub missing_owners: usize,
    pub missing_address: usize,
    pub violations: usize,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinIntegrityFinding {
    pub net: Hp67Net,
    pub at: u64,
    pub order: u32,
    pub word_bit: Option<(u64, u8)>,
    pub reason: &'static str,
    pub level: Option<LogicLevel>,
    pub owners: Option<PinOwners>,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinIntegrityAudit {
    pub pins: [PinIntegrityCoverage; Hp67Net::COUNT],
    pub cells: usize,
    pub complete_capture: bool,
    pub findings: Vec<PinIntegrityFinding>,
}
fn mask(driver: Hp67Driver) -> u16 {
    1 << driver.index()
}
fn sole_source(net: Hp67Net) -> Option<Hp67Driver> {
    use Hp67Driver::*;
    use Hp67Net::*;
    match net {
        Phi1 | Phi2 | Sync | Rcd => Some(Act1820_2530),
        Str => Some(RomDisplay1818_0268),
        F2 | Motor => Some(CardReaderController1820_1751),
        Mtrs | Hds | Wps | PgmSw => Some(External),
        _ => None,
    }
}
fn validate_masks(row: &PinRow) -> Result<(), String> {
    let allowed = (1u16 << Hp67Driver::COUNT) - 1;
    for net in Hp67Net::ALL {
        let i = net.index();
        if let Some(o) = row.owners[i] {
            let resolved = if o.high != 0 && o.low != 0 {
                LogicLevel::Contention
            } else if o.high != 0 {
                LogicLevel::High
            } else if o.low != 0 || net == Hp67Net::Isa {
                LogicLevel::Low
            } else {
                LogicLevel::Floating
            };
            if (o.high | o.low) & !allowed != 0
                || o.high & o.low != 0
                || row.levels[i] != Some(resolved)
            {
                return Err(format!(
                    "invalid ownership image for {} at {}:{}",
                    PIN_NAMES[i], row.at, row.order
                ));
            }
        }
    }
    Ok(())
}
impl PinIntegrityAudit {
    fn inspect(
        &mut self,
        row: &PinRow,
        net: Hp67Net,
        coordinate: Option<(u64, u8)>,
        address: Option<u16>,
    ) {
        let i = net.index();
        let pin = &mut self.pins[i];
        pin.checked += 1;
        let level = row.levels[i];
        let owners = row.owners[i];
        if level.is_none() {
            pin.missing_levels += 1;
        }
        if owners.is_none() {
            pin.missing_owners += 1;
        }
        let mut reason = None;
        if let Some(expected) = sole_source(net) {
            if level.is_some_and(|v| !matches!(v, LogicLevel::High | LogicLevel::Low)) {
                reason = Some("required binary pin is floating or contended");
            }
            if owners
                .is_some_and(|o| (o.high | o.low) != mask(expected) || (o.high != 0 && o.low != 0))
            {
                reason = Some("pin requires exactly its named source");
            }
        } else if net == Hp67Net::Isa {
            let bit = coordinate.expect("IS inspected only at cell checkpoint").1;
            let allowed = match bit {
                0..=7 | 16..=27 => mask(Hp67Driver::Act1820_2530),
                46..=55 => {
                    if address.is_none() {
                        pin.missing_address += 1;
                    }
                    HP67_ROM_PROFILES
                        .iter()
                        .filter(|p| {
                            address.map_or(true, |a| {
                                a >= p.first_address && a < p.first_address + 0x400
                            })
                        })
                        .fold(0, |bits, p| bits | mask(p.driver))
                }
                _ => 0,
            };
            if level.is_some_and(|v| !matches!(v, LogicLevel::High | LogicLevel::Low)) {
                reason = Some("IS must resolve to passive Low or active High");
            }
            if owners
                .is_some_and(|o| o.low != 0 || o.high.count_ones() > 1 || o.high & !allowed != 0)
            {
                reason = Some("IS source violates display/address/return/idle role");
            }
        } else if net == Hp67Net::Data {
            if level == Some(LogicLevel::Contention) {
                reason = Some("DATA contention");
            }
            let allowed = mask(Hp67Driver::Act1820_2530) | mask(Hp67Driver::RamAggregate);
            if owners.is_some_and(|o| {
                (o.high | o.low).count_ones() > 1 || (o.high | o.low) & !allowed != 0
            }) {
                reason = Some("DATA requires release or one production ACT/RAM source");
            }
        }
        if let Some(reason) = reason {
            pin.violations += 1;
            if self.findings.len() < MAX_INTEGRITY_FINDINGS {
                self.findings.push(PinIntegrityFinding {
                    net,
                    at: row.at,
                    order: row.order,
                    word_bit: coordinate,
                    reason,
                    level,
                    owners,
                });
            }
        }
    }
    /// 0 = scoped bridge contracts observed; 2 = violation; 3 = missing coverage.
    /// A scoped success never certifies hardware timing or the unassessed pins.
    pub fn exit_code(&self) -> i32 {
        if self.pins.iter().any(|p| p.violations != 0) {
            2
        } else if !self.complete_capture
            || self.pins.iter().filter(|p| p.assessed).any(|p| {
                p.checked == 0
                    || p.missing_levels != 0
                    || p.missing_owners != 0
                    || p.missing_address != 0
            })
        {
            3
        } else {
            0
        }
    }
    pub fn report(&self, trace: &Hp67PinTrace) -> String {
        use std::fmt::Write;
        let mut out = format!("HP67 PRODUCTION PIN INTEGRITY v1\nsource={:?} domain={:?}\nprovenance={}\ncells={} complete_capture={} scoped_exit={}\n", trace.source, trace.domain, trace.provenance, self.cells, self.complete_capture, self.exit_code());
        for p in &self.pins {
            writeln!(out, "pin={} scope={} checked={} missing_levels={} missing_owners={} missing_address={} violations={}", PIN_NAMES[p.net.index()], if p.assessed { "BRIDGE_CONTRACT" } else { "UNASSESSED" }, p.checked, p.missing_levels, p.missing_owners, p.missing_address, p.violations).unwrap();
        }
        for f in &self.findings {
            writeln!(
                out,
                "fault pin={} at={}:{} word_bit={:?} level={:?} owners={:?} reason={}",
                PIN_NAMES[f.net.index()],
                f.at,
                f.order,
                f.word_bit,
                f.level,
                f.owners,
                f.reason
            )
            .unwrap();
        }
        let total: usize = self.pins.iter().map(|p| p.violations).sum();
        writeln!(
            out,
            "findings_shown={} violations_total={total}",
            self.findings.len()
        )
        .unwrap();
        out.push_str("Scope: current production bridge; PHI owners checked at every row, other assessed pins at provisional PHI1-rising checkpoints. IS return page uses observed address; ROM bank identity is NOT certified. DATA direction/frame/payload and CRC logical FIFO serialization are NOT certified. F1/KC acquisition is UNASSESSED.\nSOURCE-BLOCKED: exact PHI widths/dead time, physical launch/latch edges and propagation, DATA output stage/polarity and physical RAM mapping, magnetic sense/serialization. No nanosecond calibration, hardware equivalence or complete-emulator certification follows from exit 0.\n");
        out
    }
}

pub fn audit_pin_integrity(trace: &Hp67PinTrace) -> Result<PinIntegrityAudit, String> {
    let analysis = analyze_pin_trace(trace)?;
    let mut audit = PinIntegrityAudit {
        pins: std::array::from_fn(|i| {
            let net = Hp67Net::ALL[i];
            PinIntegrityCoverage {
                net,
                assessed: sole_source(net).is_some() || matches!(net, Hp67Net::Isa | Hp67Net::Data),
                checked: 0,
                missing_levels: 0,
                missing_owners: 0,
                missing_address: 0,
                violations: 0,
            }
        }),
        cells: analysis.cells.len(),
        complete_capture: trace.first_bit == 0
            && analysis.trailing_phase == 0
            && analysis.cells.len() % 56 == 0,
        findings: Vec::new(),
    };
    for row in &trace.rows {
        validate_masks(row)?;
        for net in [Hp67Net::Phi1, Hp67Net::Phi2] {
            audit.inspect(row, net, None, None);
        }
    }
    for edge in analysis
        .clock_edges
        .iter()
        .filter(|e| e.edge == Hp67ClockEdge::Phi1Rising)
    {
        let row = &trace.rows[edge.row_index];
        let address = analysis
            .words
            .get(&edge.word)
            .filter(|w| w.address_mask == 0xfff)
            .map(|w| w.address);
        for net in Hp67Net::ALL
            .into_iter()
            .filter(|n| !matches!(n, Hp67Net::Phi1 | Hp67Net::Phi2))
        {
            if audit.pins[net.index()].assessed {
                audit.inspect(row, net, Some((edge.word, edge.bit)), address);
            }
        }
    }
    Ok(audit)
}

#[cfg(test)]
#[path = "pin_integrity_tests.rs"]
mod tests;
