//! Source-backed HP-67 DATA serial-phase primitives.
//!
//! Direct HP-67 captures establish a continuous 56-bit DATA stream whose
//! serial bit 0 appears at machine-word bit b2. Bits 54 and 55 therefore appear
//! at b0/b1 of the following machine word. This module models that cross-word
//! phase exactly at the logical-bit level while deliberately leaving electrical
//! polarity, passive bias and PHI-relative launch/sample edges unresolved.

use super::{
    act::{ActArchitecturalState, ActInstructionState, ActRegister, ACT_WORD_DIGITS},
    timing::{data_serial_bit_for_word_bit, BITS_PER_DIGIT, BITS_PER_WORD},
};

const DATA_TAIL_BITS: u8 = 2;
const DATA_BODY_LAST_WORD_BIT: u8 = BITS_PER_WORD - 1;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67DataTransferDirection {
    ActToPeripheral,
    PeripheralToAct,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67DataTransferPlan {
    pub direction: Hp67DataTransferDirection,
    pub address: u8,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67DataSerialError {
    UnexpectedWordBit { expected: u8, actual: u8 },
    MissingLogicalBit { word_bit: u8, serial_bit: u8 },
    UnexpectedLogicalBit { word_bit: u8, serial_bit: u8 },
    OverlappingFrameStart,
}

/// Decode the ACT instructions that exchange one 56-bit register through DATA.
///
/// This is an instruction-semantic classification only. It does not claim DATA
/// electrical polarity or a PHI launch/sample edge.
pub const fn act_data_transfer_plan(
    word: u16,
    state: &ActArchitecturalState,
) -> Option<Hp67DataTransferPlan> {
    if state.instruction_state == ActInstructionState::ThenGoto {
        return None;
    }

    let low = word & 0o77;
    let operand = ((word >> 6) & 0x0f) as u8;
    if word == 0o1360 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::ActToPeripheral,
            address: state.ram_address,
        })
    } else if word == 0o0070 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::PeripheralToAct,
            address: state.ram_address,
        })
    } else if low == 0o50 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::ActToPeripheral,
            address: (state.ram_address & 0xf0) | operand,
        })
    } else if low == 0o70 {
        Some(Hp67DataTransferPlan {
            direction: Hp67DataTransferDirection::PeripheralToAct,
            address: (state.ram_address & 0xf0) | operand,
        })
    } else {
        None
    }
}

pub const fn data_register_serial_bit(register: &ActRegister, serial_bit: u8) -> bool {
    debug_assert!(serial_bit < BITS_PER_WORD);
    let digit = (serial_bit / BITS_PER_DIGIT) as usize;
    let bit_in_digit = serial_bit % BITS_PER_DIGIT;
    ((register[digit] >> bit_in_digit) & 1) != 0
}

fn set_data_register_serial_bit(register: &mut ActRegister, serial_bit: u8, bit: bool) {
    debug_assert!(serial_bit < BITS_PER_WORD);
    let digit = (serial_bit / BITS_PER_DIGIT) as usize;
    let bit_in_digit = serial_bit % BITS_PER_DIGIT;
    let mask = 1u8 << bit_in_digit;
    if bit {
        register[digit] |= mask;
    } else {
        register[digit] &= !mask;
    }
}

/// Continuous DATA source whose b0/b1 output belongs to the frame started in
/// the preceding machine word and whose b2..b55 output belongs to this word.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67DataSerialSource {
    previous_tail: Option<[bool; DATA_TAIL_BITS as usize]>,
    current: Option<ActRegister>,
}

impl Hp67DataSerialSource {
    pub fn begin_word(&mut self, current: Option<ActRegister>) {
        self.current = current;
    }

    pub fn logical_bit_for_word_bit(&self, word_bit: u8) -> Option<bool> {
        debug_assert!(word_bit < BITS_PER_WORD);
        match word_bit {
            0 | 1 => self.previous_tail.map(|tail| tail[word_bit as usize]),
            _ => self.current.as_ref().map(|register| {
                data_register_serial_bit(register, data_serial_bit_for_word_bit(word_bit))
            }),
        }
    }

    pub fn complete_word(&mut self) {
        self.previous_tail = self.current.as_ref().map(|register| {
            [
                data_register_serial_bit(register, BITS_PER_WORD - 2),
                data_register_serial_bit(register, BITS_PER_WORD - 1),
            ]
        });
        self.current = None;
    }
}

/// Continuous DATA sink that completes a frame only after consuming b0/b1 of
/// the following machine word.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67DataSerialSink {
    expected_word_bit: u8,
    start_frame_at_b2: bool,
    assembling: Option<ActRegister>,
}

impl Default for Hp67DataSerialSink {
    fn default() -> Self {
        Self {
            expected_word_bit: 0,
            start_frame_at_b2: false,
            assembling: None,
        }
    }
}

impl Hp67DataSerialSink {
    pub fn begin_word(&mut self, start_frame_at_b2: bool) {
        debug_assert_eq!(self.expected_word_bit, 0);
        self.start_frame_at_b2 = start_frame_at_b2;
    }

    pub fn sample_word_bit(
        &mut self,
        word_bit: u8,
        logical_bit: Option<bool>,
    ) -> Result<Option<ActRegister>, Hp67DataSerialError> {
        if word_bit != self.expected_word_bit {
            return Err(Hp67DataSerialError::UnexpectedWordBit {
                expected: self.expected_word_bit,
                actual: word_bit,
            });
        }

        let serial_bit = data_serial_bit_for_word_bit(word_bit);
        let mut completed = None;

        if word_bit == 2 && self.start_frame_at_b2 {
            if self.assembling.is_some() {
                return Err(Hp67DataSerialError::OverlappingFrameStart);
            }
            self.assembling = Some([0; ACT_WORD_DIGITS]);
        }

        match (self.assembling.as_mut(), logical_bit) {
            (Some(register), Some(bit)) => {
                set_data_register_serial_bit(register, serial_bit, bit);
                if word_bit == 1 {
                    completed = self.assembling.take();
                }
            }
            (Some(_), None) => {
                return Err(Hp67DataSerialError::MissingLogicalBit {
                    word_bit,
                    serial_bit,
                })
            }
            (None, Some(_)) => {
                return Err(Hp67DataSerialError::UnexpectedLogicalBit {
                    word_bit,
                    serial_bit,
                })
            }
            (None, None) => {}
        }

        self.expected_word_bit = if word_bit == DATA_BODY_LAST_WORD_BIT {
            0
        } else {
            word_bit + 1
        };
        Ok(completed)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn patterned_register(seed: u8) -> ActRegister {
        std::array::from_fn(|digit| (seed.wrapping_add(digit as u8 * 3)) & 0x0f)
    }

    #[test]
    fn register_bits_are_digit_lsb_first() {
        let mut register = [0; ACT_WORD_DIGITS];
        register[0] = 0b1010;
        register[1] = 0b0101;

        assert!(!data_register_serial_bit(&register, 0));
        assert!(data_register_serial_bit(&register, 1));
        assert!(!data_register_serial_bit(&register, 2));
        assert!(data_register_serial_bit(&register, 3));
        assert!(data_register_serial_bit(&register, 4));
        assert!(!data_register_serial_bit(&register, 5));
    }

    #[test]
    fn source_keeps_previous_bits_54_55_at_next_word_b0_b1() {
        let first = patterned_register(1);
        let second = patterned_register(7);
        let mut source = Hp67DataSerialSource::default();

        source.begin_word(Some(first));
        assert_eq!(
            source.logical_bit_for_word_bit(2),
            Some(data_register_serial_bit(&first, 0))
        );
        source.complete_word();

        source.begin_word(Some(second));
        assert_eq!(
            source.logical_bit_for_word_bit(0),
            Some(data_register_serial_bit(&first, 54))
        );
        assert_eq!(
            source.logical_bit_for_word_bit(1),
            Some(data_register_serial_bit(&first, 55))
        );
        assert_eq!(
            source.logical_bit_for_word_bit(2),
            Some(data_register_serial_bit(&second, 0))
        );
    }

    #[test]
    fn sink_reconstructs_back_to_back_frames_across_word_boundaries() {
        let first = patterned_register(2);
        let second = patterned_register(9);
        let mut source = Hp67DataSerialSource::default();
        let mut sink = Hp67DataSerialSink::default();

        source.begin_word(Some(first));
        sink.begin_word(true);
        for word_bit in 0..BITS_PER_WORD {
            assert_eq!(
                sink.sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                    .unwrap(),
                None
            );
        }
        source.complete_word();

        source.begin_word(Some(second));
        sink.begin_word(true);
        let mut first_completed = None;
        for word_bit in 0..BITS_PER_WORD {
            let completed = sink
                .sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                .unwrap();
            if completed.is_some() {
                assert_eq!(word_bit, 1);
                first_completed = completed;
            }
        }
        source.complete_word();
        assert_eq!(first_completed, Some(first));

        source.begin_word(None);
        sink.begin_word(false);
        let mut second_completed = None;
        for word_bit in 0..BITS_PER_WORD {
            let completed = sink
                .sample_word_bit(word_bit, source.logical_bit_for_word_bit(word_bit))
                .unwrap();
            if completed.is_some() {
                assert_eq!(word_bit, 1);
                second_completed = completed;
            }
        }
        assert_eq!(second_completed, Some(second));
    }

    #[test]
    fn transfer_plan_matches_ram_read_write_address_semantics() {
        let mut state = ActArchitecturalState::default();
        state.ram_address = 0x27;
        assert_eq!(
            act_data_transfer_plan(0o1360, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::ActToPeripheral,
                address: 0x27,
            })
        );
        assert_eq!(
            act_data_transfer_plan(0o0070, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::PeripheralToAct,
                address: 0x27,
            })
        );
        state.ram_address = 0x20;
        assert_eq!(
            act_data_transfer_plan((3 << 6) | 0o50, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::ActToPeripheral,
                address: 0x23,
            })
        );
        assert_eq!(
            act_data_transfer_plan((5 << 6) | 0o70, &state),
            Some(Hp67DataTransferPlan {
                direction: Hp67DataTransferDirection::PeripheralToAct,
                address: 0x25,
            })
        );
        assert_eq!(act_data_transfer_plan(0o0000, &state), None);
    }

    #[test]
    fn then_goto_payload_is_never_misclassified_as_a_data_transfer() {
        let mut state = ActArchitecturalState {
            instruction_state: ActInstructionState::ThenGoto,
            ram_address: 0x20,
            ..ActArchitecturalState::default()
        };

        assert_eq!(act_data_transfer_plan(0o1360, &state), None);
        assert_eq!(act_data_transfer_plan((3 << 6) | 0o50, &state), None);
        assert_eq!(act_data_transfer_plan((5 << 6) | 0o70, &state), None);

        state.instruction_state = ActInstructionState::Normal;
        assert!(act_data_transfer_plan(0o1360, &state).is_some());
    }

    #[test]
    fn missing_or_unexpected_logical_data_is_rejected() {
        let mut sink = Hp67DataSerialSink::default();
        sink.begin_word(true);
        assert_eq!(
            sink.sample_word_bit(0, Some(false)),
            Err(Hp67DataSerialError::UnexpectedLogicalBit {
                word_bit: 0,
                serial_bit: 54,
            })
        );

        let mut sink = Hp67DataSerialSink::default();
        sink.begin_word(true);
        assert_eq!(sink.sample_word_bit(0, None), Ok(None));
        assert_eq!(sink.sample_word_bit(1, None), Ok(None));
        assert_eq!(
            sink.sample_word_bit(2, None),
            Err(Hp67DataSerialError::MissingLogicalBit {
                word_bit: 2,
                serial_bit: 0,
            })
        );
    }
}
