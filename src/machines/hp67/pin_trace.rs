//! Full-pin ordinal capture and strictly separated physical-time observations.
//! Capture metadata is provenance supplied by the caller, not hardware proof.
#[cfg(feature = "pin-capture")]
use super::machine::Hp67ElectricalBackplane;
use super::wiring::{Hp67Driver, Hp67Net};
#[cfg(feature = "pin-capture")]
use crate::emulation::Drive;
use crate::emulation::LogicLevel;

pub const MAX_PIN_ROWS: usize = 160_000;
pub const MAX_PIN_BYTES: usize = 64 * 1024 * 1024;
pub const MAX_CAPTURE_WORDS: usize = 512;
pub const PIN_NAMES: [&str; Hp67Net::COUNT] = [
    "PHI1", "PHI2", "IS", "DATA", "SYNC", "RCD", "STR", "F1", "F2", "KC1", "KC2", "KC3", "KC4",
    "KC5", "MOTOR", "MTRS",
];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinTimeDomain {
    Ordinal,
    Nanoseconds,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PinSource {
    Emulator,
    Measured,
    Synthetic,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PinOwners {
    pub high: u16,
    pub low: u16,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinRow {
    pub at: u64,
    pub order: u32,
    pub levels: [Option<LogicLevel>; Hp67Net::COUNT],
    pub owners: [Option<PinOwners>; Hp67Net::COUNT],
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67PinTrace {
    pub domain: PinTimeDomain,
    pub source: PinSource,
    pub first_bit: u8,
    pub provenance: String,
    pub rows: Vec<PinRow>,
    pub overflowed: bool,
}
impl Hp67PinTrace {
    pub fn emulator(first_bit: u8, provenance: String) -> Result<Self, String> {
        if first_bit >= 56
            || provenance.trim().is_empty()
            || provenance.chars().any(|c| matches!(c, '\n' | '\r' | '\t'))
        {
            return Err("invalid capture anchor/provenance".into());
        }
        Ok(Self {
            domain: PinTimeDomain::Ordinal,
            source: PinSource::Emulator,
            first_bit,
            provenance,
            rows: Vec::new(),
            overflowed: false,
        })
    }
    #[cfg(feature = "pin-capture")]
    pub(crate) fn record(&mut self, mut row: PinRow) {
        if self.overflowed {
            return;
        }
        if let Some(previous) = self.rows.last() {
            if previous.at == row.at
                && previous.levels == row.levels
                && previous.owners == row.owners
            {
                return;
            }
            row.order = if previous.at == row.at {
                previous.order + 1
            } else {
                0
            };
        }
        if self.rows.len() == MAX_PIN_ROWS {
            self.overflowed = true;
            return;
        }
        self.rows.push(row);
    }
    pub fn to_tsv(&self) -> Result<String, String> {
        use std::fmt::Write;
        if self.overflowed {
            return Err("pin capture overflow; no truncated success export".into());
        }
        let domain = match self.domain {
            PinTimeDomain::Ordinal => "ordinal",
            PinTimeDomain::Nanoseconds => "ns",
        };
        let source = match self.source {
            PinSource::Emulator => "emulator",
            PinSource::Measured => "measured",
            PinSource::Synthetic => "synthetic",
        };
        let mut out = format!("# hp67-pins-v1 domain={domain} source={source} first_bit={}\n# provenance={}\nat\torder", self.first_bit, self.provenance);
        for name in PIN_NAMES {
            write!(out, "\t{name}").unwrap();
        }
        out.push('\n');
        for row in &self.rows {
            write!(out, "{}\t{}", row.at, row.order).unwrap();
            for net in Hp67Net::ALL {
                let index = net.index();
                let level = level_text(row.levels[index]);
                if let Some(owner) = row.owners[index] {
                    write!(out, "\t{level}:{:04x}:{:04x}", owner.high, owner.low).unwrap();
                } else {
                    write!(out, "\t{level}").unwrap();
                }
            }
            out.push('\n');
            if out.len() > MAX_PIN_BYTES {
                return Err("pin export exceeds byte limit".into());
            }
        }
        Ok(out)
    }
}
#[cfg(feature = "pin-capture")]
pub(crate) fn snapshot(bus: &Hp67ElectricalBackplane) -> PinRow {
    PinRow {
        at: bus.tick().get(),
        order: 0,
        levels: std::array::from_fn(|i| Some(bus.level(Hp67Net::ALL[i]))),
        owners: std::array::from_fn(|i| {
            let mut owners = PinOwners { high: 0, low: 0 };
            for driver in Hp67Driver::ALL {
                match bus.driver_drive(Hp67Net::ALL[i], driver) {
                    Drive::High => owners.high |= 1 << driver.index(),
                    Drive::Low => owners.low |= 1 << driver.index(),
                    Drive::HighZ => {}
                }
            }
            Some(owners)
        }),
    }
}
fn level_text(level: Option<LogicLevel>) -> &'static str {
    match level {
        Some(LogicLevel::Low) => "0",
        Some(LogicLevel::High) => "1",
        Some(LogicLevel::Floating) => "Z",
        Some(LogicLevel::Contention) => "X",
        None => "?",
    }
}
fn parse_level(value: &str) -> Result<Option<LogicLevel>, String> {
    match value {
        "0" => Ok(Some(LogicLevel::Low)),
        "1" => Ok(Some(LogicLevel::High)),
        "Z" => Ok(Some(LogicLevel::Floating)),
        "X" => Ok(Some(LogicLevel::Contention)),
        "?" => Ok(None),
        _ => Err(format!("invalid pin level {value:?}")),
    }
}
/// Canonical TSV: explicit domain, source, b0..b55 anchor and provenance.
/// Physical inputs may omit ownership and unobserved pin columns; no CSV vendor
/// defaults, time rounding, interpolation or automatic alignment are applied.
pub fn import_pin_trace(input: &str) -> Result<Hp67PinTrace, String> {
    if input.len() > MAX_PIN_BYTES {
        return Err("pin input exceeds byte limit".into());
    }
    let mut lines = input.lines().enumerate();
    let (_, metadata) = lines.next().ok_or("missing pin metadata")?;
    let fields: Vec<_> = metadata
        .trim_start_matches('\u{feff}')
        .split_whitespace()
        .collect();
    if fields.len() != 5 || fields[0] != "#" || fields[1] != "hp67-pins-v1" {
        return Err("expected hp67-pins-v1 metadata".into());
    }
    let domain = match fields[2] {
        "domain=ordinal" => PinTimeDomain::Ordinal,
        "domain=ns" => PinTimeDomain::Nanoseconds,
        _ => return Err("unsupported pin time domain".into()),
    };
    let source = match fields[3] {
        "source=emulator" => PinSource::Emulator,
        "source=measured" => PinSource::Measured,
        "source=synthetic" => PinSource::Synthetic,
        _ => return Err("unsupported capture source".into()),
    };
    if (domain == PinTimeDomain::Ordinal && source != PinSource::Emulator)
        || (domain == PinTimeDomain::Nanoseconds && source == PinSource::Emulator)
    {
        return Err(
            "source/time domain mismatch; ordinal ticks cannot be physical nanoseconds".into(),
        );
    }
    let first_bit = fields[4]
        .strip_prefix("first_bit=")
        .ok_or("missing anchor")?
        .parse::<u8>()
        .map_err(|_| "invalid first-bit anchor")?;
    if first_bit >= 56 {
        return Err("first-bit anchor outside b0..b55".into());
    }
    let (_, provenance) = lines.next().ok_or("missing provenance")?;
    let provenance = provenance
        .strip_prefix("# provenance=")
        .filter(|s| !s.trim().is_empty() && !s.contains('\t'))
        .ok_or("missing capture provenance")?
        .to_owned();
    let (_, header) = lines.next().ok_or("missing pin header")?;
    let columns: Vec<_> = header.split('\t').collect();
    if columns.len() < 4 || columns[0..2] != ["at", "order"] {
        return Err("invalid pin header".into());
    }
    let mut nets = Vec::new();
    for name in &columns[2..] {
        let index = PIN_NAMES
            .iter()
            .position(|n| n == name)
            .ok_or_else(|| format!("unknown pin column {name}"))?;
        if nets.contains(&index) {
            return Err("duplicate pin column".into());
        }
        nets.push(index);
    }
    if !nets.contains(&Hp67Net::Phi1.index()) || !nets.contains(&Hp67Net::Phi2.index()) {
        return Err("both PHI columns are required".into());
    }
    let mut trace = Hp67PinTrace {
        domain,
        source,
        first_bit,
        provenance,
        rows: Vec::new(),
        overflowed: false,
    };
    for (line, text) in lines {
        if trace.rows.len() == MAX_PIN_ROWS {
            return Err("pin input exceeds row limit".into());
        }
        let parse = || -> Result<PinRow, String> {
            let cells: Vec<_> = text.split('\t').collect();
            if cells.len() != columns.len() {
                return Err("wrong pin column count".into());
            }
            let at = cells[0]
                .parse::<u64>()
                .map_err(|_| "invalid integer timestamp")?;
            let order = cells[1].parse::<u32>().map_err(|_| "invalid delta order")?;
            if domain == PinTimeDomain::Nanoseconds && order != 0 {
                return Err("physical rows require unique timestamps/order=0; unresolved simultaneous ordering must not be invented".into());
            }
            if let Some(previous) = trace.rows.last() {
                if at < previous.at
                    || (at == previous.at
                        && (domain != PinTimeDomain::Ordinal
                            || order
                                != previous
                                    .order
                                    .checked_add(1)
                                    .ok_or("delta order overflow")?))
                    || (at > previous.at && order != 0)
                {
                    return Err("non-monotonic capture coordinates".into());
                }
            } else if order != 0 {
                return Err("first row requires order=0".into());
            }
            let mut row = PinRow {
                at,
                order,
                levels: [None; Hp67Net::COUNT],
                owners: [None; Hp67Net::COUNT],
            };
            for (cell, &net) in cells[2..].iter().zip(&nets) {
                let parts: Vec<_> = cell.split(':').collect();
                row.levels[net] = parse_level(parts[0])?;
                match parts.len() {
                    1 => {}
                    3 if domain == PinTimeDomain::Ordinal => {
                        let high = u16::from_str_radix(parts[1], 16)
                            .map_err(|_| "invalid high ownership mask")?;
                        let low = u16::from_str_radix(parts[2], 16)
                            .map_err(|_| "invalid low ownership mask")?;
                        let allowed = (1u16 << Hp67Driver::COUNT) - 1;
                        if (high | low) & !allowed != 0 || high & low != 0 {
                            return Err("invalid/contradictory driver ownership".into());
                        }
                        let resolved = if high != 0 && low != 0 {
                            Some(LogicLevel::Contention)
                        } else if high != 0 {
                            Some(LogicLevel::High)
                        } else if low != 0 || net == Hp67Net::Isa.index() {
                            Some(LogicLevel::Low)
                        } else {
                            Some(LogicLevel::Floating)
                        };
                        if row.levels[net] != resolved {
                            return Err("ownership masks disagree with resolved pin".into());
                        }
                        row.owners[net] = Some(PinOwners { high, low });
                    }
                    _ => return Err("physical capture cannot claim internal driver masks".into()),
                }
            }
            Ok(row)
        };
        trace
            .rows
            .push(parse().map_err(|e| format!("line {}: {e}", line + 1))?);
    }
    if trace.rows.len() < 2 {
        return Err("capture needs at least two rows".into());
    }
    Ok(trace)
}

#[path = "pin_trace_analysis.rs"]
mod analysis;
pub use analysis::{
    analyze_pin_trace, compare_pin_traces, PinAnalysis, PinClockObservation, PinComparison,
};

#[cfg(all(test, feature = "pin-capture"))]
#[path = "pin_trace_tests.rs"]
mod tests;
