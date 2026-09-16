from pathlib import Path

act_path = Path("src/machines/hp67/act.rs")
act = act_path.read_text()
old = '''impl Default for ActDisplayScanSequencer {
    fn default() -> Self {
        Self { scan_slot: 1 }
    }
}

impl ActDisplayScanSequencer {
    pub const fn scan_slot(&self) -> u8 {
'''
new = '''impl Default for ActDisplayScanSequencer {
    fn default() -> Self {
        Self::new()
    }
}

impl ActDisplayScanSequencer {
    pub const fn new() -> Self {
        Self { scan_slot: 1 }
    }

    pub const fn scan_slot(&self) -> u8 {
'''
if old not in act:
    raise SystemExit("ACT display scan Default anchor not found")
act_path.write_text(act.replace(old, new, 1))

fetch_path = Path("src/machines/hp67/fetch.rs")
fetch = fetch_path.read_text()
old = "display_scan: ActDisplayScanSequencer::default(),"
new = "display_scan: ActDisplayScanSequencer::new(),"
if old not in fetch:
    raise SystemExit("ACT serial endpoint display-scan constructor anchor not found")
fetch_path.write_text(fetch.replace(old, new, 1))
