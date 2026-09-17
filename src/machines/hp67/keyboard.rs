//! Source-backed HP-67 physical keyboard input.
//!
//! The HP-67 firmware observes two distinct pieces of keyboard state: the
//! hardware key code and status bit S15 indicating that a key contact is still
//! closed.  They must not be collapsed into one optional value.  In particular,
//! firmware clears S15 itself while waiting for release, and a held contact must
//! reassert S15 on the following instruction boundary.

use super::ActArchitecturalState;

pub const HP67_KEY_PRESSED_STATUS_BIT: usize = 15;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Hp67Key {
    A,
    B,
    C,
    D,
    E,
    SigmaPlus,
    Gto,
    Dsp,
    Indirect,
    Sst,
    FunctionF,
    FunctionG,
    Sto,
    Rcl,
    FunctionH,
    Enter,
    ChangeSign,
    Exponent,
    ClearX,
    Subtract,
    Digit7,
    Digit8,
    Digit9,
    Add,
    Digit4,
    Digit5,
    Digit6,
    Multiply,
    Digit1,
    Digit2,
    Digit3,
    Divide,
    Digit0,
    Decimal,
    RunStop,
}

impl Hp67Key {
    /// Return the HP-67 hardware key code. Octal notation mirrors the preserved
    /// firmware listings and the source-backed keyboard definition.
    pub const fn scan_code(self) -> u8 {
        match self {
            Self::A => 0o244,
            Self::B => 0o243,
            Self::C => 0o242,
            Self::D => 0o241,
            Self::E => 0o240,
            Self::SigmaPlus => 0o224,
            Self::Gto => 0o223,
            Self::Dsp => 0o222,
            Self::Indirect => 0o221,
            Self::Sst => 0o220,
            Self::FunctionF => 0o024,
            Self::FunctionG => 0o023,
            Self::Sto => 0o022,
            Self::Rcl => 0o021,
            Self::FunctionH => 0o020,
            Self::Enter => 0o063,
            Self::ChangeSign => 0o062,
            Self::Exponent => 0o061,
            Self::ClearX => 0o060,
            Self::Subtract => 0o103,
            Self::Digit7 => 0o102,
            Self::Digit8 => 0o101,
            Self::Digit9 => 0o100,
            Self::Add => 0o123,
            Self::Digit4 => 0o122,
            Self::Digit5 => 0o121,
            Self::Digit6 => 0o120,
            Self::Multiply => 0o143,
            Self::Digit1 => 0o142,
            Self::Digit2 => 0o141,
            Self::Digit3 => 0o140,
            Self::Divide => 0o163,
            Self::Digit0 => 0o162,
            Self::Decimal => 0o161,
            Self::RunStop => 0o160,
        }
    }
}

/// Physical keyboard state at the ACT instruction boundary.
///
/// `pressed` is the live contact state. `code` is intentionally separate and
/// survives release; releasing a key must not clear either the last key code or
/// ACT S15. The firmware owns S15 clearing with `0 -> s 15`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67Keyboard {
    pressed: Option<Hp67Key>,
    code: Option<u8>,
}

impl Hp67Keyboard {
    pub const fn pressed(&self) -> Option<Hp67Key> {
        self.pressed
    }

    pub const fn code(&self) -> Option<u8> {
        self.code
    }

    pub fn press(&mut self, key: Hp67Key) {
        self.pressed = Some(key);
        self.code = Some(key.scan_code());
    }

    pub fn release(&mut self) {
        self.pressed = None;
    }

    /// Present external keyboard state to the instruction-boundary ACT model.
    ///
    /// A closed contact asserts S15 every boundary, so if firmware executes
    /// `0 -> s 15` while the key is still held the physical input reasserts it
    /// before the next instruction. An open contact does nothing to S15; this is
    /// essential to the firmware's release-detection loop. The last hardware
    /// key code remains available independently of contact state.
    pub fn sample_into_act(&self, act: &mut ActArchitecturalState) {
        if let Some(code) = self.code {
            act.key_buffer = Some(code);
        }
        if self.pressed.is_some() {
            act.status[HP67_KEY_PRESSED_STATUS_BIT] = true;
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::*;

    const ALL_KEYS: [Hp67Key; 35] = [
        Hp67Key::A,
        Hp67Key::B,
        Hp67Key::C,
        Hp67Key::D,
        Hp67Key::E,
        Hp67Key::SigmaPlus,
        Hp67Key::Gto,
        Hp67Key::Dsp,
        Hp67Key::Indirect,
        Hp67Key::Sst,
        Hp67Key::FunctionF,
        Hp67Key::FunctionG,
        Hp67Key::Sto,
        Hp67Key::Rcl,
        Hp67Key::FunctionH,
        Hp67Key::Enter,
        Hp67Key::ChangeSign,
        Hp67Key::Exponent,
        Hp67Key::ClearX,
        Hp67Key::Subtract,
        Hp67Key::Digit7,
        Hp67Key::Digit8,
        Hp67Key::Digit9,
        Hp67Key::Add,
        Hp67Key::Digit4,
        Hp67Key::Digit5,
        Hp67Key::Digit6,
        Hp67Key::Multiply,
        Hp67Key::Digit1,
        Hp67Key::Digit2,
        Hp67Key::Digit3,
        Hp67Key::Divide,
        Hp67Key::Digit0,
        Hp67Key::Decimal,
        Hp67Key::RunStop,
    ];

    #[test]
    fn matrix_contains_35_unique_source_backed_codes() {
        let codes: HashSet<_> = ALL_KEYS.into_iter().map(Hp67Key::scan_code).collect();
        assert_eq!(codes.len(), ALL_KEYS.len());
        assert_eq!(Hp67Key::Digit1.scan_code(), 0o142);
    }

    #[test]
    fn held_contact_reasserts_s15_and_release_does_not_clear_firmware_state() {
        let mut keyboard = Hp67Keyboard::default();
        let mut act = ActArchitecturalState::default();

        keyboard.press(Hp67Key::Digit1);
        keyboard.sample_into_act(&mut act);
        assert_eq!(act.key_buffer, Some(0o142));
        assert!(act.status[HP67_KEY_PRESSED_STATUS_BIT]);

        // Firmware may clear S15 while polling. A physically held key asserts it
        // again at the next instruction boundary.
        act.status[HP67_KEY_PRESSED_STATUS_BIT] = false;
        keyboard.sample_into_act(&mut act);
        assert!(act.status[HP67_KEY_PRESSED_STATUS_BIT]);

        // Opening the contact does not synthesize a status-bit clear and does not
        // discard the last hardware code.
        keyboard.release();
        keyboard.sample_into_act(&mut act);
        assert!(act.status[HP67_KEY_PRESSED_STATUS_BIT]);
        assert_eq!(act.key_buffer, Some(0o142));

        // Once firmware clears S15 after release, the open contact leaves it low.
        act.status[HP67_KEY_PRESSED_STATUS_BIT] = false;
        keyboard.sample_into_act(&mut act);
        assert!(!act.status[HP67_KEY_PRESSED_STATUS_BIT]);
        assert_eq!(act.key_buffer, Some(0o142));
    }
}
