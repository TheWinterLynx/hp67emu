//! Native offline pin diagnostics. New outputs use create_new; inputs survive.
use hp67emu::machines::hp67::{
    pin_trace::{
        analyze_pin_trace, compare_pin_traces, import_pin_trace, Hp67PinTrace, MAX_PIN_BYTES,
        PIN_NAMES,
    },
    wiring::Hp67Net,
};
use std::{
    fs::{File, OpenOptions},
    io::{Read, Write},
    path::Path,
};
fn read(path: &str) -> Result<Hp67PinTrace, String> {
    let file = File::open(path).map_err(|error| format!("{path}: {error}"))?;
    let mut text = String::new();
    file.take(MAX_PIN_BYTES as u64 + 1)
        .read_to_string(&mut text)
        .map_err(|error| format!("{path}: {error}"))?;
    import_pin_trace(&text).map_err(|error| format!("{path}: {error}"))
}
fn write_new(path: &str, content: &str) -> Result<(), String> {
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(path)
        .map_err(|error| format!("new output {path}: {error}"))?;
    if let Err(error) = file.write_all(content.as_bytes()) {
        drop(file);
        let _ = std::fs::remove_file(path);
        return Err(format!("write {path}: {error}"));
    }
    Ok(())
}
fn report(output: Option<&String>, content: &str) -> Result<(), String> {
    if let Some(path) = output {
        write_new(path, content)
    } else {
        print!("{content}");
        Ok(())
    }
}
pub fn handles(command: Option<&str>) -> bool {
    matches!(
        command,
        Some("--trace-pins" | "--audit-pins" | "--compare-pins")
    )
}
pub fn run(args: &[String]) -> Result<i32, String> {
    match args.first().map(String::as_str) {
        Some("--trace-pins") if args.len() == 3 => {
            if Path::new(&args[2]).exists() { return Err("trace output already exists".into()); }
            let words = args[1].parse::<usize>().map_err(|_| "invalid capture word count")?;
            let text = crate::hp67::capture_live_pin_tsv(words)?; write_new(&args[2], &text)?; Ok(0)
        },
        Some("--audit-pins") if (2..=3).contains(&args.len()) => {
            let trace = read(&args[1])?; let analysis = analyze_pin_trace(&trace)?; report(args.get(2), &analysis.report(&trace))?; Ok(0)
        },
        Some("--compare-pins") if (7..=8).contains(&args.len()) => {
            let left = read(&args[1])?; let right = read(&args[2])?;
            let lw = args[3].parse::<u64>().map_err(|_| "invalid left word index")?; let rw = args[4].parse::<u64>().map_err(|_| "invalid right word index")?; let count = args[5].parse::<usize>().map_err(|_| "invalid comparison word count")?;
            let nets: Result<Vec<_>, String> = args[6].split(',').map(|name| PIN_NAMES.iter().position(|pin| *pin == name).map(|index| Hp67Net::ALL[index]).ok_or_else(|| format!("unknown comparison pin {name:?}"))).collect();
            let result = compare_pin_traces(&left, &right, lw, rw, count, &nets?)?;
            let text = format!("left={} source={:?} domain={:?} provenance={} word={}\nright={} source={:?} domain={:?} provenance={} word={}\nwords={} pins={}\n{}", args[1], left.source, left.domain, left.provenance, lw, args[2], right.source, right.domain, right.provenance, rw, count, args[6], result.report());
            report(args.get(7), &text)?; Ok(result.exit_code())
        },
        _ => Err("usage: --trace-pins <1..512 words> <new.tsv> | --audit-pins <input.tsv> [new-report.txt] | --compare-pins <left.tsv> <right.tsv> <left-word> <right-word> <1..512 words> <IS,SYNC,...> [new-report.txt]".into()),
    }
}

#[cfg(test)]
#[path = "pin_trace_cli_tests.rs"]
mod tests;
