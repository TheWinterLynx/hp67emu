use super::*;
use hp67emu::machines::hp67::{decode_rom0_display_byte, wiring::{Hp67Net, Hp67Driver}};
use hp67emu::emulation::Drive;

#[test]
fn m15y_live_display_circuit_matches_reference_trace_keys_prefix_and_optics() {
    let mut live = Hp67LiveMachine::power_on_default().unwrap(); let mut traced = Hp67LiveMachine::power_on_default().unwrap(); let mut reference = Hp67LiveMachine::power_on_reference_fixture().unwrap();
    let mut trace = StructuralBusTrace::default(); let mut tails = 0;
    for cycle in 0..2400 {
        let contact = match cycle { 500 => Some(Some(Hp67Key::Rcl)), 700 | 1400 => Some(None), 1200 => Some(Some(Hp67Key::Digit0)), _ => None };
        if let Some(contact) = contact { for machine in [&mut live, &mut traced, &mut reference] { machine.set_key_contact(contact); } }
        tails += usize::from(live.data_serial.frame_in_progress()); let slot = live.display_circuit.scan_slot(); let enabled_before = live.machine.act.state.display_enable; let tick = live.backplane.tick().get();
        let execution = live.step_firmware_cycle_with_execution().unwrap(); assert_eq!(execution, reference.step_firmware_cycle_with_execution().unwrap()); assert_eq!(execution, traced.step_firmware_cycle_with_trace(Some(&mut trace)).unwrap());
        let emission = live.display_circuit.emission().unwrap(); assert_eq!(emission.scan_slot, slot);
        let expected = if !live.display_control_seen || enabled_before { decode_rom0_display_byte(slot, live.display_rom0.display_byte().unwrap()).unwrap() } else { Hp67SegmentMask::BLANK };
        assert_eq!(emission.anodes, expected, "cycle={cycle}");
        assert_eq!(live.display_circuit, reference.display_circuit); assert_eq!(live.display_circuit, traced.display_circuit);
        assert_eq!(live.display_frame(), reference.display_frame()); assert_eq!(live.optical_display_frame(), traced.optical_display_frame()); assert_eq!(live.machine.act.state, reference.machine.act.state);
        assert_eq!(live.backplane.tick().get() - tick, 224);
    }
    assert!(tails > 0); assert_eq!(trace.samples.len(), 2400 * 224);
    live.reset_power_on().unwrap(); assert_eq!(live.display_circuit, Hp67DisplayCircuit::default()); assert_eq!(live.optical_display_frame(), HardwareDisplayFrame::BLANK);
    live.step_firmware_cycle_with_execution().unwrap(); assert_eq!(live.display_circuit.emission().unwrap().strobe_tick, 30);
}
#[test]
fn m15y_scan_phase_poisoning_fails_before_clocks_and_pending_data_tail() {
    for tail in [false, true] {
        let mut live = Hp67LiveMachine::power_on_default().unwrap();
        if tail { for _ in 0..2000 { if live.data_serial.frame_in_progress() { break; } live.step_firmware_cycle_with_execution().unwrap(); } assert!(live.data_serial.frame_in_progress()); }
        live.act_serial.complete_display_word();
        let tick = live.backplane.tick(); let act = live.act_serial; let data = live.data_serial; let pipeline = live.pipeline; let circuit = live.display_circuit;
        let error = live.step_firmware_cycle_with_execution().unwrap_err(); assert!(error.contains("display phase failed"), "{error}");
        assert_eq!(live.backplane.tick(), tick); assert_eq!(live.act_serial, act); assert_eq!(live.data_serial, data); assert_eq!(live.pipeline, pipeline); assert_eq!(live.display_circuit, circuit);
    }
}
#[test]
fn m15y_control_faults_preserve_display_output_and_host_frames_cannot_feed_optics() {
    for net in [Hp67Net::Str, Hp67Net::Rcd] {
        for drive in [Drive::High, Drive::Low] {
            let mut live = Hp67LiveMachine::power_on_default().unwrap();
            for _ in 0..1200 { live.step_firmware_cycle_with_execution().unwrap(); }
            let circuit = live.display_circuit; let display = live.display; let optics = live.optical_display_frame(); let phase = live.act_serial.display_scan_slot();
            live.backplane.drive(net, Hp67Driver::External, drive);
            let error = live.step_firmware_cycle_with_execution().unwrap_err(); assert!(error.contains("InvalidControlSample"), "{error}");
            assert_eq!(live.display_circuit, circuit); assert_eq!(live.display, display); assert_eq!(live.optical_display_frame(), optics); assert_eq!(live.act_serial.display_scan_slot(), phase);
        }
    }
    let mut live = Hp67LiveMachine::power_on_default().unwrap();
    for _ in 0..1200 { live.step_firmware_cycle_with_execution().unwrap(); }
    let circuit = live.display_circuit; let optics = live.optical_display_frame(); live.display.clear();
    assert_eq!(live.display_circuit, circuit); assert_eq!(live.optical_display_frame(), optics);
}
