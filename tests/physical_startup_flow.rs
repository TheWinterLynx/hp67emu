//! Cross-checks a directly observed HP-67 control-flow checkpoint against the
//! semantic Woodstock reference machine.
//!
//! The test intentionally embeds only the two 10-bit instruction words needed
//! for the published control-flow observation, not a firmware image.

use hp67emu::reference::woodstock::ReferenceMachine;

const DELAYED_SELECT_ROM_15: u16 = 0x3f4; // 0o1764
const JSB_00C6: u16 = 0x319; // 0o1431

#[test]
fn physical_hp67_0068_call_resolves_to_0fc6() {
    let mut machine = ReferenceMachine::default();
    machine.cpu.pc = 0x0067;

    machine
        .step_word(DELAYED_SELECT_ROM_15)
        .expect("delayed ROM select at HP-67 PC 0x067 must execute");
    assert_eq!(machine.cpu.pc, 0x0068);
    assert_eq!(machine.cpu.delayed_rom, Some(0x0f));

    machine
        .step_word(JSB_00C6)
        .expect("JSB at HP-67 PC 0x068 must execute");

    // The JSB stores the post-fetch return address first. The delayed ROM
    // selection from PC 0x067 is then applied to the call target, producing
    // the 0x0fc6 destination reported from physical HP-67 execution.
    assert_eq!(machine.cpu.return_stack[0], 0x0069);
    assert_eq!(machine.cpu.stack_pointer, 1);
    assert_eq!(machine.cpu.pc, 0x0fc6);
    assert_eq!(machine.cpu.delayed_rom, None);
}
