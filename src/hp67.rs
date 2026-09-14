#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RunMode {
    Run,
    Program,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum KeyAction {
    Digit(u8),
    Decimal,
    ChangeSign,
    ClearX,
    Enter,
    Add,
    Subtract,
    Multiply,
    Divide,
    FunctionF,
    FunctionG,
    FunctionH,
    SigmaPlus,
    Gto,
    Dsp,
    Indirect,
    Sst,
    Sto,
    Rcl,
    A,
    B,
    C,
    D,
    E,
    RunStop,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UiEvent {
    Key(KeyAction),
    TogglePower,
    ToggleMode,
}

/// Temporary UI-facing state model.
///
/// This is not the final HP-67 execution core. Once the ACT/ROM/display scan is
/// emulated, the visible field must come from that machine state rather than
/// from this string model. Until then, its reset appearance mirrors a real
/// powered-on HP-67 closely enough to exercise the renderer.
#[derive(Debug, Clone)]
pub struct Hp67State {
    pub power_on: bool,
    pub mode: RunMode,
    display: String,
    entering: bool,
}

impl Default for Hp67State {
    fn default() -> Self {
        Self {
            power_on: true,
            mode: RunMode::Run,
            display: "0.00".to_owned(),
            entering: false,
        }
    }
}

impl Hp67State {
    #[cfg(test)]
    pub fn for_capture(display: &str) -> Self {
        Self {
            display: display.to_owned(),
            ..Self::default()
        }
    }

    pub fn display_text(&self) -> &str {
        if self.power_on {
            &self.display
        } else {
            ""
        }
    }

    pub fn handle(&mut self, event: UiEvent) {
        match event {
            UiEvent::TogglePower => {
                self.power_on = !self.power_on;
                self.entering = false;
                if self.power_on && self.display.is_empty() {
                    self.display = "0.00".to_owned();
                }
            }
            UiEvent::ToggleMode => {
                self.mode = match self.mode {
                    RunMode::Run => RunMode::Program,
                    RunMode::Program => RunMode::Run,
                };
            }
            UiEvent::Key(key) if self.power_on => self.handle_key(key),
            UiEvent::Key(_) => {}
        }
    }

    fn handle_key(&mut self, key: KeyAction) {
        match key {
            KeyAction::Digit(d) => self.push_digit(d),
            KeyAction::Decimal => self.push_decimal(),
            KeyAction::ChangeSign => self.change_sign(),
            KeyAction::ClearX => {
                self.display.clear();
                self.display.push_str("0.00");
                self.entering = false;
            }
            KeyAction::Enter => self.entering = false,
            KeyAction::Add => self.show_operator('+'),
            KeyAction::Subtract => self.show_operator('-'),
            KeyAction::Multiply => self.show_operator('x'),
            KeyAction::Divide => self.show_operator('/'),
            KeyAction::RunStop => {
                self.entering = false;
            }
            _ => {}
        }
    }

    fn push_digit(&mut self, digit: u8) {
        if digit > 9 {
            return;
        }

        if !self.entering || self.display == "0" || self.display == "0.00" {
            self.display.clear();
            self.entering = true;
        }

        let digit_count = self.display.chars().filter(|c| c.is_ascii_digit()).count();
        if digit_count < 10 {
            self.display.push(char::from(b'0' + digit));
        }
    }

    fn push_decimal(&mut self) {
        if !self.entering {
            self.display.clear();
            self.display.push('0');
            self.entering = true;
        }

        if !self.display.contains('.') {
            self.display.push('.');
        }
    }

    fn change_sign(&mut self) {
        if self.display.starts_with('-') {
            self.display.remove(0);
        } else if self.display != "0" && self.display != "0.00" {
            self.display.insert(0, '-');
        }
    }

    fn show_operator(&mut self, op: char) {
        // Visual smoke-test only. The real RPN core will own arithmetic and the
        // physical display scan state.
        self.display.clear();
        self.display.push(op);
        self.entering = false;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn power_on_placeholder_matches_real_hp67() {
        let hp = Hp67State::default();
        assert_eq!(hp.display_text(), "0.00");
    }

    #[test]
    fn demo_entry_handles_digits_decimal_and_sign() {
        let mut hp = Hp67State::default();
        hp.handle(UiEvent::Key(KeyAction::Digit(1)));
        hp.handle(UiEvent::Key(KeyAction::Digit(2)));
        hp.handle(UiEvent::Key(KeyAction::Decimal));
        hp.handle(UiEvent::Key(KeyAction::Digit(5)));
        hp.handle(UiEvent::Key(KeyAction::ChangeSign));
        assert_eq!(hp.display_text(), "-12.5");
    }

    #[test]
    fn clear_x_returns_to_power_on_placeholder() {
        let mut hp = Hp67State::default();
        hp.handle(UiEvent::Key(KeyAction::Digit(7)));
        hp.handle(UiEvent::Key(KeyAction::ClearX));
        assert_eq!(hp.display_text(), "0.00");
    }

    #[test]
    fn power_blanks_display_without_losing_value() {
        let mut hp = Hp67State::default();
        hp.handle(UiEvent::Key(KeyAction::Digit(7)));
        hp.handle(UiEvent::TogglePower);
        assert_eq!(hp.display_text(), "");
        hp.handle(UiEvent::TogglePower);
        assert_eq!(hp.display_text(), "7");
    }

    #[test]
    fn mode_toggles() {
        let mut hp = Hp67State::default();
        assert_eq!(hp.mode, RunMode::Run);
        hp.handle(UiEvent::ToggleMode);
        assert_eq!(hp.mode, RunMode::Program);
    }
}
