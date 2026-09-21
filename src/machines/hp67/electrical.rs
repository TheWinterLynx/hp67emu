//! Dense HP-67 electrical fabric and staged scheduler primitive.
//!
//! The generic scheduler remains useful as a model-independent reference, but
//! its map-backed snapshot/pending machinery is deliberately not the production
//! path for a calculator whose net and device inventory is fixed. This module
//! provides the same resolve -> snapshot -> evaluate/stage -> commit contract
//! with compile-time HP-67 net/driver slots and no per-tick heap allocation.

use crate::emulation::{Bias, Drive, LogicLevel, Tick};

use super::wiring::{Hp67Driver, Hp67Net};

const DRIVER_PENDING_MASK: u16 = (1u16 << Hp67Driver::COUNT) - 1;
const NET_PENDING_MASK: u16 = (1u16 << Hp67Net::COUNT) - 1;

#[derive(Debug, Clone)]
struct Hp67DenseNet {
    bias: Bias,
    drives: [Drive; Hp67Driver::COUNT],
    low_driver_count: u8,
    high_driver_count: u8,
}

impl Hp67DenseNet {
    const fn new(bias: Bias) -> Self {
        Self {
            bias,
            drives: [Drive::HighZ; Hp67Driver::COUNT],
            low_driver_count: 0,
            high_driver_count: 0,
        }
    }

    fn set_drive(&mut self, driver: Hp67Driver, drive: Drive) {
        let slot = &mut self.drives[driver.index()];
        let previous = *slot;
        if previous == drive {
            return;
        }

        match previous {
            Drive::HighZ => {}
            Drive::Low => self.low_driver_count -= 1,
            Drive::High => self.high_driver_count -= 1,
        }
        *slot = drive;
        match drive {
            Drive::HighZ => {}
            Drive::Low => self.low_driver_count += 1,
            Drive::High => self.high_driver_count += 1,
        }
    }

    const fn level(&self) -> LogicLevel {
        match (self.low_driver_count != 0, self.high_driver_count != 0) {
            (true, true) => LogicLevel::Contention,
            (true, false) => LogicLevel::Low,
            (false, true) => LogicLevel::High,
            (false, false) => match self.bias {
                Bias::Floating => LogicLevel::Floating,
                Bias::PullDown => LogicLevel::Low,
                Bias::PullUp => LogicLevel::High,
            },
        }
    }

    fn drive(&self, driver: Hp67Driver) -> Drive {
        self.drives[driver.index()]
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Hp67ElectricalError {
    Contention {
        tick: Tick,
        net: Hp67Net,
        drivers: Vec<Hp67Driver>,
    },
}

/// Immutable resolved input image shared by every device evaluation in one tick.
#[derive(Debug, Clone, Copy)]
pub struct Hp67ElectricalSnapshot<'a> {
    tick: Tick,
    levels: &'a [LogicLevel; Hp67Net::COUNT],
}

impl<'a> Hp67ElectricalSnapshot<'a> {
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    pub const fn level(&self, net: Hp67Net) -> LogicLevel {
        self.levels[net.index()]
    }
}

/// Mutable staging half of one HP-67 device-evaluation interval.
///
/// It deliberately has no access to committed nets, so publishing outputs cannot
/// change what any device samples during the same evaluation interval.
pub struct Hp67ElectricalStager<'a> {
    pending_drives: &'a mut [[Drive; Hp67Driver::COUNT]; Hp67Net::COUNT],
    pending_driver_masks: &'a mut [u16; Hp67Net::COUNT],
    pending_net_mask: &'a mut u16,
}

impl<'a> Hp67ElectricalStager<'a> {
    pub fn stage_drive(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        let net_index = net.index();
        let driver_index = driver.index();
        self.pending_drives[net_index][driver_index] = drive;
        self.pending_driver_masks[net_index] |= 1u16 << driver_index;
        *self.pending_net_mask |= 1u16 << net_index;
    }
}

/// Fixed-topology HP-67 electrical state and staged drive commit buffer.
///
/// Normal ticks allocate nothing. Pending output changes are indexed directly
/// by net and driver. Per-net driver bitmasks plus one dirty-net bitmask make
/// commit cost proportional to outputs that actually changed rather than all
/// possible device/net combinations.
#[derive(Debug, Clone)]
pub struct Hp67ElectricalFabric {
    tick: Tick,
    nets: [Hp67DenseNet; Hp67Net::COUNT],
    resolved_levels: [LogicLevel; Hp67Net::COUNT],
    pending_drives: [[Drive; Hp67Driver::COUNT]; Hp67Net::COUNT],
    pending_driver_masks: [u16; Hp67Net::COUNT],
    pending_net_mask: u16,
    contention_net_count: u8,
}

impl Default for Hp67ElectricalFabric {
    fn default() -> Self {
        let mut nets = std::array::from_fn(|index| {
            let net = Hp67Net::ALL[index];
            let bias = if net == Hp67Net::Isa {
                Bias::PullDown
            } else {
                Bias::Floating
            };
            Hp67DenseNet::new(bias)
        });

        // Direct HP-67 captures show both ACT clock outputs high between their
        // alternating low-going pulses.
        nets[Hp67Net::Phi1.index()].set_drive(Hp67Driver::Act1820_2530, Drive::High);
        nets[Hp67Net::Phi2.index()].set_drive(Hp67Driver::Act1820_2530, Drive::High);

        let resolved_levels = std::array::from_fn(|index| nets[index].level());

        Self {
            tick: Tick::ZERO,
            nets,
            resolved_levels,
            pending_drives: [[Drive::HighZ; Hp67Driver::COUNT]; Hp67Net::COUNT],
            pending_driver_masks: [0; Hp67Net::COUNT],
            pending_net_mask: 0,
            contention_net_count: 0,
        }
    }
}

impl Hp67ElectricalFabric {
    pub const fn tick(&self) -> Tick {
        self.tick
    }

    pub const fn level(&self, net: Hp67Net) -> LogicLevel {
        self.resolved_levels[net.index()]
    }

    pub fn drive(&self, net: Hp67Net, driver: Hp67Driver) -> Drive {
        self.nets[net.index()].drive(driver)
    }

    /// Immediate committed drive used by the current M14A structural bridge.
    ///
    /// M14B device evaluation should prefer stage_drive plus commit_staged so
    /// every device reads one immutable snapshot.
    pub fn set_drive_immediate(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        self.apply_drive(net.index(), driver, drive);
    }

    /// Advance canonical electrical time without staged outputs.
    ///
    /// This exists for the current explicit PHI bridge, where the edge itself is
    /// already the committed event. Device-scheduled M14B work uses
    /// commit_staged instead.
    pub fn advance_tick(&mut self) -> Tick {
        self.tick = Tick::new(self.tick.get().wrapping_add(1));
        self.tick
    }

    /// Borrow the already-resolved inputs and the independent output staging
    /// buffer for one device-evaluation interval without copying all net levels.
    ///
    /// The returned snapshot references only committed resolved levels. The
    /// stager references only pending outputs, so Rust enforces that staged
    /// writes cannot mutate what any device samples in this interval.
    pub fn begin_evaluation(
        &mut self,
    ) -> Result<(Hp67ElectricalSnapshot<'_>, Hp67ElectricalStager<'_>), Hp67ElectricalError> {
        if self.contention_net_count != 0 {
            return Err(self.contention_error(self.tick));
        }

        let tick = self.tick;
        let snapshot = Hp67ElectricalSnapshot {
            tick,
            levels: &self.resolved_levels,
        };
        let stager = Hp67ElectricalStager {
            pending_drives: &mut self.pending_drives,
            pending_driver_masks: &mut self.pending_driver_masks,
            pending_net_mask: &mut self.pending_net_mask,
        };
        Ok((snapshot, stager))
    }

    /// Convenience staging entry point for transitional callers that do not
    /// need to hold a zero-copy resolved snapshot.
    pub fn stage_drive(&mut self, net: Hp67Net, driver: Hp67Driver, drive: Drive) {
        let mut stager = Hp67ElectricalStager {
            pending_drives: &mut self.pending_drives,
            pending_driver_masks: &mut self.pending_driver_masks,
            pending_net_mask: &mut self.pending_net_mask,
        };
        stager.stage_drive(net, driver, drive);
    }

    /// Commit every staged device output atomically and advance one scheduler tick.
    pub fn commit_staged(&mut self) -> Result<Tick, Hp67ElectricalError> {
        debug_assert_eq!(self.pending_net_mask & !NET_PENDING_MASK, 0);

        while self.pending_net_mask != 0 {
            let net_index = self.pending_net_mask.trailing_zeros() as usize;
            self.pending_net_mask &= self.pending_net_mask - 1;

            let mut driver_mask = self.pending_driver_masks[net_index];
            debug_assert_eq!(driver_mask & !DRIVER_PENDING_MASK, 0);
            self.pending_driver_masks[net_index] = 0;

            while driver_mask != 0 {
                let driver_index = driver_mask.trailing_zeros() as usize;
                driver_mask &= driver_mask - 1;
                let drive = self.pending_drives[net_index][driver_index];
                let driver = Hp67Driver::ALL[driver_index];
                self.apply_drive(net_index, driver, drive);
            }
        }

        let tick = self.advance_tick();
        if self.contention_net_count != 0 {
            return Err(self.contention_error(tick));
        }
        Ok(tick)
    }

    fn apply_drive(&mut self, net_index: usize, driver: Hp67Driver, drive: Drive) {
        let previous_level = self.resolved_levels[net_index];
        self.nets[net_index].set_drive(driver, drive);
        let next_level = self.nets[net_index].level();
        self.resolved_levels[net_index] = next_level;

        match (
            previous_level == LogicLevel::Contention,
            next_level == LogicLevel::Contention,
        ) {
            (false, true) => self.contention_net_count += 1,
            (true, false) => self.contention_net_count -= 1,
            _ => {}
        }
    }

    fn contention_error(&self, tick: Tick) -> Hp67ElectricalError {
        let net = Hp67Net::ALL
            .into_iter()
            .find(|net| self.resolved_levels[net.index()] == LogicLevel::Contention)
            .expect("cached HP-67 contention count must identify a contentious net");
        let dense = &self.nets[net.index()];
        let drivers = Hp67Driver::ALL
            .into_iter()
            .filter(|driver| dense.drive(*driver) != Drive::HighZ)
            .collect();
        Hp67ElectricalError::Contention { tick, net, drivers }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn evidenced_idle_levels_are_encoded_in_dense_fabric() {
        let fabric = Hp67ElectricalFabric::default();
        for net in Hp67Net::ALL {
            let expected = match net {
                Hp67Net::Phi1 | Hp67Net::Phi2 => LogicLevel::High,
                Hp67Net::Isa => LogicLevel::Low,
                _ => LogicLevel::Floating,
            };
            assert_eq!(
                fabric.level(net),
                expected,
                "unexpected idle level for {net:?}"
            );
        }
    }

    #[test]
    fn staged_outputs_are_invisible_until_atomic_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        {
            let (snapshot, mut stager) = fabric.begin_evaluation().unwrap();
            assert_eq!(snapshot.tick(), Tick::ZERO);
            assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);

            stager.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
            assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);
        }

        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
        assert_eq!(fabric.commit_staged(), Ok(Tick::new(1)));
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
    }

    #[test]
    fn repeated_staging_by_one_driver_is_final_write_wins_without_duplicate_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::Low);
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::HighZ);
        assert_eq!(fabric.commit_staged(), Ok(Tick::new(1)));
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::Floating);
    }

    #[test]
    fn bitmask_commit_applies_multiple_nets_and_drivers_atomically() {
        let mut fabric = Hp67ElectricalFabric::default();
        {
            let (snapshot, mut stager) = fabric.begin_evaluation().unwrap();
            assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);
            assert_eq!(snapshot.level(Hp67Net::Str), LogicLevel::Floating);

            stager.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
            stager.stage_drive(Hp67Net::Str, Hp67Driver::RomDisplay1818_0268, Drive::Low);

            assert_eq!(snapshot.level(Hp67Net::Data), LogicLevel::Floating);
            assert_eq!(snapshot.level(Hp67Net::Str), LogicLevel::Floating);
        }

        assert_eq!(fabric.commit_staged(), Ok(Tick::new(1)));
        assert_eq!(fabric.level(Hp67Net::Data), LogicLevel::High);
        assert_eq!(fabric.level(Hp67Net::Str), LogicLevel::Low);
    }

    #[test]
    fn simultaneous_opposite_drives_report_contention_after_commit() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.stage_drive(Hp67Net::Data, Hp67Driver::Act1820_2530, Drive::High);
        fabric.stage_drive(
            Hp67Net::Data,
            Hp67Driver::StructuralRomResponder,
            Drive::Low,
        );

        assert_eq!(
            fabric.commit_staged(),
            Err(Hp67ElectricalError::Contention {
                tick: Tick::new(1),
                net: Hp67Net::Data,
                drivers: vec![Hp67Driver::Act1820_2530, Hp67Driver::StructuralRomResponder,],
            })
        );
    }

    #[test]
    fn dense_resolution_matches_generic_net_for_all_two_driver_states() {
        use crate::emulation::{DriverId, Net};

        const REF_ACT: DriverId = DriverId::new("reference-act");
        const REF_ROM: DriverId = DriverId::new("reference-rom");
        let drives = [Drive::HighZ, Drive::Low, Drive::High];

        for (net, bias) in [
            (Hp67Net::Data, Bias::Floating),
            (Hp67Net::Isa, Bias::PullDown),
        ] {
            for act_drive in drives {
                for rom_drive in drives {
                    let mut reference = Net::new(bias);
                    reference.set_drive(REF_ACT, act_drive);
                    reference.set_drive(REF_ROM, rom_drive);

                    let mut dense = Hp67ElectricalFabric::default();
                    dense.set_drive_immediate(net, Hp67Driver::Act1820_2530, act_drive);
                    dense.set_drive_immediate(net, Hp67Driver::StructuralRomResponder, rom_drive);

                    assert_eq!(
                        dense.level(net),
                        reference.level(),
                        "dense resolver drift for {net:?}: ACT={act_drive:?}, ROM={rom_drive:?}"
                    );
                }
            }
        }
    }

    #[test]
    fn dense_driver_slots_update_resolved_level_in_constant_index_space() {
        let mut fabric = Hp67ElectricalFabric::default();
        fabric.set_drive_immediate(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::High);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::High);
        fabric.set_drive_immediate(Hp67Net::Isa, Hp67Driver::Act1820_2530, Drive::HighZ);
        assert_eq!(fabric.level(Hp67Net::Isa), LogicLevel::Low);
    }
}
