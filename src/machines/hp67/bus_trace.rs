//! Strict ordinal IS/PHI trace import and explicit-index comparison.
//! This is offline diagnostics, not calibrated physical waveform alignment.

use crate::emulation::{Drive, LogicLevel};

pub const MAX_TRACE_ROWS: usize = 4096 * 224;
pub const MAX_TRACE_BYTES: usize = 64 * 1024 * 1024;

const PIN_HEADER: &str = "tick\ttransport_bit\tphi1\tphi2\tisa";
const DRIVER_HEADER: &str = "tick\ttransport_bit\tphi1\tphi2\tisa\tact_drive\trom_drive";

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusTraceRow {
    pub tick: u64,
    pub transport_bit: u8,
    pub phi1: LogicLevel,
    pub phi2: LogicLevel,
    pub isa: LogicLevel,
    pub act_drive: Option<Drive>,
    pub rom_drive: Option<Drive>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ImportedBusTrace {
    pub provenance: Vec<String>,
    pub rows: Vec<BusTraceRow>,
}

fn level(value: &str) -> Result<LogicLevel, String> {
    match value {
        "High" => Ok(LogicLevel::High),
        "Low" => Ok(LogicLevel::Low),
        "Floating" => Ok(LogicLevel::Floating),
        "Contention" => Ok(LogicLevel::Contention),
        _ => Err(format!("unknown logic level {value:?}")),
    }
}

fn drive(value: &str) -> Result<Drive, String> {
    match value {
        "High" => Ok(Drive::High),
        "Low" => Ok(Drive::Low),
        "HighZ" => Ok(Drive::HighZ),
        _ => Err(format!("unknown driver state {value:?}")),
    }
}

/// Import native seven-column TSV or explicitly normalized five-pin-column TSV.
/// Raw analyser CSV/timestamps are not silently converted to ordinal ticks.
pub fn import_bus_trace(input: &str) -> Result<ImportedBusTrace, String> {
    if input.len() > MAX_TRACE_BYTES {
        return Err("trace exceeds byte limit".to_owned());
    }
    let mut trace = ImportedBusTrace {
        provenance: Vec::new(),
        rows: Vec::new(),
    };
    let mut columns = None;
    for (index, raw) in input.lines().enumerate() {
        let line = raw
            .strip_prefix('\u{feff}')
            .filter(|_| index == 0)
            .unwrap_or(raw);
        if line.is_empty() {
            continue;
        }
        if line.starts_with('#') {
            trace.provenance.push(line.to_owned());
            continue;
        }
        if columns.is_none() {
            columns = Some(match line {
                PIN_HEADER => 5,
                DRIVER_HEADER => 7,
                _ => return Err(format!("line {}: unsupported trace header", index + 1)),
            });
            continue;
        }
        if trace.rows.len() == MAX_TRACE_ROWS {
            return Err("trace exceeds row limit".to_owned());
        }
        let parse = || -> Result<BusTraceRow, String> {
            let fields: Vec<&str> = line.split('\t').collect();
            if fields.len() != columns.unwrap() {
                return Err("wrong column count".to_owned());
            }
            let tick = fields[0]
                .parse::<u64>()
                .map_err(|_| "invalid ordinal tick")?;
            let transport_bit = fields[1]
                .parse::<u8>()
                .map_err(|_| "invalid transport bit")?;
            if transport_bit > 55 {
                return Err("transport bit outside b0..b55".to_owned());
            }
            if let Some(previous) = trace.rows.last() {
                if tick <= previous.tick {
                    return Err("ticks must be strictly increasing".to_owned());
                }
            }
            Ok(BusTraceRow {
                tick,
                transport_bit,
                phi1: level(fields[2])?,
                phi2: level(fields[3])?,
                isa: level(fields[4])?,
                act_drive: if fields.len() == 7 {
                    Some(drive(fields[5])?)
                } else {
                    None
                },
                rom_drive: if fields.len() == 7 {
                    Some(drive(fields[6])?)
                } else {
                    None
                },
            })
        };
        let row = parse().map_err(|error| format!("line {}: {error}", index + 1))?;
        trace.rows.push(row);
    }
    if trace.rows.is_empty() {
        return Err("trace has no samples".to_owned());
    }
    Ok(trace)
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BusTraceComparison {
    pub matches: bool,
    pub report: String,
}

/// Compare explicitly selected suffixes. No best-fit search, inversion,
/// resampling, truncation or physical time calibration is performed.
pub fn compare_bus_traces(
    expected: &ImportedBusTrace,
    actual: &ImportedBusTrace,
    expected_start: usize,
    actual_start: usize,
) -> Result<BusTraceComparison, String> {
    use std::fmt::Write;
    let expected_rows = expected
        .rows
        .get(expected_start..)
        .filter(|rows| !rows.is_empty())
        .ok_or("expected start outside samples")?;
    let actual_rows = actual
        .rows
        .get(actual_start..)
        .filter(|rows| !rows.is_empty())
        .ok_or("actual start outside samples")?;
    let mut differences = 0usize;
    let mut driver_rows = 0usize;
    let mut unresolved_rows = 0usize;
    let mut examples = String::new();
    for (index, (left, right)) in expected_rows.iter().zip(actual_rows).enumerate() {
        let mut fields = Vec::new();
        let left_tick = left
            .tick
            .checked_sub(expected_rows[0].tick)
            .ok_or("expected tick precedes origin")?;
        let right_tick = right
            .tick
            .checked_sub(actual_rows[0].tick)
            .ok_or("actual tick precedes origin")?;
        if left_tick != right_tick {
            fields.push("relative_tick");
        }
        if left.transport_bit != right.transport_bit {
            fields.push("transport_bit");
        }
        if left.phi1 != right.phi1 {
            fields.push("phi1");
        }
        if left.phi2 != right.phi2 {
            fields.push("phi2");
        }
        if left.isa != right.isa {
            fields.push("isa");
        }
        let drivers_known = left.act_drive.is_some()
            && right.act_drive.is_some()
            && left.rom_drive.is_some()
            && right.rom_drive.is_some();
        if drivers_known {
            driver_rows += 1;
            if left.act_drive != right.act_drive {
                fields.push("act_drive");
            }
            if left.rom_drive != right.rom_drive {
                fields.push("rom_drive");
            }
        }
        if [
            left.phi1, left.phi2, left.isa, right.phi1, right.phi2, right.isa,
        ]
        .iter()
        .any(|value| matches!(value, LogicLevel::Floating | LogicLevel::Contention))
        {
            unresolved_rows += 1;
        }
        if !fields.is_empty() {
            differences += 1;
            if differences <= 16 {
                writeln!(
                    examples,
                    "row expected={} actual={}; fields={}; expected={left:?}; actual={right:?}",
                    expected_start + index,
                    actual_start + index,
                    fields.join(",")
                )
                .unwrap();
            }
        }
    }
    let matches = differences == 0 && expected_rows.len() == actual_rows.len();
    let mut report = format!(
        "HP67 ORDINAL IS/PHI COMPARISON\nresult={}\nexpected_start={expected_start} actual_start={actual_start}\nexpected_origin={} actual_origin={}\nexpected_rows={} actual_rows={} compared_rows={} differing_rows={differences}\ndriver_rows_compared={driver_rows} unresolved_rows={unresolved_rows}\n",
        if matches { "MATCH" } else { "MISMATCH" },
        expected_rows[0].tick, actual_rows[0].tick, expected_rows.len(), actual_rows.len(),
        expected_rows.len().min(actual_rows.len()),
    );
    report.push_str("scope=ordinal samples only; absolute tick origins ignored; no calibrated edge/timing claim\n");
    report.push_str(
        "missing driver columns exclude driver comparison; unresolved levels remain explicit\n",
    );
    for (name, trace) in [("expected", expected), ("actual", actual)] {
        if trace.provenance.is_empty() {
            writeln!(report, "{name}_provenance=UNSPECIFIED").unwrap();
        } else {
            for line in &trace.provenance {
                writeln!(report, "{name}_provenance={line}").unwrap();
            }
        }
    }
    report.push_str(&examples);
    Ok(BusTraceComparison { matches, report })
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::fetch::{StructuralBusSample, StructuralBusTrace};

    fn native() -> String {
        StructuralBusTrace {
            samples: (0..8)
                .map(|i| StructuralBusSample {
                    tick: i + 1,
                    transport_bit: (i / 4) as u8,
                    phi1: if i % 4 == 0 {
                        LogicLevel::Low
                    } else {
                        LogicLevel::High
                    },
                    phi2: if i % 4 == 2 {
                        LogicLevel::Low
                    } else {
                        LogicLevel::High
                    },
                    isa: LogicLevel::Low,
                    act_drive: Drive::HighZ,
                    rom_drive: Drive::HighZ,
                })
                .collect(),
        }
        .to_tsv()
    }

    #[test]
    fn m14y_native_roundtrip_and_explicit_origin_alignment() {
        let expected = import_bus_trace(&native()).unwrap();
        let mut actual = expected.clone();
        for row in &mut actual.rows {
            row.tick += 1000;
        }
        let result = compare_bus_traces(&expected, &actual, 0, 0).unwrap();
        assert!(result.matches);
        assert!(result.report.contains("driver_rows_compared=8"));
        actual.rows.insert(
            0,
            BusTraceRow {
                tick: 1000,
                ..actual.rows[0].clone()
            },
        );
        assert!(
            compare_bus_traces(&expected, &actual, 0, 1)
                .unwrap()
                .matches
        );
        assert!(
            !compare_bus_traces(&expected, &actual, 0, 0)
                .unwrap()
                .matches
        );
        assert!(compare_bus_traces(&expected, &actual, 8, 0).is_err());
    }

    #[test]
    fn m14y_reports_level_phase_bit_tick_driver_and_length_differences() {
        let expected = import_bus_trace(&native()).unwrap();
        for field in 0..6 {
            let mut actual = expected.clone();
            match field {
                0 => actual.rows[3].isa = LogicLevel::High,
                1 => actual.rows[3].phi1 = LogicLevel::Low,
                2 => actual.rows[3].phi2 = LogicLevel::Low,
                3 => actual.rows[3].transport_bit = 55,
                4 => actual.rows[3].tick += 1,
                _ => actual.rows[3].act_drive = Some(Drive::High),
            }
            let result = compare_bus_traces(&expected, &actual, 0, 0).unwrap();
            assert!(!result.matches);
            assert!(result.report.contains("row expected=3 actual=3"));
        }
        let mut short = expected.clone();
        short.rows.pop();
        assert!(!compare_bus_traces(&expected, &short, 0, 0).unwrap().matches);
    }

    #[test]
    fn m14y_pin_only_input_preserves_provenance_and_unknown_levels() {
        let input = format!("# normalized capture; source=synthetic regression\n{PIN_HEADER}\n100\t55\tHigh\tHigh\tFloating\n101\t0\tLow\tHigh\tContention\n");
        let trace = import_bus_trace(&input).unwrap();
        assert_eq!(trace.rows[0].act_drive, None);
        let result = compare_bus_traces(&trace, &trace, 0, 0).unwrap();
        assert!(result.matches);
        assert!(result
            .report
            .contains("driver_rows_compared=0 unresolved_rows=2"));
        assert!(result.report.contains("source=synthetic regression"));
        assert_eq!(
            import_bus_trace(&input.replace('\n', "\r\n")).unwrap(),
            trace
        );
    }

    #[test]
    fn m14y_rejects_malformed_or_ambiguous_input() {
        for input in [
            "",
            PIN_HEADER,
            "time,phi1,phi2,is\n0,1,1,0",
            "tick\ttransport_bit\tphi1\tphi2\tisa\n1\t56\tHigh\tHigh\tLow",
            "tick\ttransport_bit\tphi1\tphi2\tisa\n1\t0\t1\tHigh\tLow",
            "tick\ttransport_bit\tphi1\tphi2\tisa\n1\t0\tHigh\tHigh",
            "tick\ttransport_bit\tphi1\tphi2\tisa\n1\t0\tHigh\tHigh\tLow\n1\t0\tHigh\tHigh\tLow",
        ] {
            assert!(import_bus_trace(input).is_err(), "{input:?}");
        }
        assert!(import_bus_trace(&native().replace("HighZ", "Floating")).is_err());
    }
}
