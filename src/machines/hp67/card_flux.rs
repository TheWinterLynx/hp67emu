//! Physical self-clocking flux encoding used by one logical HP-67 card track.
//!
//! HP documentation uses "side" or "track" for the user-visible 952-bit stream
//! selected by inserting one end of the card.  At the magnetic head that logical
//! stream is actually recorded on two parallel physical tracks: the 1-track has a
//! flux reversal for each logical one and the 0-track has a flux reversal for each
//! logical zero.  Exactly one physical track therefore reverses in every bit cell,
//! providing the clock as well as the data.
//!
//! This module intentionally models reversal events only.  Absolute magnetic
//! polarity, head geometry, sense-amplifier analogue waveforms and the unresolved
//! temporal bit order used to serialize a 28-bit CRC word remain later M13 work.

use super::magnetic_card::HP67_CARD_BITS_PER_TRACK;

pub const HP67_PHYSICAL_FLUX_TRACKS_PER_LOGICAL_TRACK: usize = 2;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Hp67PhysicalFluxTrack {
    Zero,
    One,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Hp67FluxCell {
    reversal_track: Hp67PhysicalFluxTrack,
}

impl Hp67FluxCell {
    pub const fn encode(bit: bool) -> Self {
        Self {
            reversal_track: if bit {
                Hp67PhysicalFluxTrack::One
            } else {
                Hp67PhysicalFluxTrack::Zero
            },
        }
    }

    pub const fn reversal_track(self) -> Hp67PhysicalFluxTrack {
        self.reversal_track
    }

    pub const fn zero_track_reverses(self) -> bool {
        matches!(self.reversal_track, Hp67PhysicalFluxTrack::Zero)
    }

    pub const fn one_track_reverses(self) -> bool {
        matches!(self.reversal_track, Hp67PhysicalFluxTrack::One)
    }

    pub const fn decoded_bit(self) -> bool {
        matches!(self.reversal_track, Hp67PhysicalFluxTrack::One)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Hp67SelfClockingFluxPair {
    cells: [Hp67FluxCell; HP67_CARD_BITS_PER_TRACK],
}

impl Hp67SelfClockingFluxPair {
    pub fn from_serial_bits(bits: &[bool; HP67_CARD_BITS_PER_TRACK]) -> Self {
        let mut cells = [Hp67FluxCell::encode(false); HP67_CARD_BITS_PER_TRACK];
        for (cell, bit) in cells.iter_mut().zip(bits.iter().copied()) {
            *cell = Hp67FluxCell::encode(bit);
        }
        Self { cells }
    }

    pub fn cell(&self, index: usize) -> Option<Hp67FluxCell> {
        self.cells.get(index).copied()
    }

    pub fn serial_bits(&self) -> [bool; HP67_CARD_BITS_PER_TRACK] {
        let mut bits = [false; HP67_CARD_BITS_PER_TRACK];
        for (bit, cell) in bits.iter_mut().zip(self.cells.iter().copied()) {
            *bit = cell.decoded_bit();
        }
        bits
    }

    pub const fn len(&self) -> usize {
        HP67_CARD_BITS_PER_TRACK
    }

    pub const fn is_empty(&self) -> bool {
        false
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_logical_bit_generates_exactly_one_physical_flux_reversal() {
        for bit in [false, true] {
            let cell = Hp67FluxCell::encode(bit);
            assert_ne!(cell.zero_track_reverses(), cell.one_track_reverses());
            assert_eq!(cell.decoded_bit(), bit);
        }
    }

    #[test]
    fn zero_and_one_bits_select_the_documented_parallel_tracks() {
        assert_eq!(
            Hp67FluxCell::encode(false).reversal_track(),
            Hp67PhysicalFluxTrack::Zero
        );
        assert_eq!(
            Hp67FluxCell::encode(true).reversal_track(),
            Hp67PhysicalFluxTrack::One
        );
    }

    #[test]
    fn complete_952_bit_stream_round_trips_without_inventing_bit_order() {
        let mut bits = [false; HP67_CARD_BITS_PER_TRACK];
        for (index, bit) in bits.iter_mut().enumerate() {
            *bit = index % 3 == 1;
        }

        let flux = Hp67SelfClockingFluxPair::from_serial_bits(&bits);
        assert_eq!(flux.len(), HP67_CARD_BITS_PER_TRACK);
        assert_eq!(flux.serial_bits(), bits);

        for (index, expected) in bits.iter().copied().enumerate() {
            let cell = flux.cell(index).unwrap();
            assert_eq!(cell.decoded_bit(), expected);
            assert_ne!(cell.zero_track_reverses(), cell.one_track_reverses());
        }
    }
}
