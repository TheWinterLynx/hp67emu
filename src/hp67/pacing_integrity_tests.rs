use super::*;

fn assert_same_machine(left: &Hp67LiveMachine, right: &Hp67LiveMachine) {
    assert_eq!(left.phase, right.phase);
    assert_eq!(left.boot_cycle, right.boot_cycle);
    assert_eq!(left.backplane.tick(), right.backplane.tick());
    assert_eq!(left.pending_us, right.pending_us);
    assert_eq!(left.pending_submicro_ns, right.pending_submicro_ns);
    assert_eq!(left.machine.act.state, right.machine.act.state);
    assert_eq!(left.machine.crc, right.machine.crc);
    assert_eq!(left.act_serial, right.act_serial);
    assert_eq!(left.serial_ram, right.serial_ram);
    assert_eq!(left.owned_crc_data, right.owned_crc_data);
    assert_eq!(left.card_transport, right.card_transport);
    assert_eq!(left.display_frame(), right.display_frame());
}

#[test]
fn core_audit_duration_partitions_preserve_real_firmware_and_reset_hold() {
    let total_ns =
        (HP67_OBSERVED_POWER_ON_SYNC_DELAY_US + 400 * HP67_OBSERVED_WORD_TIME_US) * 1000 + 731;
    let mut whole = Hp67LiveMachine::power_on_default().unwrap();
    whole.advance(Duration::from_nanos(total_ns)).unwrap();
    assert_eq!(whole.boot_cycle, 400);
    for chunks in [&[500_u64][..], &[1, 999, 31_999, 320_001, 731][..]] {
        let mut split = Hp67LiveMachine::power_on_default().unwrap();
        let mut remaining = total_ns;
        let mut index = 0;
        while remaining != 0 {
            let ns = remaining.min(chunks[index % chunks.len()]);
            split.advance(Duration::from_nanos(ns)).unwrap();
            remaining -= ns;
            index += 1;
        }
        assert_same_machine(&whole, &split);
    }
}

#[test]
fn core_audit_reset_clears_fractional_pacing_debt() {
    let mut reset = Hp67LiveMachine::power_on_default().unwrap();
    reset.advance(Duration::from_nanos(999)).unwrap();
    reset.reset_power_on().unwrap();
    let mut fresh = Hp67LiveMachine::power_on_default().unwrap();
    let ns = HP67_OBSERVED_POWER_ON_SYNC_DELAY_US * 1000 - 1;
    reset.advance(Duration::from_nanos(ns)).unwrap();
    fresh.advance(Duration::from_nanos(ns)).unwrap();
    assert_same_machine(&reset, &fresh);
    assert_eq!(reset.phase, LiveBootPhase::ResetHold);
    reset.advance(Duration::from_nanos(1)).unwrap();
    fresh.advance(Duration::from_nanos(1)).unwrap();
    assert_same_machine(&reset, &fresh);
    assert_eq!(reset.phase, LiveBootPhase::Firmware);
}

#[test]
fn core_audit_reset_rejects_retained_card_before_any_electronic_mutation() {
    for end in [CardInsertionEnd::End1, CardInsertionEnd::End2] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        let mut control = Hp67LiveMachine::power_on_default().unwrap();
        for machine in [&mut live, &mut control] {
            machine.advance(Duration::from_millis(200)).unwrap();
            machine.set_program_mode(true).unwrap();
            machine.set_key_contact(Some(Hp67Key::Digit1));
            machine.advance(Duration::from_nanos(731)).unwrap();
            machine
                .insert_magnetic_card(Hp67MagneticCard::default(), end)
                .unwrap();
        }
        assert!(live
            .reset_power_on()
            .unwrap_err()
            .contains("magnetic media"));
        assert_same_machine(&live, &control);
        assert_eq!(live.keyboard.pressed(), Some(Hp67Key::Digit1));
        // A rejected reset must also preserve hidden receiver state: subsequent
        // real words must remain identical to a machine that was never reset.
        for _ in 0..32 {
            assert_eq!(
                live.step_firmware_cycle_with_execution().unwrap(),
                control.step_firmware_cycle_with_execution().unwrap()
            );
        }
        assert_same_machine(&live, &control);
        let card = live.take_magnetic_card_for_power_off().unwrap();
        assert_eq!(card, control.take_magnetic_card_for_power_off().unwrap());
        assert!(card.is_some());
        live.reset_power_on().unwrap();
        control.reset_power_on().unwrap();
        assert_same_machine(&live, &control);
    }
}

#[test]
fn core_audit_early_key_is_not_a_boot_watchdog_failure() {
    for release in [false, true] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.advance(Duration::from_micros(
            HP67_OBSERVED_POWER_ON_SYNC_DELAY_US + 2 * HP67_OBSERVED_WORD_TIME_US,
        ))
        .unwrap();
        live.set_key_contact(Some(Hp67Key::Digit1));
        live.advance(Duration::from_micros(HP67_OBSERVED_WORD_TIME_US))
            .unwrap();
        if release {
            live.set_key_contact(None);
        }
        live.advance(Duration::from_micros(2100 * HP67_OBSERVED_WORD_TIME_US))
            .unwrap();
        assert_eq!(live.boot_cycle, 2103);
        assert_eq!(live.backplane.tick().get(), 2103 * 224);
        assert!(live.machine.act.state.key_buffer.is_some());
        assert_eq!(
            live.keyboard.pressed(),
            if release { None } else { Some(Hp67Key::Digit1) }
        );
    }
}

#[test]
fn core_audit_public_pacing_fault_is_terminal_until_reset() {
    use hp67emu::emulation::Drive;
    use hp67emu::machines::hp67::{Hp67Driver, Hp67Net};
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    live.advance(Duration::from_millis(200)).unwrap();
    let tick = live.backplane.tick();
    live.backplane
        .drive(Hp67Net::Phi1, Hp67Driver::External, Drive::High);
    let error = live.advance(Duration::from_micros(320)).unwrap_err();
    assert!(error.contains("invalid PHI image"));
    assert_eq!(live.backplane.tick(), tick);
    live.backplane
        .drive(Hp67Net::Phi1, Hp67Driver::External, Drive::HighZ);
    let debt = (live.pending_us, live.pending_submicro_ns);
    for elapsed in [Duration::ZERO, Duration::from_secs(1)] {
        assert_eq!(live.advance(elapsed), Err(error.clone()));
        assert_eq!(live.backplane.tick(), tick);
        assert_eq!((live.pending_us, live.pending_submicro_ns), debt);
    }
    live.reset_power_on().unwrap();
    let mut fresh = Hp67LiveMachine::power_on_default().unwrap();
    live.advance(Duration::from_millis(200)).unwrap();
    fresh.advance(Duration::from_millis(200)).unwrap();
    assert_same_machine(&live, &fresh);
}

#[test]
fn core_audit_fault_recovery_keeps_partial_write_and_insertion_end() {
    use hp67emu::emulation::Drive;
    use hp67emu::machines::hp67::{Hp67Driver, Hp67Net};
    for end in [CardInsertionEnd::End1, CardInsertionEnd::End2] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        live.insert_magnetic_card(Hp67MagneticCard::default(), end)
            .unwrap();
        // Exercise the actual transport write path independently of firmware
        // startup, then fault electronics before attempting power-off recovery.
        let mut crc = hp67emu::machines::hp67::CrcArchitecturalCore::default();
        crc.execute_opcode(0o660).unwrap();
        live.card_transport.set_head_active(true);
        live.card_transport.advance_us(0, true, &mut crc).unwrap();
        crc.execute_opcode(0o100).unwrap();
        live.card_transport.advance_us(0, true, &mut crc).unwrap();
        crc.queue_write_word(0x0123_4567).unwrap();
        live.card_transport
            .advance_us(28_000, true, &mut crc)
            .unwrap();
        let expected = live.card_transport.card().unwrap().clone();
        assert!(expected.track(end.track()).dirty());
        assert_eq!(expected.track(end.track()).word(0), Some(0x0123_4567));
        live.backplane.drive(
            Hp67Net::Hds,
            Hp67Driver::CardReaderController1820_1751,
            Drive::High,
        );
        assert!(live.take_magnetic_card_for_power_off().is_err());
        assert!(live.reset_power_on().is_err());
        assert_eq!(live.into_retained_card(), Some((expected, end)));
    }
}
