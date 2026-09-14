//! HP-67 semantic reference machine composed from Woodstock ACT/RAM and CRC.

use super::{
    crc::{
        decode_crc_opcode, CardSide, CrcError, CrcReference, CRC_RAM_READ_ADDRESS,
        CRC_RAM_WRITE_ADDRESS, FLAG_MOTOR_ENABLE, FLAG_PROGRAM_MODE,
    },
    rom::{RomError, RomImage},
    woodstock::{ExecutionError, InstructionState, ReferenceMachine, OPCODE_MASK},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67ReferenceError {
    OpcodeOutOfRange(u16),
    Cpu(ExecutionError),
    Crc(CrcError),
    Rom(RomError),
}

impl From<ExecutionError> for Hp67ReferenceError {
    fn from(value: ExecutionError) -> Self {
        Self::Cpu(value)
    }
}

impl From<CrcError> for Hp67ReferenceError {
    fn from(value: CrcError) -> Self {
        Self::Crc(value)
    }
}

impl From<RomError> for Hp67ReferenceError {
    fn from(value: RomError) -> Self {
        Self::Rom(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67Reference {
    pub core: ReferenceMachine,
    pub crc: CrcReference,
}

impl Default for Hp67Reference {
    fn default() -> Self {
        Self {
            core: ReferenceMachine::hp67(),
            crc: CrcReference::default(),
        }
    }
}

impl Hp67Reference {
    pub fn set_program_mode(&mut self, program: bool) -> Result<(), Hp67ReferenceError> {
        self.crc
            .set_external_flag(FLAG_PROGRAM_MODE as u8, program)?;
        Ok(())
    }

    pub fn insert_card(&mut self, card: CardSide) -> Result<(), Hp67ReferenceError> {
        self.crc.insert_card(card)?;
        Ok(())
    }

    pub fn take_completed_card(&mut self) -> Option<CardSide> {
        self.crc.take_completed_card()
    }

    pub fn step(&mut self, rom: &RomImage) -> Result<u16, Hp67ReferenceError> {
        if self.core.cpu.pc < 0o2000 && self.core.cpu.bank == 1 {
            self.core.cpu.bank = 0;
        }
        let opcode = rom.fetch(&self.core.cpu)?;
        self.step_word(opcode)?;
        Ok(opcode)
    }

    pub fn step_word(&mut self, opcode: u16) -> Result<(), Hp67ReferenceError> {
        if opcode > OPCODE_MASK {
            return Err(Hp67ReferenceError::OpcodeOutOfRange(opcode));
        }

        // A word following a conditional test is branch data, not an opcode.
        // It must bypass CRC decode even when its bit pattern resembles one of
        // the controller's special instructions.
        if self.core.cpu.instruction_state == InstructionState::ThenGoto {
            self.core.step_word(opcode)?;
            return Ok(());
        }

        if decode_crc_opcode(opcode)?.is_some() {
            // Let the Woodstock core perform the universal instruction-boundary
            // work (carry snapshot, PC increment, P history, delayed ROM) using
            // a NOP, then apply the external CRC chip's semantic side effect.
            self.core.step_word(0o0000)?;
            if self.crc.execute_opcode(opcode)? == Some(true) {
                // CRC flag tests pulse ACT F2; at the instruction boundary that
                // pulse is latched into ACT status bit S3.
                self.core.cpu.status[3] = true;
            }
            return Ok(());
        }

        let direct_crc_read = opcode == 0o0070 && self.core.cpu.ram_address == CRC_RAM_READ_ADDRESS;
        let direct_crc_write =
            opcode == 0o1360 && self.core.cpu.ram_address == CRC_RAM_WRITE_ADDRESS;
        let family = opcode & 0o77;
        let register_read = opcode != 0o0070 && family == 0o70;
        let register_write = family == 0o50;

        self.core.step_word(opcode)?;

        if direct_crc_read
            || (register_read && self.core.cpu.ram_address == CRC_RAM_READ_ADDRESS)
        {
            self.crc.read_into_c(&mut self.core.cpu.c)?;
        }
        if direct_crc_write
            || (register_write && self.core.cpu.ram_address == CRC_RAM_WRITE_ADDRESS)
        {
            self.crc.write_from_c(&self.core.cpu.c)?;
        }

        Ok(())
    }

    pub fn card_motor_running(&self) -> bool {
        self.crc.flag(FLAG_MOTOR_ENABLE) == Some(true)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::reference::crc::{FLAG_BUFFER_READY, FLAG_WRITE_MODE};
    use crate::reference::woodstock::Register;

    #[test]
    fn program_switch_crc_test_sets_act_s3() {
        let mut hp67 = Hp67Reference::default();
        hp67.set_program_mode(true).expect("switch must set");

        // CRC flag 1 test-and-clear.
        hp67.step_word(0o300).expect("CRC test must execute");
        assert!(hp67.core.cpu.status[3]);
        assert!(!hp67.card_motor_running());
    }

    #[test]
    fn card_read_port_routes_through_crc_instead_of_plain_ram() {
        let mut side = CardSide::default();
        side.set_word(0, 0x0765_4321);
        let mut hp67 = Hp67Reference::default();
        hp67.insert_card(side).expect("card must insert");

        hp67.step_word(0o260).expect("motor flag must set");
        assert!(hp67.card_motor_running());
        assert_eq!(hp67.crc.flag(FLAG_BUFFER_READY), Some(true));

        hp67.core.cpu.ram_address = CRC_RAM_READ_ADDRESS;
        hp67.step_word(0o0070).expect("CRC data read must execute");
        assert_eq!(&hp67.core.cpu.c.digits()[0..7], &[1, 2, 3, 4, 5, 6, 7]);
    }

    #[test]
    fn card_write_port_routes_through_crc() {
        let mut hp67 = Hp67Reference::default();
        hp67.insert_card(CardSide::default())
            .expect("card must insert");
        hp67.step_word(0o260).expect("motor flag must set");
        hp67.step_word(0o660).expect("write mode must set");
        assert_eq!(hp67.crc.flag(FLAG_WRITE_MODE), Some(true));

        hp67.core.cpu.ram_address = CRC_RAM_WRITE_ADDRESS;
        hp67.core.cpu.c = Register::zero();
        for index in 0..7 {
            hp67.core.cpu.c[7 + index] = (index + 1) as u8;
        }
        hp67.step_word(0o1360).expect("CRC data write must execute");
        assert_eq!(hp67.crc.head_position(), 1);
    }

    #[test]
    fn then_goto_target_never_executes_as_crc_opcode() {
        let mut hp67 = Hp67Reference::default();
        hp67.core.cpu.pc = 0x800;
        hp67.core.cpu.instruction_state = InstructionState::ThenGoto;
        hp67.core.cpu.carry = false;

        hp67.step_word(0o260).expect("branch target must be consumed");

        assert_eq!(hp67.core.cpu.pc, 0x8b0);
        assert!(!hp67.card_motor_running());
    }
}
