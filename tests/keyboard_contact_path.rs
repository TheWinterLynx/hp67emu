use hp67emu::machines::hp67::{Hp67ArchitecturalMachine, Hp67Key, Hp67Keyboard};

#[test]
fn digit_one_contact_reaches_act_key_dispatch_without_semantic_shortcut() {
    let mut keyboard = Hp67Keyboard::default();
    let mut machine = Hp67ArchitecturalMachine::default();

    keyboard.press(Hp67Key::Digit1);
    keyboard.sample_into_act(&mut machine.act.state);
    assert_eq!(machine.act.state.key_buffer, Some(0o142));

    // Woodstock opcode 0020 is the firmware key-dispatch primitive: it replaces
    // the low ROM address byte with the hardware key code.  The keyboard model
    // therefore supplies only the physical scan code; firmware chooses behavior.
    machine
        .execute_word(0o0020)
        .expect("ACT key-dispatch instruction must execute");
    assert_eq!(machine.pc(), 0o142);

    keyboard.release();
    keyboard.sample_into_act(&mut machine.act.state);
    assert_eq!(machine.act.state.key_buffer, None);
}
