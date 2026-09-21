use std::{
    env,
    hint::black_box,
    time::{Duration, Instant},
};

use hp67emu::machines::hp67::{
    run_structural_display_fetch_cycle, run_structural_fetch_cycle, ActArchitecturalState,
    ActSerialEndpoint, Hp67ArchitecturalMachine, Hp67ElectricalBackplane, Hp67RomWordSource,
    Rom0DisplayEndpoint, RomFetchEndpoint, BITS_PER_WORD, HP67_OBSERVED_WORD_TIME_US,
};

const DEFAULT_WORDS_PER_ROUND: usize = 5_000;
const DEFAULT_ROUNDS: usize = 7;
const DEFAULT_WARMUP_WORDS: usize = 250;
const CLOCK_EDGES_PER_BIT: u64 = 4;
const BENCH_ADDRESS: u16 = 0x07b;
const BENCH_EXECUTION_WORD: u16 = 0x11a;
const BENCH_FETCH_WORD: u16 = 0x04c;

struct BenchmarkRom;

impl Hp67RomWordSource for BenchmarkRom {
    fn read_word(&self, _address: u16) -> Option<u16> {
        Some(BENCH_FETCH_WORD)
    }
}

#[derive(Debug)]
struct BenchmarkStats {
    median: Duration,
    min: Duration,
    max: Duration,
}

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(default)
}

fn summarize(mut samples: Vec<Duration>) -> BenchmarkStats {
    samples.sort_unstable();
    BenchmarkStats {
        min: samples[0],
        median: samples[samples.len() / 2],
        max: samples[samples.len() - 1],
    }
}

fn measure_rounds<F>(rounds: usize, words: usize, mut run: F) -> BenchmarkStats
where
    F: FnMut(usize) -> u64,
{
    let mut samples = Vec::with_capacity(rounds);
    let mut checksum = 0u64;

    for _ in 0..rounds {
        let start = Instant::now();
        checksum ^= black_box(run(words));
        samples.push(start.elapsed());
    }

    black_box(checksum);
    summarize(samples)
}

fn us_per_word(duration: Duration, words: usize) -> f64 {
    duration.as_secs_f64() * 1_000_000.0 / words as f64
}

fn print_row(name: &str, stats: &BenchmarkStats, words: usize) {
    let median_us = us_per_word(stats.median, words);
    let min_us = us_per_word(stats.min, words);
    let max_us = us_per_word(stats.max, words);
    let words_per_second = 1_000_000.0 / median_us;
    let realtime_multiple = HP67_OBSERVED_WORD_TIME_US as f64 / median_us;

    println!(
        "{name:<42} {median_us:>12.3} {words_per_second:>14.0} {realtime_multiple:>12.2}x {min_us:>12.3} {max_us:>12.3}"
    );
}

fn raw_phi_stream(words: usize) -> u64 {
    let mut backplane = Hp67ElectricalBackplane::default();
    let edges_per_word = BITS_PER_WORD as u64 * CLOCK_EDGES_PER_BIT;
    let mut checksum = 0u64;

    for _ in 0..words {
        for _ in 0..edges_per_word {
            backplane.advance_clock_edge();
        }
        checksum ^= black_box(backplane.tick().get());
    }

    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

fn structural_fetch_stream(words: usize) -> u64 {
    let source = BenchmarkRom;
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(BENCH_ADDRESS);
    let mut rom = RomFetchEndpoint::default();
    let mut checksum = 0u64;
    let mut last_fetched = 0u16;

    for _ in 0..words {
        last_fetched =
            run_structural_fetch_cycle(&mut backplane, BENCH_ADDRESS, &mut act, &mut rom, &source)
                .expect("continuous structural fetch must complete");
        checksum = checksum.wrapping_add(last_fetched as u64);
    }

    assert_eq!(last_fetched, BENCH_FETCH_WORD);
    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

fn full_current_structural_stream(words: usize) -> u64 {
    let source = BenchmarkRom;
    let mut state = ActArchitecturalState::default();
    state.display_enable = true;
    for digit in 0..state.a.len() {
        state.a[digit] = (digit % 10) as u8;
        state.b[digit] = 0;
    }

    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(BENCH_ADDRESS);
    let mut rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut checksum = 0u64;
    let mut last_fetched = 0u16;

    for _ in 0..words {
        act.begin_execution(BENCH_EXECUTION_WORD, &state)
            .expect("serial execution must start");

        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            BENCH_ADDRESS,
            &state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("continuous display/fetch/execution word must complete");

        last_fetched = result.fetched_word;
        checksum = checksum
            .wrapping_add(result.fetched_word as u64)
            .wrapping_add(result.display_byte as u64)
            .wrapping_add(u64::from(result.rcd_falling));
    }

    assert_eq!(last_fetched, BENCH_FETCH_WORD);
    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

fn architectural_execution_stream(words: usize) -> u64 {
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut checksum = 0u64;

    for _ in 0..words {
        let execution = machine
            .execute_word(BENCH_EXECUTION_WORD)
            .expect("architectural benchmark word must execute");
        checksum = checksum
            .wrapping_add(execution.next_pc as u64)
            .wrapping_add(machine.act.state.c[0] as u64);
    }

    checksum
}

fn production_dual_path_stream(words: usize) -> u64 {
    let source = BenchmarkRom;
    let mut machine = Hp67ArchitecturalMachine::default();
    machine.act.state.display_enable = true;
    for digit in 0..machine.act.state.a.len() {
        machine.act.state.a[digit] = (digit % 10) as u8;
        machine.act.state.b[digit] = 0;
    }

    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(BENCH_ADDRESS);
    let mut rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut checksum = 0u64;
    let mut last_fetched = 0u16;

    for _ in 0..words {
        act.begin_execution(BENCH_EXECUTION_WORD, &machine.act.state)
            .expect("serial execution must start");

        let execution = machine
            .execute_word(BENCH_EXECUTION_WORD)
            .expect("architectural benchmark word must execute");

        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            BENCH_ADDRESS,
            &machine.act.state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("production-equivalent dual path must complete");

        last_fetched = result.fetched_word;
        checksum = checksum
            .wrapping_add(execution.next_pc as u64)
            .wrapping_add(result.fetched_word as u64)
            .wrapping_add(result.display_byte as u64)
            .wrapping_add(u64::from(result.rcd_falling));
    }

    assert_eq!(last_fetched, BENCH_FETCH_WORD);
    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

#[test]
#[ignore = "release-only wall-clock benchmark; run explicitly with --ignored --nocapture"]
fn hp67_electrical_realtime_benchmark() {
    let words = env_usize("HP67_BENCH_WORDS", DEFAULT_WORDS_PER_ROUND);
    let rounds = env_usize("HP67_BENCH_ROUNDS", DEFAULT_ROUNDS);
    let warmup_words = env_usize("HP67_BENCH_WARMUP_WORDS", DEFAULT_WARMUP_WORDS);

    black_box(raw_phi_stream(warmup_words));
    black_box(structural_fetch_stream(warmup_words));
    black_box(full_current_structural_stream(warmup_words));
    black_box(architectural_execution_stream(warmup_words));
    black_box(production_dual_path_stream(warmup_words));

    let raw_phi = measure_rounds(rounds, words, raw_phi_stream);
    let fetch = measure_rounds(rounds, words, structural_fetch_stream);
    let full = measure_rounds(rounds, words, full_current_structural_stream);
    let architectural = measure_rounds(rounds, words, architectural_execution_stream);
    let production = measure_rounds(rounds, words, production_dual_path_stream);

    let physical_word_us = HP67_OBSERVED_WORD_TIME_US as f64;
    let physical_words_per_second = 1_000_000.0 / physical_word_us;
    let physical_bits_per_second = physical_words_per_second * BITS_PER_WORD as f64;
    let topology_edges_per_second = physical_bits_per_second * CLOCK_EDGES_PER_BIT as f64;
    let physical_span = Duration::from_micros(HP67_OBSERVED_WORD_TIME_US * words as u64);

    println!();
    println!("HP-67 CURRENT ELECTRICAL/STRUCTURAL REALTIME BENCHMARK");
    println!("====================================================");
    println!(
        "Physical reference : ~{} us/word = {:.0} words/s = {:.0} bit-cells/s",
        HP67_OBSERVED_WORD_TIME_US, physical_words_per_second, physical_bits_per_second
    );
    println!(
        "PHI topology budget: {:.0} transitions/s (4 named transitions per bit-cell)",
        topology_edges_per_second
    );
    println!(
        "Measured stream    : {} words/round x {} rounds; each round represents {:.3} s of physical HP-67 time",
        words,
        rounds,
        physical_span.as_secs_f64()
    );
    println!();
    println!(
        "{:<42} {:>12} {:>14} {:>13} {:>12} {:>12}",
        "PATH", "MED us/word", "words/s", "vs hardware", "BEST us", "WORST us"
    );
    println!("{}", "-".repeat(111));
    print_row("PHI backplane / resolved clock nets", &raw_phi, words);
    print_row("IS ACT<->ROM structural fetch", &fetch, words);
    print_row("IS + ROM0 display + serial ACT execution", &full, words);
    print_row("architectural execution only", &architectural, words);
    print_row("production dual architectural + structural", &production, words);
    println!();
    println!(
        "Scope: current implementation only. The full row continuously executes all presently wired structural fidelity: 56 bit-cells/word, 4 PHI transitions/bit, resolved IS ownership, ACT->ROM 12-bit address, ROM->ACT 10-bit return, ROM0 display traffic, 15-slot display phase and serial ACT execution."
    );
    println!(
        "The production-dual row also includes the current instruction-boundary architectural execution that the live machine runs in parallel with serial execution. Not yet represented electrically: DATA transfers/RAM devices, PHI-relative IS/DATA launch/sample edges, electrically timed STR/RCD nets, exact PHI widths/dead time, and propagation delays. Therefore this measures realtime computational headroom, not final hardware-timing accuracy."
    );
}
