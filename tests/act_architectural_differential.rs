//! Differential regression for the independent HP-67 ACT architectural core.

use hp67emu::{
    machines::hp67::{
        ActArchitecturalCore, ActArchitecturalState, ActError, ActInstructionState, ActRamImage,
        ActRegister, ACT_WORD_DIGITS,
    },
    reference::woodstock::{
        ArchitecturalState, ExecutionError, InstructionState, ReferenceMachine, Register,
    },
};

const OPCODE_MASK: u16 = 0x03ff;

#[test]
fn complete_opcode_space_matches_reference_across_varied_states() {
    for seed in 0..5u8 {
        for opcode in 0..=OPCODE_MASK {
            compare_one(seed, opcode, false);
        }
    }
}

#[test]
fn every_ten_bit_word_matches_in_then_goto_state() {
    for carry in [false, true] {
        for opcode in 0..=OPCODE_MASK {
            let seed = if carry { 7 } else { 6 };
            compare_one(seed, opcode, true);
        }
    }
}

fn compare_one(seed: u8, opcode: u16, then_goto: bool) {
    let mut state = seeded_state(seed);
    if then_goto {
        state.instruction_state = ActInstructionState::ThenGoto;
        state.carry = seed & 1 != 0;
        state.delayed_rom = None;
    }

    let mut ours = ActArchitecturalCore {
        state: state.clone(),
        ..ActArchitecturalCore::default()
    };
    let mut ours_ram = ActRamImage::hp67();
    seed_our_ram(&mut ours_ram, seed);

    let mut reference = ReferenceMachine::hp67();
    reference.cpu = to_reference_state(&state);
    seed_reference_ram(&mut reference, seed);

    let our_result = ours.execute_word(&mut ours_ram, opcode);
    let reference_result = reference.step_word(opcode);
    let context = format!("seed={seed} opcode=0x{opcode:03x} then_goto={then_goto}");

    match (&our_result, &reference_result) {
        (Ok(_), Ok(())) => {
            assert_states_equal(&ours.state, &reference.cpu, &context);
            for address in 0u8..0x40 {
                let ours_word = ours_ram.read(address);
                let reference_word = reference.ram(address).map(|register| *register.digits());
                assert_eq!(ours_word, reference_word, "RAM mismatch: {context} addr=0x{address:02x}");
            }
        }
        (Err(our_error), Err(reference_error)) => {
            assert!(
                equivalent_error(*our_error, *reference_error),
                "error mismatch: {context}: ours={our_error:?} reference={reference_error:?}"
            );
        }
        _ => panic!(
            "result mismatch: {context}: ours={our_result:?} reference={reference_result:?}"
        ),
    }
}

fn seeded_state(seed: u8) -> ActArchitecturalState {
    let mut state = ActArchitecturalState::default();
    state.a = patterned_register(seed.wrapping_add(1));
    state.b = patterned_register(seed.wrapping_add(2));
    state.c = patterned_register(seed.wrapping_add(3));
    state.y = patterned_register(seed.wrapping_add(4));
    state.z = patterned_register(seed.wrapping_add(5));
    state.t = patterned_register(seed.wrapping_add(6));
    state.m1 = patterned_register(seed.wrapping_add(7));
    state.m2 = patterned_register(seed.wrapping_add(8));
    state.f = (seed.wrapping_mul(3).wrapping_add(1)) & 0x0f;
    state.p = [0, 4, 13, 14, 1][usize::from(seed % 5)];
    state.p_change = if seed == 4 { [1, 1, 1] } else { [0, -1, 1] };
    state.decimal = seed & 1 == 0;
    state.carry = seed & 1 != 0;
    state.previous_carry = seed & 2 != 0;
    for (index, status) in state.status.iter_mut().enumerate() {
        *status = (index + usize::from(seed)) % 3 == 0;
    }
    state.pc = 0x456 + u16::from(seed) * 0x11;
    state.delayed_rom = (seed == 4).then_some(2);
    state.bank = seed & 1;
    state.return_stack = [0x123 + u16::from(seed), 0xabc - u16::from(seed)];
    state.stack_pointer = seed & 1;
    state.key_buffer = Some(0x2d);
    state.display_enable = seed & 1 != 0;
    state.display_14_digit = seed & 2 != 0;
    state.ram_address = 0x12;
    state
}

fn patterned_register(seed: u8) -> ActRegister {
    let mut register = [0; ACT_WORD_DIGITS];
    for (index, digit) in register.iter_mut().enumerate() {
        *digit = (usize::from(seed) + index * 3) as u8 % 10;
    }
    register
}

fn seed_our_ram(ram: &mut ActRamImage, seed: u8) {
    for address in 0u8..0x40 {
        assert!(ram.write(address, patterned_register(seed.wrapping_add(address))));
    }
}

fn seed_reference_ram(machine: &mut ReferenceMachine, seed: u8) {
    for address in 0u8..0x40 {
        assert!(machine.set_ram(
            address,
            to_reference_register(patterned_register(seed.wrapping_add(address)))
        ));
    }
}

fn to_reference_register(source: ActRegister) -> Register {
    let mut register = Register::zero();
    register.digits_mut().copy_from_slice(&source);
    register
}

fn to_reference_state(source: &ActArchitecturalState) -> ArchitecturalState {
    ArchitecturalState {
        a: to_reference_register(source.a),
        b: to_reference_register(source.b),
        c: to_reference_register(source.c),
        y: to_reference_register(source.y),
        z: to_reference_register(source.z),
        t: to_reference_register(source.t),
        m1: to_reference_register(source.m1),
        m2: to_reference_register(source.m2),
        f: source.f,
        p: source.p,
        p_change: source.p_change,
        decimal: source.decimal,
        carry: source.carry,
        previous_carry: source.previous_carry,
        status: source.status,
        pc: source.pc,
        delayed_rom: source.delayed_rom,
        bank: source.bank,
        return_stack: source.return_stack,
        stack_pointer: source.stack_pointer,
        instruction_state: match source.instruction_state {
            ActInstructionState::Normal => InstructionState::Normal,
            ActInstructionState::ThenGoto => InstructionState::ThenGoto,
        },
        key_buffer: source.key_buffer,
        display_enable: source.display_enable,
        display_14_digit: source.display_14_digit,
        ram_address: source.ram_address,
    }
}

fn assert_states_equal(ours: &ActArchitecturalState, reference: &ArchitecturalState, context: &str) {
    assert_eq!(ours.a, *reference.a.digits(), "A mismatch: {context}");
    assert_eq!(ours.b, *reference.b.digits(), "B mismatch: {context}");
    assert_eq!(ours.c, *reference.c.digits(), "C mismatch: {context}");
    assert_eq!(ours.y, *reference.y.digits(), "Y mismatch: {context}");
    assert_eq!(ours.z, *reference.z.digits(), "Z mismatch: {context}");
    assert_eq!(ours.t, *reference.t.digits(), "T mismatch: {context}");
    assert_eq!(ours.m1, *reference.m1.digits(), "M1 mismatch: {context}");
    assert_eq!(ours.m2, *reference.m2.digits(), "M2 mismatch: {context}");
    assert_eq!(ours.f, reference.f, "F mismatch: {context}");
    assert_eq!(ours.p, reference.p, "P mismatch: {context}");
    assert_eq!(ours.p_change, reference.p_change, "P-change mismatch: {context}");
    assert_eq!(ours.decimal, reference.decimal, "decimal mismatch: {context}");
    assert_eq!(ours.carry, reference.carry, "carry mismatch: {context}");
    assert_eq!(ours.previous_carry, reference.previous_carry, "previous-carry mismatch: {context}");
    assert_eq!(ours.status, reference.status, "status mismatch: {context}");
    assert_eq!(ours.pc, reference.pc, "PC mismatch: {context}");
    assert_eq!(ours.delayed_rom, reference.delayed_rom, "delayed-ROM mismatch: {context}");
    assert_eq!(ours.bank, reference.bank, "bank mismatch: {context}");
    assert_eq!(ours.return_stack, reference.return_stack, "return-stack mismatch: {context}");
    assert_eq!(ours.stack_pointer, reference.stack_pointer, "stack-pointer mismatch: {context}");
    assert_eq!(
        ours.instruction_state == ActInstructionState::ThenGoto,
        reference.instruction_state == InstructionState::ThenGoto,
        "instruction-state mismatch: {context}"
    );
    assert_eq!(ours.key_buffer, reference.key_buffer, "key-buffer mismatch: {context}");
    assert_eq!(ours.display_enable, reference.display_enable, "display-enable mismatch: {context}");
    assert_eq!(
        ours.display_14_digit, reference.display_14_digit,
        "display-width mismatch: {context}"
    );
    assert_eq!(ours.ram_address, reference.ram_address, "RAM-address mismatch: {context}");
}

fn equivalent_error(ours: ActError, reference: ExecutionError) -> bool {
    match (ours, reference) {
        (ActError::UnknownSpecial { word: left, .. }, ExecutionError::UnknownSpecial(right)) => {
            left == right
        }
        (ActError::UnsupportedRomSelfTest { .. }, ExecutionError::UnsupportedRomSelfTest) => true,
        _ => false,
    }
}
