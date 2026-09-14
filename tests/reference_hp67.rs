//! Cross-module regressions for the integrated HP-67 semantic reference path.

use hp67emu::reference::hp67::Hp67Reference;
use hp67emu::reference::rom::RomImage;
use hp67emu::reference::woodstock::InstructionState;

#[test]
fn hp67_label_search_p_wrap_regression_at_nonpareil_06132_target() {
    let mut hp67 = Hp67Reference::default();
    hp67.core.cpu.pc = 0o6130;
    hp67.core.cpu.p = 13;

    hp67.step_word(0o0720).expect("first INC P must execute");
    hp67.step_word(0o0720).expect("second INC P must execute");
    assert_eq!(hp67.core.cpu.pc, 0o6132);
    assert_eq!(hp67.core.cpu.p, 1);

    // Operand 11 maps to P==0. Woodstock's transient P-wrap behaviour makes
    // this test succeed after the two increments even though the stable P value
    // is already 1. The semantic model handles that generically, not by PC.
    let test_p_zero = 0o0044 + (11 << 6);
    hp67.step_word(test_p_zero).expect("P test must execute");

    assert_eq!(hp67.core.cpu.instruction_state, InstructionState::ThenGoto);
    assert!(!hp67.core.cpu.carry);
}

#[test]
fn tiny_rom_can_drive_integrated_hp67_reference_without_host_io() {
    let mut rom = RomImage::new();
    rom.install_page(0, 0, &[0o0420, 0o0000])
        .expect("fixture ROM page must install");

    let mut hp67 = Hp67Reference::default();
    assert!(hp67.core.cpu.decimal);

    assert_eq!(hp67.step(&rom), Ok(0o0420));
    assert!(!hp67.core.cpu.decimal);
    assert_eq!(hp67.step(&rom), Ok(0o0000));
    assert_eq!(hp67.core.cpu.pc, 2);
}
