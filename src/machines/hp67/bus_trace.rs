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

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct DecodedIsWord {
    pub first_row: usize,
    pub first_tick: u64,
    pub rom_address: u16,
    pub instruction: u16,
    pub display_byte: u8,
}

/// Decode an explicit range under the native four-transition observation profile.
/// This validates scaffold shape; it does not infer physical PHI sample edges.
pub fn decode_bus_words(
    trace: &ImportedBusTrace,
    start: usize,
    words: usize,
) -> Result<Vec<DecodedIsWord>, String> {
    use super::timing::{display_data_serial_bit, isa_window_for_bit, IsaWindow};
    if words == 0 {
        return Err("decode requires at least one complete word".to_owned());
    }
    let end = words
        .checked_mul(224)
        .and_then(|rows| start.checked_add(rows))
        .ok_or("decode range overflow")?;
    let rows = trace
        .rows
        .get(start..end)
        .ok_or("decode range outside samples")?;
    let mut decoded = Vec::with_capacity(words);
    for (word, samples) in rows.chunks_exact(224).enumerate() {
        let mut result = DecodedIsWord {
            first_row: start + word * 224,
            first_tick: samples[0].tick,
            rom_address: 0,
            instruction: 0,
            display_byte: 0,
        };
        for bit in 0..56usize {
            let cell = &samples[bit * 4..bit * 4 + 4];
            for (phase, row) in cell.iter().enumerate() {
                let expected_tick = result
                    .first_tick
                    .checked_add((bit * 4 + phase) as u64)
                    .ok_or("decode tick overflow")?;
                let phi1 = if phase == 0 {
                    LogicLevel::Low
                } else {
                    LogicLevel::High
                };
                let phi2 = if phase == 2 {
                    LogicLevel::Low
                } else {
                    LogicLevel::High
                };
                if row.tick != expected_tick
                    || row.transport_bit != bit as u8
                    || row.phi1 != phi1
                    || row.phi2 != phi2
                    || row.isa != cell[0].isa
                {
                    return Err(format!("word {word} bit {bit} phase {phase} row {}: invalid native observation profile",
                        result.first_row + bit * 4 + phase));
                }
            }
            let high = match cell[0].isa {
                LogicLevel::High => true,
                LogicLevel::Low => false,
                _ => return Err(format!("word {word} bit {bit}: unresolved IS")),
            };
            if high {
                if let Some(serial_bit) = display_data_serial_bit(bit as u8) {
                    result.display_byte |= 1u8 << serial_bit;
                }
                match isa_window_for_bit(bit as u8) {
                    IsaWindow::RomAddress { serial_bit } => {
                        result.rom_address |= 1u16 << serial_bit
                    }
                    IsaWindow::RomWord { serial_bit } => result.instruction |= 1u16 << serial_bit,
                    IsaWindow::Other => {}
                }
            }
        }
        decoded.push(result);
    }
    Ok(decoded)
}

/// Human-readable numeric bus words; no bank or execution semantics are inferred.
pub fn decode_bus_report(
    trace: &ImportedBusTrace,
    start: usize,
    words: usize,
) -> Result<String, String> {
    use std::fmt::Write;
    let decoded = decode_bus_words(trace, start, words)?;
    let mut output = format!(
        "HP67 STRUCTURAL IS WORD DECODE\nprofile=native ordinal four-transition scaffold; not physical sample-edge evidence\ndecoded_words={words} start_row={start} excluded_prefix_rows={start} excluded_suffix_rows={}\n",
        trace.rows.len() - start - words * 224,
    );
    output.push_str("ROM address is 12-bit bus address; bank unknown; instruction is fetched, not same-word execution\n");
    if trace.provenance.is_empty() {
        output.push_str("provenance=UNSPECIFIED\n");
    }
    for line in &trace.provenance {
        writeln!(output, "provenance={line}").unwrap();
    }
    output.push_str("word\tfirst_row\tfirst_tick\taddress_hex\taddress_octal\tinstruction_hex\tinstruction_octal\tdisplay_hex\n");
    for (index, word) in decoded.iter().enumerate() {
        writeln!(
            output,
            "{}\t{}\t{}\t{:03X}\t{:04o}\t{:03X}\t{:04o}\t{:02X}",
            index,
            word.first_row,
            word.first_tick,
            word.rom_address,
            word.rom_address,
            word.instruction,
            word.instruction,
            word.display_byte
        )
        .unwrap();
    }
    Ok(output)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::fetch::{StructuralBusSample, StructuralBusTrace};

    fn full_native_trace() -> ImportedBusTrace {
        use crate::machines::hp67::{
            run_structural_display_fetch_cycle_traced, ActArchitecturalState, ActSerialEndpoint,
            Hp67ElectricalBackplane, Hp67RomWordSource, Rom0DisplayEndpoint, RomFetchEndpoint,
        };
        struct Source;
        impl Hp67RomWordSource for Source {
            fn read_word(&self, address: u16) -> Option<u16> {
                match address {
                    0xabc => Some(0x355),
                    0x123 => Some(0x04c),
                    _ => None,
                }
            }
        }
        let mut backplane = Hp67ElectricalBackplane::default();
        let mut act = ActSerialEndpoint::new(0);
        let mut rom = RomFetchEndpoint::default();
        let mut rom0 = Rom0DisplayEndpoint::default();
        let mut trace = StructuralBusTrace::default();
        for address in [0xabc, 0x123] {
            run_structural_display_fetch_cycle_traced(
                &mut backplane,
                address,
                &ActArchitecturalState::default(),
                &mut act,
                &mut rom,
                &mut rom0,
                &Source,
                &mut trace,
            )
            .unwrap();
        }
        import_bus_trace(&trace.to_tsv()).unwrap()
    }

    #[test]
    fn m14z_decodes_resolved_transport_and_preserves_fetched_word_scope() {
        let trace = full_native_trace();
        let words = decode_bus_words(&trace, 0, 2).unwrap();
        assert_eq!(words[0].rom_address, 0xabc);
        assert_eq!(words[0].instruction, 0x355);
        assert_eq!(words[0].display_byte, 0x20);
        assert_eq!(words[1].rom_address, 0x123);
        assert_eq!(words[1].instruction, 0x04c);
        assert_eq!(words[1].first_row, 224);
        assert_eq!(words[1].first_tick, 225);
        let report = decode_bus_report(&trace, 224, 1).unwrap();
        assert!(report.contains(
            "decoded_words=1 start_row=224 excluded_prefix_rows=224 excluded_suffix_rows=0"
        ));
        assert!(report.contains("bank unknown"));
        assert!(report.contains("instruction is fetched"));
    }

    #[test]
    fn m14z_refuses_partial_unresolved_shifted_or_non_native_samples() {
        let trace = full_native_trace();
        assert!(decode_bus_words(&trace, 0, 0).is_err());
        assert!(decode_bus_words(&trace, 0, usize::MAX).is_err());
        assert!(decode_bus_words(&trace, 1, 2).is_err());
        assert!(decode_bus_words(&trace, 1, 1).is_err());
        for mutation in 0..4 {
            let mut invalid = trace.clone();
            match mutation {
                0 => invalid.rows[68].isa = LogicLevel::Floating,
                1 => invalid.rows[68].phi1 = LogicLevel::High,
                2 => invalid.rows[68].transport_bit = 18,
                _ => invalid.rows[68].tick += 1,
            }
            assert!(decode_bus_words(&invalid, 0, 1).is_err());
        }
        let mut invalid = trace.clone();
        for row in &mut invalid.rows[68..72] {
            row.isa = LogicLevel::Contention;
        }
        assert!(decode_bus_words(&invalid, 0, 1)
            .unwrap_err()
            .contains("unresolved IS"));
    }

    #[test]
    fn m14z_stable_bit_change_is_decoded_instead_of_hidden() {
        let mut trace = full_native_trace();
        for row in &mut trace.rows[68..72] {
            row.isa = if row.isa == LogicLevel::High {
                LogicLevel::Low
            } else {
                LogicLevel::High
            };
        }
        let word = decode_bus_words(&trace, 0, 1).unwrap()[0];
        assert_eq!(word.rom_address, 0xabc ^ 2);
        assert_eq!(word.instruction, 0x355);
    }

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
