//! Clock topology, measured interval summaries and explicit word alignment.
use super::*;
use std::collections::BTreeMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinCell {
    pub word: u64,
    pub bit: u8,
    pub at: u64,
    pub levels: [Option<LogicLevel>; Hp67Net::COUNT],
    pub owners: [Option<PinOwners>; Hp67Net::COUNT],
}
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct PinWord {
    pub address: u16, pub address_mask: u16,
    pub opcode: u16, pub opcode_mask: u16,
    pub qualifier: Option<bool>, pub qualifier_conflict: bool,
    pub data: u64, pub data_mask: u64,
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinAnalysis {
    pub cells: Vec<PinCell>,
    pub words: BTreeMap<u64, PinWord>,
    pub phase_intervals: [Vec<u64>; 4],
    pub low_pulses: [Vec<u64>; Hp67Net::COUNT],
    pub str_cells: [usize; 56],
    pub rcd_cells: [usize; 56],
    pub coincident_changes: usize,
    pub trailing_phase: usize,
}
fn binary(level: Option<LogicLevel>) -> Option<bool> {
    match level { Some(LogicLevel::High) => Some(true), Some(LogicLevel::Low) => Some(false), _ => None }
}
fn clocks(row: &PinRow) -> Result<(bool, bool), String> {
    Ok((binary(row.levels[Hp67Net::Phi1.index()]).ok_or_else(|| format!("unresolved PHI1 at {}:{}", row.at, row.order))?, binary(row.levels[Hp67Net::Phi2.index()]).ok_or_else(|| format!("unresolved PHI2 at {}:{}", row.at, row.order))?))
}
pub fn analyze_pin_trace(trace: &Hp67PinTrace) -> Result<PinAnalysis, String> {
    if trace.overflowed { return Err("cannot analyze overflowed capture".into()); }
    if trace.first_bit >= 56 || trace.rows.len() < 2 || trace.rows.len() > MAX_PIN_ROWS { return Err("invalid/empty pin capture".into()); }
    if clocks(&trace.rows[0])? != (true, true) { return Err("capture must start with both PHI pins high before PHI1 falling".into()); }
    let mut result = PinAnalysis { cells: Vec::new(), words: BTreeMap::new(), phase_intervals: std::array::from_fn(|_| Vec::new()), low_pulses: std::array::from_fn(|_| Vec::new()), str_cells: [0; 56], rcd_cells: [0; 56], coincident_changes: 0, trailing_phase: 0 };
    let mut expected = 0usize; let mut last_edge: Option<(usize, u64)> = None; let mut cell_index = 0u64;
    let mut low_start = [None; Hp67Net::COUNT];
    for pair in trace.rows.windows(2) {
        let previous = &pair[0]; let row = &pair[1];
        if (row.at, row.order) <= (previous.at, previous.order) { return Err("non-monotonic pin observations".into()); }
        let old = clocks(previous)?; let new = clocks(row)?;
        if new == (false, false) { return Err(format!("overlapping PHI low pulses at {}", row.at)); }
        let coordinate = u64::from(trace.first_bit) + cell_index;
        let bit = (coordinate % 56) as u8;
        for net in Hp67Net::ALL {
            let index = net.index();
            match (binary(previous.levels[index]), binary(row.levels[index])) {
                (Some(true), Some(false)) => {
                    low_start[index] = Some(row.at);
                    if net == Hp67Net::Str { result.str_cells[usize::from(bit)] += 1; }
                    if net == Hp67Net::Rcd { result.rcd_cells[usize::from(bit)] += 1; }
                },
                (Some(false), Some(true)) => { if let Some(start) = low_start[index].take() { result.low_pulses[index].push(row.at - start); } },
                (None, _) | (_, None) => low_start[index] = None,
                _ => {},
            }
        }
        if old == new { continue; }
        let edge = match (old, new) {
            ((true, true), (false, true)) => 0,
            ((false, true), (true, true)) => 1,
            ((true, true), (true, false)) => 2,
            ((true, false), (true, true)) => 3,
            _ => return Err(format!("ambiguous/simultaneous PHI changes at {}", row.at)),
        };
        if edge != expected { return Err(format!("PHI order at {}: expected transition {expected}, observed {edge}", row.at)); }
        if let Some((prior, at)) = last_edge {
            let interval = row.at.checked_sub(at).filter(|delta| *delta > 0).ok_or("zero/backwards PHI interval; edge order is unresolved")?;
            result.phase_intervals[prior].push(interval);
        }
        if Hp67Net::ALL.into_iter().any(|net| !matches!(net, Hp67Net::Phi1 | Hp67Net::Phi2) && row.levels[net.index()] != previous.levels[net.index()]) { result.coincident_changes += 1; }
        if edge == 1 { result.cells.push(PinCell { word: coordinate / 56, bit, at: row.at, levels: row.levels, owners: row.owners }); }
        if edge == 3 { cell_index += 1; }
        expected = (edge + 1) % 4; last_edge = Some((edge, row.at));
    }
    result.trailing_phase = expected;
    if result.cells.is_empty() { return Err("capture contains no PHI1-rising sample checkpoint".into()); }
    for cell in &result.cells {
        let word = result.words.entry(cell.word).or_default();
        let is = binary(cell.levels[Hp67Net::Isa.index()]);
        if (16..=27).contains(&cell.bit) { if let Some(value) = is { word.address_mask |= 1 << (cell.bit - 16); if value { word.address |= 1 << (cell.bit - 16); } } }
        if (46..=55).contains(&cell.bit) {
            if let Some(value) = is { word.opcode_mask |= 1 << (cell.bit - 46); if value { word.opcode |= 1 << (cell.bit - 46); } }
            if let Some(value) = binary(cell.levels[Hp67Net::Sync.index()]) {
                if word.qualifier.is_some_and(|old| old != value) { word.qualifier_conflict = true; }
                word.qualifier = Some(value);
            } else { word.qualifier_conflict = true; }
        }
        // DATA bit 0 is b2; b0/b1 complete the preceding word's frame.
        let data_word = if cell.bit >= 2 { Some((cell.word, cell.bit - 2)) } else { cell.word.checked_sub(1).map(|word| (word, cell.bit + 54)) };
        if let (Some((index, serial)), Some(value)) = (data_word, binary(cell.levels[Hp67Net::Data.index()])) {
            let frame = result.words.entry(index).or_default(); frame.data_mask |= 1 << serial; if value { frame.data |= 1 << serial; }
        }
    }
    Ok(result)
}
fn statistics(values: &[u64]) -> String {
    if values.is_empty() { return "n=0 (not observed)".into(); }
    let mut sorted = values.to_vec(); sorted.sort_unstable();
    format!("n={} min={} median={} max={}", sorted.len(), sorted[0], sorted[sorted.len() / 2], sorted[sorted.len() - 1])
}
impl PinAnalysis {
    pub fn report(&self, trace: &Hp67PinTrace) -> String {
        use std::fmt::Write;
        let units = match trace.domain { PinTimeDomain::Ordinal => "ordinal ticks (UNCALIBRATED)", PinTimeDomain::Nanoseconds => "ns (input timestamps; source claim not independently verified)" };
        let mut out = format!("HP67 PIN AUDIT v1\nsource={:?}\nprovenance={}\nfirst_bit={} (explicit anchor)\ninterval_units={units}\nsample_checkpoint=PHI1 rising (provisional policy, not silicon latch proof)\nrows={} sampled_cells={} trailing_phase={} coincident_clock_and_pin_changes={}\n", trace.source, trace.provenance, trace.first_bit, trace.rows.len(), self.cells.len(), self.trailing_phase, self.coincident_changes);
        for (name, values) in ["PHI1 low", "dead time PHI1->PHI2", "PHI2 low", "dead time PHI2->PHI1"].into_iter().zip(&self.phase_intervals) { writeln!(out, "{name}: {}", statistics(values)).unwrap(); }
        for net in [Hp67Net::Str, Hp67Net::Rcd, Hp67Net::Sync] { writeln!(out, "{} low pulses: {}", PIN_NAMES[net.index()], statistics(&self.low_pulses[net.index()])).unwrap(); }
        for (name, cells) in [("STR falling cells", &self.str_cells), ("RCD falling cells", &self.rcd_cells)] {
            write!(out, "{name}:").unwrap(); for (bit, count) in cells.iter().enumerate().filter(|(_, count)| **count != 0) { write!(out, " b{bit}={count}").unwrap(); } out.push('\n');
        }
        for (index, word) in &self.words {
            writeln!(out, "word={index} address={:03x} address_mask={:03x} opcode={:03x} opcode_mask={:03x} qualifier={:?} qualifier_conflict={} DATA={:014x} DATA_mask={:014x}", word.address, word.address_mask, word.opcode, word.opcode_mask, word.qualifier, word.qualifier_conflict, word.data, word.data_mask).unwrap();
        }
        out.push_str("No timing profile is installed. No analog delay, DATA polarity or unobserved edge is inferred.\n"); out
    }
}
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PinComparison {
    pub compared: usize, pub differences: usize, pub unresolved: usize,
    pub owner_comparisons: usize, pub owner_differences: usize,
    pub examples: Vec<String>,
}
impl PinComparison {
    /// 0: resolved scoped equality; 2: difference; 3: incomplete observations.
    pub const fn exit_code(&self) -> i32 { if self.differences != 0 || self.owner_differences != 0 { 2 } else if self.unresolved != 0 { 3 } else { 0 } }
    pub fn report(&self) -> String {
        format!("HP67 PIN COMPARE v1\nscope=explicit word indices, PHI1-rising observations; physical durations NOT compared\ncompared={} differences={} unresolved={} owner_comparisons={} owner_differences={}\nexit={}\n{}\n", self.compared, self.differences, self.unresolved, self.owner_comparisons, self.owner_differences, self.exit_code(), self.examples.join("\n"))
    }
}
pub fn compare_pin_traces(left: &Hp67PinTrace, right: &Hp67PinTrace, left_word: u64, right_word: u64, count: usize, nets: &[Hp67Net]) -> Result<PinComparison, String> {
    if count == 0 || count > MAX_CAPTURE_WORDS || nets.is_empty() { return Err("comparison requires 1..512 words and explicit pins".into()); }
    for (index, net) in nets.iter().enumerate() { if nets[..index].contains(net) { return Err("duplicate comparison pin".into()); } }
    let l = analyze_pin_trace(left)?; let r = analyze_pin_trace(right)?;
    let mut result = PinComparison { compared: 0, differences: 0, unresolved: 0, owner_comparisons: 0, owner_differences: 0, examples: Vec::new() };
    for offset in 0..count {
        let lw = left_word.checked_add(offset as u64).ok_or("word range overflow")?; let rw = right_word.checked_add(offset as u64).ok_or("word range overflow")?;
        let lc = &l.cells[l.cells.partition_point(|cell| cell.word < lw)..l.cells.partition_point(|cell| cell.word <= lw)];
        let rc = &r.cells[r.cells.partition_point(|cell| cell.word < rw)..r.cells.partition_point(|cell| cell.word <= rw)];
        if lc.len() != 56 || rc.len() != 56 { return Err(format!("partial/missing word: left={lw}/{} right={rw}/{} cells", lc.len(), rc.len())); }
        for (a, b) in lc.iter().zip(rc) {
            if a.bit != b.bit { return Err("word bit alignment mismatch".into()); }
            for net in nets {
                let index = net.index(); let av = binary(a.levels[index]); let bv = binary(b.levels[index]);
                if av.is_none() || bv.is_none() { result.unresolved += 1; }
                else { result.compared += 1; if av != bv { result.differences += 1; if result.examples.len() < 16 { result.examples.push(format!("word+{offset} b{} {}: left={av:?}@{} right={bv:?}@{}", a.bit, PIN_NAMES[index], a.at, b.at)); } } }
                if let (Some(x), Some(y)) = (a.owners[index], b.owners[index]) { result.owner_comparisons += 1; if x != y { result.owner_differences += 1; if result.examples.len() < 16 { result.examples.push(format!("word+{offset} b{} {} ownership: left={x:?} right={y:?}", a.bit, PIN_NAMES[index])); } } }
            }
        }
    }
    Ok(result)
}
