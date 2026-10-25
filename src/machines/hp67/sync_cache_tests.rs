use super::*;

fn scanned_owner(fabric: &Hp67ElectricalFabric, net: Hp67Net) -> bool {
    let mask = Hp67Driver::ALL.into_iter().fold(0u16, |mask, driver| {
        mask | if fabric.drive(net, driver) == Drive::HighZ {
            0
        } else {
            1 << driver.index()
        }
    });
    mask == 1 << Hp67Driver::Act1820_2530.index()
}
fn check(fabric: &Hp67ElectricalFabric) {
    assert_eq!(
        fabric.sync_drives_valid(),
        scanned_owner(fabric, Hp67Net::Sync)
    );
    assert_eq!(
        fabric.clock_drives_valid(),
        [Hp67Net::Phi1, Hp67Net::Phi2]
            .into_iter()
            .all(|net| scanned_owner(fabric, net))
    );
}

#[test]
fn m16e_cache_all_immediate_net_source_drive_pairs_match_independent_scan() {
    for net in Hp67Net::ALL {
        for driver in Hp67Driver::ALL {
            for act in [Drive::HighZ, Drive::Low, Drive::High] {
                for other in [Drive::HighZ, Drive::Low, Drive::High] {
                    let mut fabric = Hp67ElectricalFabric::default();
                    check(&fabric);
                    fabric.set_drive_immediate(net, Hp67Driver::Act1820_2530, act);
                    check(&fabric);
                    fabric.set_drive_immediate(net, driver, other);
                    check(&fabric);
                    // Low/High changes must retain the same valid OR invalid occupancy.
                    fabric.set_drive_immediate(net, driver, Drive::Low);
                    check(&fabric);
                    fabric.set_drive_immediate(net, driver, Drive::High);
                    check(&fabric);
                    fabric.set_drive_immediate(net, driver, Drive::HighZ);
                    check(&fabric);
                }
            }
        }
    }
}

#[test]
fn m16e_cache_staged_handoffs_repeated_stages_and_contention_commits_match_scan() {
    let mut fabric = Hp67ElectricalFabric::default();
    let act = fabric.claim_driver_owner(Hp67Driver::Act1820_2530).unwrap();
    let other = fabric.claim_driver_owner(Hp67Driver::External).unwrap();
    for reverse in [false, true] {
        for a in [Drive::HighZ, Drive::Low, Drive::High] {
            for b in [Drive::HighZ, Drive::Low, Drive::High] {
                let before = fabric.sync_drives_valid();
                fabric.stage_drive(&act, Hp67Net::Sync, Drive::HighZ);
                if reverse {
                    fabric.stage_drive(&other, Hp67Net::Sync, b);
                    fabric.stage_drive(&act, Hp67Net::Sync, a);
                } else {
                    fabric.stage_drive(&act, Hp67Net::Sync, a);
                    fabric.stage_drive(&other, Hp67Net::Sync, b);
                }
                assert_eq!(fabric.sync_drives_valid(), before);
                let result = fabric.commit_staged();
                assert_eq!(
                    result.is_err(),
                    fabric.level(Hp67Net::Sync) == LogicLevel::Contention
                );
                check(&fabric);
                check(&fabric.clone());
            }
        }
    }
    fabric.stage_drive(&other, Hp67Net::Sync, Drive::HighZ);
    fabric.stage_drive(&act, Hp67Net::Sync, Drive::Low);
    fabric.commit_staged().unwrap();
    assert!(fabric.sync_drives_valid());
    check(&fabric);
    let reset = Hp67ElectricalFabric::default();
    assert!(!reset.sync_drives_valid());
    check(&reset);
}

#[test]
fn m16e_cache_atomic_serial_bundles_preserve_every_foreign_contributor() {
    for driver in Hp67Driver::ALL
        .into_iter()
        .filter(|d| *d != Hp67Driver::Act1820_2530)
    {
        for foreign in [Drive::HighZ, Drive::Low, Drive::High] {
            for chips in [false, true] {
                let mut fabric = Hp67ElectricalFabric::default();
                fabric.set_drive_immediate(Hp67Net::Sync, driver, foreign);
                // Includes first attachment, both polarity directions and release.
                for sync in [
                    Drive::Low,
                    Drive::High,
                    Drive::Low,
                    Drive::HighZ,
                    Drive::High,
                ] {
                    let roms = if chips {
                        RomIsDrives::Chips([Drive::HighZ; 5])
                    } else {
                        RomIsDrives::Aggregate(Drive::HighZ)
                    };
                    fabric.commit_serial_launch_with_data_owner(
                        Drive::HighZ,
                        roms,
                        sync,
                        Some((Drive::Low, Drive::HighZ)),
                        Hp67Driver::RamAggregate,
                    );
                    assert_eq!(fabric.drive(Hp67Net::Sync, driver), foreign);
                    check(&fabric);
                }
                fabric.set_drive_immediate(Hp67Net::Sync, driver, Drive::HighZ);
                check(&fabric);
            }
        }
    }
}

#[test]
fn m16e_cache_mixed_immediate_staged_and_serial_updates_keep_all_metadata_consistent() {
    let mut fabric = Hp67ElectricalFabric::default();
    let owners: Vec<_> = Hp67Driver::ALL
        .into_iter()
        .map(|driver| fabric.claim_driver_owner(driver).unwrap())
        .collect();
    let drives = [Drive::HighZ, Drive::Low, Drive::High];
    let mut random = 0x67_97_16_e3u64;
    for step in 0..20_000 {
        random = random.wrapping_mul(6364136223846793005).wrapping_add(1);
        let net = Hp67Net::ALL[((random >> 16) as usize) % Hp67Net::COUNT];
        let driver = Hp67Driver::ALL[((random >> 32) as usize) % Hp67Driver::COUNT];
        let drive = drives[((random >> 48) % 3) as usize];
        match step % 3 {
            0 => fabric.set_drive_immediate(net, driver, drive),
            1 => {
                let before = fabric.sync_drives_valid();
                fabric.stage_drive(&owners[driver.index()], net, drive);
                assert_eq!(fabric.sync_drives_valid(), before);
                let _ = fabric.commit_staged();
            }
            _ => fabric.commit_serial_launch_with_roms(
                Drive::HighZ,
                RomIsDrives::Aggregate(Drive::HighZ),
                drive,
                None,
            ),
        }
        check(&fabric);
    }
}

#[test]
fn m16e_cache_public_validator_retains_fault_details_and_pristine_exception() {
    use crate::machines::hp67::{sync::*, Hp67ElectricalBackplane};
    let mut bus = Hp67ElectricalBackplane::default();
    assert!(validate_sync_transport_entry(&bus).is_ok());
    assert!(validate_sync_source(&bus).is_err());
    bus.drive(Hp67Net::Sync, Hp67Driver::Act1820_2530, Drive::Low);
    for driver in Hp67Driver::ALL
        .into_iter()
        .filter(|d| *d != Hp67Driver::Act1820_2530)
    {
        bus.drive(Hp67Net::Sync, driver, Drive::Low);
        let error = validate_sync_source(&bus).unwrap_err();
        assert_eq!(error.level, LogicLevel::Low);
        assert_eq!(
            error.active_mask,
            (1 << driver.index()) | (1 << Hp67Driver::Act1820_2530.index())
        );
        bus.drive(Hp67Net::Sync, driver, Drive::High);
        assert_eq!(
            validate_sync_source(&bus).unwrap_err().level,
            LogicLevel::Contention
        );
        bus.drive(Hp67Net::Sync, driver, Drive::HighZ);
        validate_sync_source(&bus).unwrap();
    }
}

fn probe_stream<const CACHED: bool>(words: usize) {
    use crate::machines::hp67::{sync::validate_sync_source, Hp67ElectricalBackplane};
    use std::hint::black_box;
    let mut bus = Hp67ElectricalBackplane::default();
    for word in 0..words {
        for bit in 0..56 {
            bus.commit_serial_launch(
                Drive::HighZ,
                RomIsDrives::Aggregate(Drive::HighZ),
                if bit >= 46 && word % 2 == 0 {
                    Drive::High
                } else {
                    Drive::Low
                },
                None,
            );
            bus.advance_clock();
            bus.advance_clock();
            // Three independent instruction consumers plus three entry checks.
            if bit == 0 || bit >= 46 {
                for _ in 0..3 {
                    let observed = black_box(&bus);
                    let valid = if CACHED {
                        validate_sync_source(observed).is_ok()
                    } else {
                        observed.active_driver_count(Hp67Net::Sync) == 1
                            && observed.driver_drive(Hp67Net::Sync, Hp67Driver::Act1820_2530)
                                != Drive::HighZ
                            && matches!(
                                observed.level(Hp67Net::Sync),
                                LogicLevel::Low | LogicLevel::High
                            )
                    };
                    assert!(black_box(valid));
                }
            }
            bus.advance_clock();
            bus.advance_clock();
        }
    }
    assert_eq!(bus.tick().get(), words as u64 * 224);
    let _ = black_box(bus);
}

#[test]
#[ignore = "release-only source-query comparison; run explicitly with --ignored --nocapture"]
fn m16e_sync_source_query_realtime_benchmark() {
    use std::time::Instant;
    assert!(!cfg!(debug_assertions), "run with --release");
    const WORDS: usize = 50_000;
    const ROUNDS: usize = 7;
    probe_stream::<true>(2500);
    probe_stream::<false>(2500);
    let mut samples = [[0.0f64; ROUNDS]; 2];
    for round in 0..ROUNDS {
        for offset in 0..2 {
            let path = (round + offset) % 2;
            let start = Instant::now();
            if path == 0 {
                probe_stream::<true>(WORDS);
            } else {
                probe_stream::<false>(WORDS);
            }
            samples[path][round] = start.elapsed().as_secs_f64() * 1_000_000.0 / WORDS as f64;
        }
    }
    println!("\nM16E SYNC SOURCE QUERY KERNEL / SAME PROCESS");
    for (path, name) in ["cached ownership", "count + ACT slot"]
        .into_iter()
        .enumerate()
    {
        samples[path].sort_by(f64::total_cmp);
        let median = samples[path][ROUNDS / 2];
        println!(
            "{name:20} median={median:.3} us/word best={:.3} worst={:.3}",
            samples[path][0],
            samples[path][ROUNDS - 1]
        );
    }
    println!("Scope: 33 equivalent source probes/word, joint IS/SYNC publication, 56 cells/224 PHI transitions, rotating order. Both paths maintain the new cache; this isolates query cost, not whole-emulator old/new performance. No speed threshold is a correctness assertion.");
}
