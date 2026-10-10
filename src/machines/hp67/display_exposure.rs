//! Nominal ROM0 segment dwell integrated over complete structural scans.
//!
//! Teenix HP-67 captures, pages 76-77, report approximately 40 us per normal
//! segment and 30 us per decimal. These are observed dwell times, not current,
//! radiant energy, exact PHI edges or an electrical waveform. Keep all fifteen
//! scan slots, including the duplicate, without guessing its optical routing.

use super::{
    display::{Hp67SegmentMask, HP67_DISPLAY_SCAN_SLOTS},
    timing::{
        HP67_OBSERVED_DISPLAY_DP_ON_US, HP67_OBSERVED_DISPLAY_REFRESH_US,
        HP67_OBSERVED_DISPLAY_SEGMENT_ON_US, HP67_OBSERVED_WORD_TIME_US,
    },
};

pub type Hp67ScanDwellUs = [[u64; 8]; HP67_DISPLAY_SCAN_SLOTS as usize];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67DisplayExposure {
    pending: Hp67ScanDwellUs,
    completed: Hp67ScanDwellUs,
    next_slot: u8,
    completed_scans: u64,
    pending_duration_us: u64,
}

impl Default for Hp67DisplayExposure {
    fn default() -> Self {
        Self {
            pending: [[0; 8]; HP67_DISPLAY_SCAN_SLOTS as usize],
            completed: [[0; 8]; HP67_DISPLAY_SCAN_SLOTS as usize],
            next_slot: 1,
            completed_scans: 0,
            pending_duration_us: 0,
        }
    }
}

impl Hp67DisplayExposure {
    /// Published only after all fifteen slots; partial scans cannot tear output.
    pub const fn completed_scan_dwell_us(&self) -> &Hp67ScanDwellUs {
        &self.completed
    }

    pub const fn completed_scans(&self) -> u64 {
        self.completed_scans
    }

    /// Consume validated ROM0/cathode output once per complete machine word.
    /// Rejection leaves both pending and published state unchanged.
    pub fn capture_word(
        &mut self,
        scan_slot: u8,
        anodes: Hp67SegmentMask,
    ) -> Result<bool, (u8, u8)> {
        if scan_slot != self.next_slot {
            return Err((self.next_slot, scan_slot));
        }
        let mut dwell = [0; 8];
        for (segment, value) in dwell.iter_mut().enumerate() {
            if anodes.bits() & (1 << segment) != 0 {
                *value = if segment == 7 {
                    HP67_OBSERVED_DISPLAY_DP_ON_US
                } else {
                    HP67_OBSERVED_DISPLAY_SEGMENT_ON_US
                };
            }
        }
        self.pending[usize::from(scan_slot - 1)] = dwell;
        self.pending_duration_us += HP67_OBSERVED_WORD_TIME_US;
        if scan_slot == HP67_DISPLAY_SCAN_SLOTS {
            debug_assert_eq!(self.pending_duration_us, HP67_OBSERVED_DISPLAY_REFRESH_US);
            self.completed = self.pending;
            self.completed_scans += 1;
            self.next_slot = 1;
            self.pending_duration_us = 0;
            Ok(true)
        } else {
            self.next_slot += 1;
            Ok(false)
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::machines::hp67::display::decode_rom0_display_byte;

    #[test]
    fn m14t_rom0_dwell_publishes_only_complete_scans_and_replaces_blanked_slots() {
        let mut exposure = Hp67DisplayExposure::default();
        for slot in 1..=15 {
            let code = match slot {
                3 => 2, // Both signs: ROM0 E and G, not physical routing.
                4 => 8,
                5 => 0x30,
                _ => 0x20,
            };
            let complete = exposure.capture_word(slot, decode_rom0_display_byte(slot, code).unwrap()).unwrap();
            assert_eq!(complete, slot == 15);
            if slot < 15 {
                assert_eq!(exposure.completed_scan_dwell_us(), &[[0; 8]; 15]);
            }
        }
        assert_eq!(exposure.completed_scans(), 1);
        assert_eq!(exposure.completed_scan_dwell_us()[2], [0, 0, 0, 0, 40, 0, 40, 0]);
        assert_eq!(exposure.completed_scan_dwell_us()[3], [40, 40, 40, 40, 40, 40, 40, 0]);
        assert_eq!(exposure.completed_scan_dwell_us()[4], [0, 0, 0, 0, 0, 0, 0, 30]);
        let before = exposure;
        assert_eq!(exposure.capture_word(2, Hp67SegmentMask::BLANK), Err((1, 2)));
        assert_eq!(exposure, before);
        for slot in 1..=15 {
            exposure.capture_word(slot, Hp67SegmentMask::BLANK).unwrap();
        }
        assert_eq!(exposure.completed_scan_dwell_us(), &[[0; 8]; 15]);
        assert_eq!(exposure.completed_scans(), 2);
    }
}
