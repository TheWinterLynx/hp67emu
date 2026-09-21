use std::{
    env,
    hint::black_box,
    time::{Duration, Instant},
};

use hp67emu::emulation::{Drive, LogicLevel};
use hp67emu::machines::hp67::{
    run_structural_display_fetch_cycle, run_structural_fetch_cycle, ActArchitecturalState,
    ActSerialEndpoint, FetchPipelineLatch, Hp67ArchitecturalMachine, Hp67Driver,
    Hp67ElectricalBackplane, Hp67ElectricalFabric, Hp67Firmware, Hp67Net, Hp67RomWordSource,
    Rom0DisplayEndpoint, RomFetchEndpoint, BITS_PER_WORD, HP67_OBSERVED_WORD_TIME_US,
};

const DEFAULT_WORDS_PER_ROUND: usize = 50_000;
const DEFAULT_ROUNDS: usize = 7;
const DEFAULT_WARMUP_WORDS: usize = 2_500;
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

fn measure_phi_rounds_interleaved(rounds: usize, words: usize) -> [BenchmarkStats; 3] {
    let paths: [fn(usize) -> u64; 3] = [
        raw_phi_stream,
        stage_commit_phi_stream,
        staged_phi_scheduler_stream,
    ];
    let mut samples: [Vec<Duration>; 3] = std::array::from_fn(|_| Vec::with_capacity(rounds));
    let mut checksum = 0u64;

    for round in 0..rounds {
        for offset in 0..paths.len() {
            let index = (round + offset) % paths.len();
            let start = Instant::now();
            checksum ^= black_box(paths[index](words));
            samples[index].push(start.elapsed());
        }
    }

    black_box(checksum);
    samples.map(summarize)
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
            black_box(backplane.level(Hp67Net::Phi1));
            black_box(backplane.level(Hp67Net::Phi2));
            backplane.advance_clock_edge();
        }
        checksum ^= black_box(backplane.tick().get());
    }

    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

fn stage_commit_phi_stream(words: usize) -> u64 {
    let mut fabric = Hp67ElectricalFabric::default();
    let edges_per_word = BITS_PER_WORD as u64 * CLOCK_EDGES_PER_BIT;
    let mut checksum = 0u64;

    for _ in 0..words {
        for _ in 0..edges_per_word {
            black_box(fabric.level(Hp67Net::Phi1));
            black_box(fabric.level(Hp67Net::Phi2));

            match (fabric.tick().get() + 1) & 0b11 {
                1 => fabric.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::Low),
                2 => fabric.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::High),
                3 => fabric.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::Low),
                _ => fabric.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::High),
            }

            fabric
                .commit_staged()
                .expect("dense PHI stage/commit must remain contention-free");
        }
        checksum ^= black_box(fabric.tick().get());
    }

    checksum
}

fn staged_phi_scheduler_stream(words: usize) -> u64 {
    let mut fabric = Hp67ElectricalFabric::default();
    let edges_per_word = BITS_PER_WORD as u64 * CLOCK_EDGES_PER_BIT;
    let mut checksum = 0u64;

    for _ in 0..words {
        for _ in 0..edges_per_word {
            {
                let (snapshot, mut stager) = fabric
                    .begin_evaluation()
                    .expect("dense scheduler snapshot must be contention-free");
                black_box(snapshot.level(Hp67Net::Phi1));
                black_box(snapshot.level(Hp67Net::Phi2));

                match (snapshot.tick().get() + 1) & 0b11 {
                    1 => stager.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::Low),
                    2 => stager.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::High),
                    3 => stager.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::Low),
                    _ => stager.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::High),
                }
            }

            fabric
                .commit_staged()
                .expect("dense PHI scheduling must remain contention-free");
        }
        checksum ^= black_box(fabric.tick().get());
    }

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

fn firmware_production_stream(words: usize) -> u64 {
    let source = Hp67Firmware::default();
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(0);
    let mut rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut checksum = 0u64;

    for _ in 0..words {
        pipeline.begin_cycle();

        if let Some(word) = pipeline.executing_word() {
            act.begin_execution(word, &machine.act.state)
                .expect("firmware serial execution must start");
            let execution = machine
                .execute_word(word)
                .expect("versioned HP-67 firmware word must execute");
            checksum = checksum.wrapping_add(execution.next_pc as u64);
        }

        let bank = machine.prepare_hp67_fetch();
        source.select_bank(bank);
        let address = machine.pc();
        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            address,
            &machine.act.state,
            &mut act,
            &mut rom,
            &mut rom0,
            &source,
        )
        .expect("continuous real-firmware production path must complete");
        pipeline.complete_cycle(result.fetched_word);
        checksum = checksum
            .wrapping_add(result.fetched_word as u64)
            .wrapping_add(result.display_byte as u64)
            .wrapping_add(u64::from(result.rcd_falling));
    }

    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PhiTraceSample {
    tick: u64,
    phi1: LogicLevel,
    phi2: LogicLevel,
}

fn raw_phi_trace(edges: usize) -> Vec<PhiTraceSample> {
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut trace = Vec::with_capacity(edges + 1);
    trace.push(PhiTraceSample {
        tick: backplane.tick().get(),
        phi1: backplane.level(Hp67Net::Phi1),
        phi2: backplane.level(Hp67Net::Phi2),
    });

    for _ in 0..edges {
        backplane.advance_clock_edge();
        trace.push(PhiTraceSample {
            tick: backplane.tick().get(),
            phi1: backplane.level(Hp67Net::Phi1),
            phi2: backplane.level(Hp67Net::Phi2),
        });
    }

    trace
}

fn stage_commit_phi_trace(edges: usize) -> Vec<PhiTraceSample> {
    let mut fabric = Hp67ElectricalFabric::default();
    let mut trace = Vec::with_capacity(edges + 1);
    trace.push(PhiTraceSample {
        tick: fabric.tick().get(),
        phi1: fabric.level(Hp67Net::Phi1),
        phi2: fabric.level(Hp67Net::Phi2),
    });

    for _ in 0..edges {
        match (fabric.tick().get() + 1) & 0b11 {
            1 => fabric.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::Low),
            2 => fabric.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::High),
            3 => fabric.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::Low),
            _ => fabric.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::High),
        }
        fabric
            .commit_staged()
            .expect("dense PHI stage/commit trace must remain contention-free");
        trace.push(PhiTraceSample {
            tick: fabric.tick().get(),
            phi1: fabric.level(Hp67Net::Phi1),
            phi2: fabric.level(Hp67Net::Phi2),
        });
    }

    trace
}

fn staged_phi_trace(edges: usize) -> Vec<PhiTraceSample> {
    let mut fabric = Hp67ElectricalFabric::default();
    let mut trace = Vec::with_capacity(edges + 1);
    trace.push(PhiTraceSample {
        tick: fabric.tick().get(),
        phi1: fabric.level(Hp67Net::Phi1),
        phi2: fabric.level(Hp67Net::Phi2),
    });

    for _ in 0..edges {
        {
            let (snapshot, mut stager) = fabric
                .begin_evaluation()
                .expect("dense PHI trace snapshot must be contention-free");
            match (snapshot.tick().get() + 1) & 0b11 {
                1 => stager.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::Low),
                2 => stager.stage_drive(Hp67Net::Phi1, Hp67Driver::Act1820_2530, Drive::High),
                3 => stager.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::Low),
                _ => stager.stage_drive(Hp67Net::Phi2, Hp67Driver::Act1820_2530, Drive::High),
            }
        }
        fabric
            .commit_staged()
            .expect("dense staged PHI trace must remain contention-free");
        trace.push(PhiTraceSample {
            tick: fabric.tick().get(),
            phi1: fabric.level(Hp67Net::Phi1),
            phi2: fabric.level(Hp67Net::Phi2),
        });
    }

    trace
}

#[test]
fn matched_phi_paths_preserve_identical_trace_and_tick_progression() {
    let edges = BITS_PER_WORD as usize * CLOCK_EDGES_PER_BIT as usize * 3;
    let raw = raw_phi_trace(edges);
    let stage_commit = stage_commit_phi_trace(edges);
    let staged = staged_phi_trace(edges);

    assert_eq!(stage_commit, raw);
    assert_eq!(staged, raw);
    assert_eq!(
        raw.last().expect("PHI trace must contain its final sample").tick,
        edges as u64
    );
}

#[test]
#[ignore = "release-only wall-clock benchmark; run explicitly with --ignored --nocapture"]
fn hp67_electrical_realtime_benchmark() {
    let words = env_usize("HP67_BENCH_WORDS", DEFAULT_WORDS_PER_ROUND);
    let rounds = env_usize("HP67_BENCH_ROUNDS", DEFAULT_ROUNDS);
    let warmup_words = env_usize("HP67_BENCH_WARMUP_WORDS", DEFAULT_WARMUP_WORDS);

    black_box(raw_phi_stream(warmup_words));
    black_box(stage_commit_phi_stream(warmup_words));
    black_box(staged_phi_scheduler_stream(warmup_words));
    black_box(structural_fetch_stream(warmup_words));
    black_box(full_current_structural_stream(warmup_words));
    black_box(architectural_execution_stream(warmup_words));
    black_box(production_dual_path_stream(warmup_words));
    black_box(firmware_production_stream(warmup_words));

    let [raw_phi, stage_commit_phi, staged_phi] = measure_phi_rounds_interleaved(rounds, words);
    let fetch = measure_rounds(rounds, words, structural_fetch_stream);
    let full = measure_rounds(rounds, words, full_current_structural_stream);
    let architectural = measure_rounds(rounds, words, architectural_execution_stream);
    let production = measure_rounds(rounds, words, production_dual_path_stream);
    let firmware = measure_rounds(rounds, words, firmware_production_stream);

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
    print_row("dense stage+commit / PHI", &stage_commit_phi, words);
    print_row("dense staged scheduler / PHI", &staged_phi, words);
    print_row("IS ACT<->ROM structural fetch", &fetch, words);
    print_row("IS + ROM0 display + serial ACT execution", &full, words);
    print_row("architectural execution only", &architectural, words);
    print_row(
        "production dual architectural + structural",
        &production,
        words,
    );
    print_row("real firmware architectural + structural", &firmware, words);
    println!();
    println!(
        "Scope: current implementation only. The full row continuously executes all presently wired structural fidelity: 56 bit-cells/word, 4 PHI transitions/bit, resolved IS ownership, ACT->ROM 12-bit address, ROM->ACT 10-bit return, ROM0 display traffic, 15-slot display phase and serial ACT execution."
    );
    println!(
        "The production-dual row includes the current instruction-boundary architectural execution in parallel with serial execution; the real-firmware row runs that same dual path through the versioned HP-67 ROM and its actual control flow. Not yet represented electrically: DATA transfers/RAM devices, PHI-relative IS/DATA launch/sample edges, electrically timed STR/RCD nets, exact PHI widths/dead time, and propagation delays. Therefore this measures realtime computational headroom, not final hardware-timing accuracy."
    );
}
