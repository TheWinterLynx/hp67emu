use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temporary(std::path::PathBuf);
impl Temporary {
    fn new() -> Self {
        let path = std::env::temp_dir().join(format!(
            "hp67-m16l-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        std::fs::create_dir(&path).unwrap();
        Self(path)
    }
    fn file(&self, name: &str) -> String {
        self.0.join(name).to_str().unwrap().to_owned()
    }
}
impl Drop for Temporary {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
fn args(values: &[&str]) -> Vec<String> {
    values.iter().map(|s| (*s).to_owned()).collect()
}
#[test]
fn m16l_integrity_cli_handles_unknown_ownership_and_preserves_inputs_and_outputs() {
    let temp = Temporary::new();
    let input = temp.file("physical.tsv");
    let output = temp.file("report.txt");
    let text = "# hp67-pins-v1 domain=ns source=synthetic first_bit=0\n# provenance=synthetic sparse fixture, not real measurement\nat\torder\tPHI1\tPHI2\n0\t0\t1\t1\n10\t0\t0\t1\n20\t0\t1\t1\n30\t0\t1\t0\n40\t0\t1\t1\n";
    write_new(&input, text).unwrap();
    assert!(handles(Some("--integrity-pins")));
    assert_eq!(
        run(&args(&["--integrity-pins", &input, &output])).unwrap(),
        3
    );
    let report = std::fs::read_to_string(&output).unwrap();
    assert!(report.contains("scoped_exit=3"));
    assert!(report.contains("SOURCE-BLOCKED"));
    assert!(run(&args(&["--integrity-pins", &input, &output])).is_err());
    assert!(run(&args(&["--integrity-pins", &input, &input])).is_err());
    assert_eq!(std::fs::read_to_string(&input).unwrap(), text);
    assert_eq!(std::fs::read_to_string(&output).unwrap(), report);
    assert!(run(&args(&["--integrity-pins"])).is_err());
}
#[cfg(feature = "pin-capture")]
#[test]
fn m16l_integrity_cli_checks_real_firmware_capture_and_distinguishes_fault_from_missing_observation(
) {
    use hp67emu::machines::hp67::Hp67Driver;
    let temp = Temporary::new();
    let input = temp.file("live.tsv");
    let bad = temp.file("fault.tsv");
    let partial = temp.file("partial.tsv");
    let source = crate::hp67::capture_live_pin_tsv(16).unwrap();
    write_new(&input, &source).unwrap();
    assert_eq!(run(&args(&["--integrity-pins", &input])).unwrap(), 0);
    let mut trace = import_pin_trace(&source).unwrap();
    let row = trace.rows.iter_mut().find(|r| r.at == 2).unwrap();
    let owner = row.owners[Hp67Net::PgmSw.index()].as_mut().unwrap();
    owner.high |= 1 << Hp67Driver::CardReaderController1820_1751.index();
    write_new(&bad, &trace.to_tsv().unwrap()).unwrap();
    assert_eq!(run(&args(&["--integrity-pins", &bad])).unwrap(), 2);
    // A legacy capture with no MOTOR observation is unknown, not motor-off.
    let mut trace = import_pin_trace(&source).unwrap();
    for row in &mut trace.rows {
        row.levels[Hp67Net::Motor.index()] = None;
        row.owners[Hp67Net::Motor.index()] = None;
    }
    write_new(&partial, &trace.to_tsv().unwrap()).unwrap();
    assert_eq!(run(&args(&["--integrity-pins", &partial])).unwrap(), 3);
    assert_eq!(std::fs::read_to_string(&input).unwrap(), source);
}
