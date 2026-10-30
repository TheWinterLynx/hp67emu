//! Transaction parity and rejected-sample recovery against the pre-M16K lane.
use super::*;
use crate::emulation::Drive;
use crate::machines::hp67::{Hp67DataSerialWordPath, Hp67Driver, Hp67Net};

fn launch(bus: &mut Hp67ElectricalBackplane, word: u16, instruction: bool) {
    use crate::machines::hp67::{isa::{act_address_drive, rom_word_drive}, rom_devices::RomIsDrives};
    let bit = bus.word_bit();
    bus.commit_serial_launch(act_address_drive(0x123, bit), RomIsDrives::Aggregate(rom_word_drive(word, bit)), if bit >= 46 && instruction { Drive::High } else { Drive::Low }, None);
}

// Preserved M16J full-copy algorithm, test-only. It is a regression comparator,
// not physical timing evidence and not selectable in the production endpoint.
fn full_copy_sample(crc: &mut Hp67SerialCrcControl, bus: &Hp67ElectricalBackplane) -> Result<(), Hp67CrcControlError> {
    let bit = bus.word_bit();
    if !bus.at_transport_sample_edge() || bit != crc.expected_bit {
        return Err(Hp67CrcControlError::Phase { expected: crc.expected_bit, actual: bit, tick: bus.tick().get() });
    }
        // Prepare the small control image: a failed command sample cannot change
        // latches, pending instruction, contacts or the partial receive window.
        let mut next = *crc;
        if bit == 0 {
            next.executing = match next.receiver.received_frame() {
                Some(frame) if frame.instruction => decode_crc_opcode(frame.word)
                    .map_err(|_| Hp67CrcControlError::Decode(frame.word))?,
                _ => None,
            };
            next.result = None;
        }
        if bit >= 46 {
            next.receiver
                .sample_at_edge(bus)
                .map_err(Hp67CrcControlError::Command)?;
        }
        if bit == 55 {
            if let Some(instruction) = next.executing {
                let flag = match instruction {
                    CrcInstruction::SetFlag { flag }
                    | CrcInstruction::TestFlagAndClear { flag } => flag,
                };
                let index = usize::from(flag);
                let (value, condition) = match instruction {
                    CrcInstruction::SetFlag { .. } => (true, None),
                    CrcInstruction::TestFlagAndClear { .. } => {
                        (false, Some(next.flags[index] || next.external[index]))
                    }
                };
                next.flags[index] = value;
                next.result = Some(Hp67CrcControlResult {
                    instruction,
                    flag,
                    value,
                    condition,
                });
            }
        }
        next.expected_bit = if bit == 55 { 0 } else { bit + 1 };
        *crc = next;
        Ok(())
    }

fn run_pair(crc: &mut Hp67SerialCrcControl, old: &mut Hp67SerialCrcControl, bus: &mut Hp67ElectricalBackplane, opcode: u16, instruction: bool) {
    let start = bus.tick().get();
    for _ in 0..56 {
        launch(bus, opcode, instruction);
        let before = *crc;
        assert_eq!(crc.sample_at_edge(bus), full_copy_sample(old, bus));
        assert_eq!(*crc, before);
        assert_eq!(*crc, *old);
        bus.advance_clock(); bus.advance_clock();
        crc.sample_at_edge(bus).unwrap();
        full_copy_sample(old, bus).unwrap();
        assert_eq!(*crc, *old, "opcode={opcode:03x} bit={}", bus.word_bit());
        let before = *crc;
        assert_eq!(crc.sample_at_edge(bus), full_copy_sample(old, bus));
        assert_eq!(*crc, before);
        assert_eq!(*crc, *old);
        bus.advance_clock(); bus.advance_clock();
    }
    assert_eq!(bus.tick().get() - start, 224);
}

#[test]
fn m16k_crc_transaction_matches_prior_image_at_every_cell_for_all_words_and_qualifiers() {
    for seed in 0u16..4 {
        let mut crc = Hp67SerialCrcControl::default();
        crc.flags = std::array::from_fn(|i| ((i as u16 + seed) & 1) != 0);
        crc.external = std::array::from_fn(|i| ((i as u16 + seed) & 2) != 0);
        let mut old = crc;
        let mut bus = Hp67ElectricalBackplane::default();
        // Consecutive words exercise the retained prior instruction and status,
        // rather than resetting between opcodes and testing fetch alone.
        for instruction in [false, true] {
            for opcode in 0..=0x3ff {
                run_pair(&mut crc, &mut old, &mut bus, opcode, instruction);
            }
        }
        run_pair(&mut crc, &mut old, &mut bus, 0, true);
    }
}

#[test]
fn m16k_preparation_and_discard_leave_every_retained_field_unchanged() {
    let mut crc = Hp67SerialCrcControl::default();
    let mut bus = Hp67ElectricalBackplane::default();
    for opcode in [0o1000, 0o0300, 0] {
        for _ in 0..56 {
            launch(&mut bus, opcode, true);
            bus.advance_clock(); bus.advance_clock();
            let before = crc;
            let _discarded = crc.prepare_sample(&bus).unwrap();
            assert_eq!(crc, before);
            let sample = crc.prepare_sample(&bus).unwrap();
            assert_eq!(crc, before);
            crc.commit_sample(sample);
            let after = crc;
            assert!(crc.prepare_sample(&bus).is_err());
            assert_eq!(crc, after);
            bus.advance_clock(); bus.advance_clock();
        }
    }
}

#[test]
fn m16k_shared_data_fault_at_each_cell_preserves_crc_ram_receiver_and_sink_then_retries() {
    let mut path = Hp67DataSerialWordPath::default().with_ram_command_receiver().with_crc_control();
    let mut bus = Hp67ElectricalBackplane::default();
    let mut old = Hp67SerialCrcControl::default();
    for opcode in [0o1000, 0o0300, 0] {
        path.enable_electrical_level_bridge(None);
        path.begin_word(None);
        let start = bus.tick().get();
        for bit in 0..56 {
            launch(&mut bus, opcode, true);
            bus.advance_clock(); bus.advance_clock();
            let before = path;
            bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::High);
            let tick = bus.tick();
            assert!(path.sample_transport_word_bit_at_edge(bit, &bus).is_err());
            assert_eq!(path, before);
            assert_eq!(bus.tick(), tick);
            bus.drive(Hp67Net::Data, Hp67Driver::External, Drive::HighZ);
            path.sample_transport_word_bit_at_edge(bit, &bus).unwrap();
            full_copy_sample(&mut old, &bus).unwrap();
            assert_eq!(*path.crc_control().unwrap(), old);
            bus.advance_clock(); bus.advance_clock();
        }
        assert_eq!(bus.tick().get() - start, 224);
        assert_eq!(path.complete_word(), None);
    }
}

#[test]
fn m16k_fault_on_final_received_bit_cannot_publish_frame_or_execute_pending_command() {
    for net in [Hp67Net::Isa, Hp67Net::Sync] {
        let mut crc = Hp67SerialCrcControl::default();
        let mut bus = Hp67ElectricalBackplane::default();
        let mut old = crc;
        run_pair(&mut crc, &mut old, &mut bus, 0o1000, true);
        for bit in 0..56 {
            launch(&mut bus, 0o0300, true);
            bus.advance_clock(); bus.advance_clock();
            if bit == 55 {
                let before = crc;
                let mut fault = bus.clone();
                fault.drive(net, Hp67Driver::External, Drive::High);
                fault.drive(net, Hp67Driver::Act1820_2530, Drive::Low);
                assert!(crc.prepare_sample(&fault).is_err());
                assert_eq!(crc, before);
                assert!(!crc.flags[0]);
                assert_eq!(crc.result(), None);
            }
            crc.sample_at_edge(&bus).unwrap();
            bus.advance_clock(); bus.advance_clock();
        }
        assert!(crc.flags[0]);
        assert_eq!(crc.received_frame().unwrap().word, 0o0300);
    }
}

fn benchmark_lane(words: usize, previous: bool) -> (u128, Hp67SerialCrcControl) {
    use std::{hint::black_box, time::Instant};
    let mut bus = Hp67ElectricalBackplane::default();
    let mut crc = Hp67SerialCrcControl::default();
    let start = Instant::now();
    for word in 0..words {
        for _ in 0..56 {
            launch(&mut bus, (word & 0x3ff) as u16, word & 1 == 0);
            bus.advance_clock(); bus.advance_clock();
            if previous {
                // The enclosing DATA path formerly copied this image too.
                let mut next = crc;
                full_copy_sample(&mut next, &bus).unwrap();
                crc = black_box(next);
            } else {
                let sample = crc.prepare_sample(&bus).unwrap();
                crc.commit_sample(black_box(sample));
            }
            bus.advance_clock(); bus.advance_clock();
        }
        black_box(&crc);
    }
    let elapsed = start.elapsed().as_nanos();
    assert_eq!(bus.tick().get(), words as u64 * 224);
    (elapsed, crc)
}

#[test]
#[ignore = "release-only matched CRC transaction benchmark"]
fn m16k_crc_transaction_realtime_benchmark() {
    assert!(!cfg!(debug_assertions), "run with --release");
    let words = 50_000;
    let mut times = [Vec::new(), Vec::new()];
    benchmark_lane(500, false);
    benchmark_lane(500, true);
    for round in 0..7 {
        let mut images = [None, None];
        for lane in [round % 2, (round + 1) % 2] {
            let (ns, image) = benchmark_lane(words, lane == 1);
            times[lane].push(ns as f64 / words as f64 / 1000.0);
            images[lane] = Some(image);
        }
        assert_eq!(images[0], images[1]);
    }
    println!("\nM16K CRC TRANSACTION / MATCHED PRIOR COPY KERNEL");
    for (lane, name) in ["prepared fields", "prior full copy"].into_iter().enumerate() {
        times[lane].sort_by(f64::total_cmp);
        println!("{name}: median={:.3} us/word best={:.3} worst={:.3}", times[lane][3], times[lane][0], times[lane][6]);
    }
    println!("Scope: rotating same-process CRC+resolved IS/SYNC/224 PHI workload; black-box transaction boundaries. Prior lane preserves M16J nested-copy algorithm only in tests. Not whole-live A/B or hardware timing evidence; no speed threshold.");
}
