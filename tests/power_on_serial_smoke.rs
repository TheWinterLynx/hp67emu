//! End-to-end structural regression for the first physically observed HP-67 startup words.

use hp67emu::machines::hp67::{
    run_structural_fetch_cycle, ActFetchEndpoint, ActRamImage, FetchPipelineLatch,
    Hp67ElectricalBackplane, Hp67RomWordSource, PowerOnActCore, RomFetchEndpoint,
};

struct StartupFixture;

impl Hp67RomWordSource for StartupFixture {
    fn read_word(&self, address: u16) -> Option<u16> {
        match address {
            0x000 => Some(0x000),
            0x001 => Some(0x3e3),
            0x0f8 => Some(0x11a),
            // The fourth fetch is concurrent with execution of 0x11a. Its
            // value is irrelevant to this checkpoint, but the bus still needs
            // a valid responder for the complete 56-bit cycle.
            0x0f9 => Some(0x000),
            _ => None,
        }
    }
}

#[test]
fn serial_power_on_executes_the_first_physical_startup_path() {
    let source = StartupFixture;
    let mut backplane = Hp67ElectricalBackplane::default();
    let mut fetch_act = ActFetchEndpoint::new(0);
    let mut fetch_rom = RomFetchEndpoint::default();
    let mut pipeline = FetchPipelineLatch::default();
    let mut act = PowerOnActCore::default();
    let mut ram = ActRamImage::hp67();
    let mut executed = Vec::new();
    let mut fetched = Vec::new();

    for _ in 0..4 {
        pipeline.begin_cycle();
        if let Some(word) = pipeline.executing_word() {
            let execution = act
                .execute_word(&mut ram, word)
                .expect("startup word must execute");
            executed.push((execution.pc, execution.word));
        }

        let address = act.pc();
        let word = run_structural_fetch_cycle(
            &mut backplane,
            address,
            &mut fetch_act,
            &mut fetch_rom,
            &source,
        )
        .expect("serial startup fetch must complete");
        pipeline.complete_cycle(word);
        fetched.push((address, word));
    }

    assert_eq!(
        &fetched[..3],
        &[(0x000, 0x000), (0x001, 0x3e3), (0x0f8, 0x11a)]
    );
    assert_eq!(
        executed,
        vec![(0x000, 0x000), (0x001, 0x3e3), (0x0f8, 0x11a)]
    );
    assert_eq!(act.pc(), 0x0f9);
    assert_eq!(act.executed_words(), 3);
    assert_eq!(backplane.word_index(), 4);
    assert_eq!(backplane.word_bit(), 0);
}
