//! Retained ACT registers and explicit logical input ports.
//! Completed-word ownership does not establish physical silicon write edges.
use super::super::{ActRegister, Hp67Keyboard};
use super::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(super) struct ActOwnedRegisters {
    snapshot: ActSerialStateSnapshot,
    previous_carry: bool,
}
impl ActOwnedRegisters {
    fn capture(state: &ActArchitecturalState) -> Self {
        Self {
            snapshot: ActSerialStateSnapshot::capture(state),
            previous_carry: state.previous_carry,
        }
    }
    fn materialize(self) -> ActArchitecturalState {
        let s = self.snapshot;
        ActArchitecturalState {
            a: *s.register(ActSerialRegister::A),
            b: *s.register(ActSerialRegister::B),
            c: *s.register(ActSerialRegister::C),
            y: *s.y(),
            z: *s.z(),
            t: *s.t(),
            m1: *s.m1(),
            m2: *s.m2(),
            f: s.f(),
            p: s.p(),
            p_change: s.p_change(),
            decimal: s.decimal(),
            carry: s.carry(),
            previous_carry: self.previous_carry,
            status: *s.status(),
            instruction_state: s.instruction_state(),
            key_buffer: s.key_buffer(),
            display_enable: s.display_enable(),
            display_14_digit: s.display_14_digit(),
            ram_address: s.ram_address(),
            ..Default::default()
        }
    }
}
impl ActSerialEndpoint {
    pub fn with_owned_registers(mut self) -> Self {
        assert!(
            self.received_execution_enabled && self.owned_flow.is_some(),
            "owned registers require received execution and flow"
        );
        self.owned_registers = Some(ActOwnedRegisters::capture(&ActArchitecturalState::default()));
        self
    }
    /// A fixed-size source image; no architectural opcode executor runs.
    pub fn owned_register_state(&self) -> Option<ActArchitecturalState> {
        self.owned_registers.map(|registers| {
            let mut state = registers.materialize();
            self.owned_flow.expect("owned ACT flow").apply(&mut state);
            state
        })
    }
    pub fn check_owned_register_mirror(&self, state: &ActArchitecturalState) -> Result<(), String> {
        if let Some(owned) = self.owned_registers {
            let mirror = ActOwnedRegisters::capture(state);
            if mirror != owned {
                return Err(format!(
                    "ACT owned register/mirror mismatch: device={owned:?} mirror={mirror:?}"
                ));
            }
        }
        Ok(())
    }
    pub fn sample_owned_keyboard(&mut self, keyboard: &Hp67Keyboard) -> Result<(), String> {
        if let Some(mut state) = self.owned_register_state() {
            if self.execution.is_some_and(|word| !word.is_complete()) {
                return Err("ACT keyboard input requires an instruction boundary".into());
            }
            keyboard.sample_into_act(&mut state);
            self.owned_registers = Some(ActOwnedRegisters::capture(&state));
        }
        Ok(())
    }
    /// Caller supplies only successfully reconstructed resolved DATA C.
    pub fn receive_owned_data_c(
        &mut self,
        value: ActRegister,
        bus: &Hp67ElectricalBackplane,
    ) -> Result<(), String> {
        if let Some(mut state) = self.owned_register_state() {
            if bus.word_bit() != 2
                || !bus.at_transport_launch_boundary()
                || self.execution.is_some_and(|word| !word.is_complete())
            {
                return Err(format!(
                    "ACT DATA C handoff requires completed b1: bit={} tick={}",
                    bus.word_bit(),
                    bus.tick().get()
                ));
            }
            state.c = value;
            self.owned_registers = Some(ActOwnedRegisters::capture(&state));
        }
        Ok(())
    }
    /// Independent CRC command/latch result after completed execution/compare.
    pub fn receive_owned_crc_condition(&mut self, condition: Option<bool>) -> Result<(), String> {
        if let Some(mut state) = self.owned_register_state() {
            if !self.execution.is_some_and(|word| word.is_complete()) {
                return Err("ACT CRC input requires completed execution".into());
            }
            state.status[3] |= condition == Some(true);
            self.owned_registers = Some(ActOwnedRegisters::capture(&state));
        }
        Ok(())
    }
    /// Production CRC condition enters through the validated resolved F2 net.
    pub fn receive_owned_crc_f2(&mut self, bus: &Hp67ElectricalBackplane) -> Result<bool, String> {
        if bus.word_bit() != 0 || !bus.at_transport_launch_boundary() {
            return Err(format!(
                "ACT F2 input requires completed b55: tick={}",
                bus.tick().get()
            ));
        }
        let asserted = super::super::sample_crc_f2(bus)
            .map_err(|error| format!("ACT F2 sample failed: {error:?}"))?;
        self.receive_owned_crc_condition(Some(asserted))?;
        Ok(asserted)
    }
    pub fn qualify_owned_registers_from_received(&mut self) -> Result<(), String> {
        if let Some(mut state) = self.owned_register_state() {
            state.instruction_state = self
                .fetched_instruction_state()
                .map_err(|error| format!("ACT owned instruction qualification: {error:?}"))?;
            self.owned_registers = Some(ActOwnedRegisters::capture(&state));
        }
        Ok(())
    }
    /// Comparison precedes replacing the complete ACT mirror from owned state.
    pub fn commit_owned_register_mirror(
        &self,
        state: &mut ActArchitecturalState,
    ) -> Result<(), String> {
        self.check_owned_register_mirror(state)?;
        self.check_owned_flow_mirror(state)?;
        if let Some(owned) = self.owned_register_state() {
            *state = owned;
        }
        Ok(())
    }
    pub(super) fn complete_owned_register_word(&mut self) {
        let Some(mut state) = self.owned_register_state() else {
            return;
        };
        if let Some(image) = self.arithmetic_result_image {
            state.a = *image.register(ActSerialRegister::A);
            state.b = *image.register(ActSerialRegister::B);
            state.c = *image.register(ActSerialRegister::C);
            state.carry = image.final_chain();
        }
        if let Some(image) = self.control_result_image {
            assert!(image.is_complete());
            state.p = image.p();
            state.p_change = image.p_change();
            state.status = *image.status();
            state.carry = image.carry();
            state.previous_carry = image.previous_carry();
            state.instruction_state = image.instruction_state();
        }
        if let Some(image) = self.special_register_result_image {
            assert!(image.is_complete());
            state.a = *image.a();
            state.b = *image.b();
            state.c = *image.c();
            state.y = *image.y();
            state.z = *image.z();
            state.t = *image.t();
            state.m1 = *image.m1();
            state.m2 = *image.m2();
            state.f = image.f();
        }
        if let Some(image) = self.mode_latch_result_image {
            assert!(image.is_complete());
            state.decimal = image.decimal();
            state.display_enable = image.display_enable();
            state.display_14_digit = image.display_14_digit();
            state.ram_address = image.ram_address();
        }
        if let Some(image) = self.absent_ram_read_image {
            assert!(image.is_complete());
            state.c = [0; 14];
        }
        if let Some(image) = self.crc_data_result_image {
            assert!(image.is_complete());
            state.c = image.result_c();
        }
        if let Some(image) = self.ram_result_image {
            assert!(image.is_complete());
            state.ram_address = image.ram_address();
        }
        if let Some(image) = self.boundary_result_image {
            image.commit(&mut state);
        }
        if let Some(image) = self.flow_result_image {
            image.commit(&mut state);
        }
        // CRC S3 uses its independent device port; installed RAM C waits for b1.
        self.owned_registers = Some(ActOwnedRegisters::capture(&state));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::{
        act_data_transfer_plan, ActInstructionState, Hp67ArchitecturalMachine,
        Hp67ArchitecturalOperation, Hp67DataTransferDirection, Hp67Key,
    };

    fn endpoint() -> ActSerialEndpoint {
        ActSerialEndpoint::new(0)
            .with_received_execution()
            .with_owned_flow()
            .with_owned_registers()
    }
    fn prime(act: &mut ActSerialEndpoint, word: u16, instruction: bool) {
        let mut bus = Hp67ElectricalBackplane::default();
        act.begin_fetch_cycle(act.owned_flow_state().unwrap().pc);
        for bit in 0..BITS_PER_WORD {
            publish_transport_sources(
                &mut bus,
                bit,
                act_address_drive(act.address(), bit),
                RomIsDrives::Aggregate(rom_word_drive(word, bit)),
                if bit >= 46 && instruction {
                    Drive::High
                } else {
                    Drive::Low
                },
                None,
            )
            .unwrap();
            bus.advance_clock();
            bus.advance_clock();
            if bit >= 46 {
                act.sample_instruction_at_edge(&bus).unwrap();
            }
            bus.advance_clock();
            bus.advance_clock();
        }
        assert_eq!(bus.tick().get(), 224);
    }
    #[test]
    fn m15w_owned_register_images_match_all_nonperipheral_words_and_both_qualifiers() {
        let mut accepted = [0usize; 2];
        for (p, decimal) in [(0, true), (13, true), (7, false)] {
            for carry in [false, true] {
                for instruction in [false, true] {
                    for address in [0, 0x40] {
                        for word in 0..=0x3ff {
                            let mut oracle = Hp67ArchitecturalMachine::default();
                            let s = &mut oracle.act.state;
                            s.pc = 0x68;
                            s.delayed_rom = Some(15);
                            s.return_stack = [0xabc, 0x321];
                            s.stack_pointer = u8::from(carry);
                            s.a = std::array::from_fn(|i| ((i + 1) % 10) as u8);
                            s.b = [2; 14];
                            s.c = [3; 14];
                            s.y = [4; 14];
                            s.z = [5; 14];
                            s.t = [6; 14];
                            s.m1 = [7; 14];
                            s.m2 = [8; 14];
                            s.f = 9;
                            s.p = p;
                            s.decimal = decimal;
                            s.carry = carry;
                            s.previous_carry = !carry;
                            s.p_change = [1, -1, 1];
                            s.key_buffer = Some(0o244);
                            s.status = std::array::from_fn(|i| i % 2 == 0);
                            s.display_enable = true;
                            s.display_14_digit = true;
                            s.ram_address = address;
                            s.instruction_state = if instruction {
                                ActInstructionState::Normal
                            } else {
                                ActInstructionState::ThenGoto
                            };
                            let before = oracle.clone();
                            let Ok(execution) = oracle.execute_word(word) else {
                                continue;
                            };
                            if !matches!(execution.operation, Hp67ArchitecturalOperation::Act(_)) {
                                continue;
                            }
                            if act_data_transfer_plan(word, &before.act.state).is_some_and(|plan| {
                                plan.direction == Hp67DataTransferDirection::PeripheralToAct
                                    && before.ram.read(plan.address).is_some()
                            }) {
                                continue;
                            }
                            let mut act = endpoint();
                            act.owned_flow = Some(ActSerialFlowState::capture(&before.act.state));
                            act.owned_registers =
                                Some(ActOwnedRegisters::capture(&before.act.state));
                            prime(&mut act, word, instruction);
                            assert_eq!(
                                act.begin_received_execution(&before.act.state).unwrap(),
                                Some(word)
                            );
                            act.bind_ram_topology(&before.ram).unwrap();
                            act.bind_crc_control(&before.crc).unwrap();
                            let retained = act.owned_registers;
                            for bit in 0..55 {
                                act.advance_execution_for_bit(bit).unwrap();
                                assert_eq!(act.owned_registers, retained);
                            }
                            act.advance_execution_for_bit(55).unwrap();
                            oracle.prepare_hp67_fetch();
                            assert_eq!(act.owned_register_state().unwrap(), oracle.act.state, "word={word:03x} instruction={instruction} p={p} decimal={decimal} address={address}");
                            accepted[usize::from(instruction)] += 1;
                        }
                    }
                }
            }
        }
        assert!(accepted.into_iter().all(|count| count > 0));
    }
    #[test]
    fn m15w_owned_ports_reject_wrong_phase_and_preserve_sticky_keyboard_and_s3() {
        let mut act = endpoint();
        let mut keyboard = Hp67Keyboard::default();
        keyboard.press(Hp67Key::Digit7);
        act.sample_owned_keyboard(&keyboard).unwrap();
        assert_eq!(
            act.owned_register_state().unwrap().key_buffer,
            Some(Hp67Key::Digit7.scan_code())
        );
        let pressed = act.owned_register_state();
        keyboard.release();
        act.sample_owned_keyboard(&keyboard).unwrap();
        assert_eq!(act.owned_register_state(), pressed);
        let mut bus = Hp67ElectricalBackplane::default();
        let before = act;
        assert!(act.receive_owned_data_c([5; 14], &bus).is_err());
        assert!(act.receive_owned_crc_condition(Some(true)).is_err());
        assert_eq!(act, before);
        for _ in 0..8 {
            bus.advance_clock();
        }
        act.receive_owned_data_c([5; 14], &bus).unwrap();
        assert_eq!(act.owned_register_state().unwrap().c, [5; 14]);
        prime(&mut act, 0, true);
        let state = act.owned_register_state().unwrap();
        act.begin_received_execution(&state).unwrap();
        let active = act;
        assert!(act.sample_owned_keyboard(&keyboard).is_err());
        assert!(act.receive_owned_data_c([9; 14], &bus).is_err());
        assert_eq!(act, active);
        for bit in 0..56 {
            act.advance_execution_for_bit(bit).unwrap();
        }
        for condition in [Some(true), Some(false), None] {
            act.receive_owned_crc_condition(condition).unwrap();
            assert!(act.owned_register_state().unwrap().status[3]);
        }
    }
    #[test]
    fn m15w_binding_uses_owned_registers_when_reference_binder_mirror_is_poisoned() {
        let mut act = endpoint();
        let mut source = act.owned_register_state().unwrap();
        source.a = [1; 14];
        source.b = [2; 14];
        source.c = [3; 14];
        act.owned_registers = Some(ActOwnedRegisters::capture(&source));
        let mut poisoned = source.clone();
        poisoned.a = [9; 14];
        poisoned.p = 13;
        poisoned.decimal = false;
        let before = act;
        assert!(act.begin_received_execution(&poisoned).is_err());
        assert_eq!(act, before);
        act.bind_execution(0, &poisoned).unwrap();
        assert_eq!(
            act.serial_execution_state(),
            Some(ActSerialStateSnapshot::capture(&source))
        );
        assert_eq!(act.owned_register_state(), Some(source));
    }
    #[test]
    fn m16c_act_s3_consumes_resolved_f2_and_rejects_invalid_inputs_before_mutation() {
        use super::super::super::{Hp67CrcFlagBridge, Hp67Driver, Hp67Net};
        use crate::emulation::Drive;
        let mut act = endpoint();
        let mut bus = Hp67ElectricalBackplane::default();
        Hp67CrcFlagBridge::attach(&mut bus);
        let before = act;
        assert!(act.receive_owned_crc_f2(&bus).is_err());
        assert_eq!(act, before);
        prime(&mut act, 0, true);
        let state = act.owned_register_state().unwrap();
        act.begin_received_execution(&state).unwrap();
        for bit in 0..56 {
            act.advance_execution_for_bit(bit).unwrap();
        }
        bus.drive(
            Hp67Net::F2,
            Hp67Driver::CardReaderController1820_1751,
            Drive::Low,
        );
        assert_eq!(act.receive_owned_crc_f2(&bus), Ok(true));
        assert!(act.owned_register_state().unwrap().status[3]);
        bus.drive(
            Hp67Net::F2,
            Hp67Driver::CardReaderController1820_1751,
            Drive::High,
        );
        assert_eq!(act.receive_owned_crc_f2(&bus), Ok(false));
        assert!(
            act.owned_register_state().unwrap().status[3],
            "inactive F2 must not clear sticky S3"
        );
        let before = act;
        bus.drive(Hp67Net::F2, Hp67Driver::External, Drive::Low);
        assert!(act.receive_owned_crc_f2(&bus).is_err());
        assert_eq!(act, before);
        bus.drive(Hp67Net::F2, Hp67Driver::External, Drive::HighZ);
        bus.advance_clock();
        assert!(act.receive_owned_crc_f2(&bus).is_err());
        assert_eq!(act, before);
    }
}
