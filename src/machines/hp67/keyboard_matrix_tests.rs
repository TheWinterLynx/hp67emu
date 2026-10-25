use super::super::keyboard::Hp67Keyboard;
use super::*;

// Transcribed independently from PDF page 8, keyboard/display PCB sheet 3.
const WIRES: [(u8, u8); 35] = [
    (8, 1),
    (8, 2),
    (8, 3),
    (8, 4),
    (8, 5),
    (6, 1),
    (6, 2),
    (6, 3),
    (6, 4),
    (6, 5),
    (13, 1),
    (13, 2),
    (13, 3),
    (13, 4),
    (13, 5),
    (5, 2),
    (5, 3),
    (5, 4),
    (5, 5),
    (11, 2),
    (11, 3),
    (11, 4),
    (11, 5),
    (3, 2),
    (3, 3),
    (3, 4),
    (3, 5),
    (9, 2),
    (9, 3),
    (9, 4),
    (9, 5),
    (7, 2),
    (7, 3),
    (7, 4),
    (7, 5),
];

#[test]
fn m16d_all_35_contacts_match_physical_wires_and_have_unique_coordinates() {
    let mut seen = std::collections::HashSet::new();
    for (key, (cathode, column)) in Hp67Key::ALL.into_iter().zip(WIRES) {
        assert_eq!(key_contact(key), Hp67KeyContact { cathode, column });
        assert!(seen.insert((cathode, column)));
    }
    assert_eq!(seen.len(), 35);
}

#[test]
fn m16d_every_contact_conducts_both_directions_and_release_is_floating() {
    for key in Hp67Key::ALL {
        let mut matrix = Hp67KeyboardMatrix::default();
        let contact = key_contact(key);
        let row = HP67_KEYBOARD_CATHODES
            .iter()
            .position(|&c| c == contact.cathode)
            .unwrap();
        let column = usize::from(contact.column - 1);
        matrix.set_closed(key, true);
        assert_eq!(matrix.single_key(), Some(key));
        for (drive, expected) in [
            (Drive::HighZ, LogicLevel::Floating),
            (Drive::Low, LogicLevel::Low),
            (Drive::High, LogicLevel::High),
        ] {
            let mut cathodes = [Drive::HighZ; 8];
            cathodes[row] = drive;
            let pins = matrix.resolve(cathodes, [Drive::HighZ; 5]);
            for c in 0..5 {
                assert_eq!(
                    pins.columns[c],
                    if c == column {
                        expected
                    } else {
                        LogicLevel::Floating
                    }
                );
            }
            let mut columns = [Drive::HighZ; 5];
            columns[column] = drive;
            let pins = matrix.resolve([Drive::HighZ; 8], columns);
            for r in 0..8 {
                assert_eq!(
                    pins.cathodes[r],
                    if r == row {
                        expected
                    } else {
                        LogicLevel::Floating
                    }
                );
            }
        }
        matrix.set_closed(key, false);
        assert_eq!(matrix, Hp67KeyboardMatrix::default());
        assert_eq!(
            matrix.resolve([Drive::HighZ; 8], [Drive::HighZ; 5]).columns,
            [LogicLevel::Floating; 5]
        );
    }
}

#[test]
fn m16d_three_corners_backfeed_without_inventing_a_fourth_key() {
    let mut matrix = Hp67KeyboardMatrix::default();
    for key in [Hp67Key::A, Hp67Key::B, Hp67Key::SigmaPlus] {
        matrix.set_closed(key, true);
    }
    let mut sources = [Drive::HighZ; 8];
    sources[1] = Drive::Low; // cathode 6 -> Sigma+ -> KC1 -> A -> cathode 8 -> B -> KC2
    let pins = matrix.resolve(sources, [Drive::HighZ; 5]);
    assert_eq!(&pins.columns[..2], &[LogicLevel::Low; 2]);
    assert_eq!(&pins.cathodes[..2], &[LogicLevel::Low; 2]);
    assert!(!matrix.is_closed(Hp67Key::Gto));
    assert_eq!(matrix.single_key(), None);
    matrix.set_closed(Hp67Key::A, false);
    assert_eq!(
        matrix.resolve(sources, [Drive::HighZ; 5]).columns[1],
        LogicLevel::Floating
    );
}

#[test]
fn m16d_connected_opposite_sources_contend_and_agreeing_sources_do_not() {
    let mut matrix = Hp67KeyboardMatrix::default();
    for key in [Hp67Key::A, Hp67Key::SigmaPlus] {
        matrix.set_closed(key, true);
    }
    let mut sources = [Drive::HighZ; 8];
    sources[0] = Drive::Low;
    sources[1] = Drive::High;
    let pins = matrix.resolve(sources, [Drive::HighZ; 5]);
    assert_eq!(pins.columns[0], LogicLevel::Contention);
    assert_eq!(&pins.cathodes[..2], &[LogicLevel::Contention; 2]);
    sources[0] = Drive::High;
    assert_eq!(
        matrix.resolve(sources, [Drive::HighZ; 5]).columns[0],
        LogicLevel::High
    );
    matrix.set_closed(Hp67Key::SigmaPlus, false);
    sources[1] = Drive::Low;
    assert_eq!(
        matrix.resolve(sources, [Drive::HighZ; 5]).columns[0],
        LogicLevel::High
    );
}

// Independent per-sample reachability oracle: no cached component union.
fn walk_levels(keys: &[Hp67Key], sources: [Drive; 13]) -> [LogicLevel; 13] {
    let mut edges = [[false; 13]; 13];
    for key in keys {
        let (cathode, column) = WIRES[*key as usize];
        let row = HP67_KEYBOARD_CATHODES
            .iter()
            .position(|&c| c == cathode)
            .unwrap();
        let column = 8 + usize::from(column - 1);
        edges[row][column] = true;
        edges[column][row] = true;
    }
    std::array::from_fn(|root| {
        let mut reached = [false; 13];
        let mut pending = vec![root];
        let (mut low, mut high) = (false, false);
        while let Some(node) = pending.pop() {
            if reached[node] {
                continue;
            }
            reached[node] = true;
            low |= sources[node] == Drive::Low;
            high |= sources[node] == Drive::High;
            for next in 0..13 {
                if edges[node][next] && !reached[next] {
                    pending.push(next);
                }
            }
        }
        match (low, high) {
            (false, false) => LogicLevel::Floating,
            (true, false) => LogicLevel::Low,
            (false, true) => LogicLevel::High,
            (true, true) => LogicLevel::Contention,
        }
    })
}

#[test]
fn m16d_all_two_key_chords_and_all_source_pairs_match_independent_walk() {
    for (a, &first) in Hp67Key::ALL.iter().enumerate() {
        for &second in &Hp67Key::ALL[a..] {
            let mut matrix = Hp67KeyboardMatrix::default();
            for key in [first, second] {
                matrix.set_closed(key, true);
            }
            for low in 0..13 {
                for high in 0..13 {
                    let mut sources = [Drive::HighZ; 13];
                    sources[low] = Drive::Low;
                    sources[high] = Drive::High;
                    let expected = walk_levels(&[first, second], sources);
                    let pins = matrix.resolve(
                        std::array::from_fn(|i| sources[i]),
                        std::array::from_fn(|i| sources[i + 8]),
                    );
                    assert_eq!(pins.cathodes, std::array::from_fn(|i| expected[i]));
                    assert_eq!(pins.columns, std::array::from_fn(|i| expected[i + 8]));
                }
            }
        }
    }
}

#[test]
fn m16d_contact_order_idempotence_and_removal_preserve_topology() {
    let mut forward = Hp67KeyboardMatrix::default();
    let mut reverse = forward;
    for key in Hp67Key::ALL {
        forward.set_closed(key, true);
        forward.set_closed(key, true);
    }
    for key in Hp67Key::ALL.into_iter().rev() {
        reverse.set_closed(key, true);
    }
    assert_eq!(forward, reverse);
    let pins = forward.resolve([Drive::High; 8], [Drive::HighZ; 5]);
    assert_eq!(pins.columns, [LogicLevel::High; 5]);
    for key in Hp67Key::ALL {
        forward.set_closed(key, false);
        forward.set_closed(key, false);
    }
    assert_eq!(forward, Hp67KeyboardMatrix::default());
}

#[test]
fn m16d_production_single_contact_adapter_uses_matrix_and_retains_legacy_latch() {
    let mut keyboard = Hp67Keyboard::default();
    for key in Hp67Key::ALL {
        keyboard.press(key);
        assert_eq!(keyboard.pressed(), Some(key));
        for other in Hp67Key::ALL {
            assert_eq!(keyboard.contacts().is_closed(other), other == key);
        }
        assert_eq!(keyboard.code(), Some(key.scan_code()));
        keyboard.release();
        assert_eq!(keyboard.pressed(), None);
        assert_eq!(keyboard.contacts(), &Hp67KeyboardMatrix::default());
        assert_eq!(keyboard.code(), Some(key.scan_code()));
    }
}
