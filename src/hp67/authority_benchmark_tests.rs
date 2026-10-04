//! Measures the actual live authority bridge; synthetic port loads are labelled.
use super::*;
use std::{env, hint::black_box, time::Instant};
use hp67emu::machines::hp67::{BITS_PER_WORD, CRC_CARD_WORD_MASK};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Workload {
    FirmwareIdle,
    RamPorts,
    CrcPorts,
}

impl Workload {
    const ALL: [Self; 3] = [Self::FirmwareIdle, Self::RamPorts, Self::CrcPorts];
    fn name(self) -> &'static str {
        match self {
            Self::FirmwareIdle => "live firmware idle / full authority",
            Self::RamPorts => "synthetic RAM ports / full live bridge",
            Self::CrcPorts => "synthetic CRC ports / full live bridge",
        }
    }
}

struct LiveBenchmark {
    live: Hp67LiveMachine,
    workload: Workload,
    sequence: usize,
}

impl LiveBenchmark {
    fn new(workload: Workload) -> Self {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        for _ in 0..BOOT_CYCLE_LIMIT {
            live.step_firmware_cycle().expect("live benchmark boot must succeed");
            if live.phase == LiveBootPhase::Idle { break; }
        }
        assert_eq!(live.phase, LiveBootPhase::Idle);
        live.complete_pending_ram_data_before_word(live.boot_cycle).unwrap();
        if workload != Workload::FirmwareIdle {
            live.machine.act.state.instruction_state = ActInstructionState::Normal;
            live.machine.act.state.c = [3; 14];
            live.machine.act.state.ram_address = 0x20;
        }
        if workload == Workload::CrcPorts {
            live.machine.crc.commit_control_flag(CRC_FLAG_WRITE_MODE as u8, true).unwrap();
        }
        Self { live, workload, sequence: 0 }
    }

    fn run_words(&mut self, words: usize) -> u64 {
        let tick = self.live.backplane.tick().get();
        let cycle = self.live.boot_cycle;
        let executed = self.live.machine.executed_words();
        let mut checksum = 0u64;
        for _ in 0..words {
            let write = self.sequence % 2 == 0;
            match self.workload {
                Workload::FirmwareIdle => {},
                Workload::RamPorts => {
                    self.live.pipeline.complete_cycle(if write { 0o1360 } else { 0o0070 });
                },
                Workload::CrcPorts => {
                    if write {
                        self.live.machine.act.state.ram_address = 0x99;
                        self.live.pipeline.complete_cycle(0o1360);
                    } else {
                        self.live.machine.act.state.ram_address = 0x9b;
                        self.live.machine.crc.present_read_word((self.sequence as u32) & CRC_CARD_WORD_MASK).unwrap();
                        self.live.pipeline.complete_cycle(0o0070);
                    }
                },
            }
            let execution = self.live.step_firmware_cycle_with_execution().expect("full live authority cycle must succeed").expect("booted benchmark pipeline must execute every word");
            if self.workload == Workload::CrcPorts && write {
                let Hp67ArchitecturalOperation::CrcDataWrite { card_word, .. } = execution.operation else { panic!("CRC write fixture lost ownership"); };
                assert_eq!(self.live.machine.crc.take_queued_write_word(), Some(card_word));
                checksum = checksum.wrapping_add(u64::from(card_word));
            }
            checksum = checksum.wrapping_add(u64::from(execution.next_pc)).wrapping_add(u64::from(self.live.machine.act.state.c[0]));
            self.sequence += 1;
        }
        // These checks make reduced transition counts or skipped oracle words
        // observable independently of wall-clock measurements.
        assert_eq!(self.live.backplane.tick().get() - tick, words as u64 * u64::from(BITS_PER_WORD) * 4);
        assert_eq!(self.live.boot_cycle - cycle, words as u64);
        assert_eq!(self.live.machine.executed_words() - executed, words as u64);
        assert!(self.live.act_serial.serial_execution().unwrap().is_complete());
        if self.workload == Workload::RamPorts {
            assert!(self.live.pending_ram_data_transfer.is_some());
            assert!(self.live.data_serial.frame_in_progress());
        }
        if self.workload == Workload::CrcPorts {
            assert_eq!(self.live.machine.crc.queued_read_words(), 0);
            assert_eq!(self.live.machine.crc.queued_write_words(), 0);
            assert!(self.live.act_serial.serial_crc_data_result_image().unwrap().is_complete());
        }
        black_box(checksum)
    }
}

fn positive_env(name: &str, default: usize) -> usize {
    match env::var(name) {
        Ok(value) => value.parse::<usize>().ok().filter(|value| *value > 0).unwrap_or_else(|| panic!("{name} must be a positive integer")),
        Err(env::VarError::NotPresent) => default,
        Err(error) => panic!("cannot read {name}: {error}"),
    }
}

#[test]
fn m14o_live_benchmark_workloads_preserve_word_and_port_invariants() {
    for workload in Workload::ALL {
        let mut benchmark = LiveBenchmark::new(workload);
        benchmark.run_words(17);
        benchmark.run_words(16);
        assert_eq!(benchmark.sequence, 33);
        if workload == Workload::RamPorts {
            benchmark.live.complete_pending_ram_data_before_word(0).unwrap();
            assert_eq!(benchmark.live.machine.ram.read(0x20), Some([3;14]));
            assert_eq!(benchmark.live.machine.act.state.c, [3;14]);
        }
    }
}

#[test]
#[ignore = "release-only live authority benchmark; run explicitly with --ignored --nocapture"]
fn m14o_live_authority_realtime_benchmark() {
    assert!(!cfg!(debug_assertions), "run this benchmark with --release");
    let words = positive_env("HP67_BENCH_WORDS", 50_000);
    let rounds = positive_env("HP67_BENCH_ROUNDS", 7);
    let warmup = positive_env("HP67_BENCH_WARMUP_WORDS", 2_500);
    // Construction, boot and warm-up are outside timed samples. Machines remain
    // alive across rounds; path order rotates to reduce thermal/order bias.
    let mut paths = Workload::ALL.map(LiveBenchmark::new);
    for path in &mut paths { black_box(path.run_words(warmup)); }
    let mut samples: [Vec<Duration>; 3] = std::array::from_fn(|_| Vec::with_capacity(rounds));
    for round in 0..rounds {
        for offset in 0..paths.len() {
            let index = (round + offset) % paths.len();
            let start = Instant::now();
            black_box(paths[index].run_words(words));
            samples[index].push(start.elapsed());
        }
    }
    println!("\nM14O LIVE AUTHORITY REALTIME BENCHMARK");
    println!("words/round={words} rounds={rounds} warmup={warmup}; reference={} us/word", HP67_OBSERVED_WORD_TIME_US);
    println!("{:<44} {:>12} {:>12} {:>12} {:>12} {:>12}", "PATH", "MED us/word", "words/s", "vs hardware", "BEST us", "WORST us");
    for (index, path) in paths.iter().enumerate() {
        samples[index].sort_unstable();
        let us = |duration: Duration| duration.as_secs_f64() * 1_000_000.0 / words as f64;
        let median = us(samples[index][rounds / 2]);
        println!("{:<44} {:>12.3} {:>12.0} {:>11.2}x {:>12.3} {:>12.3}", path.workload.name(), median, 1_000_000.0 / median, HP67_OBSERVED_WORD_TIME_US as f64 / median, us(samples[index][0]), us(samples[index][rounds - 1]));
    }
    println!("Live firmware row uses the actual desktop live cycle, frozen inputs, restore guard, structural fetch/display, oracle comparisons and authority commits. All rows require 56 bit-cells and 224 PHI transitions per word.");
    println!("RAM/CRC rows inject synthetic port opcodes through that same live cycle; timed CRC fixture refill/drain overhead is included. They are stress loads, not firmware card throughput or magnetic/electrical timing evidence.");
    println!("Boot excluded; existing logical DATA phase and CRC buffers included; electrical DATA/physical RAM, exact PHI edges, STR/RCD propagation and magnetic sense/serialization remain source-blocked. No speed threshold is a correctness assertion.");
}
