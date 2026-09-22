use std::{
    env,
    hint::black_box,
    time::{Duration, Instant},
};

use hp67emu::machines::hp67::{
    act_data_transfer_plan, run_structural_display_fetch_cycle,
    run_structural_display_fetch_data_phase_cycle, ActRegister, ActSerialEndpoint,
    FetchPipelineLatch, Hp67ArchitecturalMachine, Hp67DataSerialSink, Hp67DataSerialSource,
    Hp67DataSerialWordPath, Hp67DataTransferDirection, Hp67ElectricalBackplane, Hp67Firmware,
    Rom0DisplayEndpoint, RomFetchEndpoint, BITS_PER_WORD, HP67_OBSERVED_WORD_TIME_US,
};

const DEFAULT_WORDS_PER_ROUND: usize = 50_000;
const DEFAULT_ROUNDS: usize = 7;
const DEFAULT_WARMUP_WORDS: usize = 2_500;

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

fn measure_firmware_data_rounds_interleaved(rounds: usize, words: usize) -> [BenchmarkStats; 3] {
    let paths: [fn(usize) -> u64; 3] = [
        firmware_production_stream,
        firmware_data_shadow_stream,
        firmware_data_fused_stream,
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
        "{name:<46} {median_us:>12.3} {words_per_second:>14.0} {realtime_multiple:>12.2}x {min_us:>12.3} {max_us:>12.3}"
    );
}

fn data_phase_stream(words: usize) -> u64 {
    let mut source = Hp67DataSerialSource::default();
    let mut sink = Hp67DataSerialSink::default();
    let mut checksum = 0u64;
    let mut completed = 0usize;

    for word_index in 0..words {
        let register =
            std::array::from_fn(|digit| (word_index as u8).wrapping_add(digit as u8 * 3) & 0x0f);
        source.begin_word(Some(register));
        sink.begin_word(true);

        for word_bit in 0..BITS_PER_WORD {
            if let Some(reconstructed) = sink
                .sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                .expect("continuous DATA phase stream must remain structurally valid")
            {
                completed += 1;
                checksum = reconstructed
                    .into_iter()
                    .fold(checksum, |sum, digit| sum.wrapping_add(u64::from(digit)));
            }
        }

        source.complete_word();
    }

    assert_eq!(completed, words.saturating_sub(1));
    checksum ^ completed as u64
}

fn firmware_data_shadow_stream(words: usize) -> u64 {
    let firmware = Hp67Firmware::default();
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(0);
    let mut rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut data_source = Hp67DataSerialSource::default();
    let mut data_sink = Hp67DataSerialSink::default();
    let mut pending_expected: Option<ActRegister> = None;
    let mut checksum = 0u64;

    for _ in 0..words {
        pipeline.begin_cycle();

        let mut current_payload = None;
        if let Some(word) = pipeline.executing_word() {
            if let Some(plan) = act_data_transfer_plan(word, &machine.act.state) {
                if let Some(ram_word) = machine.ram.read(plan.address) {
                    current_payload = Some(match plan.direction {
                        Hp67DataTransferDirection::ActToPeripheral => machine.act.state.c,
                        Hp67DataTransferDirection::PeripheralToAct => ram_word,
                    });
                    checksum = checksum.wrapping_add(u64::from(plan.address));
                }
            }

            act.begin_execution(word, &machine.act.state)
                .expect("firmware DATA shadow serial execution must start");
            let execution = machine
                .execute_word(word)
                .expect("versioned HP-67 firmware DATA shadow word must execute");
            checksum = checksum.wrapping_add(execution.next_pc as u64);
        }

        if pending_expected.is_some() || current_payload.is_some() {
            data_source.begin_word(current_payload);
            data_sink.begin_word(current_payload.is_some());

            for word_bit in 0..BITS_PER_WORD {
                if let Some(reconstructed) = data_sink
                    .sample_word_bit(word_bit, data_source.logical_bit_for_word_bit(word_bit))
                    .expect("firmware DATA shadow must preserve the source-backed phase")
                {
                    let expected = pending_expected
                        .take()
                        .expect("completed DATA shadow frame must have a pending payload");
                    assert_eq!(reconstructed, expected);
                    checksum = reconstructed
                        .into_iter()
                        .fold(checksum, |sum, digit| sum.wrapping_add(u64::from(digit)));
                }
            }

            data_source.complete_word();
            pending_expected = current_payload;
        }

        let bank = machine.prepare_hp67_fetch();
        firmware.select_bank(bank);
        let address = machine.pc();
        let result = run_structural_display_fetch_cycle(
            &mut backplane,
            address,
            &machine.act.state,
            &mut act,
            &mut rom,
            &mut rom0,
            &firmware,
        )
        .expect("continuous real-firmware DATA shadow path must complete");
        pipeline.complete_cycle(result.fetched_word);
        checksum = checksum
            .wrapping_add(result.fetched_word as u64)
            .wrapping_add(result.display_byte as u64)
            .wrapping_add(u64::from(result.rcd_falling));
    }

    assert_eq!(backplane.word_index(), words as u64);
    checksum
}

fn firmware_data_fused_stream(words: usize) -> u64 {
    let firmware = Hp67Firmware::default();
    let mut machine = Hp67ArchitecturalMachine::default();
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut act = ActSerialEndpoint::new(0);
    let mut rom = RomFetchEndpoint::default();
    let mut rom0 = Rom0DisplayEndpoint::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut data = Hp67DataSerialWordPath::default();
    let mut pending_expected: Option<ActRegister> = None;
    let mut checksum = 0u64;

    for _ in 0..words {
        pipeline.begin_cycle();

        let mut current_payload = None;
        if let Some(word) = pipeline.executing_word() {
            if let Some(plan) = act_data_transfer_plan(word, &machine.act.state) {
                if let Some(ram_word) = machine.ram.read(plan.address) {
                    current_payload = Some(match plan.direction {
                        Hp67DataTransferDirection::ActToPeripheral => machine.act.state.c,
                        Hp67DataTransferDirection::PeripheralToAct => ram_word,
                    });
                    checksum = checksum.wrapping_add(u64::from(plan.address));
                }
            }

            act.begin_execution(word, &machine.act.state)
                .expect("firmware fused DATA serial execution must start");
            let execution = machine
                .execute_word(word)
                .expect("versioned HP-67 firmware fused DATA word must execute");
            checksum = checksum.wrapping_add(execution.next_pc as u64);
        }

        let bank = machine.prepare_hp67_fetch();
        firmware.select_bank(bank);
        let address = machine.pc();

        let result = if data.frame_in_progress() || current_payload.is_some() {
            let fused = run_structural_display_fetch_data_phase_cycle(
                &mut backplane,
                address,
                &machine.act.state,
                &mut act,
                &mut rom,
                &mut rom0,
                &firmware,
                &mut data,
                current_payload,
            )
            .expect("continuous real-firmware fused DATA path must complete");

            if let Some(reconstructed) = fused.completed_data {
                let expected = pending_expected
                    .take()
                    .expect("completed fused DATA frame must have a pending payload");
                assert_eq!(reconstructed, expected);
                checksum = reconstructed
                    .into_iter()
                    .fold(checksum, |sum, digit| sum.wrapping_add(u64::from(digit)));
            }
            pending_expected = current_payload;
            fused.word
        } else {
            run_structural_display_fetch_cycle(
                &mut backplane,
                address,
                &machine.act.state,
                &mut act,
                &mut rom,
                &mut rom0,
                &firmware,
            )
            .expect("continuous real-firmware non-DATA word must complete")
        };

        pipeline.complete_cycle(result.fetched_word);
        checksum = checksum
            .wrapping_add(result.fetched_word as u64)
            .wrapping_add(result.display_byte as u64)
            .wrapping_add(u64::from(result.rcd_falling));
    }

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

#[test]
#[ignore = "release-only M14B DATA wall-clock benchmark; run explicitly with --ignored --nocapture"]
fn hp67_data_realtime_benchmark() {
    let words = env_usize("HP67_BENCH_WORDS", DEFAULT_WORDS_PER_ROUND);
    let rounds = env_usize("HP67_BENCH_ROUNDS", DEFAULT_ROUNDS);
    let warmup_words = env_usize("HP67_BENCH_WARMUP_WORDS", DEFAULT_WARMUP_WORDS);

    black_box(firmware_production_stream(warmup_words));
    black_box(firmware_data_shadow_stream(warmup_words));
    black_box(firmware_data_fused_stream(warmup_words));
    black_box(data_phase_stream(warmup_words));

    let [firmware, shadow, fused] = measure_firmware_data_rounds_interleaved(rounds, words);
    let data = measure_rounds(rounds, words, data_phase_stream);

    println!();
    println!("HP-67 M14B DATA REALTIME BENCHMARK");
    println!("================================");
    println!(
        "Measured stream    : {} words/round x {} rounds",
        words, rounds
    );
    println!();
    println!(
        "{:<46} {:>12} {:>14} {:>13} {:>12} {:>12}",
        "PATH", "MED us/word", "words/s", "vs hardware", "BEST us", "WORST us"
    );
    println!("{}", "-".repeat(115));
    print_row("real firmware architectural + structural", &firmware, words);
    print_row(
        "real firmware + conditional RAM DATA shadow",
        &shadow,
        words,
    );
    print_row("real firmware + fused RAM DATA phase", &fused, words);
    print_row("DATA logical phase source + sink", &data, words);
    println!();
    println!(
        "Scope: M14B comparison only. The three real-firmware rows are measured in rotating interleaved order. The fused row mirrors the live logical RAM DATA integration for installed RAM addresses. DATA polarity, electrical ownership, physical RAM-chip mapping, PHI-relative launch/sample edges and propagation remain unresolved."
    );
}
