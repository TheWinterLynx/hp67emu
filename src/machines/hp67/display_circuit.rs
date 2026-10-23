//! Pin-derived ROM0/cathode output and complete-scan nominal exposure.
//! Logical checkpoints and observed dwell are not calibrated PHI waveforms.
use super::{
    display::{decode_rom0_display_byte, CathodeDriver1820_1749, CathodeScanError, Hp67DisplayRole, Hp67SegmentMask, Rom0DisplayEndpoint, Rom0DisplayError},
    display_exposure::Hp67DisplayExposure,
    timing::BITS_PER_WORD,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67DisplayCircuitError {
    Receiver(Rom0DisplayError),
    Cathode(CathodeScanError),
    Exposure { expected: u8, actual: u8 },
    StrobeCadence { previous: u64, actual: u64 },
    Phase { device: u8, observer: u8 },
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67DisplayEmission {
    pub scan_slot: u8,
    pub role: Hp67DisplayRole,
    pub code: u8,
    pub anodes: Hp67SegmentMask,
    pub enabled: bool,
    pub strobe_tick: u64,
    pub completed_scan: bool,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub struct Hp67DisplayCircuit {
    cathode: CathodeDriver1820_1749,
    exposure: Hp67DisplayExposure,
    emission: Option<Hp67DisplayEmission>,
}
impl Hp67DisplayCircuit {
    pub const fn scan_slot(&self) -> u8 { self.cathode.scan_slot() }
    pub const fn exposure(&self) -> &Hp67DisplayExposure { &self.exposure }
    pub const fn emission(&self) -> Option<Hp67DisplayEmission> { self.emission }
    /// ACT's slot is a comparison only, never a cathode input or resync request.
    pub fn check_phase(&self, observer: u8) -> Result<(), Hp67DisplayCircuitError> {
        if self.scan_slot() != observer { return Err(Hp67DisplayCircuitError::Phase { device: self.scan_slot(), observer }); }
        Ok(())
    }
    /// Only a transaction reconstructed from resolved pins can clock this path.
    /// Rejection preserves cathode, anode latch, pending dwell and published scan.
    pub fn capture_rom0(&mut self, rom0: &Rom0DisplayEndpoint, enabled: bool) -> Result<Hp67DisplayEmission, Hp67DisplayCircuitError> {
        let strobe = rom0.resolved_strobe().map_err(Hp67DisplayCircuitError::Receiver)?;
        if let Some(previous) = self.emission {
            if previous.strobe_tick.checked_add(u64::from(BITS_PER_WORD) * 4) != Some(strobe.tick()) {
                return Err(Hp67DisplayCircuitError::StrobeCadence { previous: previous.strobe_tick, actual: strobe.tick() });
            }
        }
        let slot = self.scan_slot();
        let anodes = if enabled { decode_rom0_display_byte(slot, strobe.code()).map_err(Hp67DisplayCircuitError::Receiver)? } else { Hp67SegmentMask::BLANK };
        let mut cathode = self.cathode;
        let role = cathode.apply_resolved_strobe(strobe).map_err(Hp67DisplayCircuitError::Cathode)?;
        let mut exposure = self.exposure;
        let completed_scan = exposure.capture_word(slot, anodes).map_err(|(expected, actual)| Hp67DisplayCircuitError::Exposure { expected, actual })?;
        let emission = Hp67DisplayEmission { scan_slot: slot, role, code: strobe.code(), anodes, enabled, strobe_tick: strobe.tick(), completed_scan };
        self.cathode = cathode; self.exposure = exposure; self.emission = Some(emission);
        Ok(emission)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{emulation::{Drive, LogicLevel}, machines::hp67::{Hp67ElectricalBackplane, wiring::{Hp67Net, Hp67Driver}}};

    fn prepare(bus: &mut Hp67ElectricalBackplane, code: u8) {
        let bit = bus.word_bit();
        bus.drive(Hp67Net::Isa, Hp67Driver::Act1820_2530, if bit < 8 && code & (1 << bit) != 0 { Drive::High } else { Drive::HighZ });
        bus.advance_clock(); bus.advance_clock();
    }
    fn checkpoint(rom0: &mut Rom0DisplayEndpoint, bus: &mut Hp67ElectricalBackplane, rcd: bool) -> Result<(), Rom0DisplayError> {
        let mut next = *rom0; next.sample_data_at_edge(bus)?;
        let bit = bus.word_bit();
        if matches!(bit, 0 | 7 | 8) {
            bus.commit_display_controls(next.str_drive(bit)?, if bit == 7 && rcd { Drive::Low } else { Drive::High });
        }
        next.sample_controls_at_edge(bus)?; *rom0 = next; Ok(())
    }
    fn word(rom0: &mut Rom0DisplayEndpoint, bus: &mut Hp67ElectricalBackplane, code: u8, rcd: bool) {
        rom0.begin_word();
        for _ in 0..56 { prepare(bus, code); checkpoint(rom0, bus, rcd).unwrap(); bus.advance_clock(); bus.advance_clock(); }
    }
    #[test]
    fn m15y_pin_strobes_drive_own_cathode_anodes_and_complete_scan_dwell() {
        let mut bus = Hp67ElectricalBackplane::default(); let mut rom0 = Rom0DisplayEndpoint::default(); let mut circuit = Hp67DisplayCircuit::default();
        let mut expected = [[0; 8]; 15];
        for index in 0..45 {
            let slot = index % 15 + 1;
            let code = if slot == 3 { 2 } else { index % 10 };
            word(&mut rom0, &mut bus, code, slot == 15);
            let emission = circuit.capture_rom0(&rom0, true).unwrap();
            assert_eq!(emission.scan_slot, slot); assert_eq!(emission.code, code);
            assert_eq!(emission.strobe_tick, u64::from(index) * 224 + 30);
            let anodes = decode_rom0_display_byte(slot, code).unwrap(); assert_eq!(emission.anodes, anodes);
            for segment in 0..8 { expected[usize::from(slot - 1)][segment] = if anodes.bits() & (1 << segment) == 0 { 0 } else if segment == 7 { 30 } else { 40 }; }
            assert_eq!(emission.completed_scan, slot == 15);
            if slot == 15 { assert_eq!(circuit.exposure().completed_scan_dwell_us(), &expected); }
        }
        assert_eq!(circuit.scan_slot(), 1); assert_eq!(circuit.exposure().completed_scans(), 3); assert_eq!(bus.tick().get(), 45 * 224);
    }
    #[test]
    fn m15y_replay_missing_and_skipped_strobes_preserve_all_display_state() {
        let mut bus = Hp67ElectricalBackplane::default(); let mut rom0 = Rom0DisplayEndpoint::default(); let mut circuit = Hp67DisplayCircuit::default();
        assert!(circuit.capture_rom0(&rom0, true).is_err()); assert_eq!(circuit, Hp67DisplayCircuit::default());
        word(&mut rom0, &mut bus, 8, false); circuit.capture_rom0(&rom0, true).unwrap(); let before = circuit;
        assert!(matches!(circuit.capture_rom0(&rom0, true), Err(Hp67DisplayCircuitError::StrobeCadence { .. }))); assert_eq!(circuit, before);
        word(&mut rom0, &mut bus, 8, false); word(&mut rom0, &mut bus, 8, false);
        assert!(matches!(circuit.capture_rom0(&rom0, true), Err(Hp67DisplayCircuitError::StrobeCadence { .. }))); assert_eq!(circuit, before);
    }
    #[test]
    fn m15y_raw_rcd_is_checked_by_own_counter_and_decode_failure_is_atomic() {
        let mut bus = Hp67ElectricalBackplane::default(); let mut rom0 = Rom0DisplayEndpoint::default(); let mut circuit = Hp67DisplayCircuit::default();
        word(&mut rom0, &mut bus, 8, true); assert!(matches!(circuit.capture_rom0(&rom0, true), Err(Hp67DisplayCircuitError::Cathode(CathodeScanError::UnexpectedRcd { .. })))); assert_eq!(circuit, Hp67DisplayCircuit::default());
        let mut bus = Hp67ElectricalBackplane::default();
        for slot in 1..15 { word(&mut rom0, &mut bus, 0, false); circuit.capture_rom0(&rom0, slot != 3).unwrap(); }
        let before = circuit; word(&mut rom0, &mut bus, 0, false); assert_eq!(circuit.capture_rom0(&rom0, true), Err(Hp67DisplayCircuitError::Cathode(CathodeScanError::MissingRcdAtFinalSlot))); assert_eq!(circuit, before);
        let mut fresh = Hp67DisplayCircuit::default(); let mut bus = Hp67ElectricalBackplane::default(); word(&mut rom0, &mut bus, 0xff, false); let before = fresh;
        assert!(fresh.capture_rom0(&rom0, true).is_err()); assert_eq!(fresh, before);
        assert_eq!(fresh.capture_rom0(&rom0, false).unwrap().anodes, Hp67SegmentMask::BLANK);
    }
    #[test]
    fn m15y_every_control_coordinate_rejects_extra_drivers_without_latch_mutation() {
        for faulty_bit in 0..56 {
            for net in [Hp67Net::Str, Hp67Net::Rcd] {
                for drive in [Drive::High, Drive::Low] {
                    let mut rom0 = Rom0DisplayEndpoint::default(); let mut bus = Hp67ElectricalBackplane::default();
                    for bit in 0..=faulty_bit {
                        prepare(&mut bus, 8);
                        if bit == faulty_bit {
                            bus.drive(net, Hp67Driver::External, drive); let before = rom0;
                            assert!(checkpoint(&mut rom0, &mut bus, false).is_err(), "bit={bit} net={net:?} drive={drive:?}"); assert_eq!(rom0, before);
                        } else { checkpoint(&mut rom0, &mut bus, false).unwrap(); }
                        bus.advance_clock(); bus.advance_clock();
                    }
                }
            }
        }
    }
    #[test]
    fn m15y_receiver_rejects_wrong_phase_replay_floating_and_wrong_sole_owner() {
        let mut rom0 = Rom0DisplayEndpoint::default(); let mut bus = Hp67ElectricalBackplane::default(); let before = rom0;
        assert!(checkpoint(&mut rom0, &mut bus, false).is_err()); assert_eq!(rom0, before);
        prepare(&mut bus, 0); checkpoint(&mut rom0, &mut bus, false).unwrap(); let before = rom0;
        assert!(checkpoint(&mut rom0, &mut bus, false).is_err()); assert_eq!(rom0, before);
        for net in [Hp67Net::Str, Hp67Net::Rcd] {
            let mut rom0 = Rom0DisplayEndpoint::default(); let mut bus = Hp67ElectricalBackplane::default(); prepare(&mut bus, 0); rom0.sample_data_at_edge(&bus).unwrap(); bus.commit_display_controls(Drive::High, Drive::High);
            let driver = if net == Hp67Net::Str { Hp67Driver::RomDisplay1818_0268 } else { Hp67Driver::Act1820_2530 };
            bus.drive(net, driver, Drive::HighZ); assert_eq!(bus.level(net), LogicLevel::Floating); let before = rom0;
            assert!(rom0.sample_controls_at_edge(&bus).is_err()); assert_eq!(rom0, before);
            bus.drive(net, Hp67Driver::External, Drive::High); assert_eq!(bus.active_driver_count(net), 1);
            assert!(matches!(rom0.sample_controls_at_edge(&bus), Err(Rom0DisplayError::InvalidControlOwner { .. }))); assert_eq!(rom0, before);
        }
    }
}
