use super::*;

fn recomputed_ownership(fabric: &Hp67ElectricalFabric) -> bool {
    [Hp67Net::Phi1, Hp67Net::Phi2].into_iter().all(|net| {
        let mask = Hp67Driver::ALL.into_iter().fold(0u16, |mask, driver| {
            mask | if fabric.drive(net, driver) == Drive::HighZ {
                0
            } else {
                1 << driver.index()
            }
        });
        mask == 1 << Hp67Driver::Act1820_2530.index()
    })
}

#[test]
fn m16b_cache_matches_independent_driver_scan_for_all_sources_and_drive_pairs() {
    for net in Hp67Net::ALL {
        for driver in Hp67Driver::ALL {
            for act in [Drive::HighZ, Drive::Low, Drive::High] {
                for external in [Drive::HighZ, Drive::Low, Drive::High] {
                    let mut fabric = Hp67ElectricalFabric::default();
                    fabric.set_drive_immediate(net, Hp67Driver::Act1820_2530, act);
                    assert_eq!(fabric.clock_drives_valid(), recomputed_ownership(&fabric));
                    fabric.set_drive_immediate(net, driver, external);
                    assert_eq!(fabric.clock_drives_valid(), recomputed_ownership(&fabric));
                    fabric.set_drive_immediate(net, driver, Drive::HighZ);
                    assert_eq!(fabric.clock_drives_valid(), recomputed_ownership(&fabric));
                }
            }
        }
    }
}

#[test]
fn m16b_cache_tracks_staged_handoffs_contention_commits_clone_and_recovery() {
    let mut fabric = Hp67ElectricalFabric::default();
    let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
    let external = fabric.claim_driver_owner(Hp67Driver::External).unwrap();
    for reverse in [false, true] {
        for net in [Hp67Net::Phi1, Hp67Net::Phi2] {
            for a in [Drive::HighZ, Drive::Low, Drive::High] {
                for b in [Drive::HighZ, Drive::Low, Drive::High] {
                    let before = fabric.clock_drives_valid();
                    if reverse {
                        fabric.stage_drive(&external, net, b);
                        fabric.stage_drive(&act, net, a);
                    } else {
                        fabric.stage_drive(&act, net, a);
                        fabric.stage_drive(&external, net, b);
                    }
                    assert_eq!(fabric.clock_drives_valid(), before);
                    // A contention error reports the resulting committed state.
                    let _ = fabric.commit_staged();
                    assert_eq!(fabric.clock_drives_valid(), recomputed_ownership(&fabric));
                    let clone = fabric.clone();
                    assert_eq!(clone.clock_drives_valid(), recomputed_ownership(&clone));
                }
            }
            fabric.stage_drive(&act, net, Drive::High);
            fabric.stage_drive(&external, net, Drive::HighZ);
            fabric.commit_staged().unwrap();
        }
    }
    assert!(fabric.clock_drives_valid());
    assert!(Hp67ElectricalFabric::default().clock_drives_valid());
}

#[test]
fn m16b_cache_survives_mixed_net_updates_and_keeps_resolution_and_clock_faults_separate() {
    let mut fabric = Hp67ElectricalFabric::default();
    let mut random = 0x67_97_16_b2u64;
    for _ in 0..20_000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let net = Hp67Net::ALL[((random >> 16) as usize) % Hp67Net::COUNT];
        let driver = Hp67Driver::ALL[((random >> 32) as usize) % Hp67Driver::COUNT];
        let drive = [Drive::HighZ, Drive::Low, Drive::High][((random >> 48) % 3) as usize];
        fabric.set_drive_immediate(net, driver, drive);
        assert_eq!(fabric.clock_drives_valid(), recomputed_ownership(&fabric));
    }
    // Non-clock contention must not masquerade as a clock ownership fault.
    let mut clean = Hp67ElectricalFabric::default();
    clean.set_drive_immediate(Hp67Net::Data, Hp67Driver::External, Drive::Low);
    clean.set_drive_immediate(Hp67Net::Data, Hp67Driver::RamAggregate, Drive::High);
    assert_eq!(clean.level(Hp67Net::Data), LogicLevel::Contention);
    assert!(clean.clock_drives_valid());
}

fn count_based_ownership(fabric: &Hp67ElectricalFabric) -> bool {
    [Hp67Net::Phi1, Hp67Net::Phi2].into_iter().all(|net| {
        fabric.active_driver_count(net) == 1
            && fabric.drive(net, Hp67Driver::Act1820_2530) != Drive::HighZ
    })
}

fn ownership_probe_stream<const CACHED: bool>(words: usize) -> u64 {
    use std::hint::black_box;
    let mut fabric = Hp67ElectricalFabric::default();
    for _ in 0..words {
        for _ in 0..56 {
            for (net, drive) in [
                (Hp67Net::Phi1, Drive::Low),
                (Hp67Net::Phi1, Drive::High),
                (Hp67Net::Phi2, Drive::Low),
                (Hp67Net::Phi2, Drive::High),
            ] {
                fabric.set_drive_immediate(net, Hp67Driver::Act1820_2530, drive);
                fabric.advance_tick();
                black_box(fabric.level(net));
                // Two observations per edge isolate repeated ownership queries.
                // This is a synthetic electrical kernel, not desktop throughput.
                for _ in 0..2 {
                    let observed = black_box(&fabric);
                    let valid = if CACHED {
                        observed.clock_drives_valid()
                    } else {
                        count_based_ownership(observed)
                    };
                    assert!(black_box(valid));
                }
            }
        }
    }
    assert_eq!(fabric.tick().get(), words as u64 * 224);
    fabric.tick().get()
}

#[test]
#[ignore = "release-only rotating ownership query benchmark; not production throughput"]
fn m16b_cache_ownership_realtime_benchmark() {
    use std::{
        hint::black_box,
        time::{Duration, Instant},
    };
    assert!(!cfg!(debug_assertions), "run with --release");
    const WORDS: usize = 50_000;
    const ROUNDS: usize = 8;
    let paths: [fn(usize) -> u64; 2] = [
        ownership_probe_stream::<true>,
        ownership_probe_stream::<false>,
    ];
    for path in paths {
        black_box(path(2_500));
    }
    let mut samples: [Vec<Duration>; 2] = std::array::from_fn(|_| Vec::with_capacity(ROUNDS));
    for round in 0..ROUNDS {
        for offset in 0..2 {
            let index = (round + offset) % 2;
            let start = Instant::now();
            black_box(paths[index](WORDS));
            samples[index].push(start.elapsed());
        }
    }
    println!("\nM16B OWNERSHIP QUERY KERNEL — ROTATING SAME-PROCESS COMPARISON");
    for (index, name) in ["cached ownership", "M16B count-based ownership"]
        .into_iter()
        .enumerate()
    {
        samples[index].sort_unstable();
        let us = |d: Duration| d.as_secs_f64() * 1e6 / WORDS as f64;
        println!(
            "{name}: median={:.3} us/word best={:.3} worst={:.3}",
            us(samples[index][ROUNDS / 2]),
            us(samples[index][0]),
            us(samples[index][ROUNDS - 1])
        );
    }
    println!("Both kernels maintain the cache, resolve every PHI edge and assert 224 transitions/word. Eight ownership probes per bit. This isolates query representation; it is not an old/new whole-emulator A/B result. No correctness speed threshold.");
}
