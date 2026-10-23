use super::*;
use std::sync::atomic::{AtomicU64, Ordering};
static NEXT: AtomicU64 = AtomicU64::new(0);
struct Temporary(std::path::PathBuf);
impl Temporary {
    fn new() -> Self { let path = std::env::temp_dir().join(format!("hp67-m15z-{}-{}", std::process::id(), NEXT.fetch_add(1, Ordering::Relaxed))); std::fs::create_dir(&path).unwrap(); Self(path) }
    fn file(&self, name: &str) -> String { self.0.join(name).to_str().unwrap().to_owned() }
}
impl Drop for Temporary { fn drop(&mut self) { let _ = std::fs::remove_dir_all(&self.0); } }
fn args(values: &[&str]) -> Vec<String> { values.iter().map(|value| (*value).to_owned()).collect() }
#[test]
fn m15z_cli_capture_audit_compare_and_no_overwrite_contract() {
    let temp = Temporary::new(); let input = temp.file("pins.tsv"); let audit = temp.file("audit.txt"); let compared = temp.file("comparison.txt");
    assert!(handles(Some("--trace-pins"))); assert!(!handles(Some("--trace-is")));
    assert_eq!(run(&args(&["--trace-pins", "16", &input])).unwrap(), 0);
    let original = std::fs::read(&input).unwrap();
    assert_eq!(run(&args(&["--audit-pins", &input, &audit])).unwrap(), 0); assert!(std::fs::read_to_string(&audit).unwrap().contains("UNCALIBRATED"));
    assert_eq!(run(&args(&["--compare-pins", &input, &input, "0", "0", "16", "IS,SYNC", &compared])).unwrap(), 0);
    assert!(run(&args(&["--trace-pins", "16", &input])).is_err()); assert!(run(&args(&["--audit-pins", &input, &input])).is_err()); assert_eq!(std::fs::read(&input).unwrap(), original);
    assert!(run(&args(&["--trace-pins", "0", &temp.file("zero.tsv")])).is_err()); assert!(run(&args(&["--trace-pins", "513", &temp.file("big.tsv")])).is_err()); assert!(run(&args(&["--compare-pins"])).is_err());
}
#[test]
fn m15z_cli_difference_and_unresolved_status_are_distinct() {
    use hp67emu::{emulation::LogicLevel, machines::hp67::{pin_trace::PinOwners, wiring::Hp67Driver}};
    let temp = Temporary::new(); let left = temp.file("left.tsv"); let right = temp.file("right.tsv"); let partial = temp.file("partial.tsv");
    let source = crate::hp67::capture_live_pin_tsv(2).unwrap(); write_new(&left, &source).unwrap(); let mut trace = import_pin_trace(&source).unwrap();
    let row = trace.rows.iter_mut().find(|row| row.at == 16 * 4 + 2).unwrap();
    let old = row.levels[Hp67Net::Isa.index()]; row.levels[Hp67Net::Isa.index()] = if old == Some(LogicLevel::High) { Some(LogicLevel::Low) } else { Some(LogicLevel::High) };
    row.owners[Hp67Net::Isa.index()] = Some(PinOwners { high: if old == Some(LogicLevel::High) { 0 } else { 1 << Hp67Driver::Act1820_2530.index() }, low: 0 });
    write_new(&right, &trace.to_tsv().unwrap()).unwrap(); assert_eq!(run(&args(&["--compare-pins", &left, &right, "0", "0", "1", "IS"])).unwrap(), 2);
    let row = trace.rows.iter_mut().find(|row| row.at == 16 * 4 + 2).unwrap(); row.levels[Hp67Net::Isa.index()] = None; row.owners[Hp67Net::Isa.index()] = None;
    write_new(&partial, &trace.to_tsv().unwrap()).unwrap(); assert_eq!(run(&args(&["--compare-pins", &left, &partial, "0", "0", "1", "IS"])).unwrap(), 3);
}
