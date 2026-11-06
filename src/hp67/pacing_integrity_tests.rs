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
