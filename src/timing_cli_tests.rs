use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temporary(std::path::PathBuf);
impl Temporary {
    fn new() -> Self { let path = std::env::temp_dir().join(format!("hp67-m16a-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))); std::fs::create_dir(&path).unwrap(); Self(path) }
    fn file(&self, name: &str) -> String { self.0.join(name).to_str().unwrap().to_owned() }
}
impl Drop for Temporary { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
fn args(values: &[&str]) -> Vec<String> { values.iter().map(|value| (*value).to_owned()).collect() }
fn fixture() -> String {
    "# hp67-pins-v1 domain=ns source=synthetic first_bit=0\n# provenance=synthetic CLI phase margins; not hardware evidence\nat\torder\tPHI1\tPHI2\tIS\tDATA\tSYNC\n0\t0\t1\t1\t0\t0\t0\n5\t0\t1\t1\t1\t0\t0\n10\t0\t0\t1\t1\t0\t0\n12\t0\t1\t1\t1\t0\t0\n15\t0\t1\t0\t1\t0\t0\n17\t0\t1\t0\t0\t0\t0\n20\t0\t1\t1\t0\t0\t0\n25\t0\t0\t1\t0\t0\t0\n27\t0\t1\t1\t0\t0\t0\n30\t0\t1\t0\t0\t0\t0\n32\t0\t1\t0\t1\t0\t0\n35\t0\t1\t1\t1\t0\t0\n".into()
}
#[test]
fn m16a_timing_cli_keeps_offline_physical_audit_available_in_both_builds() {
    let temp = Temporary::new(); let input = temp.file("input.tsv"); let output = temp.file("report.txt"); let text = fixture(); write_new(&input, &text).unwrap();
    assert!(handles(Some("--timing-pins"))); assert_eq!(run(&args(&["--timing-pins", &input, "IS,SYNC", &output])).unwrap(), 0);
    let report = std::fs::read_to_string(&output).unwrap(); assert!(report.contains("input nanoseconds")); assert!(report.contains("setup_min=Some(7)")); assert!(report.contains("source=Synthetic"));
    assert!(run(&args(&["--timing-pins", &input, "IS", &input])).is_err()); assert_eq!(std::fs::read_to_string(&input).unwrap(), text);
    assert!(run(&args(&["--timing-pins", &input, "PHI1"])).is_err()); assert!(run(&args(&["--timing-pins", &input, "IS,IS"])).is_err()); assert!(run(&args(&["--timing-pins"])).is_err());
}
#[test]
fn m16a_timing_cli_distinguishes_missing_and_same_publication_edges() {
    use hp67emu::emulation::LogicLevel;
    let temp = Temporary::new(); let input = temp.file("missing.tsv"); let mut trace = import_pin_trace(&fixture()).unwrap();
    for row in &mut trace.rows { row.levels[Hp67Net::Data.index()] = None; }
    write_new(&input, &trace.to_tsv().unwrap()).unwrap(); assert_eq!(run(&args(&["--timing-pins", &input, "DATA"])).unwrap(), 3);
    let input = temp.file("coincident.tsv"); let mut trace = import_pin_trace(&fixture()).unwrap(); trace.rows.iter_mut().find(|row| row.at == 12).unwrap().levels[Hp67Net::Isa.index()] = Some(LogicLevel::Low);
    write_new(&input, &trace.to_tsv().unwrap()).unwrap(); assert_eq!(run(&args(&["--timing-pins", &input, "IS"])).unwrap(), 3);
}
#[cfg(not(feature = "pin-capture"))]
#[test]
fn m16a_default_capture_request_refuses_explicitly_without_creating_output() {
    let temp = Temporary::new(); let output = temp.file("capture.tsv");
    let error = run(&args(&["--trace-pins", "2", &output])).unwrap_err(); assert!(error.contains("--features pin-capture"), "{error}"); assert!(!std::path::Path::new(&output).exists());
}
#[cfg(feature = "pin-capture")]
#[test]
fn m16a_diagnostic_capture_still_feeds_phi_window_audit_through_cli() {
    let temp = Temporary::new(); let input = temp.file("capture.tsv"); let output = temp.file("report.txt");
    assert_eq!(run(&args(&["--trace-pins", "16", &input])).unwrap(), 0);
    assert_eq!(run(&args(&["--timing-pins", &input, "IS,SYNC", &output])).unwrap(), 0);
    let report = std::fs::read_to_string(&output).unwrap(); assert!(report.contains("UNCALIBRATED")); assert!(report.contains("phi1_low_with_changes=0"));
}
