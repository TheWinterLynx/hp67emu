//! Passive HP-67 keyboard connectivity, from keyboard/display PCB sheet 3.
//!
//! No implicit pull, voltage, active scan polarity or ACT latch edge is assigned.
//! Sources are explicit; ideal closed switches connect both directions.
use super::keyboard::Hp67Key;
use crate::emulation::{Drive, LogicLevel};

/// Physical cathode numbers in keyboard schematic order, not scan-slot numbers.
pub const HP67_KEYBOARD_CATHODES: [u8; 8] = [8, 6, 13, 5, 11, 3, 9, 7];
const NODES: usize = 13;

/// A key closes the named cathode wire to one KC column (1..5).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67KeyContact {
    pub cathode: u8,
    pub column: u8,
}

pub const fn key_contact(key: Hp67Key) -> Hp67KeyContact {
    // Enum order follows the physical face rows. The first three have five
    // contacts; remaining rows omit KC1. This is wiring, not ACT encoding.
    let index = key as usize;
    let (row, column) = if index < 15 {
        (index / 5, index % 5 + 1)
    } else {
        ((index - 15) / 4 + 3, (index - 15) % 4 + 2)
    };
    Hp67KeyContact {
        cathode: HP67_KEYBOARD_CATHODES[row],
        column: column as u8,
    }
}

/// Resolved wires include backfeeding into cathodes through multiple contacts.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67KeyboardPins {
    pub cathodes: [LogicLevel; 8],
    pub columns: [LogicLevel; 5],
}

/// Contact-owned graph; connected components are rebuilt only on contact change.
/// Sampling has fixed size and performs no allocation or topology traversal.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67KeyboardMatrix {
    closed: u64,
    components: [u16; NODES],
}
impl Default for Hp67KeyboardMatrix {
    fn default() -> Self {
        Self {
            closed: 0,
            components: std::array::from_fn(|i| 1 << i),
        }
    }
}
impl Hp67KeyboardMatrix {
    pub const fn is_closed(&self, key: Hp67Key) -> bool {
        self.closed & (1u64 << key as usize) != 0
    }
    /// Compatibility single-contact adapter never selects a winner for a chord.
    pub const fn single_key(&self) -> Option<Hp67Key> {
        if self.closed.count_ones() == 1 {
            Some(Hp67Key::ALL[self.closed.trailing_zeros() as usize])
        } else {
            None
        }
    }
    pub fn set_closed(&mut self, key: Hp67Key, closed: bool) {
        let bit = 1u64 << key as usize;
        let next = if closed {
            self.closed | bit
        } else {
            self.closed & !bit
        };
        if next == self.closed {
            return;
        }
        self.closed = next;
        self.components = std::array::from_fn(|i| 1 << i);
        for key in Hp67Key::ALL {
            if !self.is_closed(key) {
                continue;
            }
            let contact = key_contact(key);
            let row = HP67_KEYBOARD_CATHODES
                .iter()
                .position(|&c| c == contact.cathode)
                .expect("wired cathode");
            let column = 8 + usize::from(contact.column - 1);
            let merged = self.components[row] | self.components[column];
            for component in &mut self.components {
                if *component & merged != 0 {
                    *component = merged;
                }
            }
        }
    }
    /// Resolve explicit active sources with no guessed passive bias. High-Z
    /// contributes nothing; disagreeing connected sources expose contention.
    /// Column sources are inputs too: a switch is not a one-way logic gate.
    pub fn resolve(&self, cathodes: [Drive; 8], columns: [Drive; 5]) -> Hp67KeyboardPins {
        let mut low = 0u16;
        let mut high = 0u16;
        for (index, drive) in cathodes.into_iter().chain(columns).enumerate() {
            match drive {
                Drive::HighZ => {}
                Drive::Low => low |= 1 << index,
                Drive::High => high |= 1 << index,
            }
        }
        let levels: [LogicLevel; NODES] = std::array::from_fn(|index| {
            let component = self.components[index];
            match (component & low != 0, component & high != 0) {
                (false, false) => LogicLevel::Floating,
                (true, false) => LogicLevel::Low,
                (false, true) => LogicLevel::High,
                (true, true) => LogicLevel::Contention,
            }
        });
        Hp67KeyboardPins {
            cathodes: std::array::from_fn(|i| levels[i]),
            columns: std::array::from_fn(|i| levels[i + 8]),
        }
    }
}

#[cfg(test)]
#[path = "keyboard_matrix_tests.rs"]
mod tests;
