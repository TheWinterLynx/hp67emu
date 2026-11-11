//! Isolated firmware-backed samples for the desktop plotter.

use super::*;
use hp67emu::machines::hp67::Hp67CardTrack;

const KEY_LIMIT: usize = 512;
const RESULT_LIMIT: usize = 30_000;
const CARD_LIMIT: usize = 65_536;
const DIGITS: [u8; 10] = [0x3f, 0x06, 0x5b, 0x4f, 0x66, 0x6d, 0x7d, 0x07, 0x7f, 0x6f];

pub(crate) struct PolynomialSampler(Hp67LiveMachine);

impl PolynomialSampler {
    pub(crate) fn new(coefficients: [f64; 4]) -> Result<Self, String> {
        use Hp67Key::*;
        let mut live = Hp67LiveMachine::power_on_default()?;
        live.advance(Duration::from_micros(HP67_OBSERVED_POWER_ON_SYNC_DELAY_US))?;
        for _ in 0..2_000 {
            if live.phase == LiveBootPhase::Idle {
                break;
            }
            live.step_firmware_cycle()?;
        }
        if live.phase != LiveBootPhase::Idle {
            return Err("El firmware no terminó de arrancar.".into());
        }

        let entry = crate::program_library::PROGRAM_LIBRARY
            .iter()
            .find(|entry| entry.reference == "SD1-09A")
            .ok_or("No se encuentra SD1-09A en la biblioteca.")?;
        let mut card = entry.load_card()?.card;
        for (track, end) in [
            (Hp67CardTrack::Track1, CardInsertionEnd::End1),
            (Hp67CardTrack::Track2, CardInsertionEnd::End2),
        ] {
            if !card.track(track).is_recorded() {
                return Err("A SD1-09A le falta una pista de programa.".into());
            }
            if end == CardInsertionEnd::End2 {
                for _ in 0..16_384 {
                    if live.card_prompt_visible() {
                        break;
                    }
                    live.step_firmware_cycle()?;
                }
                if !live.card_prompt_visible() {
                    return Err("El firmware no pidió la segunda pista (Crd).".into());
                }
            }
            live.insert_magnetic_card(card, end)?;
            for _ in 0..CARD_LIMIT {
                if live.card_transport_complete() {
                    break;
                }
                live.step_firmware_cycle()?;
            }
            card = live
                .take_completed_magnetic_card()
                .ok_or("El lector no devolvió la tarjeta.")?;
        }
        let mut sampler = Self(live);
        sampler.wait_for_idle()?;
        sampler.press(FunctionF)?;
        sampler.press(A)?;
        for (value, key) in coefficients.into_iter().zip([B, C, D, E]) {
            sampler.enter_number(value)?;
            sampler.press(key)?;
        }
        Ok(sampler)
    }

    pub(crate) fn sample(&mut self, x: f64) -> Result<[f64; 2], String> {
        let entered = self.enter_number(x)?;
        self.press(Hp67Key::A)?;
        // The next completed scan can still contain part of the previous
        // display. Observe a whole subsequent scan after the idle checkpoint.
        let initial = self.0.display_circuit.exposure().completed_scans();
        for _ in 0..1_024 {
            self.0.step_firmware_cycle()?;
            if self
                .0
                .display_circuit
                .exposure()
                .completed_scans()
                .saturating_sub(initial)
                >= 2
            {
                return display_number(self.0.optical_display_frame()).map(|y| [entered, y]);
            }
        }
        Err("La pantalla no completó un barrido.".into())
    }

    fn wait_for_idle(&mut self) -> Result<(), String> {
        let live = &mut self.0;
        let before = live.main_wait_visits;
        for _ in 0..RESULT_LIMIT {
            live.step_firmware_cycle()?;
            let status = &live.machine.act.state.status;
            if live.main_wait_visits > before && !status[15] && !status[2] && !live.card_motor_on()
            {
                return Ok(());
            }
        }
        Err("El firmware no volvió a la espera de teclado.".into())
    }

    fn press(&mut self, key: Hp67Key) -> Result<(), String> {
        let live = &mut self.0;
        live.set_key_contact(Some(key));
        let dispatch = (|| {
            let mut scanned = false;
            for _ in 0..KEY_LIMIT {
                let Some(execution) = live.step_firmware_cycle_with_execution()? else {
                    continue;
                };
                if let Hp67ArchitecturalOperation::Act(ActOperation::Special { opcode }) =
                    execution.operation
                {
                    match opcode {
                        0o0120 => {
                            let a = &live.machine.act.state.a;
                            scanned = ((a[2] << 4) | a[1]) == key.scan_code();
                            if !scanned {
                                return Err(format!("Código de teclado inesperado para {key:?}."));
                            }
                        }
                        0o0220 => {
                            return if scanned {
                                Ok(())
                            } else {
                                Err(format!("Salto de teclado sin lectura de {key:?}."))
                            };
                        }
                        _ => {}
                    }
                }
            }
            Err(format!("El firmware no reconoció {key:?}."))
        })();
        live.set_key_contact(None);
        dispatch?;
        self.wait_for_idle()
    }

    fn enter_number(&mut self, value: f64) -> Result<f64, String> {
        use Hp67Key::*;
        let (mantissa, exponent) = number_entry(value)?;
        for c in mantissa.trim_start_matches('-').chars() {
            let key = match c {
                '.' => Decimal,
                '0'..='9' => digit_key(c),
                _ => return Err("Entrada numérica no válida.".into()),
            };
            self.press(key)?;
        }
        if mantissa.starts_with('-') {
            self.press(ChangeSign)?;
        }
        if exponent != 0 {
            self.press(Exponent)?;
            if exponent < 0 {
                self.press(ChangeSign)?;
            }
            for c in exponent.unsigned_abs().to_string().chars() {
                self.press(digit_key(c))?;
            }
        }
        format!("{mantissa}e{exponent}")
            .parse()
            .map_err(|_| "Entrada numérica no válida.".into())
    }
}

fn digit_key(c: char) -> Hp67Key {
    use Hp67Key::*;
    [
        Digit0, Digit1, Digit2, Digit3, Digit4, Digit5, Digit6, Digit7, Digit8, Digit9,
    ][c as usize - '0' as usize]
}

fn number_entry(value: f64) -> Result<(String, i32), String> {
    if !value.is_finite() || value.abs() > 1e20 || (value != 0.0 && value.abs() < 1e-20) {
        return Err("Usa cero o una magnitud entre 10⁻²⁰ y 10²⁰.".into());
    }
    let scientific = format!("{value:.8e}");
    let (mantissa, exponent) = scientific.split_once('e').ok_or("Falta el exponente.")?;
    let exponent: i32 = exponent.parse().map_err(|_| "Exponente no válido.")?;
    let fixed;
    let (mantissa, exponent) = if (-4..=8).contains(&exponent) {
        let precision = (8 - exponent) as usize;
        fixed = format!("{value:.precision$}");
        if fixed
            .trim_end_matches('0')
            .chars()
            .filter(char::is_ascii_digit)
            .count()
            <= 10
        {
            (fixed.as_str(), 0)
        } else {
            (mantissa, exponent)
        }
    } else {
        (mantissa, exponent)
    };
    let mantissa = if mantissa.contains('.') {
        mantissa.trim_end_matches('0').trim_end_matches('.')
    } else {
        mantissa
    };
    Ok((mantissa.to_owned(), exponent))
}

fn display_number(frame: HardwareDisplayFrame) -> Result<f64, String> {
    const MINUS: u8 = 0x40;
    const DECIMAL: u8 = 0x80;
    let slots = frame.segments;
    let invalid = || format!("La pantalla no muestra un número: {slots:02x?}");
    let digit = |mask| DIGITS.iter().position(|&value| value == mask);
    let mut number = String::new();
    match slots[0] {
        0 => {}
        MINUS => number.push('-'),
        _ => return Err(invalid()),
    }
    let mut blank = false;
    for mask in &slots[1..12] {
        if *mask == 0 {
            blank = true;
            continue;
        }
        if blank {
            return Err(invalid());
        }
        if mask & !DECIMAL != 0 {
            let n = digit(mask & !DECIMAL).ok_or_else(invalid)?;
            number.push(char::from(b'0' + n as u8));
        }
        if mask & DECIMAL != 0 {
            number.push('.');
        }
    }
    match slots[12..] {
        [0, 0, 0] => {}
        [sign @ (0 | MINUS), tens, units] => {
            let tens = digit(tens).ok_or_else(invalid)?;
            let units = digit(units).ok_or_else(invalid)?;
            number.push('e');
            if sign == MINUS {
                number.push('-');
            }
            number.push_str(&(10 * tens + units).to_string());
        }
        _ => return Err(invalid()),
    }
    number.parse().map_err(|_| invalid())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plot_reads_signed_fixed_and_scientific_display() {
        let mut frame = HardwareDisplayFrame::BLANK;
        frame.segments[..6].copy_from_slice(&[0x40, 0x06, 0x5b, 0x80, 0x4f, 0x66]);
        assert_eq!(display_number(frame).unwrap(), -12.34);
        frame.segments[12..].copy_from_slice(&[0x40, 0x3f, 0x4f]);
        assert_eq!(display_number(frame).unwrap(), -0.01234);
        assert!(display_number(HardwareDisplayFrame::BLANK).is_err());
    }

    #[test]
    fn plot_enters_numbers_with_bounded_significant_digits() {
        assert_eq!(number_entry(400.0).unwrap(), ("400".into(), 0));
        assert_eq!(number_entry(-7.596).unwrap(), ("-7.596".into(), 0));
        assert_eq!(number_entry(-0.742e-6).unwrap(), ("-7.42".into(), -7));
        assert!(number_entry(f64::NAN).is_err());
    }

    #[test]
    #[ignore = "release gate exercises the real firmware and both physical card passes"]
    fn plot_official_polynomial_samples_match_standard_pac() {
        let mut sampler = PolynomialSampler::new([-9140.0, -7.596, 4.243e-3, -0.742e-6]).unwrap();
        for (x, expected) in [(400.0, -11547.01), (600.0, -12330.39), (800.0, -12881.18)] {
            assert_eq!(sampler.sample(x).unwrap(), [x, expected]);
        }
    }
}
